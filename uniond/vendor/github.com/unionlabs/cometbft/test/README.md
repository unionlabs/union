# CometBFT Tests

## Unit tests
The unit tests (ie. `go test`) can be run with `make test` from the root directory of the repository.

## Integration tests

The integration tests can be run with `make test_integrations` from the root directory of the repository.

Running the integrations test will build a docker container with local version of CometBFT
and run the following tests in docker containers:

- go tests, with --race
    - includes test coverage
- app tests
    - kvstore app over socket
- persistence tests
    - crash cometbft at each of many predefined points, restart, and ensure it syncs properly with the app

## End-to-end tests

You can run e2e nightly tests locally by running
```
make -j2 docker generator runner && ./build/generator -g 5 -d networks/nightly/ -p && ./run-multiple.sh networks/nightly/*-group*-*.toml
```
from the root directory of the repository.

Please refer to the [README.MD](e2e/README.md) in the `e2e` folder for more information.

## `localnet` tests
During development you might need a local network that runs modified code.

1. Build a linux binary with the code modifications
```
make build-linux
```

2. Run a local network with four nodes
```
make localnet-start
```
This command will build a `localnet` docker image and run it using Docker Compose.

3. You can send arbitrary transactions on your local network by building and running the `loadtime` application.
```
make -C test/loadtime build
test/loadtime/build/load -c 2 --broadcast-tx-method sync -T 1200 --endpoints ws://127.0.0.1:26657/websocket
```
Refer to the [README.md](loadtime/README.md) for more information.

4. You can also monitor your network using Prometheus and Grafana.
```
make monitoring-start
```
Refer to the [README.md](monitoring/README.md) for more information.


## Fuzzing

[Fuzzing](https://en.wikipedia.org/wiki/Fuzzing) of various system inputs.

See `./fuzz/README.md` for more details.
