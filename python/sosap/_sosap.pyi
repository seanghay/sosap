"""Type stubs for the compiled Rust extension `sosap._sosap`.

The actual implementation lives in Rust (`rust/src/python.rs`). Static
analyzers can't see inside a `.so`/`.pyd`, so this file declares the
public surface for IDEs and type checkers.
"""

from typing import overload

class PathData:
    """A single G2P decoding result. Mirrors the upstream Phonetisaurus
    `PathData` struct."""

    path_weight: float
    path_weights: list[float]
    ilabels: list[int]
    olabels: list[int]
    uniques: list[int]

    def __repr__(self) -> str: ...

class Model:
    """Loaded G2P model. Mirrors the upstream `PhonetisaurusScript` class."""

    def __init__(self, path: str, delim: str = "") -> None:
        """Load an OpenFST `.fst` model from disk.

        Args:
            path: Filesystem path to the FST file produced by `phonetisaurus-train`.
            delim: Optional delimiter string between input tokens. Defaults to "".
        """

    def phoneticize(self, word: str) -> list[str]:
        """Top-1 phoneme sequence for `word`. Backward-compatible with the
        previous Cython binding."""

    def phoneticize_sampling(
        self,
        word: str,
        nbest: int = 1,
        beam: int = 10000,
        threshold: float = 99.0,
        pmass: float = 99.0,
    ) -> list[list[str]]:
        """N-best list of phoneme sequences. Each sublist is the cluster-
        decomposed, veto-filtered output of one path."""

    def phoneticize_paths(
        self,
        word: str,
        nbest: int = 1,
        beam: int = 10000,
        threshold: float = 99.0,
        write_fsts: bool = False,
        accumulate: bool = False,
        pmass: float = 99.0,
    ) -> list[PathData]:
        """Full upstream `Phoneticize()` surface. Returns `PathData` objects
        with per-arc weights and raw input/output labels."""

    @overload
    def find_isym(self, key: str) -> int | None: ...
    @overload
    def find_isym(self, key: int) -> str | None: ...
    def find_isym(self, key: str | int) -> int | str | None:
        """Look up an input symbol by name (-> id) or by id (-> name)."""

    @overload
    def find_osym(self, key: str) -> int | None: ...
    @overload
    def find_osym(self, key: int) -> str | None: ...
    def find_osym(self, key: str | int) -> int | str | None:
        """Look up an output symbol by name (-> id) or by id (-> name)."""
