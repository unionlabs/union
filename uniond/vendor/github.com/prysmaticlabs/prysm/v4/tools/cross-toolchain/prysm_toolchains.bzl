def _pryms_toolchains_impl(ctx):
    ctx.template(
        "BUILD.bazel",
        ctx.attr._build_tpl,
    )
    ctx.template(
        "cc_toolchain_config_linux_arm64.bzl",
        ctx.attr._cc_toolchain_config_linux_arm_tpl,
    )
    ctx.template(
        "cc_toolchain_config_osx.bzl",
        ctx.attr._cc_toolchain_config_osx_tpl,
    )
    ctx.template(
        "cc_toolchain_config_windows.bzl",
        ctx.attr._cc_toolchain_config_windows_tpl,
    )

prysm_toolchains = repository_rule(
    implementation = _pryms_toolchains_impl,
    attrs = {
        "_build_tpl": attr.label(
            default = "@prysm//tools/cross-toolchain:cc_toolchain.BUILD.bazel.tpl",
        ),
        "_cc_toolchain_config_linux_arm_tpl": attr.label(
            default = "@prysm//tools/cross-toolchain:cc_toolchain_config_linux_arm64.bzl.tpl",
        ),
        "_cc_toolchain_config_osx_tpl": attr.label(
            default = "@prysm//tools/cross-toolchain:cc_toolchain_config_osx.bzl.tpl",
        ),
        "_cc_toolchain_config_windows_tpl": attr.label(
            default = "@prysm//tools/cross-toolchain:cc_toolchain_config_windows.bzl.tpl",
        ),
    },
    doc = "Configures Prysm custom toolchains for cross compilation and remote build execution.",
)

def configure_prysm_toolchains():
    prysm_toolchains(name = "prysm_toolchains")
    native.register_toolchains("@prysm_toolchains//:cc-toolchain-multiarch")
