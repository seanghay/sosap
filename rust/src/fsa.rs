//! Build the input acceptor FSA. Ports `Entry2FSA` non-superfinal branch
//! from `src/Phonetisaurus/include/PhonetisaurusRex.h:105-136`.

use rustfst::fst_impls::VectorFst;
use rustfst::fst_traits::MutableFst;
use rustfst::semirings::{Semiring, TropicalWeight};
use rustfst::{Label, StateId, Tr};

use crate::symbols::SymbolMapM21;

pub fn entry_to_fsa(
    word: &[Label],
    invmap: &SymbolMapM21,
    maxlen: usize,
) -> VectorFst<TropicalWeight> {
    let mut fsa = VectorFst::<TropicalWeight>::new();
    let s0 = fsa.add_state();
    fsa.set_start(s0).unwrap();

    for _ in 0..word.len() {
        fsa.add_state();
    }

    let one = TropicalWeight::one();
    for i in 0..word.len() {
        let from = i as StateId;
        // single-char arc
        fsa.add_tr(from, Tr::new(word[i], word[i], one, (i + 1) as StateId))
            .unwrap();

        // multi-char cluster arcs
        let mut j = 2usize;
        while j <= maxlen && i + j <= word.len() {
            let subv = &word[i..i + j];
            if let Some(&cluster_id) = invmap.get(subv) {
                fsa.add_tr(
                    from,
                    Tr::new(cluster_id, cluster_id, one, (i + j) as StateId),
                )
                .unwrap();
            }
            j += 1;
        }
    }

    fsa.set_final(word.len() as StateId, one).unwrap();
    fsa
}
