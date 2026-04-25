<img width=144 src="https://github.com/seanghay/sosap/assets/15277233/25c2ae30-4dd6-4350-a387-c30353cb2a98">

Phonetisaurus G2P inference, pure-Rust implementation. Ships as a Python package, a Rust crate (with a CLI), and a WASM module — same algorithm, four distribution targets.

### Install

```shell
# pypi
pip install sosap

# GitHub
pip install git+https://github.com/seanghay/sosap.git
```

### Phoneticize

```python
from sosap import Model

model = Model("g2p.fst")
model.phoneticize("hello")
# => ['h', 'ɛɛ', 'l', 'oo']
```

### N-best sampling

```python
from sosap import Model

model = Model("g2p.fst")
results = model.phoneticize_sampling("hello", nbest=4)
# => [['h', 'ɛɛ', 'l', 'oo'], ['h', 'ee', '.', 'l', 'oo'], ['h', 'ɛɛ', '.', 'l', 'oo'], ['h', 'ɛɛ', 'l', '.', 'l', 'ɔɔ']]

results = model.phoneticize_sampling("hello", nbest=4, beam=1000, threshold=99.0, pmass=99.0)
```

For full access to the underlying `PhonetisaurusScript` interface (per-arc weights, raw input/output labels, accumulate/pmass modes), use `model.phoneticize_paths(word, ...)` which returns `PathData` objects with `.path_weight`, `.path_weights`, `.ilabels`, `.olabels`, `.uniques`.

### Other targets

- **Rust crate** at [`rust/`](rust/) — `cargo build --release` produces a CLI (`sosap <model.fst> <word>`) and a `rustfst`-compatible library.
- **WebAssembly** — `cd rust && wasm-pack build --target web --release --no-default-features --no-typescript` builds a browser-ready bundle in `rust/pkg/`. The `Model` class accepts the FST as raw bytes (`new Model(uint8Array, "")`).

---

### License

`MIT`
