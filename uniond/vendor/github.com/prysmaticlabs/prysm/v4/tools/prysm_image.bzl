load("@rules_oci//oci:defs.bzl", "oci_image", "oci_image_index", "oci_push")
load("@rules_pkg//:pkg.bzl", "pkg_tar")
load("//tools:multi_arch.bzl", "multi_arch")

def prysm_image_upload(
        name,
        binary,
        entrypoint,
        symlinks,
        repository,
        tags):
    pkg_tar(
        name = "binary_tar",
        srcs = [binary],
        symlinks = symlinks,
        tags = tags,
    )

    oci_image(
        name = "oci_image",
        base = "@linux_debian11_multiarch_base",
        entrypoint = entrypoint,
        tars = [
            "//tools:passwd_tar",
            "//tools:libtinfo6_tar",
            "//tools:bash_tar",
            ":binary_tar",
        ],
        labels = {
          "org.opencontainers.image.source": "https://github.com/prysmaticlabs/prysm",
        },
        tags = tags,
    )

    multi_arch(
        name = "oci_multiarch",
        image = ":oci_image",
        platforms = [
            "@io_bazel_rules_go//go/toolchain:linux_amd64_cgo",
            "@io_bazel_rules_go//go/toolchain:linux_arm64_cgo",
        ],
        tags = tags,
    )

    oci_image_index(
        name = "oci_image_index",
        images = [
            ":oci_multiarch",
        ],
        tags = tags,
    )

    oci_push(
        name = name,
        image = ":oci_image_index",
        repository = repository,
        tags = tags,
    )
