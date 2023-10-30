# Small script to run tests for a target (or all targets) inside all the
# respective docker images.

set -ex

run() {
    docker build -t backtrace -f ci/docker/$1/Dockerfile ci
    mkdir -p target
    docker run \
      --user `id -u`:`id -g` \
      --rm \
      --init \
      --volume $(dirname $(dirname `which cargo`)):/cargo \
      --env CARGO_HOME=/cargo \
      --volume `rustc --print sysroot`:/rust:ro \
      --env TARGET=$1 \
      --volume `pwd`:/checkout:ro \
      --volume `pwd`/target:/checkout/target \
      --workdir /checkout \
      --privileged \
      --env RUSTFLAGS \
      backtrace \
      bash \
      -c 'PATH=$PATH:/rust/bin exec ci/run.sh'
}

if [ -z "$1" ]; then
  for d in `ls ci/docker/`; do
    run $d
  done
else
  run $1
fi
