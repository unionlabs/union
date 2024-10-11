package(default_visibility = ["//visibility:public"])

load(":cc_toolchain_config_osx.bzl", "osx_cc_toolchain_config")
load(":cc_toolchain_config_linux_arm64.bzl", "arm64_cc_toolchain_config")
load(":cc_toolchain_config_windows.bzl", "windows_cc_toolchain_config")

cc_toolchain_suite(
    name = "multiarch_toolchain",
    toolchains = {
        "k8|osxcross": ":cc-clang-osx-amd64",
        "aarch64|osxcross": ":cc-clang-osx-arm64",
        "k8|clang": "cc-clang-amd64",
        "aarch64|clang": ":cc-clang-arm64",
        "k8": "cc-clang-amd64",
        "aarch64": ":cc-clang-arm64",
        "k8|mingw-w64": ":cc-mingw-amd64",
    },
    tags = ["manual"],
)

cc_toolchain_suite(
    name = "hostonly_toolchain",
    toolchains = {
        "k8": "cc-clang-amd64",
    },
    tags = ["manual"],
)

filegroup(
    name = "empty",
    srcs = [],
    tags = ["manual"],
)

config_setting(
    name = "osx_amd64",
    constraint_values = [
        "@platforms//os:osx",
        "@platforms//cpu:x86_64",
    ],
    tags = ["manual"],
)

config_setting(
    name = "osx_arm64",
    constraint_values = [
        "@platforms//os:osx",
        "@platforms//cpu:arm64",
    ],
    tags = ["manual"],
)

config_setting(
    name = "linux_arm64",
    constraint_values = [
        "@platforms//os:linux",
        "@platforms//cpu:aarch64",
    ],
    tags = ["manual"],
)

config_setting(
    name = "linux_amd64",
    constraint_values = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
    tags = ["manual"],
)

config_setting(
    name = "windows_amd64",
    constraint_values = [
        "@platforms//os:windows",
        "@platforms//cpu:x86_64",
    ],
    tags = ["manual"],
)

arm64_cc_toolchain_config(
    name = "local-arm64",
    target = "aarch64-linux-gnu",
    tags = ["manual"],
)

arm64_cc_toolchain_config(
    name = "local-amd64",
    target = "x86_64-unknown-linux-gnu",
    tags = ["manual"],
)

osx_cc_toolchain_config(
    name = "local-osxcross-amd64",
    target = "darwin_x86_64",
    target_cpu = "x86_64",
    tags = ["manual"],
)

osx_cc_toolchain_config(
    name = "local-osxcross-arm64",
    target = "darwin_aarch64",
    target_cpu = "aarch64",
    tags = ["manual"],
)

windows_cc_toolchain_config(
    name = "local-windows",
    target = "x86_64-w64",
    tags = ["manual"],
)

cc_toolchain(
    name = "cc-mingw-amd64",
    all_files = ":empty",
    ar_files = ":empty",
    as_files = ":empty",
    compiler_files = ":empty",
    dwp_files = ":empty",
    linker_files = ":empty",
    objcopy_files = ":empty",
    strip_files = ":empty",
    supports_param_files = 0,
    toolchain_config = ":local-windows",
    tags = ["manual"],
)

cc_toolchain(
    name = "cc-clang-arm64",
    all_files = ":empty",
    compiler_files = ":empty",
    dwp_files = ":empty",
    linker_files = ":empty",
    objcopy_files = ":empty",
    strip_files = ":empty",
    supports_param_files = 1,
    toolchain_config = ":local-arm64",
    tags = ["manual"],
)

cc_toolchain(
    name = "cc-clang-osx-amd64",
    all_files = ":empty",
    compiler_files = ":empty",
    dwp_files = ":empty",
    linker_files = ":empty",
    objcopy_files = ":empty",
    strip_files = ":empty",
    supports_param_files = 1,
    toolchain_config = ":local-osxcross-amd64",
    tags = ["manual"],
)

cc_toolchain(
    name = "cc-clang-osx-arm64",
    all_files = ":empty",
    compiler_files = ":empty",
    dwp_files = ":empty",
    linker_files = ":empty",
    objcopy_files = ":empty",
    strip_files = ":empty",
    supports_param_files = 1,
    toolchain_config = ":local-osxcross-arm64",
    tags = ["manual"],
)

cc_toolchain(
    name = "cc-clang-amd64",
    all_files = ":empty",
    compiler_files = ":empty",
    dwp_files = ":empty",
    linker_files = ":empty",
    objcopy_files = ":empty",
    strip_files = ":empty",
    supports_param_files = 1,
    toolchain_config = ":local-amd64",
    tags = ["manual"],
)

toolchain(
    name = "cc-toolchain-multiarch",
    exec_compatible_with = [
        "@platforms//os:linux",
        "@platforms//cpu:x86_64",
    ],
    target_compatible_with = [],
    toolchain = select({
        ":linux_arm64": ":cc-clang-arm64",
        ":linux_amd64": ":cc-clang-amd64",
        ":osx_amd64": ":cc-clang-osx-amd64",
        ":osx_arm64": ":cc-clang-osx-arm64",
        ":windows_amd64": ":cc-mingw-amd64",
    }),
    toolchain_type = "@bazel_tools//tools/cpp:toolchain_type",
    tags = ["manual"],
)
