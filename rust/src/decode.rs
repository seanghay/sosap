//! High-level Phoneticize: load model, normalize, compose, n-best decode,
//! optional pmass post-processing. Ports `PhonetisaurusScript::Phoneticize`
//! from `src/Phonetisaurus/include/PhonetisaurusScript.h:107-192`.

use std::collections::HashSet;
use std::sync::Arc;

use rustfst::algorithms::{compose::compose, tr_sort};
use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::{Fst, SerializableFst};
use rustfst::prelude::ILabelCompare;
use rustfst::semirings::TropicalWeight;
use rustfst::{Label, SymbolTable};

use crate::fsa::entry_to_fsa;
use crate::nbest::nbest_paths;
use crate::path_filter::M2MPathFilter;
use crate::symbols::{load_clusters, Clusters};
use crate::tokenize::tokenize_to_labels;
use crate::Error;

#[derive(Debug, Clone)]
pub struct PathData {
    pub path_weight: f32,
    pub path_weights: Vec<f32>,
    pub ilabels: Vec<i32>,
    pub olabels: Vec<i32>,
    pub uniques: Vec<i32>,
}

#[derive(Debug, Clone)]
pub struct PhoneticizeOptions {
    pub nbest: usize,
    pub beam: usize,
    pub threshold: f32,
    pub write_fsts: bool,
    pub accumulate: bool,
    pub pmass: f64,
}

impl Default for PhoneticizeOptions {
    fn default() -> Self {
        Self {
            nbest: 1,
            beam: 10_000,
            threshold: 99.0,
            write_fsts: false,
            accumulate: false,
            pmass: 99.0,
        }
    }
}

pub struct Model {
    fst: VectorFst<TropicalWeight>,
    isyms: Arc<SymbolTable>,
    osyms: Arc<SymbolTable>,
    iclusters: Clusters,
    oclusters: Clusters,
    veto_set: HashSet<Label>,
    delim: String,
}

impl Model {
    pub fn from_fst(
        mut fst: VectorFst<TropicalWeight>,
        delim: &str,
    ) -> Result<Self, Error> {
        // ArcSort by ilabel (PhonetisaurusScript.h:76).
        tr_sort(&mut fst, ILabelCompare {});

        let isyms = fst
            .input_symbols()
            .ok_or(Error::MissingSymbolTable("input"))?
            .clone();
        let osyms = fst
            .output_symbols()
            .ok_or(Error::MissingSymbolTable("output"))?
            .clone();

        let iclusters = load_clusters(&isyms);
        let oclusters = load_clusters(&osyms);

        let mut veto_set: HashSet<Label> = HashSet::new();
        veto_set.insert(0);
        veto_set.insert(1);
        veto_set.insert(2);

        Ok(Self {
            fst,
            isyms,
            osyms,
            iclusters,
            oclusters,
            veto_set,
            delim: delim.to_string(),
        })
    }

    pub fn from_bytes(bytes: &[u8], delim: &str) -> Result<Self, Error> {
        let fst = VectorFst::<TropicalWeight>::load(bytes)
            .map_err(|e| Error::FstRead(e.to_string()))?;
        Self::from_fst(fst, delim)
    }

    #[cfg(feature = "std-fs")]
    pub fn open(path: impl AsRef<std::path::Path>, delim: &str) -> Result<Self, Error> {
        let fst = VectorFst::<TropicalWeight>::read(path)
            .map_err(|e| Error::FstRead(e.to_string()))?;
        Self::from_fst(fst, delim)
    }

    pub fn isyms(&self) -> &SymbolTable {
        &self.isyms
    }
    pub fn osyms(&self) -> &SymbolTable {
        &self.osyms
    }

    pub fn find_isym_id(&self, name: &str) -> Option<i32> {
        self.isyms.get_label(name).map(|l| l as i32)
    }
    pub fn find_isym_name(&self, id: i32) -> Option<&str> {
        if id < 0 {
            return None;
        }
        self.isyms.get_symbol(id as Label)
    }
    pub fn find_osym_id(&self, name: &str) -> Option<i32> {
        self.osyms.get_label(name).map(|l| l as i32)
    }
    pub fn find_osym_name(&self, id: i32) -> Option<&str> {
        if id < 0 {
            return None;
        }
        self.osyms.get_symbol(id as Label)
    }

