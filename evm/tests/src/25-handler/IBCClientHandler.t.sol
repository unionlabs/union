pragma solidity ^0.8.23;

import "solidity-bytes-utils/BytesLib.sol";
import "@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol";

import {IBCHandler} from "../../../contracts/core/25-handler/IBCHandler.sol";
import {IBCConnectionImpl} from
    "../../../contracts/core/03-connection/IBCConnection.sol";
import {IBCClient} from "../../../contracts/core/02-client/IBCClient.sol";
import {IBCChannelImpl} from "../../../contracts/core/04-channel/IBCChannel.sol";
import {
    IBCPacket,
    IBCPacketLib
} from "../../../contracts/core/04-channel/IBCPacket.sol";
import {IBCClientLib} from "../../../contracts/core/02-client/IBCClient.sol";
import {
    ILightClient,
    ConsensusStateUpdate
} from "../../../contracts/core/02-client/ILightClient.sol";
import {
    CometblsClient,
    CometblsClientLib
} from "../../../contracts/clients/CometblsClientV2.sol";
import {IBCMsgs} from "../../../contracts/core/25-handler/IBCMsgs.sol";
import {IBCCommitment} from "../../../contracts/core/24-host/IBCCommitment.sol";
import {IbcCoreClientV1Height} from
    "../../../contracts/proto/ibc/core/client/v1/client.sol";
import {
    TendermintTypesCommit,
    TendermintTypesSignedHeader,
    TendermintVersionConsensus,
    TendermintTypesCommitSig,
    TendermintTypesBlockID,
    TendermintTypesPartSetHeader
} from "../../../contracts/proto/tendermint/types/types.sol";
import
    "../../../contracts/proto/union/ibc/lightclients/cometbls/v1/cometbls.sol";

import "../TestPlus.sol";

contract TestCometblsClient is CometblsClient {
    uint256 validProof = 0;

    function pushValidProof() public {
        validProof += 1;
    }

    uint256 validMembership = 0;

    function pushValidMembership() public {
        validMembership += 1;
    }

    function internalVerifyZKP(
        bytes calldata,
        string memory,
        bytes32,
        UnionIbcLightclientsCometblsV1LightHeader.Data calldata
    ) internal override returns (bool) {
        bool ok = validProof > 0;
        if (validProof > 0) {
            validProof -= 1;
        }
        return ok;
    }

    function verifyMembership(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height,
        uint64 delayPeriodTime,
        uint64 delayPeriodBlocks,
        bytes calldata,
        bytes memory,
        bytes calldata,
        bytes calldata
    ) external override returns (bool) {
        validateDelayPeriod(
            clientId, height, delayPeriodTime, delayPeriodBlocks
        );

        bool ok = validMembership > 0;
        if (validMembership > 0) {
            validMembership -= 1;
        }
        return ok;
    }

    function verifyNonMembership(
        string calldata clientId,
        IbcCoreClientV1Height.Data calldata height,
        uint64 delayPeriodTime,
        uint64 delayPeriodBlocks,
        bytes calldata,
        bytes calldata,
        bytes calldata
    ) external override returns (bool) {
        validateDelayPeriod(
            clientId, height, delayPeriodTime, delayPeriodBlocks
        );

        bool ok = validMembership > 0;
        if (validMembership > 0) {
            validMembership -= 1;
        }
        return ok;
    }
}

