pragma solidity ^0.8.23;

import {ILightClient, ConsensusStateUpdate} from "../../../contracts/core/02-client/ILightClient.sol";
import {IZKVerifierV2} from "../../../contracts/core/IZKVerifierV2.sol";
import {CometblsClient} from "../../../contracts/clients/CometblsClientV2.sol";
import {IBCMsgs} from "../../../contracts/core/25-handler/IBCMsgs.sol";
import {IBCCommitment} from "../../../contracts/core/24-host/IBCCommitment.sol";
import {CometblsHelp} from "../../../contracts/lib/CometblsHelp.sol";
import {IMembershipVerifier} from "../../../contracts/core/IMembershipVerifier.sol";
import {IbcCoreClientV1Height} from "../../../contracts/proto/ibc/core/client/v1/client.sol";

import {IBCHandler_Testable} from "../utils/IBCHandler_Testable.sol";

import {TendermintTypesSignedHeader} from "../../../contracts/proto/tendermint/types/canonical.sol";
import {TendermintTypesCommit, TendermintTypesHeader, TendermintTypesSignedHeader, TendermintVersionConsensus, TendermintTypesCommitSig, TendermintTypesBlockID, TendermintTypesPartSetHeader} from "../../../contracts/proto/tendermint/types/types.sol";

import "../TestPlus.sol";
import "solady/utils/LibString.sol";
import "solidity-bytes-utils/BytesLib.sol";

contract TestVerifier is IZKVerifierV2 {
    uint256 valid = 0;

    function pushValid() public {
        valid += 1;
    }

    function verifyProof(
        uint256[8] memory proof,
        uint256[2] memory proofCommitment,
        uint256[5] calldata input
    ) external returns (bool) {
        bool ok = valid > 0;
        if (valid > 0) {
            valid -= 1;
        }
        return ok;
    }
}

contract TestMembershipVerifier is IMembershipVerifier {
    uint256 valid = 0;

    function pushValid() public {
        valid += 1;
    }

    function verifyMembership(
        bytes memory root,
        bytes calldata proof,
        bytes memory prefix,
        bytes calldata path,
        bytes calldata value
    ) external returns (bool) {
        bool ok = valid > 0;
        if (valid > 0) {
            valid -= 1;
        }
        return ok;
    }

    function verifyNonMembership(
        bytes memory root,
        bytes calldata proof,
        bytes calldata prefix,
        bytes calldata path
    ) external returns (bool) {
        bool ok = valid > 0;
        if (valid > 0) {
            valid -= 1;
        }
        return ok;
    }
}

