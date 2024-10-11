"""
Easily add exclusions to nogo_config.json. The tool also allows for hand written entries in the 
input file to be preserved.

Example usage:

nogo_config_exclude(
    name = "nogo_config_with_excludes",
    input = "nogo_config.json",
    exclude_files = [
        "third_party/.*",
        ".*_test\\.go",
    ],
    checks = [
        "ifferr",
        "sa0000",
        "neverbuild",
    ],
)

nogo(
    name = "nogo",
    config = ":nogo_config_with_excludes",
    ...
)
"""

def _nogo_config_exclude_impl(ctx):
    input_file = ctx.attr.input.files.to_list()[0].path
    output_file = ctx.outputs.out.path
    exclude_files = ctx.attr.exclude_files
    checks = ctx.attr.checks

    ctx.actions.run(
        executable = ctx.executable.tool,
        inputs = ctx.attr.input.files,
        outputs = [ctx.outputs.out],
        arguments = [
            "--input=%s" % input_file,
            "--output=%s" % output_file,
            "--checks=%s" % ",".join(checks),
            "--exclude_files=%s" % ",".join(exclude_files),
            "--silent",
        ],
        progress_message = "Generating nogo_config.json with exclusions.",
    )

nogo_config_exclude = rule(
    implementation = _nogo_config_exclude_impl,
    attrs = {
        "input": attr.label(
            mandatory = True,
            allow_single_file = True,
            doc = "The input nogo_config.json file.",
        ),
        "exclude_files": attr.string_list(
            mandatory = False,
            doc = "A list of regexes to exclude from the input file.",
        ),
        "checks": attr.string_list(
            mandatory = True,
            doc = "A list of checks to exclude.",
        ),
        "tool": attr.label(
            executable = True,
            cfg = "exec",
            default = Label("@prysm//tools/nogo_config:nogo_config"),
            doc = "The nogo config exclusion tool.",
        ),
    },
    doc = "Generate a nogo_config.json file with exclusions.",
    outputs = {
        "out": "nogo_config.generated.json",
    },
)
