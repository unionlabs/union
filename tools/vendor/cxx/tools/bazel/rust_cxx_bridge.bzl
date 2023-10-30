# buildifier: disable=module-docstring
load("@bazel_skylib//rules:run_binary.bzl", "run_binary")
load("@rules_cc//cc:defs.bzl", "cc_library")

def rust_cxx_bridge(name, src, deps = []):
    """A macro defining a cxx bridge library

    Args:
        name (string): The name of the new target
        src (string): The rust source file to generate a bridge for
        deps (list, optional): A list of dependencies for the underlying cc_library. Defaults to [].
    """
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
        tool = "@cxx.rs//:codegen",
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
