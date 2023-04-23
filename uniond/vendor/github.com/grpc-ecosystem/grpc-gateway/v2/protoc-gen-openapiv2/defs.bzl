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
        return [struct(file = src, import_path = src.path) for src in proto_info.check_deps_sources.to_list()]

    offset = len(source_root) + 1  # + '/'.

    infos = []
    for src in proto_info.check_deps_sources.to_list():
        # TODO(yannic): Remove this hack when we drop support for Bazel < 1.0.
        local_offset = offset
        if src.root.path and not source_root.startswith(src.root.path):
            # Before Bazel 1.0, `proto_source_root` wasn't guaranteed to be a
            # prefix of `src.path`. This could happend, e.g., if `file` was
            # generated (https://github.com/bazelbuild/bazel/issues/9215).
            local_offset += len(src.root.path) + 1  # + '/'.
        infos.append(struct(file = src, import_path = src.path[local_offset:]))

    return infos

def _run_proto_gen_openapi(
        actions,
        proto_info,
        target_name,
        transitive_proto_srcs,
        protoc,
        protoc_gen_openapiv2,
        single_output,
        allow_delete_body,
        grpc_api_configuration,
        json_names_for_fields,
        repeated_path_param_separator,
        include_package_in_tags,
        fqn_for_openapi_name,
        openapi_naming_strategy,
        use_go_templates,
        disable_default_errors,
        disable_service_tags,
        enums_as_ints,
        omit_enum_default_value,
        output_format,
        simple_operation_ids,
        proto3_optional_nullable,
        openapi_configuration,
        generate_unbound_methods,
        visibility_restriction_selectors,
        use_allof_for_refs):
    args = actions.args()

    args.add("--plugin", "protoc-gen-openapiv2=%s" % protoc_gen_openapiv2.path)

    args.add("--openapiv2_opt", "logtostderr=true")

    extra_inputs = []
    if grpc_api_configuration:
        extra_inputs.append(grpc_api_configuration)
        args.add("--openapiv2_opt", "grpc_api_configuration=%s" % grpc_api_configuration.path)

    if openapi_configuration:
        extra_inputs.append(openapi_configuration)
        args.add("--openapiv2_opt", "openapi_configuration=%s" % openapi_configuration.path)

    if not json_names_for_fields:
        args.add("--openapiv2_opt", "json_names_for_fields=false")

    if fqn_for_openapi_name:
        args.add("--openapiv2_opt", "fqn_for_openapi_name=true")

    if openapi_naming_strategy:
        args.add("--openapiv2_opt", "openapi_naming_strategy=%s" % openapi_naming_strategy)

    if generate_unbound_methods:
        args.add("--openapiv2_opt", "generate_unbound_methods=true")

    if simple_operation_ids:
        args.add("--openapiv2_opt", "simple_operation_ids=true")

    if allow_delete_body:
        args.add("--openapiv2_opt", "allow_delete_body=true")

    if include_package_in_tags:
        args.add("--openapiv2_opt", "include_package_in_tags=true")

    if use_go_templates:
        args.add("--openapiv2_opt", "use_go_templates=true")

    if disable_default_errors:
        args.add("--openapiv2_opt", "disable_default_errors=true")

    if disable_service_tags:
        args.add("--openapiv2_opt", "disable_service_tags=true")

    if enums_as_ints:
        args.add("--openapiv2_opt", "enums_as_ints=true")

    if omit_enum_default_value:
        args.add("--openapiv2_opt", "omit_enum_default_value=true")

    if output_format:
        args.add("--openapiv2_opt", "output_format=%s" % output_format)

    if proto3_optional_nullable:
        args.add("--openapiv2_opt", "proto3_optional_nullable=true")

    for visibility_restriction_selector in visibility_restriction_selectors:
        args.add("--openapiv2_opt", "visibility_restriction_selectors=%s" % visibility_restriction_selector)

    if use_allof_for_refs:
        args.add("--openapiv2_opt", "use_allof_for_refs=true")

    args.add("--openapiv2_opt", "repeated_path_param_separator=%s" % repeated_path_param_separator)

    proto_file_infos = _direct_source_infos(proto_info)

    # TODO(yannic): Use |proto_info.transitive_descriptor_sets| when
    # https://github.com/bazelbuild/bazel/issues/9337 is fixed.
    args.add_all(proto_info.transitive_proto_path, format_each = "--proto_path=%s")

    if single_output:
        args.add("--openapiv2_opt", "allow_merge=true")
        args.add("--openapiv2_opt", "merge_file_name=%s" % target_name)

        openapi_file = actions.declare_file("%s.swagger.json" % target_name)
        args.add("--openapiv2_out", openapi_file.dirname)

        args.add_all([f.import_path for f in proto_file_infos])

        actions.run(
            executable = protoc,
            tools = [protoc_gen_openapiv2],
            inputs = depset(
                direct = extra_inputs,
                transitive = [transitive_proto_srcs],
            ),
            outputs = [openapi_file],
            arguments = [args],
        )

        return [openapi_file]

    # TODO(yannic): We may be able to generate all files in a single action,
    # but that will change at least the semantics of `use_go_template.proto`.
    openapi_files = []
    for proto_file_info in proto_file_infos:
        # TODO(yannic): This probably doesn't work as expected: we only add this
        # option after we have seen it, so `.proto` sources that happen to be
        # in the list of `.proto` files before `use_go_template.proto` will be
        # compiled without this option, and all sources that get compiled after
        # `use_go_template.proto` will have this option on.
        if proto_file_info.file.basename == "use_go_template.proto":
            args.add("--openapiv2_opt", "use_go_templates=true")

        file_name = "%s.swagger.json" % proto_file_info.import_path[:-len(".proto")]
        openapi_file = actions.declare_file(
            "_virtual_imports/%s/%s" % (target_name, file_name),
        )

        file_args = actions.args()

        offset = len(file_name) + 1  # + '/'.
        file_args.add("--openapiv2_out", openapi_file.path[:-offset])

        file_args.add(proto_file_info.import_path)

        actions.run(
            executable = protoc,
            tools = [protoc_gen_openapiv2],
            inputs = depset(
                direct = extra_inputs,
                transitive = [transitive_proto_srcs],
            ),
            outputs = [openapi_file],
            arguments = [args, file_args],
        )
        openapi_files.append(openapi_file)

    return openapi_files

