{{#title Bazel, Buck — Rust ♡ C++}}
## Bazel, Buck, potentially other similar environments

Starlark-based build systems with the ability to compile a code generator and
invoke it as a `genrule` will run CXX's C++ code generator via its `cxxbridge`
command line interface.

The tool is packaged as the `cxxbridge-cmd` crate on crates.io or can be built
from the *gen/cmd/* directory of the CXX GitHub repo.

```console
$  cargo install cxxbridge-cmd

$  cxxbridge src/bridge.rs --header > path/to/bridge.rs.h
$  cxxbridge src/bridge.rs > path/to/bridge.rs.cc
```

The CXX repo maintains working Bazel `BUILD` and Buck `BUCK` targets for the
complete blobstore tutorial (chapter 3) for your reference, tested in CI. These
aren't meant to be directly what you use in your codebase, but serve as an
illustration of one possible working pattern.

```python
# tools/bazel/rust_cxx_bridge.bzl

load("@bazel_skylib//rules:run_binary.bzl", "run_binary")
load("@rules_cc//cc:defs.bzl", "cc_library")

def rust_cxx_bridge(name, src, deps = []):
    native.alias(
        name = "%s/header" % name,
        actual = src + ".h",
    )

    native.alias(
        name = "%s/source" % name,
        actual = src + ".cc",
    )

    run_binary(
        name = "%s/generated" % name,
        srcs = [src],
        outs = [
            src + ".h",
            src + ".cc",
        ],
        args = [
            "$(location %s)" % src,
            "-o",
            "$(location %s.h)" % src,
            "-o",
            "$(location %s.cc)" % src,
        ],
        tool = "//:codegen",
    )

    cc_library(
        name = name,
        srcs = [src + ".cc"],
        deps = deps + [":%s/include" % name],
    )

    cc_library(
        name = "%s/include" % name,
        hdrs = [src + ".h"],
    )
```

```python
# demo/BUILD

load("@rules_cc//cc:defs.bzl", "cc_library")
load("@rules_rust//rust:defs.bzl", "rust_binary")
load("//tools/bazel:rust_cxx_bridge.bzl", "rust_cxx_bridge")

rust_binary(
    name = "demo",
    srcs = glob(["src/**/*.rs"]),
    deps = [
        ":blobstore-sys",
        ":bridge",
        "//:cxx",
    ],
)

rust_cxx_bridge(
    name = "bridge",
    src = "src/main.rs",
    deps = [":blobstore-include"],
)

cc_library(
    name = "blobstore-sys",
    srcs = ["src/blobstore.cc"],
    deps = [
        ":blobstore-include",
        ":bridge/include",
    ],
)

cc_library(
    name = "blobstore-include",
    hdrs = ["include/blobstore.h"],
    deps = ["//:core"],
)
```
