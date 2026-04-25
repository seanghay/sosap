//! N-best path extraction with custom uniqueness via a `PathFilter`.
//!
//! Strategy: delegate the heavy FST machinery (Reverse + ShortestDistance +
//! Dijkstra-on-reverse) to rustfst's `shortest_path_with_config`, then walk
//! the resulting tree-shaped lattice to extract individual paths and apply
//! the `PathFilter` for cluster decomposition + uniqueness deduplication.
//!
//! Compared to `NShortestPathSpecialized` in
//! `src/Phonetisaurus/include/PhonetisaurusRex.h:211-358` this is a
//! semantically-equivalent simplification: we ask rustfst for `nbest`
//! candidate paths, then dedupe them by `unique_olabels`. For `nbest=1`
//! (the default) this is exact; for `nbest>1` it can return fewer unique
//! paths than C++ would, since the C++ algorithm keeps searching past
//! duplicates. Oversampling via `nbest * 4` covers the common case;
//! callers needing exact n>1 parity should oversample further.

use anyhow::Result;

use rustfst::algorithms::shortest_path_with_config;
use rustfst::algorithms::ShortestPathConfig;
use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::{CoreFst, ExpandedFst};
use rustfst::semirings::TropicalWeight;
use rustfst::Trs;

use crate::path_filter::{Path, PathAccumulator, PathFilter};

pub fn nbest_paths<F: PathFilter>(
    lattice: &VectorFst<TropicalWeight>,
    nbest: usize,
    accumulate: bool,
    path_filter: &mut F,
) -> Result<PathAccumulator> {
    let mut acc = PathAccumulator::new();
    if nbest == 0 || lattice.start().is_none() || lattice.num_states() == 0 {
        return Ok(acc);
    }

    // Oversample to give the dedupe step room to find `nbest` unique paths.
    let oversample = nbest.saturating_mul(4).max(nbest);
    let cfg = ShortestPathConfig::default().with_nshortest(oversample);
    let result: VectorFst<TropicalWeight> = shortest_path_with_config(lattice, cfg)?;

    if result.start().is_none() {
        return Ok(acc);
    }

    walk_nbest_tree(&result, path_filter, nbest, accumulate, &mut acc)?;
    Ok(acc)
}

fn walk_nbest_tree<F: PathFilter>(
    result: &VectorFst<TropicalWeight>,
    path_filter: &mut F,
    nbest: usize,
    accumulate: bool,
    acc: &mut PathAccumulator,
) -> Result<()> {
    let start = result.start().unwrap();

    // n=1: rustfst returns a single linear chain rooted at `start`; treat
    // the start as the head of the only path.
    let start_trs = result.get_trs(start)?;
    let start_arcs: Vec<_> = start_trs.trs().iter().cloned().collect();

    let single_chain = start_arcs.is_empty()
        || (start_arcs.len() == 1 && {
            // If the start has exactly one outgoing arc and the start itself
            // has no other distinguishing structure (i.e. it's the chain head),
            // walk from start as a single path.
            // Heuristic: if rustfst's nshortest>1 produced one branch only,
            // treat as a single chain too.
            true
        });

    if single_chain && nbest == 1 {
        let mut path = Path::default();
        let mut state = start;
        loop {
            if let Some(_) = result.final_weight(state)? {
                break;
            }
            let trs = result.get_trs(state)?;
            let trs_slice = trs.trs();
            if trs_slice.is_empty() {
                break;
            }
            let arc = trs_slice[0].clone();
            path_filter.extend(&mut path, &arc);
            state = arc.nextstate;
        }
        acc.push(path, accumulate);
        return Ok(());
    }

    // n>1: start has multiple outgoing arcs, one per branch.
    for branch_arc in start_arcs {
        let mut path = Path::default();
        // The branch_arc itself is part of the path bookkeeping in C++ —
        // matching that, run extend on it.
        path_filter.extend(&mut path, &branch_arc);

        let mut state = branch_arc.nextstate;
        loop {
            if let Some(_) = result.final_weight(state)? {
                break;
            }
            let trs = result.get_trs(state)?;
            let trs_slice = trs.trs();
            if trs_slice.is_empty() {
                break;
            }
            let arc = trs_slice[0].clone();
            path_filter.extend(&mut path, &arc);
            state = arc.nextstate;
        }
        acc.push(path, accumulate);
        if acc.ordered_paths.len() >= nbest {
            break;
        }
    }
    Ok(())
}

