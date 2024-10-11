package(default_visibility = ["//visibility:public"])

cc_library(
    name = "fp",
    srcs = [
        "src/fp.cpp",
    ] + select({
        "@io_bazel_rules_go//go/platform:android_arm": [
            "src/asm/arm.s",
        ],
        "@io_bazel_rules_go//go/platform:linux_arm64": [
            "src/asm/aarch64.s",
        ],
        "@io_bazel_rules_go//go/platform:android_arm64": [
            "src/asm/aarch64.s",
        ],
        "@io_bazel_rules_go//go/platform:darwin_amd64": [
            "src/asm/x86-64mac.s",
        ],
        "@io_bazel_rules_go//go/platform:darwin_arm64": [
            "@prysm//third_party/herumi:mcl_darwin_arm64_base64.o",
        ],
        "@io_bazel_rules_go//go/platform:linux_amd64": [
            "src/asm/x86-64.s",
        ],
        "@io_bazel_rules_go//go/platform:windows_amd64": [
            "src/asm/x86-64.s",
        ],
    }),
    includes = [
        "include",
    ],
    hdrs = glob([
        "src/xbyak/*.h",
        "include/cybozu/*.hpp",
    ]) + [
        "include/mcl/op.hpp",
        "include/mcl/gmp_util.hpp",
        "include/mcl/vint.hpp",
        "include/mcl/randgen.hpp",
        "include/mcl/array.hpp",
        "include/mcl/config.hpp",
        "include/mcl/util.hpp",
        "include/mcl/fp_tower.hpp",
        "include/mcl/fp.hpp",
        "include/mcl/conversion.hpp",
        "src/low_func.hpp",
        "src/fp_generator.hpp",
        "src/proto.hpp",
        "src/low_func_llvm.hpp",
    ],
)

cc_library(
    name = "bn",
    srcs = [
        "src/bn_c384_256.cpp",
    ],
    deps = [":fp"],
    hdrs = [
        "include/mcl/bn.h",
        "include/mcl/curve_type.h",
        "include/mcl/impl/bn_c_impl.hpp",
        "include/mcl/bls12_381.hpp",
        "include/mcl/bn_c384_256.h",
        "include/mcl/ec.hpp",
        "include/mcl/mapto_wb19.hpp",
        "include/mcl/ecparam.hpp",
        "include/mcl/lagrange.hpp",
        "include/mcl/bn.hpp",
        "include/mcl/operator.hpp",
        "include/mcl/window_method.hpp",
    ],
    includes = ["include"],
)

# src_gen is a tool to generate some llvm assembly language file.
cc_binary(
    name = "src_gen",
    srcs = [
        "src/gen.cpp",
        "src/llvm_gen.hpp",
    ] + glob([
        "include/cybozu/*.hpp",
        "include/mcl/*.hpp",
    ]),
    includes = ["include"],
)
