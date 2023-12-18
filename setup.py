from setuptools import setup, Extension
import sys

COMPILE_ARGS = ["-std=c++11", "-w"]

if sys.platform.startswith("darwin"):
    COMPILE_ARGS.append("-stdlib=libc++")
    COMPILE_ARGS.append("-mmacosx-version-min=10.7")
    
phonetisaurus_extension = Extension(
    name="_phonetisaurus",
    sources=[
        "src/openfst/lib/compat.cc",
        "src/openfst/lib/flags.cc",
        "src/openfst/lib/fst-types.cc",
        "src/openfst/lib/fst.cc",
        "src/openfst/lib/mapped-file.cc",
        "src/openfst/lib/properties.cc",
        "src/openfst/lib/symbol-table-ops.cc",
        "src/openfst/lib/symbol-table.cc",
        "src/openfst/lib/util.cc",
        "src/openfst/lib/weight.cc",
        "src/Phonetisaurus/lib/util.cc",
        "src/core.pyx",
    ],
    language="c++",
    include_dirs=[
        "src/openfst/include",
        "src/Phonetisaurus",
        "src/Phonetisaurus/3rdparty/utfcpp",
    ],
    extra_compile_args=COMPILE_ARGS,
)

setup(ext_modules=[phonetisaurus_extension])
