"""sosap — Phonetisaurus G2P inference, pure-Rust implementation.

Public API mirrors the upstream ``PhonetisaurusScript`` C++ class plus the
backward-compatible top-1 ``phoneticize`` shape from earlier sosap releases.
"""

from sosap._sosap import Model, PathData

__all__ = ["Model", "PathData"]
