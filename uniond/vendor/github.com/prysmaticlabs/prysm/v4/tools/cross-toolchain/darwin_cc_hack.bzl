load("@bazel_tools//tools/cpp:unix_cc_configure.bzl", "configure_unix_toolchain")
load(
    "@bazel_tools//tools/cpp:lib_cc_configure.bzl",
    "get_cpu_value",
    "resolve_labels",
)

"""
This file is a copy of https://github.com/bazelbuild/bazel/blob/master/tools/cpp/cc_configure.bzl
with some minor changes. The original file is licensed under Apache 2.0 license. The gist of this
is that we want darwin to register the local toolchain and disregard the environment variable of
BAZEL_DO_NOT_DETECT_CPP_TOOLCHAIN. We must support a local toolchain for darwin until
hermetic_cc_toolchain supports darwin's sysroot in a hermetic way.
"""

def cc_autoconf_toolchains_impl(repository_ctx):
    """Generate BUILD file with 'toolchain' targets for the local host C++ toolchain.

    Args:
      repository_ctx: repository context
    """

    cpu_value = get_cpu_value(repository_ctx)

    if cpu_value.startswith("darwin"):
        paths = resolve_labels(repository_ctx, [
            "@bazel_tools//tools/cpp:BUILD.toolchains.tpl",
        ])
        repository_ctx.template(
            "BUILD",
            paths["@bazel_tools//tools/cpp:BUILD.toolchains.tpl"],
            {"%{name}": get_cpu_value(repository_ctx)},
        )
    else:
        repository_ctx.file("BUILD", "# C++ toolchain autoconfiguration was disabled by BAZEL_DO_NOT_DETECT_CPP_TOOLCHAIN env variable.")

def cc_autoconf_impl(repository_ctx, overriden_tools = dict()):
    """Generate BUILD file with 'cc_toolchain' targets for the local host C++ toolchain.

    Args:
       repository_ctx: repository context
       overriden_tools: dict of tool paths to use instead of autoconfigured tools
    """
    cpu_value = get_cpu_value(repository_ctx)

    if cpu_value.startswith("darwin"):
        print("Configuring local C++ toolchain for Darwin. This is non-hermetic and builds may " +
              "not be reproducible. Consider building on linux for a hermetic build.")
        configure_unix_toolchain(repository_ctx, cpu_value, overriden_tools)
    else:
        paths = resolve_labels(repository_ctx, [
            "@bazel_tools//tools/cpp:BUILD.empty.tpl",
            "@bazel_tools//tools/cpp:empty_cc_toolchain_config.bzl",
        ])
        repository_ctx.symlink(paths["@bazel_tools//tools/cpp:empty_cc_toolchain_config.bzl"], "cc_toolchain_config.bzl")
        repository_ctx.template("BUILD", paths["@bazel_tools//tools/cpp:BUILD.empty.tpl"], {
            "%{cpu}": get_cpu_value(repository_ctx),
        })

cc_autoconf_toolchains = repository_rule(
    environ = [
        "BAZEL_DO_NOT_DETECT_CPP_TOOLCHAIN",
    ],
    implementation = cc_autoconf_toolchains_impl,
    configure = True,
)

cc_autoconf = repository_rule(
    environ = [
        "ABI_LIBC_VERSION",
        "ABI_VERSION",
        "BAZEL_COMPILER",
        "BAZEL_HOST_SYSTEM",
        "BAZEL_CONLYOPTS",
        "BAZEL_CXXOPTS",
        "BAZEL_LINKOPTS",
        "BAZEL_LINKLIBS",
        "BAZEL_LLVM_COV",
        "BAZEL_LLVM_PROFDATA",
        "BAZEL_PYTHON",
        "BAZEL_SH",
        "BAZEL_TARGET_CPU",
        "BAZEL_TARGET_LIBC",
        "BAZEL_TARGET_SYSTEM",
        "BAZEL_DO_NOT_DETECT_CPP_TOOLCHAIN",
        "BAZEL_USE_LLVM_NATIVE_COVERAGE",
        "BAZEL_LLVM",
        "BAZEL_IGNORE_SYSTEM_HEADERS_VERSIONS",
        "USE_CLANG_CL",
        "CC",
        "CC_CONFIGURE_DEBUG",
        "CC_TOOLCHAIN_NAME",
        "CPLUS_INCLUDE_PATH",
        "DEVELOPER_DIR",
        "GCOV",
        "LIBTOOL",
        "HOMEBREW_RUBY_PATH",
        "SYSTEMROOT",
        "USER",
    ],
    implementation = cc_autoconf_impl,
    configure = True,
)

def configure_nonhermetic_darwin():
    """A C++ configuration rules that generate the crosstool file."""
    cc_autoconf_toolchains(name = "local_config_cc_toolchains")
    cc_autoconf(name = "local_config_cc")
    native.bind(name = "cc_toolchain", actual = "@local_config_cc//:toolchain")
    native.register_toolchains(
        # Use register_toolchain's target pattern expansion to register all toolchains in the package.
        "@local_config_cc_toolchains//:all",
    )
