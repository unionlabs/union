FROM ubuntu:20.04
RUN apt-get update && apt-get install -y --no-install-recommends \
  gcc \
  libc6-dev \
  ca-certificates \
  gcc-mingw-w64-x86-64

# No need to run tests, we're just testing that it compiles
ENV CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUNNER=echo \
    CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc
