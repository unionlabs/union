load("@rules_cc//cc:defs.bzl", "cc_library")
load("@rules_rust//rust:defs.bzl", "rust_binary", "rust_library", "rust_proc_macro")

rust_library(
    name = "cxx",
    srcs = glob(["src/**/*.rs"]),
    crate_features = [
        "alloc",
        "std",
    ],
    edition = "2018",
    proc_macro_deps = [
        ":cxxbridge-macro",
    ],
    visibility = ["//visibility:public"],
    deps = [":core-lib"],
)

rust_binary(
    name = "codegen",
    srcs = glob(["gen/cmd/src/**/*.rs"]),
    data = ["gen/cmd/src/gen/include/cxx.h"],
    edition = "2018",
    visibility = ["//visibility:public"],
    deps = [
        "//third-party:clap",
        "//third-party:codespan-reporting",
        "//third-party:proc-macro2",
        "//third-party:quote",
        "//third-party:syn",
    ],
)

cc_library(
    name = "core",
    hdrs = ["include/cxx.h"],
    include_prefix = "rust",
    strip_include_prefix = "include",
    visibility = ["//visibility:public"],
)

cc_library(
    name = "core-lib",
    srcs = ["src/cxx.cc"],
    hdrs = ["include/cxx.h"],
)

rust_proc_macro(
    name = "cxxbridge-macro",
    srcs = glob(["macro/src/**/*.rs"]),
    edition = "2018",
    deps = [
        "//third-party:proc-macro2",
        "//third-party:quote",
        "//third-party:syn",
    ],
)

rust_library(
    name = "build",
    srcs = glob(["gen/build/src/**/*.rs"]),
    data = ["gen/build/src/gen/include/cxx.h"],
    edition = "2018",
    visibility = ["//visibility:public"],
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
    srcs = glob(["gen/lib/src/**/*.rs"]),
    data = ["gen/lib/src/gen/include/cxx.h"],
    edition = "2018",
    visibility = ["//visibility:public"],
    deps = [
        "//third-party:cc",
        "//third-party:codespan-reporting",
        "//third-party:proc-macro2",
        "//third-party:quote",
        "//third-party:syn",
    ],
)
