"""Generated an open-api spec for a grpc api spec.

Reads the the api spec in protobuf format and generate an open-api spec.
Optionally applies settings from the grpc-service configuration.
"""

load("@rules_proto//proto:defs.bzl", "ProtoInfo")

# TODO(yannic): Replace with |proto_common.direct_source_infos| when
# https://github.com/bazelbuild/rules_proto/pull/22 lands.
def _direct_source_infos(proto_info, provided_sources = []):
    """Returns sequence of `ProtoFileInfo` for `proto_info`'s direct sources.

    Files that are both in `proto_info`'s direct sources and in
    `provided_sources` are skipped. This is useful, e.g., for well-known
    protos that are already provided by the Protobuf runtime.

    Args:
      proto_info: An instance of `ProtoInfo`.
      provided_sources: Optional. A sequence of files to ignore.
          Usually, these files are already provided by the
          Protocol Buffer runtime (e.g. Well-Known protos).

    Returns: A sequence of `ProtoFileInfo` containing information about
        `proto_info`'s direct sources.
    """

    source_root = proto_info.proto_source_root
    if "." == source_root:
        return [struct(file = src, import_path = src.path) for src in proto_info.direct_sources]

    offset = len(source_root) + 1  # + '/'.

    infos = []
    for src in proto_info.direct_sources:
        # TODO(yannic): Remove this hack when we drop support for Bazel < 1.0.
        local_offset = offset
        if src.root.path and not source_root.startswith(src.root.path):
            # Before Bazel 1.0, `proto_source_root` wasn't guaranteed to be a
            # prefix of `src.path`. This could happend, e.g., if `file` was
            # generated (https://github.com/bazelbuild/bazel/issues/9215).
            local_offset += len(src.root.path) + 1  # + '/'.
        infos.append(struct(file = src, import_path = src.path[local_offset:]))

    return infos

def _run_proto_gen_swagger(
        actions,
        proto_info,
        target_name,
        transitive_proto_srcs,
        protoc,
        protoc_gen_swagger,
        grpc_api_configuration,
        single_output,
        json_names_for_fields,
        fqn_for_swagger_name):
    args = actions.args()

    args.add("--plugin", "protoc-gen-swagger=%s" % protoc_gen_swagger.path)

    args.add("--swagger_opt", "logtostderr=true")
    args.add("--swagger_opt", "allow_repeated_fields_in_body=true")

    extra_inputs = []
    if grpc_api_configuration:
        extra_inputs.append(grpc_api_configuration)
        args.add("--swagger_opt", "grpc_api_configuration=%s" % grpc_api_configuration.path)

    if json_names_for_fields:
        args.add("--swagger_opt", "json_names_for_fields=true")

    if fqn_for_swagger_name:
        args.add("--swagger_opt", "fqn_for_swagger_name=true")

    proto_file_infos = _direct_source_infos(proto_info)

    # TODO(yannic): Use |proto_info.transitive_descriptor_sets| when
    # https://github.com/bazelbuild/bazel/issues/9337 is fixed.
    args.add_all(proto_info.transitive_proto_path, format_each = "--proto_path=%s")

    if single_output:
        args.add("--swagger_opt", "allow_merge=true")
        args.add("--swagger_opt", "merge_file_name=%s" % target_name)

        swagger_file = actions.declare_file("%s.swagger.json" % target_name)
        args.add("--swagger_out", swagger_file.dirname)

        args.add_all([f.import_path for f in proto_file_infos])

        actions.run(
            executable = protoc,
            tools = [protoc_gen_swagger],
            inputs = depset(
                direct = extra_inputs,
                transitive = [transitive_proto_srcs],
            ),
            outputs = [swagger_file],
            arguments = [args],
        )

        return [swagger_file]

    # TODO(yannic): We may be able to generate all files in a single action,
    # but that will change at least the semantics of `use_go_template.proto`.
    swagger_files = []
    for proto_file_info in proto_file_infos:
        # TODO(yannic): This probably doesn't work as expected: we only add this
        # option after we have seen it, so `.proto` sources that happen to be
        # in the list of `.proto` files before `use_go_template.proto` will be
        # compiled without this option, and all sources that get compiled after
        # `use_go_template.proto` will have this option on.
        if proto_file_info.file.basename == "use_go_template.proto":
            args.add("--swagger_opt", "use_go_templates=true")

        file_name = "%s.swagger.json" % proto_file_info.import_path[:-len(".proto")]
        swagger_file = actions.declare_file(
            "_virtual_imports/%s/%s" % (target_name, file_name),
        )

        file_args = actions.args()

        offset = len(file_name) + 1  # + '/'.
        file_args.add("--swagger_out", swagger_file.path[:-offset])

        file_args.add(proto_file_info.import_path)

        actions.run(
            executable = protoc,
            tools = [protoc_gen_swagger],
            inputs = depset(
                direct = extra_inputs,
                transitive = [transitive_proto_srcs],
            ),
            outputs = [swagger_file],
            arguments = [args, file_args],
        )
        swagger_files.append(swagger_file)

    return swagger_files

def _proto_gen_swagger_impl(ctx):
    proto = ctx.attr.proto[ProtoInfo]
    return [
        DefaultInfo(
            files = depset(
                _run_proto_gen_swagger(
                    actions = ctx.actions,
                    proto_info = proto,
                    target_name = ctx.attr.name,
                    transitive_proto_srcs = depset(
                        direct = ctx.files._well_known_protos,
                        transitive = [proto.transitive_sources],
                    ),
                    protoc = ctx.executable._protoc,
                    protoc_gen_swagger = ctx.executable._protoc_gen_swagger,
                    grpc_api_configuration = ctx.file.grpc_api_configuration,
                    single_output = ctx.attr.single_output,
                    json_names_for_fields = ctx.attr.json_names_for_fields,
                    fqn_for_swagger_name = ctx.attr.fqn_for_swagger_name,
                ),
            ),
        ),
    ]

protoc_gen_swagger = rule(
    attrs = {
        "proto": attr.label(
            mandatory = True,
            providers = [ProtoInfo],
        ),
        "grpc_api_configuration": attr.label(
            allow_single_file = True,
            mandatory = False,
        ),
        "single_output": attr.bool(
            default = False,
            mandatory = False,
        ),
        "json_names_for_fields": attr.bool(
            default = False,
            mandatory = False,
        ),
        "fqn_for_swagger_name": attr.bool(
            default = False,
            mandatory = False,
        ),
        "_protoc": attr.label(
            default = "@com_google_protobuf//:protoc",
            executable = True,
            cfg = "host",
        ),
        "_well_known_protos": attr.label(
            default = "@com_google_protobuf//:well_known_protos",
            allow_files = True,
        ),
        "_protoc_gen_swagger": attr.label(
            default = Label("//protoc-gen-swagger:protoc-gen-swagger"),
            executable = True,
            cfg = "host",
        ),
    },
    implementation = _proto_gen_swagger_impl,
)
