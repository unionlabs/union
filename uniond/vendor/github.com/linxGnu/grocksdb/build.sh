#!/bin/bash
DIRECTORY="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

INSTALL_PREFIX=$1

BUILD_PATH=/tmp/build
mkdir -p $BUILD_PATH

CMAKE_REQUIRED_PARAMS="-DCMAKE_POSITION_INDEPENDENT_CODE=ON -DCMAKE_INSTALL_PREFIX=${INSTALL_PREFIX}"

snappy_version="1.1.10"
cd $BUILD_PATH && wget https://github.com/google/snappy/archive/${snappy_version}.tar.gz && tar xzf ${snappy_version}.tar.gz && cd snappy-${snappy_version} && \
    mkdir -p build_place && cd build_place && \
    cmake $CMAKE_REQUIRED_PARAMS -DSNAPPY_BUILD_TESTS=OFF -DSNAPPY_BUILD_BENCHMARKS=OFF .. && \
    make install/strip -j16 && \
    cd $BUILD_PATH && rm -rf *

export CFLAGS='-fPIC -O3 -pipe'
export CXXFLAGS='-fPIC -O3 -pipe -Wno-uninitialized'

zlib_version="1.3"
cd $BUILD_PATH && wget https://github.com/madler/zlib/archive/v${zlib_version}.tar.gz && tar xzf v${zlib_version}.tar.gz && cd zlib-${zlib_version} && \
    ./configure --prefix=$INSTALL_PREFIX --static && make -j16 install && \
    cd $BUILD_PATH && rm -rf *

lz4_version="1.9.4"
cd $BUILD_PATH && wget https://github.com/lz4/lz4/archive/v${lz4_version}.tar.gz && tar xzf v${lz4_version}.tar.gz && cd lz4-${lz4_version}/build/cmake && \
    cmake $CMAKE_REQUIRED_PARAMS -DLZ4_BUILD_LEGACY_LZ4C=OFF -DBUILD_SHARED_LIBS=OFF -DLZ4_POSITION_INDEPENDENT_LIB=ON && make -j16 install && \
    cd $BUILD_PATH && rm -rf *

zstd_version="1.5.5"
cd $BUILD_PATH && wget https://github.com/facebook/zstd/archive/v${zstd_version}.tar.gz && tar xzf v${zstd_version}.tar.gz && \
    cd zstd-${zstd_version}/build/cmake && mkdir -p build_place && cd build_place && \
    cmake $CMAKE_REQUIRED_PARAMS -DZSTD_BUILD_PROGRAMS=OFF -DZSTD_BUILD_CONTRIB=OFF -DZSTD_BUILD_STATIC=ON -DZSTD_BUILD_SHARED=OFF -DZSTD_BUILD_TESTS=OFF \
    -DCMAKE_POSITION_INDEPENDENT_CODE=ON -DZSTD_ZLIB_SUPPORT=ON -DZSTD_LZMA_SUPPORT=OFF -DCMAKE_BUILD_TYPE=Release .. && make -j$(nproc) install && \
    cd $BUILD_PATH && rm -rf * && ldconfig

# Note: if you don't have a good reason, please do not set -DPORTABLE=ON
# This one is set here on purpose of compatibility with github action runtime processor
rocksdb_version="8.11.3"
cd $BUILD_PATH && wget https://github.com/facebook/rocksdb/archive/v${rocksdb_version}.tar.gz && tar xzf v${rocksdb_version}.tar.gz && cd rocksdb-${rocksdb_version}/ && \
    mkdir -p build_place && cd build_place && cmake -DCMAKE_BUILD_TYPE=Release $CMAKE_REQUIRED_PARAMS -DCMAKE_PREFIX_PATH=$INSTALL_PREFIX -DWITH_TESTS=OFF -DWITH_GFLAGS=OFF \
    -DWITH_BENCHMARK_TOOLS=OFF -DWITH_TOOLS=OFF -DWITH_MD_LIBRARY=OFF -DWITH_RUNTIME_DEBUG=OFF -DROCKSDB_BUILD_SHARED=OFF -DWITH_SNAPPY=ON -DWITH_LZ4=ON -DWITH_ZLIB=ON -DWITH_LIBURING=OFF \
    -DWITH_ZSTD=ON -DWITH_BZ2=OFF -WITH_GFLAGS=OFF -DPORTABLE=1 .. && make -j16 install/strip && \
    cd $BUILD_PATH && rm -rf *
