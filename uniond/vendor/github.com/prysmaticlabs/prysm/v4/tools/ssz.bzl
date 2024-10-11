"""
A rule that uses the generated pb.go files from a go_proto_library target to generate SSZ marshal
and unmarshal functions as pointer receivers on the specified objects. To use this rule, provide a
go_proto_library target and specify the structs to generate methods in the "objs" field. Lastly,
include your new target as a source for the go_library that embeds the go_proto_library.
Example:
go_proto_library(
  name = "example_go_proto",
   ...
)
ssz_gen_marshal(
  name = "ssz_generated_sources",
  go_proto = ":example_go_proto",
  objs = [ # omit this field to generate for all structs in the package.
    "AddressBook",
    "Person",
  ],
)
go_library(
  name = "go_default_library",
  srcs = [":ssz_generated_sources"],
  embed = [":example_go_proto"],
  deps = SSZ_DEPS,
)
"""

load(
    "@io_bazel_rules_go//go:def.bzl",
    "GoLibrary",
    "GoSource",
)

def _ssz_go_proto_library_impl(ctx):
    if ctx.var.get("ssz"):
        fail("--define=ssz=<value> is no longer supported, please use --//proto:network=<value>")

    if ctx.attr.go_proto != None:
        go_proto = ctx.attr.go_proto
        input_files = go_proto[OutputGroupInfo].go_generated_srcs.to_list()
        package_path = input_files[0].dirname
    elif hasattr(ctx.attr, "srcs") and len(ctx.attr.srcs) > 0:
        package_path = ctx.attr.srcs[0].files.to_list()[0].dirname
        input_files = ctx.attr.srcs[0].files.to_list()
    else:
        fail("Must have go_proto or srcs")

    # Run the tool.
    output = ctx.outputs.out
    args = [
        "--output=%s" % output.path,
        "--path=%s" % package_path,
    ]
    if hasattr(ctx.attr, "includes") and len(ctx.attr.includes) > 0:
        incs = []
        for include in ctx.attr.includes:
            incs.append(include[GoSource].srcs[0].dirname)
            input_files += include[GoSource].srcs
        args.append("--include=%s" % ",".join(incs))

    if len(ctx.attr.objs) > 0:
        args.append("--objs=%s" % ",".join(ctx.attr.objs))

    ctx.actions.run(
        executable = ctx.executable.sszgen,
        progress_message = "Generating ssz marshal and unmarshal functions",
        inputs = input_files,
        arguments = args,
        outputs = [output],
    )

ssz_gen_marshal = rule(
    implementation = _ssz_go_proto_library_impl,
    attrs = {
        "srcs": attr.label_list(allow_files = True),
        "go_proto": attr.label(providers = [GoLibrary]),
        "sszgen": attr.label(
            default = Label("@com_github_prysmaticlabs_fastssz//sszgen:sszgen"),
            executable = True,
            cfg = "exec",
        ),
        "objs": attr.string_list(),
        "includes": attr.label_list(providers = [GoLibrary]),
    },
    outputs = {"out": "generated.ssz.go"},
)

SSZ_DEPS = ["@com_github_prysmaticlabs_fastssz//:go_default_library"]
