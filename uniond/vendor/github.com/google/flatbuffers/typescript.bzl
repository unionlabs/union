"""
Rules for building typescript flatbuffers with Bazel.
"""

load("@aspect_rules_js//js:defs.bzl", "js_library")
load(":build_defs.bzl", "flatbuffer_library_public")

DEFAULT_FLATC_TS_ARGS = [
    "--gen-object-api",
    "--gen-mutable",
    "--reflect-names",
    "--gen-name-strings",
    "--ts-flat-files",
    "--keep-prefix",
]

def flatbuffer_ts_library(
        name,
        srcs,
        compatible_with = None,
        target_compatible_with = None,
        deps = [],
        include_paths = None,
        flatc_args = DEFAULT_FLATC_TS_ARGS,
        visibility = None,
        restricted_to = None,
        gen_reflections = False):
    """Generates a ts_library rule for a given flatbuffer definition.

    Args:
      name: Name of the generated ts_library rule.
      srcs: Source .fbs file(s).
      deps: Other flatbuffer_ts_library's to depend on. Note that currently
            you must specify all your transitive dependencies manually.
      include_paths: Optional, list of paths the includes files can be found in.
      flatc_args: Optional list of additional arguments to pass to flatc
          (e.g. --gen-mutable).
      visibility: The visibility of the generated cc_library. By default, use the
          default visibility of the project.
      compatible_with: Optional, The list of environments this rule can be built
        for, in addition to default-supported environments.
      restricted_to: Optional, The list of environments this rule can be built
        for, instead of default-supported environments.
      target_compatible_with: Optional, The list of target platform constraints
        to use.
      gen_reflections: Optional, if true this will generate the flatbuffer
        reflection binaries for the schemas.
    """
    srcs_lib = "%s_srcs" % (name)
    out_base = [s.replace(".fbs", "").split("/")[-1].split(":")[-1] for s in srcs]

    if len(srcs) != 1:
        fail("flatbuffer_ts_library only supports one .fbs file per target currently.")

    outs = ["%s_generated.cjs" % s for s in out_base]
    includes = [d + "_includes" for d in deps]
    reflection_name = "%s_reflection" % name if gen_reflections else ""
    flatbuffer_library_public(
        name = srcs_lib,
        srcs = srcs,
        outs = outs,
        language_flag = "--ts",
        includes = includes,
        include_paths = include_paths,
        extra_env = "ESBUILD_BIN=$(ESBUILD_BIN)",
        flatc_args = flatc_args + ["--filename-suffix _generated"],
        compatible_with = compatible_with,
        restricted_to = restricted_to,
        reflection_name = reflection_name,
        reflection_visibility = visibility,
        target_compatible_with = target_compatible_with,
        flatc_path = "@com_github_google_flatbuffers//ts:compile_flat_file",
        toolchains = ["@aspect_rules_esbuild//esbuild:resolved_toolchain"],
        tools = ["@aspect_rules_esbuild//esbuild:resolved_toolchain"],
    )
    js_library(
        name = name,
        visibility = visibility,
        compatible_with = compatible_with,
        restricted_to = restricted_to,
        target_compatible_with = target_compatible_with,
        srcs = outs,
    )
    native.filegroup(
        name = "%s_includes" % (name),
        srcs = srcs + includes,
        compatible_with = compatible_with,
        restricted_to = restricted_to,
        visibility = visibility,
    )
