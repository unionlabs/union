#!/bin/bash

grpc_1_39_0_githash=58602e20a3f3e48f24a4114c757099b25b947f7b

function build_grpc () {
  git clone https://github.com/grpc/grpc.git google/grpc
  cd google/grpc
  git checkout ${grpc_1_39_0_githash}
  git submodule update --init
  # Apply boringssl build patch
  cd third_party/boringssl-with-bazel
  git apply ../../../../grpc/boringssl.patch
  cd ../..
  mkdir ../grpc_build
  cd ../grpc_build
  cmake ../grpc -DgRPC_INSTALL=ON -DgRPC_BUILD_TESTS=OFF -DABSL_ENABLE_INSTALL=ON -DBUILD_SHARED_LIBS=ON -DCMAKE_INSTALL_PREFIX=`pwd`/../grpc/install
  cmake --build . --target install ${JOBS:+-j$JOBS}
  cd ../..
}

GRPC_INSTALL_PATH=`pwd`/google/grpc/install
PROTOBUF_DOWNLOAD_PATH=`pwd`/google/grpc/third_party/protobuf

build_grpc
