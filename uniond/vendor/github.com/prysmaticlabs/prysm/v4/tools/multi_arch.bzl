"a rule transitioning an oci_image to multiple platforms"

def _multiarch_transition(settings, attr):
    return [
        {"//command_line_option:platforms": str(platform)}
        for platform in attr.platforms
    ]

multiarch_transition = transition(
    implementation = _multiarch_transition,
    inputs = [],
    outputs = ["//command_line_option:platforms"],
)

# multiarch_transition impl returns a DefaultInfo provider with the image deps as declared outputs to
# build.
def _impl(ctx):
    return DefaultInfo(files = depset(ctx.files.image))

# The multi_arch rule builds the image for multiple platforms defined in the platforms attribute.
multi_arch = rule(
    implementation = _impl,
    attrs = {
        "image": attr.label(cfg = multiarch_transition),
        "platforms": attr.label_list(),
        "_allowlist_function_transition": attr.label(
            default = "@bazel_tools//tools/allowlists/function_transition_allowlist",
        ),
    },
)
