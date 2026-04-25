# Legacy C++/Cython implementation

These files were the original Phonetisaurus C++ + Cython binding that shipped as
the `sosap` Python package up to the Rust rewrite. They are kept here for
reference; the live build no longer touches them.

`setup.py` and `core.pyx` reference `phonemizer.cc` and the vendored
`openfst/` and `Phonetisaurus/` trees, but the build is broken on recent macOS
clang (see `openfst/include/fst/bi-table.h:320` — references a missing `s_`
member of `VectorHashBiTable`). Reanimating it would mean either patching that
header or pinning to an older OpenFST snapshot. Out of scope here.

The current Python package lives at `../python/sosap/` and is built by maturin
from the Rust crate at `../rust/`.
