//! UTF-8 tokenization. Ports `tokenize_utf8_string` and `tokenize2ints` from
//! `src/Phonetisaurus/lib/util.cc:51-126`.

use rustfst::{Label, SymbolTable};

pub fn tokenize_utf8_string(s: &str, delim: &str) -> Vec<String> {
    if delim.is_empty() {
        return s.chars().map(|c| c.to_string()).collect();
    }

    let mut out = vec![String::new()];
    for c in s.chars() {
        let cs = c.to_string();
        if cs == delim {
            out.push(String::new());
        } else {
            out.last_mut().unwrap().push_str(&cs);
        }
    }
    out
}

pub fn tokenize_to_labels(word: &str, delim: &str, syms: &SymbolTable) -> Vec<Label> {
    let tokens = tokenize_utf8_string(word, delim);
    let mut entry = Vec::with_capacity(tokens.len());
    for t in tokens {
        match syms.get_label(&t) {
            Some(label) => entry.push(label),
            None => {
                eprintln!(
                    "Symbol: '{t}' not found in input symbols table.\nMapping to null..."
                );
            }
        }
    }
    entry
}
