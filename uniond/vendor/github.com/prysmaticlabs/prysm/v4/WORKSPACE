workspace(name = "prysm")

load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")
load("@bazel_tools//tools/build_defs/repo:git.bzl", "git_repository")

http_archive(
    name = "rules_pkg",
    sha256 = "8c20f74bca25d2d442b327ae26768c02cf3c99e93fad0381f32be9aab1967675",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_pkg/releases/download/0.8.1/rules_pkg-0.8.1.tar.gz",
        "https://github.com/bazelbuild/rules_pkg/releases/download/0.8.1/rules_pkg-0.8.1.tar.gz",
    ],
)

load("@rules_pkg//:deps.bzl", "rules_pkg_dependencies")

rules_pkg_dependencies()

http_archive(
    name = "hermetic_cc_toolchain",
    sha256 = "973ab22945b921ef45b8e1d6ce01ca7ce1b8a462167449a36e297438c4ec2755",
    strip_prefix = "hermetic_cc_toolchain-5098046bccc15d2962f3cc8e7e53d6a2a26072dc",
    urls = [
        "https://github.com/uber/hermetic_cc_toolchain/archive/5098046bccc15d2962f3cc8e7e53d6a2a26072dc.tar.gz",  # 2023-06-28
    ],
)

load("@hermetic_cc_toolchain//toolchain:defs.bzl", zig_toolchains = "toolchains")

# Temporarily use a nightly build until 0.12.0 is released.
# See: https://github.com/prysmaticlabs/prysm/issues/13130
zig_toolchains(
    host_platform_sha256 = {
        "linux-aarch64": "45afb8e32adde825165f4f293fcea9ecea503f7f9ec0e9bf4435afe70e67fb70",
        "linux-x86_64": "f136c6a8a0f6adcb057d73615fbcd6f88281b3593f7008d5f7ed514ff925c02e",
        "macos-aarch64": "05d995853c05243151deff47b60bdc2674f1e794a939eaeca0f42312da031cee",
        "macos-x86_64": "721754ba5a50f31e8a1f0e1a74cace26f8246576878ac4a8591b0ee7b6db1fc1",
        "windows-x86_64": "93f5248b2ea8c5ee8175e15b1384e133edc1cd49870b3ea259062a2e04164343",
    },
    url_formats = [
        "https://ziglang.org/builds/zig-{host_platform}-{version}.{_ext}",
        "https://mirror.bazel.build/ziglang.org/builds/zig-{host_platform}-{version}.{_ext}",
        "https://prysmaticlabs.com/mirror/ziglang.org/builds/zig-{host_platform}-{version}.{_ext}",
    ],
    version = "0.12.0-dev.1349+fa022d1ec",
)

# Register zig sdk toolchains with support for Ubuntu 20.04 (Focal Fossa) which has an EOL date of April, 2025.
# For ubuntu glibc support, see https://launchpad.net/ubuntu/+source/glibc
register_toolchains(
    "@zig_sdk//toolchain:linux_amd64_gnu.2.31",
    "@zig_sdk//toolchain:linux_arm64_gnu.2.31",
    # Hermetic cc toolchain is not yet supported on darwin. Sysroot needs to be provided.
    # See https://github.com/uber/hermetic_cc_toolchain#osx-sysroot
    #    "@zig_sdk//toolchain:darwin_amd64",
    #    "@zig_sdk//toolchain:darwin_arm64",
    # Windows builds are not supported yet.
    #    "@zig_sdk//toolchain:windows_amd64",
)

load("@prysm//tools/cross-toolchain:darwin_cc_hack.bzl", "configure_nonhermetic_darwin")

configure_nonhermetic_darwin()

load("@prysm//tools/cross-toolchain:prysm_toolchains.bzl", "configure_prysm_toolchains")

configure_prysm_toolchains()

http_archive(
    name = "bazel_skylib",
    sha256 = "1c531376ac7e5a180e0237938a2536de0c54d93f5c278634818e0efc952dd56c",
    urls = [
        "https://github.com/bazelbuild/bazel-skylib/releases/download/1.0.3/bazel-skylib-1.0.3.tar.gz",
        "https://mirror.bazel.build/github.com/bazelbuild/bazel-skylib/releases/download/1.0.3/bazel-skylib-1.0.3.tar.gz",
    ],
)

