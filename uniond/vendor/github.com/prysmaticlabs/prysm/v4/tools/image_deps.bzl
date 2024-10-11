load("@bazel_tools//tools/build_defs/repo:http.bzl", "http_file")

def prysm_image_deps():
    http_file(
        name = "bash_amd64",
        sha256 = "f702ef058e762d7208a9c83f6f6bbf02645533bfd615c54e8cdcce842cd57377",
        urls = [
            "http://ftp.us.debian.org/debian/pool/main/b/bash/bash_5.1-2+deb11u1_amd64.deb",
            "http://http.us.debian.org/debian/pool/main/b/bash/bash_5.1-2+deb11u1_amd64.deb",
            "http://ftp.uk.debian.org/debian/pool/main/b/bash/bash_5.1-2+deb11u1_amd64.deb",
            "http://ftp.au.debian.org/debian/pool/main/b/bash/bash_5.1-2+deb11u1_amd64.deb",
            "https://prysmaticlabs.com/uploads/bash_5.1-2+deb11u1_amd64.deb",
        ],
    )

    http_file(
        name = "bash_arm64",
        sha256 = "d7c7af5d86f43a885069408a89788f67f248e8124c682bb73936f33874e0611b",
        urls = [
            "http://ftp.us.debian.org/debian/pool/main/b/bash/bash_5.1-2+deb11u1_arm64.deb",
            "http://http.us.debian.org/debian/pool/main/b/bash/bash_5.1-2+deb11u1_arm64.deb",
            "http://ftp.uk.debian.org/debian/pool/main/b/bash/bash_5.1-2+deb11u1_arm64.deb",
            "http://ftp.au.debian.org/debian/pool/main/b/bash/bash_5.1-2+deb11u1_arm64.deb",
            "https://prysmaticlabs.com/uploads/bash_5.1-2+deb11u1_arm64.deb",
        ],
    )

    http_file(
        name = "libtinfo6_amd64",
        sha256 = "96ed58b8fd656521e08549c763cd18da6cff1b7801a3a22f29678701a95d7e7b",
        urls = [
            "http://ftp.us.debian.org/debian/pool/main/n/ncurses/libtinfo6_6.2+20201114-2+deb11u2_amd64.deb",
            "http://http.us.debian.org/debian/pool/main/n/ncurses/libtinfo6_6.2+20201114-2+deb11u2_amd64.deb",
            "http://ftp.uk.debian.org/debian/pool/main/n/ncurses/libtinfo6_6.2+20201114-2+deb11u2_amd64.deb",
            "http://ftp.au.debian.org/debian/pool/main/n/ncurses/libtinfo6_6.2+20201114-2+deb11u2_amd64.deb",
            "https://prysmaticlabs.com/uploads/libtinfo6_6.2+20201114-2+deb11u2_amd64.deb",
        ],
    )

    http_file(
        name = "libtinfo6_arm64",
        sha256 = "58027c991756930a2abb2f87a829393d3fdbfb76f4eca9795ef38ea2b0510e27",
        urls = [
            "http://ftp.us.debian.org/debian/pool/main/n/ncurses/libtinfo6_6.2+20201114-2+deb11u1_arm64.deb",
            "http://http.us.debian.org/debian/pool/main/n/ncurses/libtinfo6_6.2+20201114-2+deb11u1_arm64.deb",
            "http://ftp.uk.debian.org/debian/pool/main/n/ncurses/libtinfo6_6.2+20201114-2+deb11u1_arm64.deb",
            "http://ftp.au.debian.org/debian/pool/main/n/ncurses/libtinfo6_6.2+20201114-2+deb11u1_arm64.deb",
            "https://prysmaticlabs.com/uploads/libtinfo6_6.2+20201114-2+deb11u2_arm64.deb",
        ],
    )
