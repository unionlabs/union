load("@io_bazel_rules_go//go:def.bzl", "go_library", "go_test")

config_setting(
    name = "use_gmp",
    values = {"define": "BLS_USE_GMP=true"},
)

config_setting(
    name = "use_openssl",
    values = {"define": "BLS_USE_OPENSSL=true"},
)

OPTS = [
    "-DMCL_LLVM_BMI2=0",
    "-DMCL_USE_LLVM=1",
    "-DMCL_VINT_FIXED_BUFFER",
    "-DMCL_SIZEOF_UNIT=8",
    "-DMCL_MAX_BIT_SIZE=384",
    "-DCYBOZU_DONT_USE_EXCEPTION",
    "-DCYBOZU_DONT_USE_STRING",
    "-DBLS_SWAP_G",
    "-DBLS_ETH",
] + select({
    ":use_gmp": [],
    "//conditions:default": [
        "-DMCL_USE_VINT",
    ],
}) + select({
    ":use_openssl": [],
    "//conditions:default": [
        "-DMCL_DONT_USE_OPENSSL",
    ],
})

genrule(
    name = "base64_ll",
    outs = ["src/base64.ll"],  # llvm assembly language file.
    tools = [
        "@herumi_mcl//:src_gen",
    ],
    cmd = "touch func.list && $(location @herumi_mcl//:src_gen) -u 64 -f func.list > $@",
)

genrule(
    name = "base64_o",
    srcs = [
        "src/base64.ll",
    ],
    outs = ["base64.o"],
    cmd = "$(CC) $(CC_FLAGS) -c -o $@ $(location src/base64.ll)",
    toolchains = [
        "@bazel_tools//tools/cpp:current_cc_toolchain",
        "@bazel_tools//tools/cpp:cc_flags",
    ],
)

cc_library(
    name = "lib",
    srcs = [
        "@herumi_mcl//:src/fp.cpp",
        "@herumi_bls//:src/bls_c384_256.cpp",
        "@herumi_bls//:src/bls_c_impl.hpp",
        ":base64_o",
    ],
    deps = ["@herumi_mcl//:bn"],
    includes = [
        "bls/include",
    ],
    hdrs = [
        "bls/include/bls/bls.h",
        "bls/include/bls/bls384_256.h",
        "bls/include/mcl/bn.h",
        "bls/include/mcl/bn_c384_256.h",
        "@herumi_mcl//:include/mcl/curve_type.h",
    ],
    copts = OPTS + [
        "-std=c++03",
    ],
    linkopts = select({
        ":use_gmp": ["-lgmp"],
        "//conditions:default": [],
    }) + select({
        ":use_openssl": [
            "-lssl",
            "-lcrypto",
        ],
        "//conditions:default": [],
    }),
    visibility = ["//visibility:public"],
)

cc_library(
    name = "precompiled",
    srcs = select({
        "@io_bazel_rules_go//go/platform:android_arm": [
            "bls/lib/android/armeabi-v7a/libbls384_256.a",
        ],
        "@io_bazel_rules_go//go/platform:linux_arm64": [
            "bls/lib/linux/arm64/libbls384_256.a",
        ],
        "@io_bazel_rules_go//go/platform:android_arm64": [
            "bls/lib/android/arm64-v8a/libbls384_256.a",
        ],
        "@io_bazel_rules_go//go/platform:darwin_amd64": [
            "bls/lib/darwin/amd64/libbls384_256.a",
        ],
        "@io_bazel_rules_go//go/platform:darwin_arm64": [
            "bls/lib/darwin/arm64/libbls384_256.a",
        ],
        "@io_bazel_rules_go//go/platform:linux_amd64": [
            "bls/lib/linux/amd64/libbls384_256.a",
        ],
        "@io_bazel_rules_go//go/platform:windows_amd64": [
            "bls/lib/windows/amd64/libbls384_256.a",
        ],
        "//conditions:default": [],
    }),
    hdrs = [
        "bls/include/bls/bls.h",
        "bls/include/bls/bls384_256.h",
        "bls/include/mcl/bn.h",
        "bls/include/mcl/bn_c384_256.h",
        "bls/include/mcl/curve_type.h",
    ],
    includes = [
        "bls/include",
    ],
    deprecation = "Using precompiled BLS archives. To build BLS from source with llvm, use --config=llvm.",
)

config_setting(
    name = "llvm_compiler_enabled",
    define_values = {
        "compiler": "llvm",
    },
)

go_library(
    name = "go_default_library",
    importpath = "github.com/herumi/bls-eth-go-binary/bls",
    srcs = [
        "bls/bls.go",
        "bls/eth.go",
        "bls/callback.go",
        "bls/cast.go",
        "bls/mcl.go",
    ],
    cdeps = select({
        ":llvm_compiler_enabled": [":lib"],
        "//conditions:default": [":precompiled"],
    }),
    cgo = True,
    copts = OPTS,
    visibility = [
        # Additional access will require security approval.
        "@prysm//crypto/bls/herumi:__pkg__",
        "@com_github_wealdtech_go_eth2_types_v2//:__pkg__",
    ],
    clinkopts = select({
        "//conditions:default": [],
    }),
)
