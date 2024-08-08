pragma solidity ^0.8.23;

import "forge-std/Test.sol";
import {IBCMsgs} from "../../../contracts/core/25-handler/IBCMsgs.sol";
import {
    IbcLightclientsMockV1ClientState as MockClientState,
    IbcLightclientsMockV1Header as MockHeader,
    IbcLightclientsMockV1ConsensusState as MockConsensusState,
    IbcCoreClientV1Height as ClientHeight
} from "../../../contracts/proto/MockClient.sol";
import {GoogleProtobufAny as Any} from
    "../../../contracts/proto/GoogleProtobufAny.sol";
import {
    GoogleProtobufDuration as Duration,
    GoogleProtobufTimestamp as Timestamp
} from "../../../contracts/proto/ProtoBufRuntime.sol";
import {
    IbcCoreChannelV1Counterparty as ChannelCounterparty,
    IbcCoreChannelV1Channel as Channel,
    IbcCoreChannelV1GlobalEnums as ChannelEnums,
    IbcCoreChannelV1Counterparty as ChannelCounterparty
} from "../../../contracts/proto/ibc/core/channel/v1/channel.sol";
import {
    IbcCoreConnectionV1Counterparty as ConnectionCounterparty,
    IbcCoreConnectionV1Version as ConnectionVersion,
    IbcCoreConnectionV1ConnectionEnd as ConnectionEnd,
    IbcCoreConnectionV1GlobalEnums as ConnectionEnums
} from "../../../contracts/proto/ibc/core/connection/v1/connection.sol";
import {IbcCoreCommitmentV1MerklePrefix as CommitmentMerklePrefix} from
    "../../../contracts/proto/ibc/core/commitment/v1/commitment.sol";
import {
    CometblsClientLib,
    OptimizedConsensusState
} from "../../../contracts/clients/CometblsClientV2.sol";
import {
    UnionIbcLightclientsCometblsV1ClientState as CometblsClientState,
    UnionIbcLightclientsCometblsV1Header as CometblsHeader,
    UnionIbcLightclientsCometblsV1Header as CometblsHeader,
    UnionIbcLightclientsCometblsV1LightHeader as CometblsLightHeader
} from
    "../../../contracts/proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import {IbcLightclientsWasmV1ClientState as WasmClientState} from
    "../../../contracts/proto/ibc/lightclients/wasm/v1/wasm.sol";
import {
    TendermintTypesCommit,
    TendermintTypesHeader,
    TendermintTypesSignedHeader,
    TendermintVersionConsensus
} from "../../../contracts/proto/tendermint/types/types.sol";
import {IbcLightclientsTendermintV1Fraction as Fraction} from
    "../../../contracts/proto/ibc/lightclients/tendermint/v1/tendermint.sol";

library Cometbls {
    using CometblsClientLib for *;

    uint64 constant HOUR = 3600;
    uint64 constant DAY = 24 * HOUR;
    uint64 constant WEEK = 7 * DAY;
    uint64 constant MONTH = 4 * WEEK;
    uint64 constant YEAR = 12 * MONTH;

    /*
     * The max clock drift allow the local chain clock to drift from the counterparty clock
     */
    uint64 constant MAX_CLOCK_DRIFT = HOUR;

    /*
     * The trusting period is the maximum difference in timestamp between two blocks that client will accept to process
     */
    uint64 constant TRUSTING_PERIOD = WEEK;

    function createClientState(
        string memory chainId,
        uint64 latestHeight
    ) internal pure returns (CometblsClientState.Data memory) {
        return CometblsClientState.Data({
            chain_id: chainId,
            // TODO: all this could be fuzzed
            trusting_period: TRUSTING_PERIOD * 1e9,
            unbonding_period: 300,
            max_clock_drift: MAX_CLOCK_DRIFT * 1e9,
            frozen_height: ClientHeight.Data({
                revision_number: 0,
                revision_height: 0
            }),
            latest_height: ClientHeight.Data({
                revision_number: 0,
                revision_height: latestHeight
            })
        });
    }

    function createConsensusState(
        bytes32 appHash,
        bytes32 nextValidatorsHash,
        uint64 timestamp
    ) internal pure returns (OptimizedConsensusState memory) {
        return OptimizedConsensusState({
            timestamp: timestamp,
            appHash: appHash,
            nextValidatorsHash: nextValidatorsHash
        });
    }

    function createClient(
        string memory clientType,
        string memory chainId,
        uint64 latestHeight,
        bytes32 appHash,
        bytes32 nextValidatorsHash,
        uint64 timestamp
    ) internal pure returns (IBCMsgs.MsgCreateClient memory m) {
        m.clientType = clientType;
        m.clientStateBytes =
            createClientState(chainId, latestHeight).encodeMemory();
        m.consensusStateBytes = createConsensusState(
            appHash, nextValidatorsHash, timestamp
        ).encodeMemory();
    }

    function updateClient(
        string memory clientId,
        CometblsLightHeader.Data memory signedHeader,
        uint64 trustedHeight,
        bytes memory zkp
    ) internal pure returns (IBCMsgs.MsgUpdateClient memory m) {
        m.clientId = clientId;
        m.clientMessage = CometblsHeader.Data({
            signed_header: signedHeader,
            trusted_height: ClientHeight.Data({
                revision_number: 0,
                revision_height: trustedHeight
            }),
            zero_knowledge_proof: zkp
        }).encodeMemory();
    }
}
