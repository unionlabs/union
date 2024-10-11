load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_archive")

"""
Herumi's BLS library for go depends on
- herumi/mcl
- herumi/bls
- herumi/bls-eth-go-binary
"""

def bls_dependencies():
    _maybe(
        http_archive,
        name = "herumi_bls_eth_go_binary",
        strip_prefix = "bls-eth-go-binary-d37c07cfda4e5369f269368f92c42209400e0742",
        urls = [
            "https://github.com/herumi/bls-eth-go-binary/archive/d37c07cfda4e5369f269368f92c42209400e0742.tar.gz",
        ],
        sha256 = "9d811ed2a4a9fd06d9549c392aefc6ac1ffb6685d15f940b4d14976cfcfd26fa",
        build_file = "@prysm//third_party/herumi:bls_eth_go_binary.BUILD",
    )
    _maybe(
        http_archive,
        name = "herumi_mcl",
        strip_prefix = "mcl-0c31ab9648e81f54177325e55ea96dd8e9c8ba6b",
        urls = [
            "https://github.com/herumi/mcl/archive/0c31ab9648e81f54177325e55ea96dd8e9c8ba6b.tar.gz",
        ],
        sha256 = "0be6f61660ad85ab1fdead420f75d59e3ecbf84da7fa1752daf5157c810727c8",
        build_file = "@prysm//third_party/herumi:mcl.BUILD",
    )
    _maybe(
        http_archive,
        name = "herumi_bls",
        strip_prefix = "bls-02060e20d81c2714e481922b182b43e8e26d1fee",
        urls = [
            "https://github.com/herumi/bls/archive/02060e20d81c2714e481922b182b43e8e26d1fee.tar.gz",
        ],
        sha256 = "60b405c934514816f5559538dccf95fbdfdcd86ed08bf1fb95daae45f1cabbfd",
        build_file = "@prysm//third_party/herumi:bls.BUILD",
    )

def _maybe(repo_rule, name, **kwargs):
    if name not in native.existing_rules():
        repo_rule(name = name, **kwargs)
