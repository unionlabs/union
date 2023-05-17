Our networks configuration is structured like this:

## Networks

We define the following **networks**:

- **devnet**: what a developer runs _locally on their machine_ to simulate a full end-to-end network setup.
- **testnet**: what Union runs on their nodes in order to _test a mainnet-like environment_.
- **mainnet**: what runs on the public Union mainnet. _("the production environment")_

## Genesis

`genesis/` contains the genesis configurations for each _network_.

## Services

`services/` contains all of the **service-generating functions**. They are defined as Nix functions so that dependencies and network-specific configuration can be injected as needed for the network in which they are used. These functions are then included in [arion](https://docs.hercules-ci.com/arion/) specs.

## Arion

In `devnet.nix` we combine _Genesis configuration_ and _service-generating functions_ so that they are injected in an [arion](https://docs.hercules-ci.com/arion/) spec. Arion is a Nix wrapper around `docker-compose`. This allows us to create reproducible networks.
