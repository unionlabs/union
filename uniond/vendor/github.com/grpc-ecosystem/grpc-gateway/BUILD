load("@bazel_gazelle//:def.bzl", "gazelle")
load("@com_github_bazelbuild_buildtools//buildifier:def.bzl", "buildifier")

buildifier(
    name = "buildifier",
)

buildifier(
    name = "buildifier_check",
    mode = "check",
)

# gazelle:exclude third_party
# gazelle:exclude vendor
# gazelle:exclude _output
# gazelle:prefix github.com/grpc-ecosystem/grpc-gateway

gazelle(name = "gazelle")

package_group(
    name = "generators",
    packages = [
        "//protoc-gen-grpc-gateway/...",
        "//protoc-gen-swagger/...",
    ],
)
