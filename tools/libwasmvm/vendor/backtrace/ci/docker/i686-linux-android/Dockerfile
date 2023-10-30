FROM ubuntu:20.04

RUN apt-get update && apt-get install -y --no-install-recommends \
  curl \
  ca-certificates \
  unzip \
  openjdk-8-jre \
  python \
  gcc \
  libc6-dev

COPY android-ndk.sh /
RUN /android-ndk.sh x86
ENV PATH=$PATH:/android-toolchain/bin

# TODO: run tests in an emulator eventually
ENV CARGO_TARGET_I686_LINUX_ANDROID_LINKER=i686-linux-android-gcc \
    CARGO_TARGET_I686_LINUX_ANDROID_RUNNER=echo
