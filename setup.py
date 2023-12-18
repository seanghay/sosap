from setuptools import setup, Extension
from Cython.Build import cythonize
import sys

COMPILE_ARGS = ["-std=c++11", "-w"]

if sys.platform.startswith("darwin"):
    COMPILE_ARGS.append("-stdlib=libc++")
    COMPILE_ARGS.append("-mmacosx-version-min=10.7")

setup(
    name="py_phonetisaurus",
    ext_modules=cythonize(
        Extension(
            "g2p",
            [
                "openfst/lib/compat.cc",
                "openfst/lib/flags.cc",
                "openfst/lib/fst-types.cc",
                "openfst/lib/fst.cc",
                "openfst/lib/mapped-file.cc",
                "openfst/lib/properties.cc",
                "openfst/lib/symbol-table-ops.cc",
                "openfst/lib/symbol-table.cc",
                "openfst/lib/util.cc",
                "openfst/lib/weight.cc",
                "Phonetisaurus/lib/util.cc",
                "g2p.pyx",
            ],
            language="c++",
            include_dirs=[
                "openfst/include",
                "Phonetisaurus",
                "Phonetisaurus/3rdparty/utfcpp",
            ],
            extra_compile_args=COMPILE_ARGS,
        )
    ),
)