    pub fn phoneticize(&self, word: &str, opts: &PhoneticizeOptions) -> Vec<PathData> {
        // 1. Tokenize input string -> input symbol IDs.
        let entry = tokenize_to_labels(word, &self.delim, &self.isyms);
        if entry.is_empty() {
            return Vec::new();
        }

        // 2. Build input acceptor FSA covering all valid segmentations.
        let mut input_fsa = entry_to_fsa(&entry, &self.iclusters.inv, self.iclusters.max_len);
        input_fsa.set_input_symbols(self.isyms.clone());
        input_fsa.set_output_symbols(self.isyms.clone());

        #[cfg(feature = "std-fs")]
        if opts.write_fsts {
            let _ = input_fsa.write(format!("{word}.fst"));
        }

        // 3. Compose input acceptor with G2P model.
        let lattice: VectorFst<TropicalWeight> = match compose::<
            TropicalWeight,
            VectorFst<TropicalWeight>,
            VectorFst<TropicalWeight>,
            VectorFst<TropicalWeight>,
            _,
            _,
        >(input_fsa, &self.fst)
        {
            Ok(l) => l,
            Err(_) => return Vec::new(),
        };

        #[cfg(feature = "std-fs")]
        if opts.write_fsts {
            let _ = lattice.write(format!("{word}.lat.fst"));
        }

        // 4. N-best with cluster decomposition + uniqueness via M2MPathFilter.
        let mut filter = M2MPathFilter::new(&self.oclusters.map, &self.veto_set);
        let acc = match nbest_paths(&lattice, opts.nbest, opts.accumulate, &mut filter) {
            Ok(a) => a,
            Err(_) => return Vec::new(),
        };

        // 5. Collect ordered_paths into PathData. Optional pmass thresholding.
        let mut paths: Vec<PathData> = Vec::with_capacity(acc.ordered_paths.len());
        let mut total_neg_log = f32::INFINITY;
        if opts.pmass < 99.0 {
            for key in &acc.ordered_paths {
                if let Some(p) = acc.path_map.get(key) {
                    total_neg_log = log_add(total_neg_log, p.path_weight);
                }
            }
        }

        let mut nbest_pmass = f32::INFINITY;
        for key in &acc.ordered_paths {
            let p = match acc.path_map.get(key) {
                Some(p) => p,
                None => continue,
            };
            let mut pweight = p.path_weight;
            if opts.pmass < 99.0 {
                pweight -= total_neg_log;
                nbest_pmass = log_add(nbest_pmass, pweight);
            }
            // Above weight_threshold? Skip. (C++ does this inside the search;
            // with our oversampled n-best, mimic with a post-filter.)
            if opts.threshold < 99.0 && pweight > opts.threshold {
                continue;
            }
            paths.push(PathData {
                path_weight: pweight,
                path_weights: p.path_weights.clone(),
                ilabels: p.ilabels.iter().map(|&l| l as i32).collect(),
                olabels: p.olabels.iter().map(|&l| l as i32).collect(),
                uniques: p.unique_olabels.iter().map(|&l| l as i32).collect(),
            });
            if opts.pmass < 99.0 && (nbest_pmass as f64) < opts.pmass {
                break;
            }
            // Beam: cap on number of returned paths in addition to nbest.
            if paths.len() >= opts.beam {
                break;
            }
        }

        paths
    }

    pub fn phoneticize_simple(&self, word: &str) -> Vec<String> {
        let paths = self.phoneticize(word, &PhoneticizeOptions::default());
        if let Some(p) = paths.first() {
            p.uniques
                .iter()
                .filter_map(|&id| {
                    if id < 0 {
                        None
                    } else {
                        self.osyms.get_symbol(id as Label).map(str::to_string)
                    }
                })
                .collect()
        } else {
            Vec::new()
        }
    }
}

fn log_add(a: f32, b: f32) -> f32 {
    if a.is_infinite() {
        return b;
    }
    if b.is_infinite() {
        return a;
    }
    let (lo, hi) = if a < b { (a, b) } else { (b, a) };
    lo - (-(hi - lo)).exp().ln_1p()
}
