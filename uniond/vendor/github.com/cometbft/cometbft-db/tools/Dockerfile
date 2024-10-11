# This file defines the container image used to build and test tm-db in CI.
# The CI workflows use the latest tag of cometbft/cometbft-db-testing built
# from these settings.
#
# The jobs defined in the Build & Push workflow will build and update the image
# when changes to this file are merged.  If you have other changes that require
# updates here, merge the changes here first and let the image get updated (or
# push a new version manually) before PRs that depend on them.

FROM golang:1.23.1 AS build

ENV LD_LIBRARY_PATH=/usr/local/lib

RUN apt update \
    && apt install -y \
       libbz2-dev libgflags-dev libsnappy-dev libzstd-dev zlib1g-dev liblz4-dev \
       make tar wget build-essential \
       libleveldb-dev libleveldb1d

FROM build AS install
ARG ROCKSDB=9.3.1

# Install Rocksdb
RUN \
  wget -q https://github.com/facebook/rocksdb/archive/refs/tags/v${ROCKSDB}.tar.gz \
  && tar -zxf v${ROCKSDB}.tar.gz \
  && cd rocksdb-${ROCKSDB} \
  && DEBUG_LEVEL=0 make -j4 shared_lib \
  && make install-shared \
  && ldconfig \
  && cd .. \
  && rm -rf v${ROCKSDB}.tar.gz rocksdb-${ROCKSDB}