contract IBCClientHandlerTests is TestPlus {
    using BytesLib for bytes;
    using CometblsClientLib for *;

    IBCHandler_Testable handler;

    string constant CLIENT_TYPE = "mock";
    TestCometblsClient client;
    TestCometblsClient client2;

    bytes32 constant ARBITRARY_INITIAL_APP_HASH =
        hex"A8158610DD6858F3D26149CC0DB3339ABD580EA217DE0A151C9C451DED418E35";

    function setUp() public {
        handler = IBCHandler_Testable(
            address(
                new ERC1967Proxy(
                    address(new IBCHandler_Testable()),
                    abi.encodeCall(
                        IBCHandler.initialize,
                        (
                            address(new IBCClient()),
                            address(new IBCConnection()),
                            address(new IBCChannelHandshake()),
                            address(new IBCPacket()),
                            address(this)
                        )
                    )
                )
            )
        );
        client = TestCometblsClient(
            address(
                new ERC1967Proxy(
                    address(new TestCometblsClient()),
                    abi.encodeCall(
                        CometblsClient.initialize,
                        (address(handler), address(this))
                    )
                )
            )
        );
        client2 = TestCometblsClient(
            address(
                new ERC1967Proxy(
                    address(new TestCometblsClient()),
                    abi.encodeCall(
                        CometblsClient.initialize,
                        (address(handler), address(this))
                    )
                )
            )
        );
        vm.warp(1);
    }

    function getValidTransition()
        internal
        pure
        returns (
            bytes memory zkp,
            UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader
        )
    {
        zkp =
            hex"195562CC376E9265A7FD89A086855C100173B717B0DEA58AC9F50120E9CBDD7402D59ADAC8A274C5DDB199915B03B5CFB7A91032A71723876F946A7662135D4912EB1FAD1FCA5E88AD1D9097870391D1D477F4CD2A26F27DB3CFC8B511922C482F374A4821BEE34818589A052995CC5994CE787538207F1BA0D595890EB96D751D947274566F6338FC14BB1728C9E42F47F9D47A8A7F46CFA341D3EC71F0A8E80ECDAA9E38B4D6090989B165E536C4332BDF470E860D85001362EC7B369DE0092FD13C85FE2A16247E574B759B7B8EBFE8C7ED19CE7520A693BD09FD604CA54E2FA277AC176ACEC9626313DA7022E8B8DB599E1B02C25DA90AD508AA315DA67C0EAF8A0F41C4CDC897A4941F3BFA7D0E0C2BDD3030D5B0025FB4030A31C886F417B2509E9ECFEA86AA22F75402599E72C21623E9C32A499D7B14B6DBC3A1251E119244B7DC12B54A74FBC3B23E7954435491D89AFA7ABF6F07E1DADE0B28F0DA1978EC72A2C2C0F1FE8DEDA8DD8DDA7E82454618C3DFF1341C9901456F7E656A";
        signedHeader = UnionIbcLightclientsCometblsV1LightHeader.Data({
            height: 139,
            time: Timestamp.Data({secs: 1691496777, nanos: 793918988}),
            validators_hash: hex"F09E25471B41514B2F8B08B5F4C9093C5D6ED134E107FF491CED2374B947DF60",
            next_validators_hash: hex"F09E25471B41514B2F8B08B5F4C9093C5D6ED134E107FF491CED2374B947DF60",
            app_hash: hex"983EF85676937CEC783601B5B50865733A72C3DF88E4CC0B3F11C108C9688459"
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
        vm.assume(bytes(chainId).length < 32);
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

    function test_createClient_chainIdExceedScalarField(
        string memory chainId,
        bytes32 rootHash,
        bytes32 nextValidatorsHash
    ) public {
        vm.assume(bytes(chainId).length > 31);

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

    function test_createClient_onlyIBC(uint64 trustedHeight) public {
        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        vm.assume(
            0 < trustedHeight && trustedHeight < uint64(signedHeader.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        client.pushValidProof();
        vm.expectRevert(CometblsClientLib.ErrNotIBC.selector);
        client.createClient("blabla", m.clientStateBytes, m.consensusStateBytes);
    }

    function test_updateClient_onlyIBC(
        uint64 trustedHeight,
        uint64 updateLatency
    ) public {
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight && trustedHeight < uint64(signedHeader.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory id = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 =
            Cometbls.updateClient(id, signedHeader, trustedHeight, zkp);

        vm.warp(uint64(signedHeader.time.secs) + updateLatency);

        client.pushValidProof();
        vm.expectRevert(CometblsClientLib.ErrNotIBC.selector);
        client.updateClient(m2.clientId, m2.clientMessage);
    }

    function test_updateClient_newCommitment(
        uint64 trustedHeight,
        uint64 updateLatency
    ) public {
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight && trustedHeight < uint64(signedHeader.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory id = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 =
            Cometbls.updateClient(id, signedHeader, trustedHeight, zkp);

        vm.warp(uint64(signedHeader.time.secs) + updateLatency);

        client.pushValidProof();
        vm.prank(address(handler));
        (bytes32 clientStateCommitment, ConsensusStateUpdate[] memory updates) =
            client.updateClient(m2.clientId, m2.clientMessage);
        assertEq(
            clientStateCommitment,
            keccak256(
                Cometbls.createClientState(
                    "union-devnet-10", uint64(signedHeader.height)
                ).encodeMemory()
            )
        );
        assertEq(updates.length, 1);
        assertEq(
            updates[0].consensusStateCommitment,
            keccak256(
                Cometbls.createConsensusState(
                    signedHeader.app_hash.toBytes32(0),
                    signedHeader.validators_hash.toBytes32(0),
                    uint64(signedHeader.time.secs) * 1e9
                        + uint64(signedHeader.time.nanos)
                ).encodeMemory()
            )
        );
        assertEq(updates[0].height.revision_height, uint64(signedHeader.height));
    }

    function test_updateClient_validZKP(
        uint64 trustedHeight,
        uint64 updateLatency
    ) public {
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight && trustedHeight < uint64(signedHeader.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory id = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 =
            Cometbls.updateClient(id, signedHeader, trustedHeight, zkp);

        vm.warp(uint64(signedHeader.time.secs) + updateLatency);

        client.pushValidProof();
        handler.updateClient(m2);
    }

    function test_updateClient_invalidZKP(
        uint64 trustedHeight,
        uint64 updateLatency
    ) public {
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight && trustedHeight < uint64(signedHeader.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory id = handler.createClient(m);

        // Tamper the ZKP
        zkp[0] = 0xCA;
        zkp[1] = 0xFE;
        zkp[2] = 0xBA;
        zkp[3] = 0xBE;

        IBCMsgs.MsgUpdateClient memory m2 =
            Cometbls.updateClient(id, signedHeader, trustedHeight, zkp);

        vm.warp(uint64(signedHeader.time.secs) + updateLatency);

        vm.expectRevert(CometblsClientLib.ErrInvalidZKP.selector);
        handler.updateClient(m2);
    }

    // Test that the on-chain hash(signedHeader.header) is equal to the off-chain provided signedHeader.commit.block_id.hash.
    // NOTE: Optimization-wise, this is probably unecessary as we direcyly compute that on chain. i.e. we could remove commit.block_id entirely.
    function test_updateClient_invalidBlockRoot(
        uint64 trustedHeight,
        uint64 updateLatency
    ) public {
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight && trustedHeight < uint64(signedHeader.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory id = handler.createClient(m);

        // Tamper the header such that the block root != commit, as if a relayer tampered the commit or the block.
        signedHeader.app_hash = abi.encodePacked(uint256(0));

        IBCMsgs.MsgUpdateClient memory m2 =
            Cometbls.updateClient(id, signedHeader, trustedHeight, zkp);

        vm.warp(uint64(signedHeader.time.secs) + updateLatency);

        vm.expectRevert(CometblsClientLib.ErrInvalidZKP.selector);
        handler.updateClient(m2);
    }

    function test_updateClient_nextRevisionLower(
        uint64 trustedHeight,
        uint64 updateLatency
    ) public {
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(trustedHeight > uint64(signedHeader.height));

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory id = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 =
            Cometbls.updateClient(id, signedHeader, trustedHeight, zkp);

        vm.warp(uint64(signedHeader.time.secs) + updateLatency);

        vm.expectRevert(
            CometblsClientLib.ErrUntrustedHeightLTETrustedHeight.selector
        );
        handler.updateClient(m2);
    }

    function test_updateClient_trustingPeriodExpired(uint64 trustedHeight)
        public
    {
        (
            bytes memory zkp,
            UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight && trustedHeight < uint64(signedHeader.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory id = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 =
            Cometbls.updateClient(id, signedHeader, trustedHeight, zkp);

        // The block timestamp will be out of the trusting period window
        vm.warp(uint64(signedHeader.time.secs) + Cometbls.TRUSTING_PERIOD + 1);

        vm.expectRevert(CometblsClientLib.ErrHeaderExpired.selector);
        handler.updateClient(m2);
    }

    function test_getTimestampAtHeight() public {
        (
            bytes memory zkp,
            UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader
        ) = getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;
        uint64 updateLatency = Cometbls.TRUSTING_PERIOD - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory clientId = handler.createClient(m);

        IBCMsgs.MsgUpdateClient memory m2 =
            Cometbls.updateClient(clientId, signedHeader, trustedHeight, zkp);

        vm.warp(uint64(signedHeader.time.secs) + updateLatency);

        client.pushValidProof();
        handler.updateClient(m2);

        uint64 timestamp = client.getTimestampAtHeight(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: uint64(signedHeader.height)
            })
        );
        assertEq(
            timestamp,
            uint64(signedHeader.time.secs) * 1e9
                + uint64(signedHeader.time.nanos)
        );
    }

    function test_getClientState() public {
        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory clientId = handler.createClient(m);

        bytes memory clientStateBytes = client.getClientState(clientId);
        assertEq(clientStateBytes, m.clientStateBytes);
    }

    function test_getClientState_noClientState() public {
        getValidTransition();

        handler.registerClient(CLIENT_TYPE, client);

        client.getClientState("blabla");
        // REVIEW: Not sure how best to check if the bytes is the zero of the type
        // assertEq(clientStateBytes, "");
    }

    function test_getClientState_step(
        uint64 trustedHeight,
        uint64 updateLatency
    ) public {
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight && trustedHeight < uint64(signedHeader.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory clientId = handler.createClient(m);

        bytes memory clientStateBytes = client.getClientState(clientId);
        assertEq(clientStateBytes, m.clientStateBytes);

        vm.warp(uint64(signedHeader.time.secs) + updateLatency);

        IBCMsgs.MsgUpdateClient memory m2 =
            Cometbls.updateClient(clientId, signedHeader, trustedHeight, zkp);

        client.pushValidProof();
        handler.updateClient(m2);

        clientStateBytes = client.getClientState(clientId);
        assertEq(
            clientStateBytes,
            Cometbls.createClientState(
                "union-devnet-10", uint64(signedHeader.height)
            ).encodeMemory()
        );
    }

    function test_getConsensusState() public {
        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory clientId = handler.createClient(m);

        bytes memory consensusStateBytes = client.getConsensusState(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            })
        );

        assertEq(
            consensusStateBytes,
            Cometbls.createConsensusState(
                ARBITRARY_INITIAL_APP_HASH,
                signedHeader.validators_hash.toBytes32(0),
                (uint64(signedHeader.time.secs) - 10) * 1e9
            ).encodeMemory()
        );
    }

    function test_getConsensusState_noConsensus() public {
        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        client.getConsensusState(
            "blabla",
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: trustedHeight
            })
        );
        // REVIEW: Not sure how best to check if the bytes is the zero of the type
        // assertEq(consensusStateBytes, "");
    }

    function test_getConsensusState_step(
        uint64 trustedHeight,
        uint64 updateLatency
    ) public {
        vm.assume(updateLatency < Cometbls.TRUSTING_PERIOD);

        (
            bytes memory zkp,
            UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader
        ) = getValidTransition();

        vm.assume(
            0 < trustedHeight && trustedHeight < uint64(signedHeader.height)
        );

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        vm.warp(uint64(signedHeader.time.secs));

        string memory clientId = handler.createClient(m);

        bytes memory consensusStateBytes = client.getConsensusState(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: uint64(signedHeader.height)
            })
        );
        // REVIEW: Not sure how best to check if the bytes is the zero of the type
        // assertEq(consensusStateBytes, "");

        IBCMsgs.MsgUpdateClient memory m2 =
            Cometbls.updateClient(clientId, signedHeader, trustedHeight, zkp);

        vm.warp(uint64(signedHeader.time.secs) + updateLatency);

        client.pushValidProof();
        handler.updateClient(m2);

        consensusStateBytes = client.getConsensusState(
            clientId,
            IbcCoreClientV1Height.Data({
                revision_number: 0,
                revision_height: uint64(signedHeader.height)
            })
        );
        assertEq(
            consensusStateBytes,
            Cometbls.createConsensusState(
                signedHeader.app_hash.toBytes32(0),
                signedHeader.validators_hash.toBytes32(0),
                uint64(signedHeader.time.secs) * 1e9
                    + uint64(signedHeader.time.nanos)
            ).encodeMemory()
        );
    }

    function test_verifyMembership_noConsensus() public {
        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
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
        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory clientId = handler.createClient(m);

        client.pushValidMembership();
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
        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
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

        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
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
            // Expected to be in nanos
            delayPeriodTime * 1e9,
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

        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
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

        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory clientId = handler.createClient(m);

        vm.warp(vm.getBlockTimestamp() + delayPeriodTime);

        client.pushValidMembership();
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

        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory clientId = handler.createClient(m);

        vm.roll(vm.getBlockNumber() + delayPeriodBlocks);

        client.pushValidMembership();
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
        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory clientId = handler.createClient(m);

        client.pushValidMembership();
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
        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
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

        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
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
            // Expected to be in nanos
            delayPeriodTime * 1e9,
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

        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
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

        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory clientId = handler.createClient(m);

        vm.warp(vm.getBlockTimestamp() + delayPeriodTime);

        client.pushValidMembership();
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

        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory clientId = handler.createClient(m);

        vm.roll(vm.getBlockNumber() + delayPeriodBlocks);

        client.pushValidMembership();
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
        getValidTransition();

        handler.registerClient(CLIENT_TYPE, client);

        IbcCoreClientV1Height.Data memory latestHeight =
            client.getLatestHeight("blabla");

        assertEq(uint64(latestHeight.revision_height), 0);
        assertEq(uint64(latestHeight.revision_number), 0);
    }

    function test_getLatestHeight_ok() public {
        (, UnionIbcLightclientsCometblsV1LightHeader.Data memory signedHeader) =
            getValidTransition();

        uint64 trustedHeight = uint64(signedHeader.height) - 1;

        handler.registerClient(CLIENT_TYPE, client);

        IBCMsgs.MsgCreateClient memory m = Cometbls.createClient(
            CLIENT_TYPE,
            "union-devnet-10",
            trustedHeight,
            ARBITRARY_INITIAL_APP_HASH,
            signedHeader.validators_hash.toBytes32(0),
            (uint64(signedHeader.time.secs) - 10) * 1e9
        );

        string memory clientId = handler.createClient(m);

        IbcCoreClientV1Height.Data memory latestHeight =
            client.getLatestHeight(clientId);

        assertEq(uint64(latestHeight.revision_height), trustedHeight);
        assertEq(uint64(latestHeight.revision_number), 0);
    }
}
