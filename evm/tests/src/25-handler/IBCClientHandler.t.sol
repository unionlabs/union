pragma solidity ^0.8.23;

import "solidity-bytes-utils/BytesLib.sol";

import {IBCClientLib} from "../../../contracts/core/02-client/IBCClient.sol";
import {ILightClient, ConsensusStateUpdate} from "../../../contracts/core/02-client/ILightClient.sol";
import {IZKVerifierV2} from "../../../contracts/core/IZKVerifierV2.sol";
import {CometblsClient, CometblsClientLib} from "../../../contracts/clients/CometblsClientV2.sol";
import {IBCMsgs} from "../../../contracts/core/25-handler/IBCMsgs.sol";
import {IBCCommitment} from "../../../contracts/core/24-host/IBCCommitment.sol";
import {CometblsHelp} from "../../../contracts/lib/CometblsHelp.sol";
import {IMembershipVerifier} from "../../../contracts/core/IMembershipVerifier.sol";
import {IbcCoreClientV1Height} from "../../../contracts/proto/ibc/core/client/v1/client.sol";
import {TendermintTypesSignedHeader} from "../../../contracts/proto/tendermint/types/canonical.sol";
import {TendermintTypesCommit, TendermintTypesHeader, TendermintTypesSignedHeader, TendermintVersionConsensus, TendermintTypesCommitSig, TendermintTypesBlockID, TendermintTypesPartSetHeader} from "../../../contracts/proto/tendermint/types/types.sol";

import "../TestPlus.sol";

contract TestVerifier is IZKVerifierV2 {
    uint256 valid = 0;

    function pushValid() public {
        valid += 1;
    }

    function verifyProof(
        uint256[8] memory proof,
        uint256[2] memory proofCommitment,
        uint256[2] calldata proofCommitmentPOK,
        uint256[4] calldata input
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

    function setUp() public {
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
        zkp = hex"195562CC376E9265A7FD89A086855C100173B717B0DEA58AC9F50120E9CBDD7402D59ADAC8A274C5DDB199915B03B5CFB7A91032A71723876F946A7662135D4912EB1FAD1FCA5E88AD1D9097870391D1D477F4CD2A26F27DB3CFC8B511922C482F374A4821BEE34818589A052995CC5994CE787538207F1BA0D595890EB96D751D947274566F6338FC14BB1728C9E42F47F9D47A8A7F46CFA341D3EC71F0A8E80ECDAA9E38B4D6090989B165E536C4332BDF470E860D85001362EC7B369DE0092FD13C85FE2A16247E574B759B7B8EBFE8C7ED19CE7520A693BD09FD604CA54E2FA277AC176ACEC9626313DA7022E8B8DB599E1B02C25DA90AD508AA315DA67C0EAF8A0F41C4CDC897A4941F3BFA7D0E0C2BDD3030D5B0025FB4030A31C886F417B2509E9ECFEA86AA22F75402599E72C21623E9C32A499D7B14B6DBC3A1251E119244B7DC12B54A74FBC3B23E7954435491D89AFA7ABF6F07E1DADE0B28F0DA1978EC72A2C2C0F1FE8DEDA8DD8DDA7E82454618C3DFF1341C9901456F7E656A";
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

        vm.expectRevert(IBCClientLib.ErrClientTypeAlreadyExists.selector);
        handler.registerClient(CLIENT_TYPE, client);
    }

    function test_registerClient_self() public {
        vm.expectRevert(IBCClientLib.ErrClientMustNotBeSelf.selector);
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

        vm.expectRevert(IBCClientLib.ErrFailedToCreateClient.selector);
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

        vm.expectRevert(IBCClientLib.ErrClientTypeNotFound.selector);
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
        vm.expectRevert(CometblsClientLib.ErrUnauthorized.selector);
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
        vm.expectRevert(CometblsClientLib.ErrUnauthorized.selector);
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

        vm.expectRevert(CometblsClientLib.ErrInvalidZKP.selector);
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

        vm.expectRevert(
            CometblsClientLib.ErrPrecomputedRootAndBlockRootMismatch.selector
        );
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

        vm.expectRevert(
            CometblsClientLib.ErrUntrustedHeightLTETrustedHeight.selector
        );
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

        vm.expectRevert(CometblsClientLib.ErrHeaderExpired.selector);
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

        vm.expectRevert(
            CometblsClientLib.ErrTrustedConsensusStateNotFound.selector
        );
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

        vm.expectRevert(CometblsClientLib.ErrDelayPeriodNotExpired.selector);
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

        vm.expectRevert(CometblsClientLib.ErrDelayPeriodNotExpired.selector);
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

        vm.expectRevert(CometblsClientLib.ErrDelayPeriodNotExpired.selector);
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

        vm.expectRevert(CometblsClientLib.ErrDelayPeriodNotExpired.selector);
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
