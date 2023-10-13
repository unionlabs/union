pragma solidity ^0.8.21;

import {ILightClient} from "../../../contracts/core/02-client/ILightClient.sol";
import {MockClient} from "../../../contracts/clients/MockClient.sol";
import {IZKVerifier} from "../../../contracts/core/IZKVerifier.sol";
import {CometblsClient} from "../../../contracts/clients/CometblsClient.sol";
import {TestVerifier} from "../../../contracts/clients/TestVerifier.sol";
import {IBCMsgs} from "../../../contracts/core/25-handler/IBCMsgs.sol";
import {IBCCommitment} from "../../../contracts/core/24-host/IBCCommitment.sol";
import {CometblsHelp} from "../../../contracts/lib/CometblsHelp.sol";

import "../TestPlus.sol";
import {IBCHandler_Testable} from "../utils/IBCHandler_Testable.sol";
import {MembershipVerifier_Testable} from "../utils/MembershipVerifier_Testable.sol";

import {TendermintTypesSignedHeader} from "../../../contracts/proto/tendermint/types/canonical.sol";
import {TendermintTypesCommit, TendermintTypesHeader, TendermintTypesSignedHeader, TendermintVersionConsensus, TendermintTypesCommitSig, TendermintTypesBlockID, TendermintTypesPartSetHeader} from "../../../contracts/proto/tendermint/types/types.sol";

import "solady/utils/LibString.sol";
import "solidity-bytes-utils/BytesLib.sol";

