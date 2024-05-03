# Galoisd

Galois daemon (galoisd) is a gRPC service providing consensus proof generation for CometBLS block headers. It requires another service, such as an IBC relayer, to send block data for zpk generation.

The circuits developed under this service are tailored for the light client. Various implementations are partial and may not be sound if reused in a different context. We do not recommend reusing part of the circuits independently.

## Usage

Call `galoisd --help` to see an up-to-date overview of functions. The CLI is self-documenting.

### Local development

- Enter the union devshell with `nix develop`
- Run galoisd `nix run .#galoisd -- --help`

## Production Deployments

> **Warning**
>
> galoisd is not designed to be a public service. Proving is a computationally intensive process. Incorrectly configuring the service can lead to denial-of-service attacks.

### Docker

For production deployments, use the [docker image](https://github.com/unionlabs/union/pkgs/container/galoisd) provided in our package registry.

```sh
docker pull ghcr.io/unionlabs/galoisd:<VERSION>
```

### Nix

`nix run github:unionlabs/union/<COMMIT_OR_VERSION>#galoisd -- --help`
