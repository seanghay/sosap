//! JavaScript-callable wrapper. Built only for `wasm32-unknown-unknown`.

use wasm_bindgen::prelude::*;

use crate::{Model as InnerModel, PhoneticizeOptions};

#[wasm_bindgen]
pub struct Model {
    inner: InnerModel,
}

#[wasm_bindgen]
impl Model {
    /// Construct a Model from the raw bytes of an OpenFST `.fst` file
    /// (the same format Phonetisaurus emits). `delim` is usually `""`.
    #[wasm_bindgen(constructor)]
    pub fn new(bytes: &[u8], delim: &str) -> Result<Model, JsError> {
        let inner = InnerModel::from_bytes(bytes, delim)
            .map_err(|e| JsError::new(&e.to_string()))?;
        Ok(Model { inner })
    }

    /// Phoneticize a word, returning the top-1 phoneme strings as a `string[]`.
    pub fn phoneticize(&self, word: &str) -> Vec<String> {
        self.inner.phoneticize_simple(word)
    }

    /// Phoneticize a word with an explicit n-best count. Returns the i-th
    /// path's phoneme strings joined by `" "`. Multiple paths are joined by
    /// newlines so callers can split client-side. (JsValue interop with
    /// nested arrays would balloon the binding surface; keep it primitive.)
    pub fn phoneticize_nbest(&self, word: &str, nbest: usize) -> Vec<String> {
        let opts = PhoneticizeOptions { nbest, ..Default::default() };
        let paths = self.inner.phoneticize(word, &opts);
        paths
            .into_iter()
            .map(|p| {
                p.uniques
                    .iter()
                    .filter_map(|&id| {
                        if id < 0 {
                            None
                        } else {
                            self.inner
                                .osyms()
                                .get_symbol(id as u32)
                                .map(str::to_string)
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect()
    }
}
