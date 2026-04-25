//! Cluster maps over a SymbolTable. Ports `LoadClusters` from
//! `src/Phonetisaurus/include/PhonetisaurusRex.h:73-103`.

use std::collections::HashMap;

use rustfst::{Label, SymbolTable};

pub type SymbolMap12M = HashMap<Label, Vec<Label>>;
pub type SymbolMapM21 = HashMap<Vec<Label>, Label>;

pub struct Clusters {
    pub map: SymbolMap12M,
    pub inv: SymbolMapM21,
    pub max_len: usize,
}

pub fn load_clusters(syms: &SymbolTable) -> Clusters {
    // Tie character is the *name* of symbol ID 1.
    let tie = syms.get_symbol(1).unwrap_or("|").to_string();
    let mut map: SymbolMap12M = HashMap::new();
    let mut inv: SymbolMapM21 = HashMap::new();
    let mut max_len: usize = 1;

    let n = syms.len() as Label;
    for id in 2..n {
        let Some(sym) = syms.get_symbol(id) else { continue };
        let cluster: Vec<Label> = if sym.contains(&tie) {
            sym.split(&tie)
                .filter(|s| !s.is_empty())
                .filter_map(|piece| syms.get_label(piece))
                .collect()
        } else {
            vec![id]
        };

        if cluster.len() > max_len {
            max_len = cluster.len();
        }
        inv.insert(cluster.clone(), id);
        map.insert(id, cluster);
    }

    Clusters { map, inv, max_len }
}