contract IBCClientHandlerTests is TestPlus {
    using BytesLib for bytes;
    using CometblsHelp for *;

    IBCHandler_Testable handler;

    string constant CLIENT_TYPE = "mock";
    ILightClient client;
    ILightClient client2;

    TestVerifier verifier;
    TestMembershipVerifier membershipVerifier;

    bytes32 constant ARBITRARY_INITIAL_APP_HASH =
        hex"A8158610DD6858F3D26149CC0DB3339ABD580EA217DE0A151C9C451DED418E35";

    constructor() {
        handler = new IBCHandler_Testable();
        membershipVerifier = new TestMembershipVerifier();
        verifier = new TestVerifier();

        client = new CometblsClient(
            address(handler),
            verifier,
            membershipVerifier
        );
        client2 = new CometblsClient(
            address(handler),
            verifier,
            membershipVerifier
        );

        vm.warp(1);
    }

    function getValidTransition()
        internal
        pure
        returns (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        )
    {
        zkp = hex"09f57b8b308d9c57bd1e30cd493212314f5b680e685bc91402193fac45389c42064968d6db298707b5405431621b96bd73756907b2b5137ca4966c270d0b9b461a60936e4cdf9b77b993f25cdeb7d1c5623f082dc2c88b20d33a9b40c14dc39115aba4e371dc0443465b4d9b69aece3a4f15f0503c6d0f56dc1237356c32de80271ca20c7eb2bcb9bc56be7256a93d925fa3bae73829dbc53c4e9056f99046b80277d0bbc45741e6eb1e6a6a9e1d795f384cca1d3836e29ffecdebf6b0a9db5e15ec13c943d68283a8a781f4d5cb330ca1b02a7515990eb8c3c3e4da4ba9ef1717980acd29ff4c6ba58036337faae8def7355243b2449b9c5637f85ebecec1a42e4570af6b520476ccc96665d3d92dc7a22c0864169072332f17f7664223ea4004e3860aa093da597dc7f6b28284c45d9bc56e4d8e44ee5b784ec23b5309649116db1c88de8aaa9057b41b94939806fd8910bc1d5b33f3d4db7f568197c63f55";
        TendermintTypesHeader.Data memory header = TendermintTypesHeader.Data({
            version: TendermintVersionConsensus.Data({block: 11, app: 0}),
            chain_id: "union-devnet-1",
            height: 139,
            time: Timestamp.Data({secs: 1691496777, nanos: 793918988}),
            last_block_id: TendermintTypesBlockID.Data({
                hash: hex"80DF3A892BF2586E3B22201D2AC5A65EDAB66ECE7BB6F51077F3B50CCE7526E1",
                part_set_header: TendermintTypesPartSetHeader.Data({
                    total: 1,
                    hash: hex"0468D541CAD891D571E2AD1DD9F43480993BDF18A1016F4C956555A417EFE681"
                })
            }),
            last_commit_hash: hex"DA6FCBD48131808D58B54E8B44737AB2B6F3A3DD1AFF946D0F6CEFD25306FD48",
            data_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            validators_hash: hex"F09E25471B41514B2F8B08B5F4C9093C5D6ED134E107FF491CED2374B947DF60",
            next_validators_hash: hex"F09E25471B41514B2F8B08B5F4C9093C5D6ED134E107FF491CED2374B947DF60",
            consensus_hash: hex"048091BC7DDC283F77BFBF91D73C44DA58C3DF8A9CBC867405D8B7F3DAADA22F",
            app_hash: hex"983EF85676937CEC783601B5B50865733A72C3DF88E4CC0B3F11C108C9688459",
            last_results_hash: hex"357B78587B9CD4469F1F63C29B96EAC1D7F643520B97D396B20A20505122AA01",
            evidence_hash: hex"E3B0C44298FC1C149AFBF4C8996FB92427AE41E4649B934CA495991B7852B855",
            proposer_address: hex"4CE57693C82B50F830731DAB14FA759327762456"
        });
        signedHeader = TendermintTypesSignedHeader.Data({
            header: header,
            // NOTE: validators are signing the block root which is computed
            // from the above TendermintTypesHeader field. Relayers can tamper
            // the commit but the client ensure that the block_id.hash matches
            // hash(header). Signatures are not required as the ZKP is a proof
            // that the validators signed the correct hash.
            commit: TendermintTypesCommit.Data({
                height: header.height,
                round: 0,
                block_id: TendermintTypesBlockID.Data({
                    hash: hex"90548CD1E2BA8603261FE6400ADFD97EA48150CBD5B24B9828B2542BAB9E27EC",
                    part_set_header: TendermintTypesPartSetHeader.Data({
                        total: 1,
                        hash: hex"153E8B1F5B189A140FE5DA85DAB72FBD4A1DFA7E69C6FE5CE1FD66F0CCB5F6A1"
                    })
                }),
                signatures: new TendermintTypesCommitSig.Data[](0)
            })
        });
    }

    function test_registerClient() public {
        handler.registerClient(CLIENT_TYPE, client);
        handler.registerClient("other", client2);

        assertEq(handler.clientRegistry(CLIENT_TYPE), address(client));
        assertEq(handler.clientRegistry("other"), address(client2));
    }

    function test_registerClient_alreadyRegistered() public {
        handler.registerClient(CLIENT_TYPE, client);

        vm.expectRevert("registerClient: client type already exists");
        handler.registerClient(CLIENT_TYPE, client);
    }

    function test_registerClient_self() public {
        vm.expectRevert("registerClient: must not be self");
        handler.registerClient(CLIENT_TYPE, ILightClient(address(handler)));
    }

    function test_createClient(
        string memory chainId,
        uint64 revisionHeight,
        bytes32 rootHash,
        bytes32 nextValidatorsHash
    ) public {
        vm.assume(revisionHeight > 0);

        handler.registerClient(CLIENT_TYPE, client);
        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            chainId,
            revisionHeight,
            rootHash,
            nextValidatorsHash,
            uint64(vm.getBlockTimestamp())
        );

        string memory id = handler.createClient(m);

        assertEq(handler.clientTypes(id), m.clientType);
        assertEq(handler.clientImpls(id), address(client));
        assertEq(
            handler.commitments(keccak256(IBCCommitment.clientStatePath(id))),
            keccak256(m.clientStateBytes)
        );
        assertEq(
            handler.commitments(
                IBCCommitment.consensusStateCommitmentKey(id, 0, revisionHeight)
            ),
            keccak256(m.consensusStateBytes)
        );
    }

    function test_createClient_zeroHeight(
        string memory chainId,
        bytes32 rootHash,
        bytes32 nextValidatorsHash
    ) public {
        handler.registerClient(CLIENT_TYPE, client);
        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            chainId,
            0,
            rootHash,
            nextValidatorsHash,
            uint64(vm.getBlockTimestamp())
        );

        vm.expectRevert("createClient: failed to create client");
        handler.createClient(m);
    }

    function test_createClient_invalidType(
        string memory chainId,
        uint64 revisionHeight,
        bytes32 rootHash,
        bytes32 nextValidatorsHash
    ) public {
        handler.registerClient(CLIENT_TYPE, client);
        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            "other",
            chainId,
            revisionHeight,
            rootHash,
            nextValidatorsHash,
            uint64(vm.getBlockTimestamp())
        );

        vm.expectRevert("createClient: unregistered client type");
        handler.createClient(m);
    }

    function test_createClient_onlyIBC(
        uint64 trustedHeight,
        uint64 clockDrift,
        uint64 updateLatency
    ) public {
        vm.assume(clockDrift < Cometbls.MAX_CLOCK_DRIFT);
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight &&
                trustedHeight < uint64(signedHeader.header.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        verifier.pushValid();
        vm.expectRevert("LC: unauthorized");
        client.createClient(
            "blabla",
            m.clientStateBytes,
            m.consensusStateBytes
        );
    }

    function test_updateClient_onlyIBC(
        uint64 trustedHeight,
        uint64 clockDrift,
        uint64 updateLatency
    ) public {
        vm.assume(clockDrift < Cometbls.MAX_CLOCK_DRIFT);
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight &&
                trustedHeight < uint64(signedHeader.header.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory id = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            id,
            signedHeader,
            trustedHeight,
            zkp
        );

        vm.warp(uint64(signedHeader.header.time.secs) + updateLatency);

        verifier.pushValid();
        vm.expectRevert("LC: unauthorized");
        client.updateClient(m2.clientId, m2.clientMessage);
    }

    function test_updateClient_newCommitment(
        uint64 trustedHeight,
        uint64 clockDrift,
        uint64 updateLatency
    ) public {
        vm.assume(clockDrift < Cometbls.MAX_CLOCK_DRIFT);
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight &&
                trustedHeight < uint64(signedHeader.header.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory id = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            id,
            signedHeader,
            trustedHeight,
            zkp
        );

        vm.warp(uint64(signedHeader.header.time.secs) + updateLatency);

        verifier.pushValid();
        vm.prank(address(handler));
        (
            bytes32 clientStateCommitment,
            ConsensusStateUpdate[] memory updates,
            bool ok
        ) = client.updateClient(m2.clientId, m2.clientMessage);
        assertTrue(ok);
        assertEq(
            clientStateCommitment,
            keccak256(
                Cometbls
                    .createClientState(
                        signedHeader.header.chain_id,
                        uint64(signedHeader.header.height)
                    )
                    .marshalEthABI()
            )
        );
        assertEq(updates.length, 1);
        assertEq(
            updates[0].consensusStateCommitment,
            keccak256(
                Cometbls
                    .createConsensusState(
                        signedHeader.header.app_hash.toBytes32(0),
                        signedHeader.header.validators_hash.toBytes32(0),
                        uint64(signedHeader.header.time.secs)
                    )
                    .marshalEthABI()
            )
        );
        assertEq(
            updates[0].height.revision_height,
            uint64(signedHeader.header.height)
        );
    }

    function test_updateClient_validZKP(
        uint64 trustedHeight,
        uint64 clockDrift,
        uint64 updateLatency
    ) public {
        vm.assume(clockDrift < Cometbls.MAX_CLOCK_DRIFT);
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight &&
                trustedHeight < uint64(signedHeader.header.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory id = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            id,
            signedHeader,
            trustedHeight,
            zkp
        );

        vm.warp(uint64(signedHeader.header.time.secs) + updateLatency);

        verifier.pushValid();
        handler.updateClient(m2);
    }

    function test_updateClient_invalidZKP(
        uint64 trustedHeight,
        uint64 clockDrift,
        uint64 updateLatency
    ) public {
        vm.assume(clockDrift < Cometbls.MAX_CLOCK_DRIFT);
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight &&
                trustedHeight < uint64(signedHeader.header.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory id = handler.createClient(m);

        // Tamper the ZKP
        zkp[0] = 0xCA;
        zkp[1] = 0xFE;
        zkp[2] = 0xBA;
        zkp[3] = 0xBE;

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            id,
            signedHeader,
            trustedHeight,
            zkp
        );

        vm.warp(uint64(signedHeader.header.time.secs) + updateLatency);

        vm.expectRevert();
        handler.updateClient(m2);
    }

    // Test that the on-chain hash(signedHeader.header) is equal to the off-chain provided signedHeader.commit.block_id.hash.
    // NOTE: Optimization-wise, this is probably unecessary as we direcyly compute that on chain. i.e. we could remove commit.block_id entirely.
    function test_updateClient_invalidBlockRoot(
        uint64 trustedHeight,
        uint64 clockDrift,
        uint64 updateLatency
    ) public {
        vm.assume(clockDrift < Cometbls.MAX_CLOCK_DRIFT);
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight &&
                trustedHeight < uint64(signedHeader.header.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory id = handler.createClient(m);

        // Tamper the header such that the block root != commit, as if a relayer tampered the commit or the block.
        signedHeader.header.last_block_id.part_set_header.total = 0xC0DE;

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            id,
            signedHeader,
            trustedHeight,
            zkp
        );

        vm.warp(uint64(signedHeader.header.time.secs) + updateLatency);

        vm.expectRevert("LC: commit.block_id.hash != header.root()");
        handler.updateClient(m2);
    }

    function test_updateClient_nextRevisionLower(
        uint64 trustedHeight,
        uint64 clockDrift,
        uint64 updateLatency
    ) public {
        vm.assume(clockDrift < Cometbls.MAX_CLOCK_DRIFT);
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(trustedHeight > uint64(signedHeader.header.height));

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory id = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            id,
            signedHeader,
            trustedHeight,
            zkp
        );

        vm.warp(uint64(signedHeader.header.time.secs) + updateLatency);

        vm.expectRevert("LC: header height <= consensus state height");
        handler.updateClient(m2);
    }

    function test_updateClient_trustingPeriodExpired(
        uint64 trustedHeight,
        uint64 clockDrift
    ) public {
        vm.assume(clockDrift < Cometbls.MAX_CLOCK_DRIFT);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight &&
                trustedHeight < uint64(signedHeader.header.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory id = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            id,
            signedHeader,
            trustedHeight,
            zkp
        );

        // The block timestamp will be out of the trusting period window
        vm.warp(
            uint64(signedHeader.header.time.secs) + Cometbls.TRUSTING_PERIOD + 1
        );

        vm.expectRevert("LC: header expired");
        handler.updateClient(m2);
    }

    function test_getTimestampAtHeight() public {
        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;
        uint64 clockDrift = Cometbls.MAX_CLOCK_DRIFT - 1;
        uint64 updateLatency = Cometbls.TRUSTING_PERIOD - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            clientId,
            signedHeader,
            trustedHeight,
            zkp
        );

        vm.warp(uint64(signedHeader.header.time.secs) + updateLatency);

        verifier.pushValid();
        handler.updateClient(m2);

        (uint64 timestamp, bool ok) = client.getTimestampAtHeight(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: uint64(signedHeader.header.height)
            })
        );
        assertTrue(ok);
        assertEq(timestamp, uint64(signedHeader.header.time.secs));
    }

    function test_getClientState() public {
        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        (bytes memory clientStateBytes, bool ok) = client.getClientState(
            clientId
        );
        assertTrue(ok);
        assertEq(clientStateBytes, m.clientStateBytes);
    }

    function test_getClientState_noClientState() public {
        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        (bytes memory clientStateBytes, bool ok) = client.getClientState(
            "blabla"
        );
        assertFalse(ok);
    }

    function test_getClientState_step(
        uint64 trustedHeight,
        uint64 clockDrift,
        uint64 updateLatency
    ) public {
        vm.assume(clockDrift < Cometbls.MAX_CLOCK_DRIFT);
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight &&
                trustedHeight < uint64(signedHeader.header.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            clientId,
            signedHeader,
            trustedHeight,
            zkp
        );

        vm.warp(uint64(signedHeader.header.time.secs) + updateLatency);

        (bytes memory clientStateBytes, bool ok) = client.getClientState(
            clientId
        );
        assertTrue(ok);
        assertEq(clientStateBytes, m.clientStateBytes);

        verifier.pushValid();
        handler.updateClient(m2);

        (clientStateBytes, ok) = client.getClientState(clientId);
        assertTrue(ok);
        assertEq(
            clientStateBytes,
            Cometbls
                .createClientState(
                    signedHeader.header.chain_id,
                    uint64(signedHeader.header.height)
                )
                .marshalEthABI()
        );
    }

    function test_getConsensusState() public {
        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        (bytes memory consensusStateBytes, bool ok) = client.getConsensusState(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            })
        );
        assertTrue(ok);
        assertEq(
            consensusStateBytes,
            Cometbls
                .createConsensusState(
                    ARBITRARY_INITIAL_APP_HASH,
                    signedHeader.header.validators_hash.toBytes32(0),
                    uint64(signedHeader.header.time.secs - 10)
                )
                .marshalEthABI()
        );
    }

    function test_getConsensusState_noConsensus() public {
        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        (bytes memory consensusStateBytes, bool ok) = client.getConsensusState(
            "blabla",
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            })
        );
        assertFalse(ok);
    }

    function test_getConsensusState_step(
        uint64 trustedHeight,
        uint64 clockDrift,
        uint64 updateLatency
    ) public {
        vm.assume(clockDrift < Cometbls.MAX_CLOCK_DRIFT);
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight &&
                trustedHeight < uint64(signedHeader.header.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            clientId,
            signedHeader,
            trustedHeight,
            zkp
        );

        vm.warp(uint64(signedHeader.header.time.secs) + updateLatency);

        (bytes memory consensusStateBytes, bool ok) = client.getConsensusState(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: uint64(signedHeader.header.height)
            })
        );
        assertFalse(ok);

        verifier.pushValid();
        handler.updateClient(m2);

        (consensusStateBytes, ok) = client.getConsensusState(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: uint64(signedHeader.header.height)
            })
        );
        assertTrue(ok);
        assertEq(
            consensusStateBytes,
            Cometbls
                .createConsensusState(
                    signedHeader.header.app_hash.toBytes32(0),
                    signedHeader.header.validators_hash.toBytes32(0),
                    uint64(signedHeader.header.time.secs)
                )
                .marshalEthABI()
        );
    }

    function test_verifyMembership_noConsensus() public {
        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        vm.expectRevert("LC: verifyMembership: consensusState does not exist");
        client.verifyMembership(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight - 1
            }),
            0,
            0,
            bytes(""),
            bytes(""),
            bytes(""),
            bytes("")
        );
    }

    function test_verifyMembership_ok() public {
        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        membershipVerifier.pushValid();
        bool ok = client.verifyMembership(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            }),
            0,
            0,
            bytes(""),
            bytes(""),
            bytes(""),
            bytes("")
        );
        assertTrue(ok);
    }

    function test_verifyMembership_ko() public {
        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        bool ok = client.verifyMembership(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            }),
            0,
            0,
            bytes(""),
            bytes(""),
            bytes(""),
            bytes("")
        );
        assertFalse(ok);
    }

    function test_verifyMembership_delayPeriodNotExpired_time(
        uint64 delayPeriodTime,
        uint64 delayTime
    ) public {
        vm.assume(0 < delayPeriodTime && delayPeriodTime < 360000);
        vm.assume(0 < delayTime && delayTime < delayPeriodTime);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        vm.warp(vm.getBlockTimestamp() + delayTime);

        vm.expectRevert("LC: delayPeriod not expired");
        client.verifyMembership(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            }),
            delayPeriodTime,
            0,
            bytes(""),
            bytes(""),
            bytes(""),
            bytes("")
        );
    }

    function test_verifyMembership_delayPeriodNotExpired_block(
        uint64 delayPeriodBlocks,
        uint64 delayBlocks
    ) public {
        vm.assume(0 < delayPeriodBlocks && delayPeriodBlocks < 1000000);
        vm.assume(0 < delayBlocks && delayBlocks < delayPeriodBlocks);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        vm.warp(vm.getBlockNumber() + delayBlocks);

        vm.expectRevert("LC: delayPeriod not expired");
        client.verifyMembership(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            }),
            0,
            delayPeriodBlocks,
            bytes(""),
            bytes(""),
            bytes(""),
            bytes("")
        );
    }

    function test_verifyMembership_delayPeriodExpired_time(
        uint64 delayPeriodTime
    ) public {
        vm.assume(0 < delayPeriodTime && delayPeriodTime < 360000);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        vm.warp(vm.getBlockTimestamp() + delayPeriodTime);

        membershipVerifier.pushValid();
        bool ok = client.verifyMembership(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            }),
            delayPeriodTime,
            0,
            bytes(""),
            bytes(""),
            bytes(""),
            bytes("")
        );
        assertTrue(ok);
    }

    function test_verifyMembership_delayPeriodExpired_block(
        uint64 delayPeriodBlocks
    ) public {
        vm.assume(0 < delayPeriodBlocks && delayPeriodBlocks < 1000000);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        vm.roll(vm.getBlockNumber() + delayPeriodBlocks);

        membershipVerifier.pushValid();
        bool ok = client.verifyMembership(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            }),
            0,
            delayPeriodBlocks,
            bytes(""),
            bytes(""),
            bytes(""),
            bytes("")
        );
        assertTrue(ok);
    }

    function test_verifyNonMembership_ok() public {
        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        membershipVerifier.pushValid();
        bool ok = client.verifyNonMembership(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            }),
            0,
            0,
            bytes(""),
            bytes(""),
            bytes("")
        );
        assertTrue(ok);
    }

    function test_verifyNonMembership_ko() public {
        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        bool ok = client.verifyNonMembership(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            }),
            0,
            0,
            bytes(""),
            bytes(""),
            bytes("")
        );
        assertFalse(ok);
    }

    function test_verifyNonMembership_delayPeriodNotExpired_time(
        uint64 delayPeriodTime,
        uint64 delayTime
    ) public {
        vm.assume(0 < delayPeriodTime && delayPeriodTime < 360000);
        vm.assume(0 < delayTime && delayTime < delayPeriodTime);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        vm.warp(vm.getBlockTimestamp() + delayTime);

        vm.expectRevert("LC: delayPeriod not expired");
        client.verifyNonMembership(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            }),
            delayPeriodTime,
            0,
            bytes(""),
            bytes(""),
            bytes("")
        );
    }

    function test_verifyNonMembership_delayPeriodNotExpired_block(
        uint64 delayPeriodBlocks,
        uint64 delayBlocks
    ) public {
        vm.assume(0 < delayPeriodBlocks && delayPeriodBlocks < 1000000);
        vm.assume(0 < delayBlocks && delayBlocks < delayPeriodBlocks);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        vm.warp(vm.getBlockNumber() + delayBlocks);

        vm.expectRevert("LC: delayPeriod not expired");
        client.verifyNonMembership(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            }),
            0,
            delayPeriodBlocks,
            bytes(""),
            bytes(""),
            bytes("")
        );
    }

    function test_verifyNonMembership_delayPeriodExpired_time(
        uint64 delayPeriodTime
    ) public {
        vm.assume(0 < delayPeriodTime && delayPeriodTime < 360000);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        vm.warp(vm.getBlockTimestamp() + delayPeriodTime);

        membershipVerifier.pushValid();
        bool ok = client.verifyNonMembership(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            }),
            delayPeriodTime,
            0,
            bytes(""),
            bytes(""),
            bytes("")
        );
        assertTrue(ok);
    }

    function test_verifyNonMembership_delayPeriodExpired_block(
        uint64 delayPeriodBlocks
    ) public {
        vm.assume(0 < delayPeriodBlocks && delayPeriodBlocks < 1000000);

        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        vm.roll(vm.getBlockNumber() + delayPeriodBlocks);

        membershipVerifier.pushValid();
        bool ok = client.verifyNonMembership(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            }),
            0,
            delayPeriodBlocks,
            bytes(""),
            bytes(""),
            bytes("")
        );
        assertTrue(ok);
    }

    function test_getLatestHeight_noClientState() public {
        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;
        uint64 clockDrift = Cometbls.MAX_CLOCK_DRIFT - 1;
        uint64 updateLatency = Cometbls.TRUSTING_PERIOD - 1;

        handler.registerClient(CLIENT_TYPE, client);

        (IbcCoreClientV1Height.Data memory latestHeight, bool ok) = client
            .getLatestHeight("blabla");
        assertFalse(ok);
    }

    function test_getLatestHeight_ok() public {
        (
            bytes memory zkp,
            TendermintTypesSignedHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.header.height) - 1;
        uint64 clockDrift = Cometbls.MAX_CLOCK_DRIFT - 1;
        uint64 updateLatency = Cometbls.TRUSTING_PERIOD - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            signedHeader.header.chain_id,
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.header.validators_hash.toBytes32(0),
            uint64(signedHeader.header.time.secs - 10)
        );

        string memory clientId = handler.createClient(m);

        (IbcCoreClientV1Height.Data memory latestHeight, bool ok) = client
            .getLatestHeight(clientId);

        assertTrue(ok);
        assertEq(uint64(latestHeight.revision_height), trustedHeight);
        assertEq(uint64(latestHeight.revision_number), 0);
    }
}
