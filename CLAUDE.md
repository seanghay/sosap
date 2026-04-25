# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

`sosap` is a Phonetisaurus-style grapheme-to-phoneme tool. The Python package is now built by **maturin** from a pure-Rust core. The Rust crate also ships as a library, a CLI binary, and a WASM module — same code, four distribution targets:

| Target | How to build | Output |
|---|---|---|
| Python wheel | `maturin develop --release` (in venv) or CI (`.github/workflows/wheels.yml`) | `sosap` package importable via `from sosap import Model` |
| Rust CLI | `cd rust && cargo build --release` | `rust/target/release/sosap` |
| WASM (browser) | `cd rust && wasm-pack build --target web --release --no-default-features --no-typescript` | `rust/pkg/` (ES modules) |
| WASM (Node) | `cd rust && wasm-pack build --target nodejs --release --no-default-features --no-typescript --out-dir pkg-node` | `rust/pkg-node/` (CommonJS) |

The Rust API mirrors the upstream `PhonetisaurusScript` C++ class (see `feedback_mirror_upstream_api.md` in memory for rationale).

## Layout

```
.
├── pyproject.toml         # maturin build backend; points at rust/Cargo.toml
├── python/sosap/          # Python source (just re-exports from sosap._sosap)
├── rust/                  # Rust crate
│   ├── Cargo.toml         # cdylib + rlib + bin; features: std-fs (default), python
│   ├── src/
│   │   ├── lib.rs         # public Rust API
│   │   ├── decode.rs      # Model + PhoneticizeOptions + PathData
│   │   ├── nbest.rs       # n-best wrapper around rustfst::shortest_path
│   │   ├── path_filter.rs # M2MPathFilter + IdentityPathFilter + PathAccumulator
│   │   ├── fsa.rs         # Entry2FSA (input acceptor)
│   │   ├── tokenize.rs    # UTF-8 -> input symbol IDs
│   │   ├── symbols.rs     # LoadClusters
│   │   ├── wasm.rs        # wasm-bindgen exports (target_arch=wasm32)
│   │   ├── python.rs      # PyO3 exports (feature = "python")
│   │   └── bin/sosap.rs   # native CLI
│   └── tests/g2p.rs       # integration tests against ../g2p.fst (#[ignore])
├── g2p.fst                # 54 MB Khmer G2P test fixture
├── legacy/                # Old C++/Cython binding, preserved for reference
└── .github/workflows/
    └── wheels.yml         # maturin-action wheel builds for linux/macos/windows + sdist
```

## Common commands

```shell
# Python package — local development install
VIRTUAL_ENV=$(pwd)/venv ./venv/bin/maturin develop --release
./venv/bin/python -c "from sosap import Model; print(Model('g2p.fst').phoneticize('hello'))"

# Build a wheel locally (without installing)
VIRTUAL_ENV=$(pwd)/venv ./venv/bin/maturin build --release --out dist

# Rust CLI
cd rust && cargo run --release --bin sosap -- ../g2p.fst hello

# Rust integration tests (need ../g2p.fst)
cd rust && cargo test --release -- --include-ignored

# WASM build
cd rust && wasm-pack build --target web --release --no-default-features --no-typescript
cd rust && wasm-pack build --target nodejs --release --no-default-features --no-typescript --out-dir pkg-node
```

## Python API

```python
from sosap import Model, PathData

m = Model("g2p.fst")                            # Model(path, delim="")
m.phoneticize("hello")                          # -> ['h', 'ɛɛ', 'l', 'oo']  (top-1, backward-compat)
m.phoneticize_paths("hello", nbest=3)           # -> [PathData, PathData, PathData]
m.find_isym("h"); m.find_isym(77)               # name <-> id, mirrors PhonetisaurusScript.FindIsym
m.find_osym("h"); m.find_osym(5)
```

`PathData` exposes `.path_weight`, `.path_weights`, `.ilabels`, `.olabels`, `.uniques` — same shape as the upstream C++ `PathData` struct.

## Things to watch out for

- **Python-side import errors from Pylance/IDE**: `from sosap._sosap import Model` is a compiled extension; static analyzers can't resolve it. The runtime import works.
- **`maturin develop` needs `VIRTUAL_ENV` set** when invoked outside an activated venv.
- **WASM build needs `getrandom` with `wasm_js` feature** (already wired in `rust/Cargo.toml` under a target-cfg block).
- **`pyo3` is gated behind the `python` feature** so `cargo build` (default features) doesn't pull in Python headers. Maturin enables it automatically.
- **`legacy/` contains the old C++ tree** — don't try to fix or build it. The C++ doesn't compile on recent macOS clang. See `legacy/README.md`.
- **N-best caveat**: `rust/src/nbest.rs` uses `rustfst::shortest_path_with_config` with oversampling rather than a faithful port of `NShortestPathSpecialized`. Exact for `nbest=1`; for `nbest>1` may return fewer unique paths than C++ when many raw paths decompose to the same monophone sequence.

## CI

`.github/workflows/wheels.yml` builds wheels for linux x86_64+aarch64 (manylinux + musllinux), macos x86_64+aarch64, windows x86_64, plus an sdist. Triggers: push to `main`, tags `v*`, PRs, manual. Wheels are uploaded as artifacts named `wheels-<platform>-<arch>` plus an `all-wheels` aggregate. There's no PyPI publish step yet — add one when ready to release.
