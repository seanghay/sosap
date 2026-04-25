//! Path filters used to define path uniqueness and cluster decomposition
//! when extracting n-best results. Ports `Path`, `IdentityPathFilter`, and
//! `M2MPathFilter` from `src/Phonetisaurus/include/PhonetisaurusRex.h:138-208`.

use std::collections::{HashMap, HashSet};

use rustfst::semirings::{Semiring, TropicalWeight};
use rustfst::{Label, Tr};

use crate::symbols::SymbolMap12M;

#[derive(Default, Clone, Debug)]
pub struct Path {
    pub path_weight: f32,
    pub path_weights: Vec<f32>,
    pub ilabels: Vec<Label>,
    pub olabels: Vec<Label>,
    /// Filtered uniqueness key: epsilons removed, tied tokens decomposed
    /// into their monophone components.
    pub unique_olabels: Vec<Label>,
}

pub trait PathFilter {
    fn extend(&mut self, path: &mut Path, arc: &Tr<TropicalWeight>);
}

/// Filters epsilons only. Mirrors `IdentityPathFilter` in the C++. Not used
/// internally yet — kept so external callers can plug it in via the trait.
#[allow(dead_code)]
pub struct IdentityPathFilter;

impl PathFilter for IdentityPathFilter {
    fn extend(&mut self, path: &mut Path, arc: &Tr<TropicalWeight>) {
        if arc.ilabel == 0 && arc.olabel == 0 && arc.weight == TropicalWeight::one() {
            return;
        }
        if arc.olabel != 0 && arc.olabel != 1 && arc.olabel != 2 {
            path.unique_olabels.push(arc.olabel);
        }
        let w = *arc.weight.value();
        path.ilabels.push(arc.ilabel);
        path.olabels.push(arc.olabel);
        path.path_weights.push(w);
        path.path_weight += w;
    }
}

/// Decomposes cluster output labels into monophones via `omap` and drops
/// any veto IDs. Mirrors `M2MPathFilter` in the C++.
pub struct M2MPathFilter<'a> {
    pub label_map: &'a SymbolMap12M,
    pub veto_set: &'a HashSet<Label>,
}

impl<'a> M2MPathFilter<'a> {
    pub fn new(label_map: &'a SymbolMap12M, veto_set: &'a HashSet<Label>) -> Self {
        Self { label_map, veto_set }
    }
}

impl<'a> PathFilter for M2MPathFilter<'a> {
    fn extend(&mut self, path: &mut Path, arc: &Tr<TropicalWeight>) {
        if arc.ilabel == 0 && arc.olabel == 0 && arc.weight == TropicalWeight::one() {
            return;
        }
        if let Some(tokens) = self.label_map.get(&arc.olabel) {
            for &t in tokens {
                if !self.veto_set.contains(&t) {
                    path.unique_olabels.push(t);
                }
            }
        }
        let w = *arc.weight.value();
        path.ilabels.push(arc.ilabel);
        path.olabels.push(arc.olabel);
        path.path_weights.push(w);
        path.path_weight += w;
    }
}

/// Accumulator that walks an FST returned by rustfst's `shortest_path_with_config`
/// and emits up to `nbest` paths whose `unique_olabels` are distinct.
///
/// The shortest-path output FST has a single start state with up to `nbest`
/// outgoing arcs, each leading to a unique chain to a final state. We walk
/// each branch, run `path_filter.extend` per arc, and dedupe by `unique_olabels`.
pub struct PathAccumulator {
    pub path_map: HashMap<Vec<Label>, Path>,
    pub ordered_paths: Vec<Vec<Label>>,
}

impl PathAccumulator {
    pub fn new() -> Self {
        Self {
            path_map: HashMap::new(),
            ordered_paths: Vec::new(),
        }
    }

    /// Returns true if a *new* unique path was added; false if duplicate.
    pub fn push(&mut self, path: Path, accumulate: bool) -> bool {
        let key = path.unique_olabels.clone();
        match self.path_map.get_mut(&key) {
            Some(existing) if accumulate => {
                // log-add weights via -ln(exp(-a) + exp(-b))
                existing.path_weight = log_add(existing.path_weight, path.path_weight);
                false
            }
            Some(_) => false,
            None => {
                self.ordered_paths.push(key.clone());
                self.path_map.insert(key, path);
                true
            }
        }
    }
}

/// `LogWeight::Plus` over two negative-log probabilities.
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