load("@bazel_skylib//:workspace.bzl", "bazel_skylib_workspace")

bazel_skylib_workspace()

http_archive(
    name = "bazel_gazelle",
    sha256 = "d3fa66a39028e97d76f9e2db8f1b0c11c099e8e01bf363a923074784e451f809",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/bazel-gazelle/releases/download/v0.33.0/bazel-gazelle-v0.33.0.tar.gz",
        "https://github.com/bazelbuild/bazel-gazelle/releases/download/v0.33.0/bazel-gazelle-v0.33.0.tar.gz",
    ],
)

http_archive(
    name = "com_github_atlassian_bazel_tools",
    sha256 = "60821f298a7399450b51b9020394904bbad477c18718d2ad6c789f231e5b8b45",
    strip_prefix = "bazel-tools-a2138311856f55add11cd7009a5abc8d4fd6f163",
    urls = ["https://github.com/atlassian/bazel-tools/archive/a2138311856f55add11cd7009a5abc8d4fd6f163.tar.gz"],
)

http_archive(
    name = "rules_distroless",
    sha256 = "e64f06e452cd153aeab81f752ccf4642955b3af319e64f7bc7a7c9252f76b10e",
    strip_prefix = "rules_distroless-f5e678217b57ce3ad2f1c0204bd4e9d416255773",
    url = "https://github.com/GoogleContainerTools/rules_distroless/archive/f5e678217b57ce3ad2f1c0204bd4e9d416255773.tar.gz",
)

load("@rules_distroless//distroless:dependencies.bzl", "rules_distroless_dependencies")

rules_distroless_dependencies()

load("@aspect_bazel_lib//lib:repositories.bzl", "aspect_bazel_lib_dependencies", "aspect_bazel_lib_register_toolchains")

aspect_bazel_lib_dependencies()

aspect_bazel_lib_register_toolchains()

http_archive(
    name = "rules_oci",
    sha256 = "c71c25ed333a4909d2dd77e0b16c39e9912525a98c7fa85144282be8d04ef54c",
    strip_prefix = "rules_oci-1.3.4",
    url = "https://github.com/bazel-contrib/rules_oci/releases/download/v1.3.4/rules_oci-v1.3.4.tar.gz",
)

load("@rules_oci//oci:dependencies.bzl", "rules_oci_dependencies")

rules_oci_dependencies()

load("@rules_oci//oci:repositories.bzl", "LATEST_CRANE_VERSION", "oci_register_toolchains")

oci_register_toolchains(
    name = "oci",
    crane_version = LATEST_CRANE_VERSION,
)

http_archive(
    name = "io_bazel_rules_go",
    patch_args = ["-p1"],
    patches = [
        # Expose internals of go_test for custom build transitions.
        "//third_party:io_bazel_rules_go_test.patch",
    ],
    sha256 = "d6ab6b57e48c09523e93050f13698f708428cfd5e619252e369d377af6597707",
    urls = [
        "https://mirror.bazel.build/github.com/bazelbuild/rules_go/releases/download/v0.43.0/rules_go-v0.43.0.zip",
        "https://github.com/bazelbuild/rules_go/releases/download/v0.43.0/rules_go-v0.43.0.zip",
    ],
)

# Override default import in rules_go with special patch until
# https://github.com/gogo/protobuf/pull/582 is merged.
git_repository(
    name = "com_github_gogo_protobuf",
    commit = "b03c65ea87cdc3521ede29f62fe3ce239267c1bc",
    patch_args = ["-p1"],
    patches = [
        "@io_bazel_rules_go//third_party:com_github_gogo_protobuf-gazelle.patch",
        "//third_party:com_github_gogo_protobuf-equal.patch",
    ],
    remote = "https://github.com/gogo/protobuf",
    shallow_since = "1610265707 +0000",
    # gazelle args: -go_prefix github.com/gogo/protobuf -proto legacy
)

load("@rules_oci//oci:pull.bzl", "oci_pull")

