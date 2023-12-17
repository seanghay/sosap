from setuptools import setup, Extension
from Cython.Build import cythonize
import sys

COMPILE_ARGS = ["-std=c++11", "-w"]

if sys.platform.startswith("darwin"):
    COMPILE_ARGS.append("-stdlib=libc++")
    COMPILE_ARGS.append("-mmacosx-version-min=10.7")

setup(
    name="phonetisaurus",
    ext_modules=cythonize(
        Extension(
            "g2p",
            [
                "openfst/src/lib/compat.cc",
                "openfst/src/lib/flags.cc",
                "openfst/src/lib/fst-types.cc",
                "openfst/src/lib/fst.cc",
                "openfst/src/lib/mapped-file.cc",
                "openfst/src/lib/properties.cc",
                "openfst/src/lib/symbol-table-ops.cc",
                "openfst/src/lib/symbol-table.cc",
                "openfst/src/lib/util.cc",
                "openfst/src/lib/weight.cc",
                "Phonetisaurus/src/lib/util.cc",
                "g2p.pyx",
            ],
            language="c++",
            include_dirs=[
                "openfst/src/include",
                "Phonetisaurus/src",
                "Phonetisaurus/src/3rdparty/utfcpp",
            ],
            extra_compile_args=COMPILE_ARGS,
        )
    ),
)
