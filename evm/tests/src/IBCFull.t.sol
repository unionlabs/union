pragma solidity ^0.8.18;

import "forge-std/Test.sol";
import "../../contracts/core/25-handler/IBCHandler.sol";
import "../../contracts/core/02-client/IBCClient.sol";
import "../../contracts/core/03-connection/IBCConnection.sol";
import "../../contracts/core/04-channel/IBCChannelHandshake.sol";
import "../../contracts/core/04-channel/IBCPacket.sol";
import "../../contracts/core/24-host/IBCCommitment.sol";
import "../../contracts/proto/ibc/core/connection/v1/connection.sol";
import "../../contracts/proto/ibc/core/channel/v1/channel.sol";
import "../../contracts/proto/ibc/lightclients/wasm/v1/wasm.sol";
import "../../contracts/proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";
import "../../contracts/proto/tendermint/types/canonical.sol";
import "../../contracts/lib/CometblsHelp.sol";
import "../../contracts/lib/Encoder.sol";
import "../../contracts/clients/TestnetVerifier.sol";
import "../../contracts/clients/CometblsClient.sol";
import "../../contracts/clients/ICS23MembershipVerifier.sol";
import "./TestableIBCHandler.t.sol";

contract IBCFullTest is Test {
    using CometblsHelp for TendermintTypesHeader.Data;
    using CometblsHelp for TendermintTypesHeader.Data;
    using CometblsHelp for TendermintTypesCommit.Data;
    using CometblsHelp for UnionIbcLightclientsCometblsV1ConsensusState.Data;
    using CometblsHelp for UnionIbcLightclientsCometblsV1ClientState.Data;
    using CometblsHelp for UnionIbcLightclientsCometblsV1Header.Data;
    using CometblsHelp for OptimizedConsensusState;
    using CometblsHelp for bytes;

    TestableIBCHandler handler;

    string constant CHAIN_ID = "union-devnet-1";
    string constant CLIENT_TYPE = "cometbls";
    string constant CLIENT_ID = "cometbls-0";
    string constant CONNECTION_ID = "cometbls-0";
    string constant CHANNEL_ID = "cometbls-0";
    string constant PORT_ID = "cometbls:0";
    string constant CHANNEL_VERSION = "1";
    bytes constant MERKLE_PREFIX = "ibc";

    bytes32 constant GENESIS_APP_ROOT =
        hex"03B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B850";
    bytes32 constant GENESIS_VALIDATOR_ROOT =
        hex"811594B875D1BF0C7992459AE166367C094CB7EAD07DF3F849F4D01EBFBF9A07";

    function setUp() public {
        address ibcClient = address(new IBCClient());
        address ibcConnection = address(new IBCConnection());
        address ibcChannelHandshake = address(new IBCChannelHandshake());
        address ibcPacket = address(new IBCPacket());
        handler = new TestableIBCHandler(
            ibcClient,
            ibcConnection,
            ibcChannelHandshake,
            ibcPacket
        );
        handler.registerClient(
            CLIENT_TYPE,
            new CometblsClient(
                address(handler),
                new TestnetVerifier(),
                new ICS23MembershipVerifier()
            )
        );
        setUpClient();
        setUpConnection();
        setUpChannel();
    }

    function setUpClient() internal {
        createClient(0, GENESIS_VALIDATOR_ROOT, GENESIS_APP_ROOT);
    }

    function getConnectionVersions()
        internal
        pure
        returns (IbcCoreConnectionV1Version.Data[] memory)
    {
        IbcCoreConnectionV1Version.Data[]
            memory versions = new IbcCoreConnectionV1Version.Data[](1);
        string[] memory features = new string[](2);
        features[0] = "ORDER_ORDERED";
        features[1] = "ORDER_UNORDERED";
        versions[0] = IbcCoreConnectionV1Version.Data({
            identifier: "1",
            features: features
        });
        return versions;
    }

    function setUpConnection() internal {
        handler.setConnection(
            CONNECTION_ID,
            IbcCoreConnectionV1ConnectionEnd.Data({
                client_id: CLIENT_ID,
                versions: getConnectionVersions(),
                state: IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN,
                delay_period: 0,
                counterparty: IbcCoreConnectionV1Counterparty.Data({
                    client_id: CLIENT_ID,
                    connection_id: CONNECTION_ID,
                    prefix: IbcCoreCommitmentV1MerklePrefix.Data({
                        key_prefix: MERKLE_PREFIX
                    })
                })
            })
        );
        handler.setNextConnectionSequence(1);
    }

    function setUpChannel() internal {
        string[] memory hops = new string[](1);
        hops[0] = CONNECTION_ID;
        handler.setChannel(
            PORT_ID,
            CHANNEL_ID,
            IbcCoreChannelV1Channel.Data({
                state: IbcCoreChannelV1GlobalEnums.State.STATE_OPEN,
                ordering: IbcCoreChannelV1GlobalEnums.Order.ORDER_UNORDERED,
                counterparty: IbcCoreChannelV1Counterparty.Data({
                    port_id: PORT_ID,
                    channel_id: CHANNEL_ID
                }),
                connection_hops: hops,
                version: CHANNEL_VERSION
            })
        );
        handler.setNextChannelSequence(1);
        handler.setNextSequenceSend(PORT_ID, CHANNEL_ID, 1);
        handler.setNextSequenceRecv(PORT_ID, CHANNEL_ID, 1);
        handler.setNextSequenceAck(PORT_ID, CHANNEL_ID, 1);
    }

    function createClient(
        uint64 height,
        bytes32 nextValidatorsHash,
        bytes32 rootHash
    ) internal {
        handler.createClient(
            IBCMsgs.MsgCreateClient({
                clientType: CLIENT_TYPE,
                clientStateBytes: UnionIbcLightclientsCometblsV1ClientState
                    .Data({
                        chain_id: CHAIN_ID,
                        trust_level: UnionIbcLightclientsCometblsV1Fraction
                            .Data({numerator: 1, denominator: 3}),
                        trusting_period: GoogleProtobufDuration.Data({
                            Seconds: 300,
                            nanos: 0
                        }),
                        unbonding_period: GoogleProtobufDuration.Data({
                            Seconds: 300,
                            nanos: 0
                        }),
                        max_clock_drift: GoogleProtobufDuration.Data({
                            Seconds: 3600,
                            nanos: 0
                        }),
                        frozen_height: IbcCoreClientV1Height.Data({
                            revision_number: 0,
                            revision_height: 0
                        })
                    })
                    .marshalToProto(
                        IbcCoreClientV1Height.Data({
                            revision_number: 0,
                            revision_height: height
                        }),
                        hex"CAFEBABE"
                    ),
                consensusStateBytes: OptimizedConsensusState({
                    root: rootHash,
                    nextValidatorsHash: nextValidatorsHash,
                    timestamp: 1682000000
                }).marshalToProto()
            })
        );
    }

    function updateClient(bytes memory clientMessage) internal {
        handler.updateClient(
            IBCMsgs.MsgUpdateClient({
                clientId: CLIENT_ID,
                clientMessage: clientMessage
            })
        );
    }

    function testCreateClient() public {
        createClient(0, GENESIS_VALIDATOR_ROOT, GENESIS_APP_ROOT);
    }

    function testUpdateClient() public {
        // FIXME, data outdated as of the circuit upgrade
        /* bytes memory partSetHeaderHash = hex"39C604A64DDBDA8F2E0F31F0DF30315CE4B8E65DB91F74F29A5ED6926C70A03F"; */
        /* TendermintTypesHeader.Data memory header = TendermintTypesHeader.Data({ */
        /*     version: TendermintVersionConsensus.Data({ */
        /*         block: 11, */
        /*         app: 0 */
        /*         }), */
        /*     chain_id: "union-devnet-1", */
        /*     height: 1, */
        /*     time: GoogleProtobufTimestamp.Data({ */
        /*         secs: 1682000030, */
        /*         nanos: 835848794 */
        /*         }), */
        /*     last_block_id: TendermintTypesBlockID.Data({ */
        /*         hash: bytes(""), */
        /*         part_set_header: TendermintTypesPartSetHeader.Data({ */
        /*             total: 0, */
        /*             hash: bytes("") */
        /*             }) */
        /*         }), */
        /*     last_commit_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855", */
        /*     data_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855", */
        /*     validators_hash: hex"811594B875D1BF0C7992459AE166367C094CB7EAD07DF3F849F4D01EBFBF9A07", */
        /*     next_validators_hash: hex"811594B875D1BF0C7992459AE166367C094CB7EAD07DF3F849F4D01EBFBF9A07", */
        /*     consensus_hash: hex"048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F", */
        /*     app_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855", */
        /*     last_results_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855", */
        /*     evidence_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855", */
        /*     proposer_address: hex"02BE8D2FFE4B72308F7FA92D7C9E6DC3509CD4AD" */
        /*     }); */
        /* bytes memory blockHash = abi.encodePacked(header.merkleRoot()); */
        /* bytes memory zero_knowledge_proof = hex"1024a565ecf146aab820b3f92b98af07ed6e9483aaddae75ed170ddbe4ad650e21f87beb59edd575f4bf87e6ecc5f4e65d969108ba648e827acd47bae6f1f4a62506c89e22c35f465a4a0e6b196bb4b279c8ffbfe3e976c70febc2676ad8f2760b0aebc3e026c5426bedf9ef5a1123dd8791f312fa1b495e84111f59f3b795a82989e1335c662e2e641314c437e5d87688a4f065b95310f722eb7fc033d5f4212a33f014593cbc4cef8e01b9f6b65459d6b0e9d7cc5ddf0b24183fc8260634031d97b77ba34a2c35bd7b615fbb545a47e955619fd25f0d084ea3a273c84189e31cc912765a00b0ad04fef92f04fa5dd88494178e6b940264e41bfacf182e1e310320a02aaf8831d5ca039b6d1fe9adb2959c41ae5c0752750147440c5c7624b9034c55a9e1aabd27fd8edb56200ccd3de9f2dbb7f160ed38fe188d5cf9bb384d010bc55236131830f6eff8d3b9e992a70cbe2ebb9bea52f70b598e9b8eaab713"; */
        /* bytes memory untrustedValidatorsHash = hex"811594B875D1BF0C7992459AE166367C094CB7EAD07DF3F849F4D01EBFBF9A07"; */
        /* UnionIbcLightclientsCometblsV1Header.Data memory cometblsHeader = UnionIbcLightclientsCometblsV1Header.Data({ */
        /*         signed_header: TendermintTypesSignedHeader.Data({ */
        /*             header: header, */
        /*             commit:TendermintTypesCommit.Data({ */
        /*                 height: 1, */
        /*                 round: 0, */
        /*                 block_id: TendermintTypesBlockID.Data({ */
        /*                     hash: blockHash, */
        /*                     part_set_header: TendermintTypesPartSetHeader.Data({ */
        /*                         total: 1, */
        /*                         hash: partSetHeaderHash */
        /*                         }) */
        /*                     }), */
        /*                 signatures: new TendermintTypesCommitSig.Data[](0) */
        /*                 }) */
        /*             }), */
        /*         untrusted_validator_set_root: untrustedValidatorsHash, */
        /*         trusted_height: IbcCoreClientV1Height.Data({ */
        /*             revision_number: 0, */
        /*             revision_height: 0 */
        /*             }), */
        /*         zero_knowledge_proof: zero_knowledge_proof */
        /*         }); */
        /* vm.warp(1682000040); */
        /* bytes memory clientMessage = cometblsHeader.marshalHeaderEthABI(); */
        /* uint256 gas = gasleft(); */
        /* updateClient(clientMessage); */
        /* console.log("IBCFull.updateClient(): ", gas - gasleft()); */
    }
}