def _proto_gen_openapi_impl(ctx):
    proto = ctx.attr.proto[ProtoInfo]
    return [
        DefaultInfo(
            files = depset(
                _run_proto_gen_openapi(
                    actions = ctx.actions,
                    proto_info = proto,
                    target_name = ctx.attr.name,
                    transitive_proto_srcs = depset(
                        direct = ctx.files._well_known_protos,
                        transitive = [proto.transitive_sources],
                    ),
                    protoc = ctx.executable._protoc,
                    protoc_gen_openapiv2 = ctx.executable._protoc_gen_openapi,
                    single_output = ctx.attr.single_output,
                    allow_delete_body = ctx.attr.allow_delete_body,
                    grpc_api_configuration = ctx.file.grpc_api_configuration,
                    json_names_for_fields = ctx.attr.json_names_for_fields,
                    repeated_path_param_separator = ctx.attr.repeated_path_param_separator,
                    include_package_in_tags = ctx.attr.include_package_in_tags,
                    fqn_for_openapi_name = ctx.attr.fqn_for_openapi_name,
                    openapi_naming_strategy = ctx.attr.openapi_naming_strategy,
                    use_go_templates = ctx.attr.use_go_templates,
                    disable_default_errors = ctx.attr.disable_default_errors,
                    disable_service_tags = ctx.attr.disable_service_tags,
                    enums_as_ints = ctx.attr.enums_as_ints,
                    omit_enum_default_value = ctx.attr.omit_enum_default_value,
                    output_format = ctx.attr.output_format,
                    simple_operation_ids = ctx.attr.simple_operation_ids,
                    proto3_optional_nullable = ctx.attr.proto3_optional_nullable,
                    openapi_configuration = ctx.file.openapi_configuration,
                    generate_unbound_methods = ctx.attr.generate_unbound_methods,
                    visibility_restriction_selectors = ctx.attr.visibility_restriction_selectors,
                    use_allof_for_refs = ctx.attr.use_allof_for_refs,
                ),
            ),
        ),
    ]

