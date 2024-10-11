- `[proto]` Add definitions and generated code for
  [ADR-101](./docs/architecture/adr-101-data-companion-pull-api.md)
  `PruningService` in the `cometbft.services.pruning.v1` proto package
  ([\#1097](https://github.com/cometbft/cometbft/issues/1097))
- `[rpc/grpc]` Add privileged gRPC server and client facilities, in
  `server/privileged` and `client/privileged` packages respectively, to
  enable a separate API server within the node which serves trusted clients
  without authentication and should never be exposed to public internet
  ([\#1097](https://github.com/cometbft/cometbft/issues/1097))
- `[rpc/grpc]` Add a pruning service adding on the privileged gRPC server API to
  give an [ADR-101](./docs/architecture/adr-101-data-companion-pull-api.md) data
  companion control over block data retained by the node. The
  `WithPruningService` option method in `server/privileged` is provided to
  configure the pruning service
  ([\#1097](https://github.com/cometbft/cometbft/issues/1097))
- `[rpc/grpc]` Add `PruningServiceClient` interface
  for the gRPC client in `client/privileged` along with a configuration option
  to enable it
  ([\#1097](https://github.com/cometbft/cometbft/issues/1097))
- `[config]` Add `[grpc.privileged]` section to configure the privileged
  gRPC server for the node, and `[grpc.privileged.pruning_service]` section
  to control the pruning service
  ([\#1097](https://github.com/cometbft/cometbft/issues/1097))
