- `[proto]` Remove stateful block data retrieval methods from the
  data companion gRPC API as per
  [RFC 106](https://github.com/cometbft/cometbft/blob/main/docs/references/rfc/rfc-106-separate-stateful-methods.md)
  ([\#2230](https://github.com/cometbft/cometbft/issues/2230)):
  * `GetLatest` from `cometbft.services.block.v1.BlockService`;
  * `GetLatestBlockResults` from `cometbft.services.block_results.v1.BlockResultsService`.
- `[rpc/grpc]` Remove support for stateful block data retrieval methods from the
  data companion APIs as per [RFC 106](https://github.com/cometbft/cometbft/blob/main/docs/references/rfc/rfc-106-separate-stateful-methods.md)
  * `GetLatestBlock` method removed from the `BlockServiceClient` interface.
  * `GetLatestBlockResults` method removed from the `BlockResultServiceClient` interface.
  * `GetLatest` endpoint is no longer served by `BlockServiceServer` instances.
  * `GetLatestBlockResults` endpoint is no longer served by `BlockResultServiceServer` instances.
