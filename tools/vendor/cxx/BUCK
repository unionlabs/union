rust_library(
    name = "cxx",
    srcs = glob(["src/**/*.rs"]),
    edition = "2018",
    features = [
        "alloc",
        "std",
    ],
    visibility = ["PUBLIC"],
    deps = [
        ":core",
        ":macro",
    ],
)

rust_binary(
    name = "codegen",
    srcs = glob(["gen/cmd/src/**/*.rs"]) + [
        "gen/cmd/src/gen",
        "gen/cmd/src/syntax",
    ],
    crate = "cxxbridge",
    edition = "2018",
    visibility = ["PUBLIC"],
    deps = [
        "//third-party:clap",
        "//third-party:codespan-reporting",
        "//third-party:proc-macro2",
        "//third-party:quote",
        "//third-party:syn",
    ],
)

cxx_library(
    name = "core",
    srcs = ["src/cxx.cc"],
    exported_headers = {
        "cxx.h": "include/cxx.h",
    },
    exported_linker_flags = ["-lstdc++"],
    header_namespace = "rust",
    visibility = ["PUBLIC"],
)

rust_library(
    name = "macro",
    srcs = glob(["macro/src/**/*.rs"]) + ["macro/src/syntax"],
    crate = "cxxbridge_macro",
    edition = "2018",
    proc_macro = True,
    deps = [
        "//third-party:proc-macro2",
        "//third-party:quote",
        "//third-party:syn",
    ],
)

rust_library(
    name = "build",
    srcs = glob(["gen/build/src/**/*.rs"]) + [
        "gen/build/src/gen",
        "gen/build/src/syntax",
    ],
    edition = "2018",
    visibility = ["PUBLIC"],
    deps = [
        "//third-party:cc",
        "//third-party:codespan-reporting",
        "//third-party:once_cell",
        "//third-party:proc-macro2",
        "//third-party:quote",
        "//third-party:scratch",
        "//third-party:syn",
    ],
)

rust_library(
    name = "lib",
    srcs = glob(["gen/lib/src/**/*.rs"]) + [
        "gen/lib/src/gen",
        "gen/lib/src/syntax",
    ],
    edition = "2018",
    visibility = ["PUBLIC"],
    deps = [
        "//third-party:cc",
        "//third-party:codespan-reporting",
        "//third-party:proc-macro2",
        "//third-party:quote",
        "//third-party:syn",
    ],
)
