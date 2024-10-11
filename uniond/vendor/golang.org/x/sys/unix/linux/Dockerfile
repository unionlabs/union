FROM ubuntu:20.04

# Disable interactive prompts on package installation
ENV DEBIAN_FRONTEND noninteractive

# Dependencies to get the git sources and go binaries
RUN apt-get update && apt-get install -y  --no-install-recommends \
        ca-certificates \
        curl \
        git \
        rsync \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Get the git sources. If not cached, this takes O(5 minutes).
WORKDIR /git
RUN git config --global advice.detachedHead false
# Linux Kernel: Released 14 July 2024
RUN git clone --branch v6.10 --depth 1 https://kernel.googlesource.com/pub/scm/linux/kernel/git/torvalds/linux
# GNU C library: Released 21 July 2024
RUN git clone --branch release/2.40/master --depth 1 https://sourceware.org/git/glibc.git

# Get Go
ENV GOLANG_VERSION 1.21.0
ENV GOLANG_DOWNLOAD_URL https://golang.org/dl/go$GOLANG_VERSION.linux-amd64.tar.gz
ENV GOLANG_DOWNLOAD_SHA256 d0398903a16ba2232b389fb31032ddf57cac34efda306a0eebac34f0965a0742

RUN curl -fsSL "$GOLANG_DOWNLOAD_URL" -o golang.tar.gz \
    && echo "$GOLANG_DOWNLOAD_SHA256  golang.tar.gz" | sha256sum -c - \
    && tar -C /usr/local -xzf golang.tar.gz \
    && rm golang.tar.gz

ENV PATH /usr/local/go/bin:$PATH

# Linux and Glibc build dependencies and emulator
RUN apt-get update && apt-get install -y  --no-install-recommends \
        bison gawk make python3 \
        gcc gcc-multilib \
        gettext texinfo \
        qemu-user \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*
# Cross compilers (install recommended packages to get cross libc-dev)
RUN apt-get update && apt-get install -y \
        gcc-aarch64-linux-gnu       gcc-arm-linux-gnueabi     \
        gcc-mips-linux-gnu          gcc-mips64-linux-gnuabi64 \
        gcc-mips64el-linux-gnuabi64 gcc-mipsel-linux-gnu      \
        gcc-powerpc-linux-gnu       gcc-powerpc64-linux-gnu   \
        gcc-powerpc64le-linux-gnu   gcc-riscv64-linux-gnu     \
        gcc-s390x-linux-gnu         gcc-sparc64-linux-gnu     \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# Only for loong64, getting tools of qemu-user and gcc-cross-compiler
ENV LOONG64_BASE_URL https://github.com/loongson/build-tools/releases/download/2023.08.08
ENV LOONG64_GCC  CLFS-loongarch64-8.1-x86_64-cross-tools-gcc-glibc.tar.xz
ENV LOONG64_QEMU qemu-loongarch64
ENV LOONG64_GCC_DOWNLOAD_URL  $LOONG64_BASE_URL/$LOONG64_GCC
ENV LOONG64_QEMU_DOWNLOAD_URL $LOONG64_BASE_URL/$LOONG64_QEMU

RUN apt-get update && apt-get install xz-utils -y && mkdir /loong64 && cd /loong64 \
    && curl -fsSL "$LOONG64_QEMU_DOWNLOAD_URL" -o /usr/bin/"$LOONG64_QEMU" \
    && chmod +x /usr/bin/"$LOONG64_QEMU" \
    && curl -fsSL "$LOONG64_GCC_DOWNLOAD_URL" -o "$LOONG64_GCC" \
    && tar xf "$LOONG64_GCC" -C /usr/local/ \
    && ln -s /usr/local/cross-tools/bin/loongarch64-unknown-linux-gnu-gcc /usr/bin/loongarch64-linux-gnu-gcc \
    && rm -rf /loong64

# Let the scripts know they are in the docker environment
ENV GOLANG_SYS_BUILD docker
WORKDIR /build/unix
ENTRYPOINT ["go", "run", "linux/mkall.go", "/git/linux", "/git/glibc"]
