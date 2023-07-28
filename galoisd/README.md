# Galoisd

Galois daemon (galoisd) is a gRPC service providing consensus verification for CometBLS block headers. It requires another service, such as an IBC relayer, to send block data for zpk generation.

## Usage

Call `galoisd --help` to see an up-to-date overview of functions. The CLI is self-documenting.

## Production Deployments

For production deployments, use the [docker image](https://github.com/unionlabs/union/pkgs/container/galoisd) provided in our package registry.

```sh
docker pull ghcr.io/unionlabs/galoisd:v0.6.0
```

> **Warning**
>
> galoisd is not designed to be a public service. Proving is a computationally intensive process. Incorrectly configuring the service can lead to denial-of-service attacks.