# A multi-arch base image
oci_pull(
    name = "linux_debian11_multiarch_base",  # Debian bullseye
    digest = "sha256:b82f113425c5b5c714151aaacd8039bc141821cdcd3c65202d42bdf9c43ae60b",  # 2023-12-12
    image = "gcr.io/distroless/cc-debian11",
    platforms = [
        "linux/amd64",
        "linux/arm64/v8",
    ],
    reproducible = True,
)

load("@prysm//tools:image_deps.bzl", "prysm_image_deps")

prysm_image_deps()

load("@io_bazel_rules_go//go:deps.bzl", "go_register_toolchains", "go_rules_dependencies")

go_rules_dependencies()

go_register_toolchains(
    go_version = "1.21.6",
    nogo = "@//:nogo",
)

http_archive(
    name = "io_kubernetes_build",
    sha256 = "b84fbd1173acee9d02a7d3698ad269fdf4f7aa081e9cecd40e012ad0ad8cfa2a",
    strip_prefix = "repo-infra-6537f2101fb432b679f3d103ee729dd8ac5d30a0",
    url = "https://github.com/kubernetes/repo-infra/archive/6537f2101fb432b679f3d103ee729dd8ac5d30a0.tar.gz",
)

http_archive(
    name = "eip3076_spec_tests",
    build_file_content = """
filegroup(
    name = "test_data",
    srcs = glob([
        "**/*.json",
    ]),
    visibility = ["//visibility:public"],
)
    """,
    sha256 = "516d551cfb3e50e4ac2f42db0992f4ceb573a7cb1616d727a725c8161485329f",
    url = "https://github.com/eth-clients/slashing-protection-interchange-tests/archive/refs/tags/v5.3.0.tar.gz",
)

http_archive(
    name = "eip4881_spec_tests",
    build_file_content = """
filegroup(
    name = "test_data",
    srcs = glob([
        "**/*.yaml",
    ]),
    visibility = ["//visibility:public"],
)
    """,
    sha256 = "89cb659498c0d196fc9f957f8b849b2e1a5c041c3b2b3ae5432ac5c26944297e",
    url = "https://github.com/ethereum/EIPs/archive/5480440fe51742ed23342b68cf106cefd427e39d.tar.gz",
)

consensus_spec_version = "v1.4.0-beta.5"

bls_test_version = "v0.1.1"

http_archive(
    name = "consensus_spec_tests_general",
    build_file_content = """
filegroup(
    name = "test_data",
    srcs = glob([
        "**/*.ssz_snappy",
        "**/*.yaml",
    ]),
    visibility = ["//visibility:public"],
)
    """,
    sha256 = "9017ffff84d64a7c4c9e6ff9f421f9479f71d3b463b738f54e02158dbb4f50f0",
    url = "https://github.com/ethereum/consensus-spec-tests/releases/download/%s/general.tar.gz" % consensus_spec_version,
)

http_archive(
    name = "consensus_spec_tests_minimal",
    build_file_content = """
filegroup(
    name = "test_data",
    srcs = glob([
        "**/*.ssz_snappy",
        "**/*.yaml",
    ]),
    visibility = ["//visibility:public"],
)
    """,
    sha256 = "f08711682553fe7c9362f1400ed8c56b2fa9576df08581fcad4c508ba8ad4788",
    url = "https://github.com/ethereum/consensus-spec-tests/releases/download/%s/minimal.tar.gz" % consensus_spec_version,
)

http_archive(
    name = "consensus_spec_tests_mainnet",
    build_file_content = """
filegroup(
    name = "test_data",
    srcs = glob([
        "**/*.ssz_snappy",
        "**/*.yaml",
    ]),
    visibility = ["//visibility:public"],
)
    """,
    sha256 = "7ea3189e3879f2ac62467cbf2945c00b6c94d30cdefb2d645c630b1018c50e10",
    url = "https://github.com/ethereum/consensus-spec-tests/releases/download/%s/mainnet.tar.gz" % consensus_spec_version,
)

http_archive(
    name = "consensus_spec",
    build_file_content = """
filegroup(
    name = "spec_data",
    srcs = glob([
        "**/*.yaml",
    ]),
    visibility = ["//visibility:public"],
)
    """,
    sha256 = "4119992a2efc79e5cb2bdc07ed08c0b1fa32332cbd0d88e6467f34938df97026",
    strip_prefix = "consensus-specs-" + consensus_spec_version[1:],
    url = "https://github.com/ethereum/consensus-specs/archive/refs/tags/%s.tar.gz" % consensus_spec_version,
)

