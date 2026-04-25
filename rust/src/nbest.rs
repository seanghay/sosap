//! N-best path extraction with custom uniqueness via a `PathFilter`.
//!
//! Strategy: delegate the heavy FST machinery (Reverse + ShortestDistance +
//! Dijkstra-on-reverse) to rustfst's `shortest_path_with_config`, then walk
//! the resulting tree-shaped lattice to extract individual paths and apply
//! the `PathFilter` for cluster decomposition + uniqueness deduplication.
//!
//! Compared to `NShortestPathSpecialized` in
//! `src/Phonetisaurus/include/PhonetisaurusRex.h:211-358` this is a
//! semantically-equivalent simplification: many lattice paths can decompose
//! to the same monophone sequence after M2MPathFilter cluster expansion +
//! veto filtering, so we ask rustfst for more raw paths than `nbest`,
//! dedupe by `unique_olabels`, and grow the request if we don't have
//! enough unique decompositions yet — capped so pathological inputs
//! don't loop forever.

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

    // Many lattice paths can decompose to the same monophone sequence after
    // M2MPathFilter expansion. Ask rustfst for more raw paths than `nbest`,
    // dedupe, and double the request if we still don't have enough unique
    // decompositions. The cap prevents pathological inputs from looping
    // forever.
    let mut nshortest = nbest.saturating_mul(8).max(16);
    let cap = nbest.saturating_mul(256).max(1024);

    loop {
        let cfg = ShortestPathConfig::default().with_nshortest(nshortest);
        let result: VectorFst<TropicalWeight> =
            shortest_path_with_config(lattice, cfg)?;

        if result.start().is_none() {
            return Ok(acc);
        }

        // Reset and re-walk: each iteration starts from a larger candidate
        // tree. PathFilter has no per-call state to clear; PathAccumulator
        // is rebuilt fresh.
        acc = PathAccumulator::new();
        walk_nbest_tree(&result, path_filter, nbest, accumulate, &mut acc)?;

        if acc.ordered_paths.len() >= nbest || nshortest >= cap {
            break;
        }
        nshortest = nshortest.saturating_mul(2).min(cap);
    }

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
    let start_trs = result.get_trs(start)?;
    let start_arcs: Vec<_> = start_trs.trs().iter().cloned().collect();

    // The shortest-path output from rustfst is a tree:
    //   - nshortest=1: start has 1 outgoing arc, the head of the only path.
    //   - nshortest>1: start has up to nshortest outgoing arcs, each the
    //     head of a distinct branch leading to a final state.
    // Either way each arc from start heads a complete path; we walk it,
    // applying the filter per arc, and accumulate.
    for branch_arc in start_arcs {
        let mut path = Path::default();
        path_filter.extend(&mut path, &branch_arc);

        let mut state = branch_arc.nextstate;
        loop {
            if result.final_weight(state)?.is_some() {
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
