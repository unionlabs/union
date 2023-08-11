pragma solidity ^0.8.18;

import {ILightClient} from "../../../contracts/core/02-client/ILightClient.sol";
import {MockClient} from "../../../contracts/clients/MockClient.sol";
import {IZKVerifier} from "../../../contracts/core/IZKVerifier.sol";
import {CometblsClient} from "../../../contracts/clients/CometblsClient.sol";
import {DevnetVerifier} from "../../../contracts/clients/DevnetVerifier.sol";
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

struct FixtureCommit {
    bool canonical;
    TendermintTypesSignedHeader.Data signed_header;
}

struct FixtureTransition {
    bytes evm_zkp;
    bytes gnark_zkp;
    bytes untrusted_root;
}

contract IBCClientTest is TestPlus {
    using BytesLib for bytes;
    using CometblsHelp for *;

    IBCHandler_Testable handler;
    MembershipVerifier_Testable membershipVerifier;

    string constant CLIENT_TYPE = "mock";
    ILightClient mockClient;
    ILightClient client;
    ILightClient client2;

    constructor() {
        handler = new IBCHandler_Testable();
        membershipVerifier = new MembershipVerifier_Testable();

        mockClient = new MockClient(address(handler));

        IZKVerifier devnetVerifier = new DevnetVerifier();

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

    /*
     * Dump of a devnet block and it's transition ZKP
     * We need the new height to be > trusted height and the untrusted timestamp to be < current timestamp.
     * We also need the clock drift to be respected.
     * TODO: introduce more parameters to Cometbls.createClient for the clock drift, delay_period etc...
     */
    function test_updateClient_validZKP(uint64 trustedHeight) public {
        vm.assume(0 < trustedHeight && trustedHeight < 139);

        handler.registerClient(CLIENT_TYPE, client);

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

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            header.chain_id,
            trustedHeight,
            // Initial trusted app_hash
            hex"A8158610DD6858F3D26149CC0DB3339ABD580EA217DE0A151C9C451DED418E35",
            header.validators_hash.toBytes32(0),
            // TODO: fuzz on a window with clock drift
            uint64(header.time.secs - 10)
        );

        string memory id = handler.createClient(m);

        TendermintTypesSignedHeader.Data
            memory signedHeader = TendermintTypesSignedHeader.Data({
                header: header,
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
                    // We don't need the signature, this is verified by the ZKP
                    signatures: new TendermintTypesCommitSig.Data[](0)
                })
            });

        bytes
            memory zkp = hex"09f57b8b308d9c57bd1e30cd493212314f5b680e685bc91402193fac45389c42064968d6db298707b5405431621b96bd73756907b2b5137ca4966c270d0b9b461a60936e4cdf9b77b993f25cdeb7d1c5623f082dc2c88b20d33a9b40c14dc39115aba4e371dc0443465b4d9b69aece3a4f15f0503c6d0f56dc1237356c32de80271ca20c7eb2bcb9bc56be7256a93d925fa3bae73829dbc53c4e9056f99046b80277d0bbc45741e6eb1e6a6a9e1d795f384cca1d3836e29ffecdebf6b0a9db5e15ec13c943d68283a8a781f4d5cb330ca1b02a7515990eb8c3c3e4da4ba9ef1717980acd29ff4c6ba58036337faae8def7355243b2449b9c5637f85ebecec1a42e4570af6b520476ccc96665d3d92dc7a22c0864169072332f17f7664223ea4004e3860aa093da597dc7f6b28284c45d9bc56e4d8e44ee5b784ec23b5309649116db1c88de8aaa9057b41b94939806fd8910bc1d5b33f3d4db7f568197c63f55";
        bytes
            memory untrustedValidatorsRoot = hex"f09e25471b41514b2f8b08b5f4c9093c5d6ed134e107ff491ced2374b947df60";

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            id,
            signedHeader,
            trustedHeight,
            untrustedValidatorsRoot,
            zkp
        );

        // TODO: fuzz clock drift
        vm.warp(uint64(header.time.secs + 10));

        handler.updateClient(m2);
    }


    // Simple test case where a byte of the ZKP has been tampered
    function test_updateClient_invalidZKP(uint64 trustedHeight) public {
        vm.assume(0 < trustedHeight && trustedHeight < 139);

        handler.registerClient(CLIENT_TYPE, client);

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

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            header.chain_id,
            trustedHeight,
            // Initial trusted app_hash
            hex"A8158610DD6858F3D26149CC0DB3339ABD580EA217DE0A151C9C451DED418E35",
            header.validators_hash.toBytes32(0),
            // TODO: fuzz on a window with clock drift
            uint64(header.time.secs - 10)
        );

        string memory id = handler.createClient(m);

        TendermintTypesSignedHeader.Data
            memory signedHeader = TendermintTypesSignedHeader.Data({
                header: header,
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
                    // We don't need the signature, this is verified by the ZKP
                    signatures: new TendermintTypesCommitSig.Data[](0)
                })
            });

        bytes
            memory zkp = hex"09f57b8b308d9c57bd1e30cd493212314f5b680e685bc91402193fac45389c42064968d6db298707b5405431621b96bd73756907b2b5137ca4966c270d0b9b461a60936e4cdf9b77b993f25cdeb7d1c5623f082dc2c88b20d33a9b40c14dc39115aba4e371dc0443465b4d9b69aece3a4f15f0503c6d0f56dc1237356c32de80271ca20c7eb2bcb9bc56be7256a93d925fa3bae73829dbc53c4e9056f99046b80277d0bbc45741e6eb1e6a6a9e1d795f384cca1d3836e29ffecdebf6b0a9db5e15ec13c943d68283a8a781f4d5cb330ca1b02a7515990eb8c3c3e4da4ba9ef1717980acd29ff4c6ba58036337faae8def7355243b2449b9c5637f85ebecec1a42e4570af6b520476ccc96665d3d92dc7a22c0864169072332f17f7664223ea4004e3860aa093da597dc7f6b28284c45d9bc56e4d8e44ee5b784ec23b5309649116db1c88de8aaa9057b41b94939806fd8910bc1d5b33f3d4db7f568197c63f54";
        bytes
            memory untrustedValidatorsRoot = hex"f09e25471b41514b2f8b08b5f4c9093c5d6ed134e107ff491ced2374b947df60";

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            id,
            signedHeader,
            trustedHeight,
            untrustedValidatorsRoot,
            zkp
        );

        vm.warp(uint64(header.time.secs + 10));

        vm.expectRevert();
        handler.updateClient(m2);
    }


    /*
        Here we ensure that even if a ZKP is correct from the math POV (points
        are on the curve etc), the ZKP must be correlated to this specific
        transition, i.e. we can't reuse a ZKP created for another transition
        (would be a big problem, right?)
     */
    function test_updateClient_unrelatedZKP(uint64 trustedHeight) public {
        vm.assume(0 < trustedHeight && trustedHeight < 139);

        handler.registerClient(CLIENT_TYPE, client);

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

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            header.chain_id,
            trustedHeight,
            // Initial trusted app_hash
            hex"A8158610DD6858F3D26149CC0DB3339ABD580EA217DE0A151C9C451DED418E35",
            header.validators_hash.toBytes32(0),
            // TODO: fuzz on a window with clock drift
            uint64(header.time.secs - 10)
        );

        string memory id = handler.createClient(m);

        TendermintTypesSignedHeader.Data
            memory signedHeader = TendermintTypesSignedHeader.Data({
                header: header,
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
                    // We don't need the signature, this is verified by the ZKP
                    signatures: new TendermintTypesCommitSig.Data[](0)
                })
            });

        bytes
            memory zkp = hex"2b2d669ea8d67d1621b6d96445a1cc26e430a435fd005e7d3a35984431ccec8528d503b5eedd105937732322c5a505fc10ebaa16909bec384d58777409c59ce51dc39c79f42e2a449656d9912c4ccda211704aaca45e48f361637d5e5be3047f11e54d4a140da4733d833cb209b7d5e798966434a5cd04210a3b4c3f045306511f84025ce881782a283ae6b1a9cf55e14ae76857b3c1010dca2f10f7ab2d0ce00c28829d8376da5b857c6c1f996cc1d0ac9e4bd914e008a1f534f1cd764e86e8078201d2caeb8d063f398a60a5a1b173eb5b101ca411a3ed24c18417f11aabe002bccc08ffb65ef48394e11dffbcbc29e83603228506c535610b288b058308501ae0c37422e232d6afea4197a0305b0634d7cd5de25d1e3d0fd4ff3c12088e4e1585646970e7691561a3edc2ce0e9a4219132799ef5d6a9f688fadff6550e8930b5bb25d141b7cbc2af85d3a9d1a16aefd9c2feef12410b55b0dd53ac9c61f97";
        bytes
            memory untrustedValidatorsRoot = hex"f09e25471b41514b2f8b08b5f4c9093c5d6ed134e107ff491ced2374b947df60";

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            id,
            signedHeader,
            trustedHeight,
            untrustedValidatorsRoot,
            zkp
        );

        vm.warp(uint64(header.time.secs + 10));

        vm.expectRevert("LC: invalid ZKP");
        handler.updateClient(m2);
    }

    /*
       The validators are signing the block root, which is a hashing done on the
       TendermintTypesHeader. The TendermintTypesCommit.block_id.hash must match
       the hash of the TendermintTypesHeader. We tweak a field (in this case set
       header.last_block_id.part_set_header.total to 0) of the header and verify
       that the client correctly check the hash against the provided commit hash.
     */
    function test_updateClient_invalidBlockRoot(uint64 trustedHeight) public {
        vm.assume(0 < trustedHeight && trustedHeight < 139);

        handler.registerClient(CLIENT_TYPE, client);

        TendermintTypesHeader.Data memory header = TendermintTypesHeader.Data({
            version: TendermintVersionConsensus.Data({block: 11, app: 0}),
            chain_id: "union-devnet-1",
            height: 139,
            time: Timestamp.Data({secs: 1691496777, nanos: 793918988}),
            last_block_id: TendermintTypesBlockID.Data({
                hash: hex"80DF3A892BF2586E3B22201D2AC5A65EDAB66ECE7BB6F51077F3B50CCE7526E1",
                part_set_header: TendermintTypesPartSetHeader.Data({
                    total: 0,
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

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            header.chain_id,
            trustedHeight,
            // Initial trusted app_hash
            hex"A8158610DD6858F3D26149CC0DB3339ABD580EA217DE0A151C9C451DED418E35",
            header.validators_hash.toBytes32(0),
            // TODO: fuzz on a window with clock drift
            uint64(header.time.secs - 10)
        );

        string memory id = handler.createClient(m);

        TendermintTypesSignedHeader.Data
            memory signedHeader = TendermintTypesSignedHeader.Data({
                header: header,
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
                    // We don't need the signature, this is verified by the ZKP
                    signatures: new TendermintTypesCommitSig.Data[](0)
                })
            });

        bytes
            memory zkp = hex"09f57b8b308d9c57bd1e30cd493212314f5b680e685bc91402193fac45389c42064968d6db298707b5405431621b96bd73756907b2b5137ca4966c270d0b9b461a60936e4cdf9b77b993f25cdeb7d1c5623f082dc2c88b20d33a9b40c14dc39115aba4e371dc0443465b4d9b69aece3a4f15f0503c6d0f56dc1237356c32de80271ca20c7eb2bcb9bc56be7256a93d925fa3bae73829dbc53c4e9056f99046b80277d0bbc45741e6eb1e6a6a9e1d795f384cca1d3836e29ffecdebf6b0a9db5e15ec13c943d68283a8a781f4d5cb330ca1b02a7515990eb8c3c3e4da4ba9ef1717980acd29ff4c6ba58036337faae8def7355243b2449b9c5637f85ebecec1a42e4570af6b520476ccc96665d3d92dc7a22c0864169072332f17f7664223ea4004e3860aa093da597dc7f6b28284c45d9bc56e4d8e44ee5b784ec23b5309649116db1c88de8aaa9057b41b94939806fd8910bc1d5b33f3d4db7f568197c63f55";
        bytes
            memory untrustedValidatorsRoot = hex"f09e25471b41514b2f8b08b5f4c9093c5d6ed134e107ff491ced2374b947df60";

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            id,
            signedHeader,
            trustedHeight,
            untrustedValidatorsRoot,
            zkp
        );

        // TODO: fuzz clock drift
        vm.warp(uint64(header.time.secs + 10));

        vm.expectRevert("LC: commit.block_id.hash != header.root()");
        handler.updateClient(m2);
    }

    function test_updateClient_nextRevisionLower(uint64 trustedHeight) public {
        vm.assume(trustedHeight > 139);

        handler.registerClient(CLIENT_TYPE, client);

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

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-1",
            trustedHeight,
            hex"A8158610DD6858F3D26149CC0DB3339ABD580EA217DE0A151C9C451DED418E35",
            hex"F09E25471B41514B2F8B08B5F4C9093C5D6ED134E107FF491CED2374B947DF60",
            // TODO: fuzz on a window with clock drift
            uint64(header.time.secs - 10)
        );

        string memory id = handler.createClient(m);

        TendermintTypesSignedHeader.Data
            memory signedHeader = TendermintTypesSignedHeader.Data({
                header: header,
                commit: TendermintTypesCommit.Data({
                    height: header.height,
                    round: 0,
                    block_id: TendermintTypesBlockID.Data({
                        hash: hex"90548CD1E2BA8603261FE6400ADFD97EA48150CBD5B24B9828B2542BAB9E27EC",
                        part_set_header: TendermintTypesPartSetHeader.Data({
                            total: 1,
                            hash: hex"F9A503AC6A6E51C4D86B4202220BD011DDA34F2ED95E3D5CA7C90E6CDAA6491F"
                        })
                    }),
                    // We don't need the signature, this is verified by the ZKP
                    signatures: new TendermintTypesCommitSig.Data[](0)
                })
            });

        bytes
            memory zkp = hex"09f57b8b308d9c57bd1e30cd493212314f5b680e685bc91402193fac45389c42064968d6db298707b5405431621b96bd73756907b2b5137ca4966c270d0b9b461a60936e4cdf9b77b993f25cdeb7d1c5623f082dc2c88b20d33a9b40c14dc39115aba4e371dc0443465b4d9b69aece3a4f15f0503c6d0f56dc1237356c32de80271ca20c7eb2bcb9bc56be7256a93d925fa3bae73829dbc53c4e9056f99046b80277d0bbc45741e6eb1e6a6a9e1d795f384cca1d3836e29ffecdebf6b0a9db5e15ec13c943d68283a8a781f4d5cb330ca1b02a7515990eb8c3c3e4da4ba9ef1717980acd29ff4c6ba58036337faae8def7355243b2449b9c5637f85ebecec1a42e4570af6b520476ccc96665d3d92dc7a22c0864169072332f17f7664223ea4004e3860aa093da597dc7f6b28284c45d9bc56e4d8e44ee5b784ec23b5309649116db1c88de8aaa9057b41b94939806fd8910bc1d5b33f3d4db7f568197c63f55";
        bytes
            memory untrustedValidatorsRoot = hex"F09E25471B41514B2F8B08B5F4C9093C5D6ED134E107FF491CED2374B947DF60";

        IBCMsgs.MsgUpdateClient memory m2 = Cometbls.updateClient(
            id,
            signedHeader,
            trustedHeight,
            untrustedValidatorsRoot,
            zkp
        );

        vm.expectRevert("LC: header height <= consensus state height");
        handler.updateClient(m2);
    }

    // TODO: hardcode the sequence instead of reading because of ABI/JSON being unfriendly in foundry
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
