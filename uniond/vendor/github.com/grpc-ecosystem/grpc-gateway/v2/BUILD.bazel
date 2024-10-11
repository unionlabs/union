load("@bazel_gazelle//:def.bzl", "gazelle")
load("@com_github_bazelbuild_buildtools//buildifier:def.bzl", "buildifier")
load("@io_bazel_rules_go//proto:compiler.bzl", "go_proto_compiler")
load("@io_bazel_rules_go//proto/wkt:well_known_types.bzl", "PROTO_RUNTIME_DEPS", "WELL_KNOWN_TYPES_APIV2")

exports_files(["LICENSE.txt"])

buildifier(
    name = "buildifier",
)

buildifier(
    name = "buildifier_check",
    mode = "check",
)

# gazelle:exclude _output
# gazelle:prefix github.com/grpc-ecosystem/grpc-gateway/v2
# gazelle:go_proto_compilers //:go_apiv2
# gazelle:go_grpc_compilers //:go_apiv2, //:go_grpc
# gazelle:go_naming_convention import_alias

gazelle(name = "gazelle")

package_group(
    name = "generators",
    packages = [
        "//protoc-gen-grpc-gateway/...",
        "//protoc-gen-openapiv2/...",
    ],
)

go_proto_compiler(
    name = "go_apiv2",
    options = [
        "paths=source_relative",
    ],
    plugin = "@org_golang_google_protobuf//cmd/protoc-gen-go",
    suffix = ".pb.go",
    visibility = ["//visibility:public"],
    deps = PROTO_RUNTIME_DEPS + WELL_KNOWN_TYPES_APIV2,
)

go_proto_compiler(
    name = "go_grpc",
    options = [
        "paths=source_relative",
        "require_unimplemented_servers=false",
    ],
    plugin = "@org_golang_google_grpc_cmd_protoc_gen_go_grpc//:protoc-gen-go-grpc",
    suffix = "_grpc.pb.go",
    visibility = ["//visibility:public"],
    deps = PROTO_RUNTIME_DEPS + [
        "@org_golang_google_grpc//:go_default_library",
        "@org_golang_google_grpc//codes:go_default_library",
        "@org_golang_google_grpc//status:go_default_library",
    ],
)