protoc_gen_openapiv2 = rule(
    attrs = {
        "proto": attr.label(
            mandatory = True,
            providers = [ProtoInfo],
        ),
        "single_output": attr.bool(
            default = False,
            mandatory = False,
            doc = "if set, the rule will generate a single OpenAPI file",
        ),
        "allow_delete_body": attr.bool(
            default = False,
            mandatory = False,
            doc = "unless set, HTTP DELETE methods may not have a body",
        ),
        "grpc_api_configuration": attr.label(
            allow_single_file = True,
            mandatory = False,
            doc = "path to file which describes the gRPC API Configuration in YAML format",
        ),
        "json_names_for_fields": attr.bool(
            default = True,
            mandatory = False,
            doc = "if disabled, the original proto name will be used for generating OpenAPI definitions",
        ),
        "repeated_path_param_separator": attr.string(
            default = "csv",
            mandatory = False,
            values = ["csv", "pipes", "ssv", "tsv"],
            doc = "configures how repeated fields should be split." +
                  " Allowed values are `csv`, `pipes`, `ssv` and `tsv`",
        ),
        "include_package_in_tags": attr.bool(
            default = False,
            mandatory = False,
            doc = "if unset, the gRPC service name is added to the `Tags`" +
                  " field of each operation. If set and the `package` directive" +
                  " is shown in the proto file, the package name will be " +
                  " prepended to the service name",
        ),
        "fqn_for_openapi_name": attr.bool(
            default = False,
            mandatory = False,
            doc = "if set, the object's OpenAPI names will use the fully" +
                  " qualified names from the proto definition" +
                  " (ie my.package.MyMessage.MyInnerMessage",
        ),
        "openapi_naming_strategy": attr.string(
            default = "",
            mandatory = False,
            values = ["", "simple", "legacy", "fqn"],
            doc = "configures how OpenAPI names are determined." +
                  " Allowed values are `` (empty), `simple`, `legacy` and `fqn`." +
                  " If unset, either `legacy` or `fqn` are selected, depending" +
                  " on the value of the `fqn_for_openapi_name` setting",
        ),
        "use_go_templates": attr.bool(
            default = False,
            mandatory = False,
            doc = "if set, you can use Go templates in protofile comments",
        ),
        "disable_default_errors": attr.bool(
            default = False,
            mandatory = False,
            doc = "if set, disables generation of default errors." +
                  " This is useful if you have defined custom error handling",
        ),
        "disable_service_tags": attr.bool(
            default = False,
            mandatory = False,
            doc = "if set, disables generation of service tags." +
                  " This is useful if you do not want to expose the names of your backend grpc services.",
        ),
        "enums_as_ints": attr.bool(
            default = False,
            mandatory = False,
            doc = "whether to render enum values as integers, as opposed to string values",
        ),
        "omit_enum_default_value": attr.bool(
            default = False,
            mandatory = False,
            doc = "if set, omit default enum value",
        ),
        "output_format": attr.string(
            default = "json",
            mandatory = False,
            values = ["json", "yaml"],
            doc = "output content format. Allowed values are: `json`, `yaml`",
        ),
        "simple_operation_ids": attr.bool(
            default = False,
            mandatory = False,
            doc = "whether to remove the service prefix in the operationID" +
                  " generation. Can introduce duplicate operationIDs, use with caution.",
        ),
        "proto3_optional_nullable": attr.bool(
            default = False,
            mandatory = False,
            doc = "whether Proto3 Optional fields should be marked as x-nullable",
        ),
        "openapi_configuration": attr.label(
            allow_single_file = True,
            mandatory = False,
            doc = "path to file which describes the OpenAPI Configuration in YAML format",
        ),
        "generate_unbound_methods": attr.bool(
            default = False,
            mandatory = False,
            doc = "generate swagger metadata even for RPC methods that have" +
                  " no HttpRule annotation",
        ),
        "visibility_restriction_selectors": attr.string_list(
            mandatory = False,
            doc = "list of `google.api.VisibilityRule` visibility labels to include" +
                  " in the generated output when a visibility annotation is defined." +
                  " Repeat this option to supply multiple values. Elements without" +
                  " visibility annotations are unaffected by this setting.",
        ),
        "use_allof_for_refs": attr.bool(
            default = False,
            mandatory = False,
            doc = "if set, will use allOf as container for $ref to preserve" +
                  " same-level properties.",
        ),
        "_protoc": attr.label(
            default = "@com_google_protobuf//:protoc",
            executable = True,
            cfg = "exec",
        ),
        "_well_known_protos": attr.label(
            default = "@com_google_protobuf//:well_known_type_protos",
            allow_files = True,
        ),
        "_protoc_gen_openapi": attr.label(
            default = Label("//protoc-gen-openapiv2:protoc-gen-openapiv2"),
            executable = True,
            cfg = "exec",
        ),
    },
    implementation = _proto_gen_openapi_impl,
)
