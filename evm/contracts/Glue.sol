pragma solidity ^0.8.23;

import "./core/02-client/ILightClient.sol";
import "./core/02-client/IBCHeight.sol";
import "./proto/ibc/core/client/v1/client.sol";
import "./proto/ibc/core/commitment/v1/commitment.sol";
import "./proto/ibc/core/connection/v1/connection.sol";
import "./proto/ibc/lightclients/tendermint/v1/tendermint.sol";
import "./proto/cosmos/ics23/v1/proofs.sol";
import "./proto/tendermint/types/types.sol";
import "./proto/tendermint/types/canonical.sol";
import "./proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import "./proto/union/ibc/lightclients/cosmosincosmos/v1/cosmosincosmos.sol";
import "./proto/ibc/lightclients/wasm/v1/wasm.sol";
import "./lib/Common.sol";
import "./clients/CosmosInCosmosClient.sol";
import "./clients/CometblsClientV2.sol";

contract Glue {
    function typesTelescope(
        UnionIbcLightclientsCometblsV1ClientState.Data memory,
        UnionIbcLightclientsCometblsV1ConsensusState.Data memory,
        UnionIbcLightclientsCometblsV1Header.Data memory,
        UnionIbcLightclientsCosmosincosmosV1ClientState.Data memory,
        UnionIbcLightclientsCosmosincosmosV1Header.Data memory,
        OptimizedCosmosInCosmosConsensusState memory,
        TendermintTypesHeader.Data memory,
        TendermintTypesCommit.Data memory,
        IbcCoreClientV1Height.Data memory,
        OptimizedConsensusState memory,
        ProcessedMoment memory,
        TendermintTypesCanonicalVote.Data memory,
        IbcLightclientsTendermintV1ClientState.Data memory,
        IbcLightclientsTendermintV1ConsensusState.Data memory,
        IbcLightclientsTendermintV1Header.Data memory,
        IbcCoreCommitmentV1MerkleProof.Data memory,
        IbcCoreConnectionV1ConnectionEnd.Data memory
    ) public pure {}
}
