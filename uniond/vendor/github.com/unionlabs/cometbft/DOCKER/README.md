# Docker

## Supported tags and respective `Dockerfile` links

DockerHub tags for official releases are [here](https://hub.docker.com/r/cometbft/cometbft/tags/). The "latest" tag will always point to the highest version number.

Official releases can be found [here](https://github.com/cometbft/cometbft/releases).

The Dockerfile for CometBFT is not expected to change in the near future. The main file used for all builds can be found [here](https://raw.githubusercontent.com/cometbft/cometbft/main/DOCKER/Dockerfile).

Respective versioned files can be found at `https://raw.githubusercontent.com/cometbft/cometbft/vX.XX.XX/DOCKER/Dockerfile` (replace the Xs with the version number).

## Quick reference

- **Where to get help:** <https://cometbft.com/>
- **Where to file issues:** <https://github.com/cometbft/cometbft/issues>
- **Supported Docker versions:** [the latest release](https://github.com/moby/moby/releases) (down to 1.6 on a best-effort basis)

## CometBFT

CometBFT is Byzantine Fault Tolerant (BFT) middleware that takes a state transition machine, written in any programming language, and securely replicates it on many machines.

For more background, see the [the docs](https://docs.cometbft.com/main/introduction/#quick-start).

To get started developing applications, see the [application developers guide](https://docs.cometbft.com/main/introduction/quick-start.html).

## How to use this image

### Start one instance of the CometBFT with the `kvstore` app

A quick example of a built-in app and CometBFT in one container.

```sh
docker run -it --rm -v "/tmp:/cometbft" cometbft/cometbft init
docker run -it --rm -v "/tmp:/cometbft" cometbft/cometbft node --proxy_app=kvstore
```

## Local cluster

To run a 4-node network, see the `Makefile` in the root of [the repo](https://github.com/cometbft/cometbft/blob/main/Makefile) and run:

```sh
make build-linux
make build-docker-localnode
make localnet-start
```

Note that this will build and use a different image than the ones provided here.

## License

- CometBFT's license is [Apache 2.0](https://github.com/cometbft/cometbft/blob/main/LICENSE).

## Contributing

Contributions are most welcome! See the [contributing file](https://github.com/cometbft/cometbft/blob/main/CONTRIBUTING.md) for more information.
