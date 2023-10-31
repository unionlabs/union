# Small script to run debuglink tests inside a docker image.
# Creates a writable mount on /usr/lib/debug.

set -ex

run() {
    cargo generate-lockfile --manifest-path crates/debuglink/Cargo.toml
    mkdir -p target crates/debuglink/target debug
    docker build -t backtrace -f ci/docker/$1/Dockerfile ci
    docker run \
      --user `id -u`:`id -g` \
      --rm \
      --init \
      --volume $(dirname $(dirname `which cargo`)):/cargo \
      --env CARGO_HOME=/cargo \
      --volume `rustc --print sysroot`:/rust:ro \
      --env TARGET=$1 \
      --volume `pwd`:/checkout:ro \
      --volume `pwd`/target:/checkout/crates/debuglink/target \
      --workdir /checkout \
      --volume `pwd`/debug:/usr/lib/debug \
      --privileged \
      --env RUSTFLAGS \
      backtrace \
      bash \
      -c 'PATH=$PATH:/rust/bin exec ci/debuglink.sh'
}

run x86_64-unknown-linux-gnu
