load("@rules_cc//cc:defs.bzl", "cc_library")
load("@rules_rust//rust:defs.bzl", "rust_library", "rust_test")
load("//tools/bazel:rust_cxx_bridge.bzl", "rust_cxx_bridge")

rust_test(
    name = "test",
    size = "small",
    srcs = ["test.rs"],
    edition = "2018",
    deps = [
        ":cxx_test_suite",
        "//:cxx",
    ],
)

rust_library(
    name = "cxx_test_suite",
    srcs = [
        "ffi/cast.rs",
        "ffi/lib.rs",
        "ffi/module.rs",
    ],
    edition = "2018",
    deps = [
        ":impl",
        "//:cxx",
    ],
)

cc_library(
    name = "impl",
    srcs = [
        "ffi/tests.cc",
        ":bridge/source",
        ":module/source",
    ],
    hdrs = ["ffi/tests.h"],
    deps = [
        ":bridge/include",
        ":module/include",
        "//:core",
    ],
)

rust_cxx_bridge(
    name = "bridge",
    src = "ffi/lib.rs",
    deps = [":impl"],
)

rust_cxx_bridge(
    name = "module",
    src = "ffi/module.rs",
    deps = [":impl"],
)
