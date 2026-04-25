//! End-to-end smoke / sanity tests against the real `g2p.fst` model
//! (Khmer G2P model, ~54 MB, located at the repo root).
//!
//! These are `#[ignore]` by default because they need the FST present.
//! Run with: `cargo test --release -- --include-ignored`

use sosap::{Model, PhoneticizeOptions};

fn model_path() -> std::path::PathBuf {
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // -> repo root
    p.push("g2p.fst");
    p
}

#[test]
#[ignore]
fn loads_model_and_returns_nonempty_paths() {
    let m = Model::open(model_path(), "").expect("load g2p.fst");

    for w in &["hello", "world", "cat", "dog"] {
        let paths = m.phoneticize(w, &PhoneticizeOptions::default());
        assert!(!paths.is_empty(), "no paths for {w}");
        assert!(!paths[0].uniques.is_empty(), "empty uniques for {w}");
        assert!(paths[0].path_weight > 0.0 && paths[0].path_weight < 1000.0);
    }
}

#[test]
#[ignore]
fn nbest_returns_distinct_paths_in_weight_order() {
    let m = Model::open(model_path(), "").expect("load g2p.fst");
    let opts = PhoneticizeOptions { nbest: 5, ..Default::default() };
    let paths = m.phoneticize("hello", &opts);

    assert!(paths.len() >= 2, "expected several n-best paths, got {}", paths.len());
    for w in paths.windows(2) {
        assert!(w[0].path_weight <= w[1].path_weight, "paths not in weight order");
    }
    // All `uniques` should differ (M2M filter dedupes).
    let uniqs: std::collections::HashSet<Vec<i32>> =
        paths.iter().map(|p| p.uniques.clone()).collect();
    assert_eq!(uniqs.len(), paths.len(), "duplicate unique sequences slipped through");
}

#[test]
#[ignore]
fn phoneticize_simple_returns_phoneme_strings() {
    let m = Model::open(model_path(), "").expect("load g2p.fst");
    let phonemes = m.phoneticize_simple("hello");
    assert!(!phonemes.is_empty());
    // All phonemes should be non-empty strings.
    for p in &phonemes {
        assert!(!p.is_empty());
    }
    eprintln!("hello -> {:?}", phonemes);
}

#[test]
#[ignore]
fn handles_khmer_input() {
    let m = Model::open(model_path(), "").expect("load g2p.fst");
    // "កម្ពុជា" = Cambodia
    let phonemes = m.phoneticize_simple("កម្ពុជា");
    assert!(!phonemes.is_empty(), "Khmer input produced no phonemes");
    eprintln!("កម្ពុជា -> {:?}", phonemes);
}

#[test]
#[ignore]
fn empty_input_returns_no_paths() {
    let m = Model::open(model_path(), "").expect("load g2p.fst");
    let paths = m.phoneticize("", &PhoneticizeOptions::default());
    assert!(paths.is_empty());
}

#[test]
#[ignore]
fn unknown_chars_only_returns_no_paths() {
    let m = Model::open(model_path(), "").expect("load g2p.fst");
    // Characters very unlikely to be in the input symbol table.
    let paths = m.phoneticize("✨🚀💫", &PhoneticizeOptions::default());
    assert!(paths.is_empty());
}

#[test]
#[ignore]
fn from_bytes_matches_open() {
    let bytes = std::fs::read(model_path()).expect("read g2p.fst");
    let m_bytes = Model::from_bytes(&bytes, "").expect("load via bytes");
    let m_open = Model::open(model_path(), "").expect("load via open");

    for w in &["hello", "world", "cat"] {
        let a = m_bytes.phoneticize_simple(w);
        let b = m_open.phoneticize_simple(w);
        assert_eq!(a, b, "from_bytes vs open diverged for {w}");
    }
}