contract IBCClientTest is TestPlus {
    using BytesLib for bytes;
    using CometblsHelp for *;

    IBCHandler_Testable handler;
    MembershipVerifier_Testable membershipVerifier;

    string constant CLIENT_TYPE = "mock";
    ILightClient mockClient;
    ILightClient client;
    ILightClient client2;

    bytes32 constant ARBITRARY_INITIAL_APP_HASH =
        hex"A8158610DD6858F3D26149CC0DB3339ABD580EA217DE0A151C9C451DED418E35";

    constructor() {
        handler = new IBCHandler_Testable();
        membershipVerifier = new MembershipVerifier_Testable();

        mockClient = new MockClient(address(handler));

        IZKVerifier devnetVerifier = new TestVerifier();

        client = new CometblsClient(
            address(handler),
            devnetVerifier,
            membershipVerifier
        );
        client2 = new CometblsClient(
            address(handler),
            devnetVerifier,
            membershipVerifier
        );

        vm.warp(1);
    }

    //
    // registerClient
    //
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

    //
    // createClient
    //
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
            uint64(block.timestamp)
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
            uint64(block.timestamp)
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
            uint64(block.timestamp)
        );

        vm.expectRevert("createClient: unregistered client type");
        handler.createClient(m);
    }

    //
    // updateClient
    //
    function test_updateClient_newCommitment(
        uint64 revision,
        uint64 nextRevision
    ) public {
        vm.assume(revision > 0);
        vm.assume(nextRevision > revision);

        handler.registerClient(CLIENT_TYPE, mockClient);
        IBCMsgs.MsgCreateClient memory m = MsgMocks.createClient(
            CLIENT_TYPE,
            revision
        );

        string memory id = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 = MsgMocks.updateClient(
            id,
            nextRevision
        );
        handler.updateClient(m2);

        assertEq(handler.clientTypes(id), m.clientType);
        assertEq(handler.clientImpls(id), address(mockClient));
        // TODO: assert new commitments
    }

    // fixtures/commit-000114.json
    // fixtures/zk-000114-000139.json
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

    /*
     * Dump of a devnet block and it's transition ZKP
     * We need the new height to be > trusted height and the untrusted timestamp to be < current timestamp.
     * We also need the clock drift to be respected.
     * TODO: introduce more parameters to Cometbls.createClient for the clock drift, delay_period etc...
     */
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

        handler.updateClient(m2);
    }

    // Test that an ZKP containing an invalid point is rejected.
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

    // Test that a valid ZKP generated by the correct circuit but for another block transition is rejected.
    function test_updateClient_unrelatedZKP(
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
        zkp = hex"149651eef8b2b0be74b315ff22c07be1cfdf90cd1e37f9c6fc1c7f2fd33a6db10e9c1f0555b57ec6786026c488e92a27a609980c80272688895d8a29b1b80d46146b866454f8ac823953ed1a72460e58456d5194372e6d6eefe00d47caa487a90eb9ca5a4f10bc1be5f90418a4b96f99083008087e6557c96704d696d684111725855d23ea7fdc68b8a5722c3941b90c5d5ca42d57a2dee9ad0c6729becda6881a9159f67035aaecb6fc346fe553258f7ae6d8020ac6591857441bf6d09c6cd417a4a67c90ba3d591c91b579a2447c698c7564b24f6866b5b0159e3cd5b3aa932d036be11a3fcc617be88cd690383e4a4f623cc2468b0eded22588440f2e2c691f27a7c2a1527cb927dc8b6611a2f45f63257343d149964371236aca437024e8244eae3a91036115177a4b954716536e9d76d64582b85c3974948e1db3555ffb1531376fe1db46f50507d2ecdeaec4935d60bf8b5b5390883327551670186cf4";

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            id,
            signedHeader,
            trustedHeight,
            zkp
        );

        vm.warp(uint64(signedHeader.header.time.secs) + updateLatency);

        vm.expectRevert("LC: invalid ZKP");
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

        // Tamper the header.
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

    // TODO: hardcode the sequence instead of reading because of ABI/JSON being unfriendly in foundry

    /* struct FixtureCommit { */
    /*     bool canonical; */
    /*     TendermintTypesSignedHeader.Data signed_header; */
    /* } */

    /* struct FixtureTransition { */
    /*     bytes evm_zkp; */
    /*     bytes gnark_zkp; */
    /*     bytes untrusted_root; */
    /* } */

    /* function test_updateClient_sequence() public { */
    /*     uint64[7] memory checkpoints = [ */
    /*         uint64(114), */
    /*         139, */
    /*         149, */
    /*         167, */
    /*         192, */
    /*         210, */
    /*         227 */
    /*     ]; */

    /*     for (uint256 i = 0; i < 7; i++) { */
    /*         bytes memory commitBytes = vm.parseJson( */
    /*             vm.readFile( */
    /*                 string.concat( */
    /*                     "tests/src/fixtures/commit-000", */
    /*                     LibString.toString(checkpoints[i]), */
    /*                     ".json" */
    /*                 ) */
    /*             ) */
    /*         ); */
    /*         FixtureCommit memory commitFixture = abi.decode( */
    /*             commitBytes, */
    /*             (FixtureCommit) */
    /*         ); */

    /*         string memory clientId; */
    /*         if (i == 0) { */
    /*             handler.registerClient(CLIENT_TYPE, client); */
    /*             IBCMsgs.MsgCreateClient memory m = Cometbls.createClient( */
    /*                 CLIENT_TYPE, */
    /*                 commitFixture.signed_header.header.chain_id, */
    /*                 uint64(commitFixture.signed_header.header.height), */
    /*                 commitFixture.signed_header.header.app_hash.toBytes32(0), */
    /*                 commitFixture */
    /*                     .signed_header */
    /*                     .header */
    /*                     .next_validators_hash */
    /*                     .toBytes32(0) */
    /*             ); */
    /*             clientId = handler.createClient(m); */
    /*         } else { */
    /*             bytes memory transitionBytes = vm.parseJson( */
    /*                 vm.readFile( */
    /*                     string.concat( */
    /*                         "tests/src/fixtures/zk-000", */
    /*                         LibString.toString(checkpoints[i - 1]), */
    /*                         "-000", */
    /*                         LibString.toString( */
    /*                             commitFixture.signed_header.header.height */
    /*                         ), */
    /*                         ".json" */
    /*                     ) */
    /*                 ) */
    /*             ); */
    /*             FixtureTransition memory transitionFixture = abi.decode( */
    /*                 transitionBytes, */
    /*                 (FixtureTransition) */
    /*             ); */
    /*             IBCMsgs.MsgUpdateClient memory m = Cometbls.updateClient( */
    /*                 clientId, */
    /*                 commitFixture.signed_header, */
    /*                 checkpoints[i], */
    /*                 transitionFixture.untrusted_root, */
    /*                 transitionFixture.evm_zkp */
    /*             ); */
    /*             handler.updateClient(m); */
    /*         } */
    /*     } */
    /* } */
}
