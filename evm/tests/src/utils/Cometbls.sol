pragma solidity ^0.8.18;

import "forge-std/Test.sol";
import {IBCMsgs} from "../../../contracts/core/25-handler/IBCMsgs.sol";
import {IbcLightclientsMockV1ClientState as MockClientState, IbcLightclientsMockV1Header as MockHeader, IbcLightclientsMockV1ConsensusState as MockConsensusState, IbcCoreClientV1Height as ClientHeight} from "../../../contracts/proto/MockClient.sol";
import {GoogleProtobufAny as Any} from "../../../contracts/proto/GoogleProtobufAny.sol";
import {GoogleProtobufDuration as Duration, GoogleProtobufTimestamp as Timestamp} from "../../../contracts/proto/ProtoBufRuntime.sol";
import {IbcCoreChannelV1Counterparty as ChannelCounterparty, IbcCoreChannelV1Channel as Channel, IbcCoreChannelV1GlobalEnums as ChannelEnums, IbcCoreChannelV1Counterparty as ChannelCounterparty} from "../../../contracts/proto/ibc/core/channel/v1/channel.sol";
import {IbcCoreConnectionV1Counterparty as ConnectionCounterparty, IbcCoreConnectionV1Version as ConnectionVersion, IbcCoreConnectionV1ConnectionEnd as ConnectionEnd, IbcCoreConnectionV1GlobalEnums as ConnectionEnums} from "../../../contracts/proto/ibc/core/connection/v1/connection.sol";
import {IbcCoreCommitmentV1MerklePrefix as CommitmentMerklePrefix} from "../../../contracts/proto/ibc/core/commitment/v1/commitment.sol";
import {CometblsHelp, OptimizedConsensusState as CometblsConsensusState} from "../../../contracts/lib/CometblsHelp.sol";
import {UnionIbcLightclientsCometblsV1ClientState as CometblsClientState, UnionIbcLightclientsCometblsV1Header as CometblsHeader, UnionIbcLightclientsCometblsV1Header as CometblsHeader} from "../../../contracts/proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import {IbcLightclientsWasmV1ClientState as WasmClientState} from "../../../contracts/proto/ibc/lightclients/wasm/v1/wasm.sol";
import {TendermintTypesCommit, TendermintTypesHeader, TendermintTypesSignedHeader, TendermintVersionConsensus} from "../../../contracts/proto/tendermint/types/types.sol";
import {IbcLightclientsTendermintV1Fraction as Fraction} from "../../../contracts/proto/ibc/lightclients/tendermint/v1/tendermint.sol";

library Cometbls {
    using CometblsHelp for *;

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

    function createClient(
        string memory clientType,
        string memory chainId,
        uint64 revisionHeight,
        bytes32 appHash,
        bytes32 nextValidatorsHash,
        uint64 timestamp
    ) internal view returns (IBCMsgs.MsgCreateClient memory m) {
        m.clientType = clientType;
        m.clientStateBytes = CometblsClientState
            .Data({
                chain_id: chainId,
                // TODO: Unused and must be removed from the proto definition as it is hardcoded in the ZK circuit.
                trust_level: Fraction.Data({numerator: 1, denominator: 3}),
                // TODO: all this could be fuzzed
                trusting_period: Duration.Data({
                    Seconds: int64(TRUSTING_PERIOD),
                    nanos: 0
                }),
                unbonding_period: Duration.Data({Seconds: 300, nanos: 0}),
                max_clock_drift: Duration.Data({
                    Seconds: int64(MAX_CLOCK_DRIFT),
                    nanos: 0
                }),
                frozen_height: ClientHeight.Data({
                    revision_number: 0,
                    revision_height: 0
                })
            })
            .marshalToProto(
                ClientHeight.Data({
                    revision_number: 0,
                    revision_height: revisionHeight
                }),
                // NOTE: Cometbls wasm code_id from union, this data is required as per the IBC wasm-08 spec but unused in the counterparty side (our side)
                hex"CAFEBABE"
            );

        m.consensusStateBytes = CometblsConsensusState({
            root: appHash,
            nextValidatorsHash: nextValidatorsHash,
            timestamp: timestamp
        }).marshalToProto();
    }

    function updateClient(
        string memory clientId,
        TendermintTypesSignedHeader.Data memory signedHeader,
        uint64 trustedHeight,
        bytes memory untrustedValidatorsHash,
        bytes memory zkp
    ) internal view returns (IBCMsgs.MsgUpdateClient memory m) {
        m.clientId = clientId;
        m.clientMessage = CometblsHeader
            .Data({
                signed_header: signedHeader,
                untrusted_validator_set_root: untrustedValidatorsHash,
                trusted_height: ClientHeight.Data({
                    revision_number: 0,
                    revision_height: trustedHeight
                }),
                zero_knowledge_proof: zkp
            })
            .marshalHeaderEthABI();
    }
}
