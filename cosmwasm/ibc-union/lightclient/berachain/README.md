# Berachain (BeaconKit) Light Client

Berachain's consensus is provided by [BeaconKit], which is built using a modified version of the [CometBFT] consensus engine.

Similar to Ethereum's consensus, Berachain is actually two chains: a consensus and an execution chain. The consensus chain, implementing the [Ethereum Engine API], provides finality to the execution chain.

The BeaconKit lightclient first verifies the CometBFT conensus, verifying the aggregated signature in the commit, and then verifies the inclusion of the EVM state in the consensus node's state.

The execution state is verified by verifying an ICS-23 proof of the `LatestExecutionPayloadHeader`, which is stored [SSZ]-encoded under the [`LatestExecutionPayloadHeaderPrefix`] prefix, in the [`"beacon"`] store. Membership proofs are verified using proofs of the execution layer's state, using [`evm-storage-verifier`].

[BeaconKit]: https://github.com/berachain/beacon-kit
[CometBFT]: https://github.com/berachain/cometbft/tree/v1.x-bera
[Ethereum Engine API]: https://github.com/ethereum/execution-apis/tree/main/src/engine
[`LatestExecutionPayloadHeaderPrefix`]: https://github.com/berachain/beacon-kit/blob/360265c6a0dca8459d451eaf0dfbdb500d941342/storage/beacondb/keys/keys.go#L41
[`"beacon"`]: https://github.com/berachain/beacon-kit/blob/360265c6a0dca8459d451eaf0dfbdb500d941342/storage/kv_store_service.go#L33
[SSZ]: ../../../../ssz/README.md 
[`evm-storage-verifier`]: ../../../../lib/evm-storage-verifier