http_archive(
    name = "bls_spec_tests",
    build_file_content = """
filegroup(
    name = "test_data",
    srcs = glob([
        "**/*.yaml",
    ]),
    visibility = ["//visibility:public"],
)
    """,
    sha256 = "93c7d006e7c5b882cbd11dc9ec6c5d0e07f4a8c6b27a32f964eb17cf2db9763a",
    url = "https://github.com/ethereum/bls12-381-tests/releases/download/%s/bls_tests_yaml.tar.gz" % bls_test_version,
)

http_archive(
    name = "eth2_networks",
    build_file_content = """
filegroup(
    name = "configs",
    srcs = glob([
        "shared/**/config.yaml",
    ]),
    visibility = ["//visibility:public"],
)
    """,
    sha256 = "2701e1e1a3ec10c673fe7dbdbbe6f02c8ae8c922aebbf6e720d8c72d5458aafe",
    strip_prefix = "eth2-networks-7b4897888cebef23801540236f73123e21774954",
    url = "https://github.com/eth-clients/eth2-networks/archive/7b4897888cebef23801540236f73123e21774954.tar.gz",
)

http_archive(
    name = "goerli_testnet",
    build_file_content = """
filegroup(
    name = "configs",
    srcs = [
        "prater/config.yaml",
    ],
    visibility = ["//visibility:public"],
)
    """,
    sha256 = "43fc0f55ddff7b511713e2de07aa22846a67432df997296fb4fc09cd8ed1dcdb",
    strip_prefix = "goerli-6522ac6684693740cd4ddcc2a0662e03702aa4a1",
    url = "https://github.com/eth-clients/goerli/archive/6522ac6684693740cd4ddcc2a0662e03702aa4a1.tar.gz",
)

http_archive(
    name = "holesky_testnet",
    build_file_content = """
filegroup(
    name = "configs",
    srcs = [
        "custom_config_data/config.yaml",
    ],
    visibility = ["//visibility:public"],
)
""",
    sha256 = "5f4be6fd088683ea9db45c863b9c5a1884422449e5b59fd2d561d3ba0f73ffd9",
    strip_prefix = "holesky-9d9aabf2d4de51334ee5fed6c79a4d55097d1a43",
    url = "https://github.com/eth-clients/holesky/archive/9d9aabf2d4de51334ee5fed6c79a4d55097d1a43.tar.gz",  # 2024-01-22
)

http_archive(
    name = "com_google_protobuf",
    sha256 = "9bd87b8280ef720d3240514f884e56a712f2218f0d693b48050c836028940a42",
    strip_prefix = "protobuf-25.1",
    urls = [
        "https://github.com/protocolbuffers/protobuf/archive/v25.1.tar.gz",
    ],
)

# External dependencies
http_archive(
    name = "googleapis",
    sha256 = "9d1a930e767c93c825398b8f8692eca3fe353b9aaadedfbcf1fca2282c85df88",
    strip_prefix = "googleapis-64926d52febbf298cb82a8f472ade4a3969ba922",
    urls = [
        "https://github.com/googleapis/googleapis/archive/64926d52febbf298cb82a8f472ade4a3969ba922.zip",
    ],
)

load("@googleapis//:repository_rules.bzl", "switched_rules_by_language")

switched_rules_by_language(
    name = "com_google_googleapis_imports",
    go = True,
)

load("//:deps.bzl", "prysm_deps")

# gazelle:repository_macro deps.bzl%prysm_deps
prysm_deps()

load("@prysm//third_party/herumi:herumi.bzl", "bls_dependencies")

bls_dependencies()

load("@prysm//testing/endtoend:deps.bzl", "e2e_deps")

e2e_deps()

load("@com_github_atlassian_bazel_tools//gometalinter:deps.bzl", "gometalinter_dependencies")

gometalinter_dependencies()

load("@bazel_gazelle//:deps.bzl", "gazelle_dependencies")

gazelle_dependencies()

load("@com_google_protobuf//:protobuf_deps.bzl", "protobuf_deps")

protobuf_deps()

# Do NOT add new go dependencies here! Refer to DEPENDENCIES.md!
