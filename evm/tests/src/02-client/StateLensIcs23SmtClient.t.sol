// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../../../contracts/clients/StateLensIcs23SmtClient.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

/*//////////////////////////////////////////////////////////////
                        MOCKS
//////////////////////////////////////////////////////////////*/

contract MockLightClient is ILightClient {
    bool private frozenReturn;
    bool private verifyMembershipReturn = true;

    function setIsFrozenReturn(
        bool isFrozen
    ) public {
        frozenReturn = isFrozen;
    }

    function isFrozen(
        uint32 /*clientId*/
    ) external view override returns (bool) {
        return frozenReturn;
    }

    function setVerifyMembershipReturn(
        bool val
    ) external {
        verifyMembershipReturn = val;
    }

    // For membership checks (like in updateClient), always return our stored bool
    function verifyMembership(
        uint32, /*clientId*/
        uint64, /*height*/
        bytes calldata, /*proof*/
        bytes calldata, /*path*/
        bytes calldata /*value*/
    ) external view override returns (bool) {
        return verifyMembershipReturn;
    }

    function verifyNonMembership(
        uint32,
        uint64,
        bytes calldata,
        bytes calldata
    ) external pure override returns (bool) {
        return true;
    }

    // The other ILightClient functions are unused in these tests. We stub them out:
    function createClient(
        address,
        uint32, /*clientId*/
        bytes calldata, /*clientStateBytes*/
        bytes calldata, /*consensusStateBytes*/
        address
    ) external returns (ConsensusStateUpdate memory, string memory) {
        return (ConsensusStateUpdate(bytes32(0), bytes32(0), 0), "");
    }

    function updateClient(
        address,
        uint32, /*clientId*/
        bytes calldata, /*clientMessageBytes*/
        address
    ) external returns (ConsensusStateUpdate memory) {
        return ConsensusStateUpdate(bytes32(0), bytes32(0), 0);
    }

    function getLatestHeight(
        uint32 /*clientId*/
    ) external pure override returns (uint64) {
        return 0;
    }

    function getTimestampAtHeight(
        uint32, /*clientId*/
        uint64 /*height*/
    ) external pure override returns (uint64) {
        return 0;
    }

    function getConsensusState(
        uint32, /*clientId*/
        uint64 /*height*/
    ) external pure returns (bytes memory) {
        return new bytes(0);
    }

    function misbehaviour(address, uint32, bytes calldata, address) external {}

    function getClientState(
        uint32
    ) external view returns (bytes memory) {
        return abi.encodePacked("test");
    }
}

// Minimal mock IBCStore that returns our MockLightClient
contract MockIBCStore {
    address public client;

    function setClient(
        address _client
    ) external {
        client = _client;
    }

    function getClient(
        uint32 clientId
    ) external view returns (ILightClient) {
        return ILightClient(client);
    }

    // Unused IBCStore stubs:
    function setExpectedTimePerBlock(
        uint64 /*timePerBlock*/
    ) external {}

    function getExpectedTimePerBlock() external view returns (uint64) {
        return 0;
    }

    function setClientImpl(
        uint32, /*clientType*/
        address /*clientImpl*/
    ) external {}

    function getClientImpl(
        uint32 /*clientType*/
    ) external view returns (address) {
        return address(0);
    }

    function getClientType(
        uint32 /*clientId*/
    ) external pure returns (uint32) {
        return 0;
    }
}

/*//////////////////////////////////////////////////////////////
                        TEST CONTRACT
//////////////////////////////////////////////////////////////*/

contract StateLensIcs23SmtClientTest is Test {
    StateLensIcs23SmtClient client;
    MockIBCStore ibcStore;
    MockLightClient lightClient;
    address ibcHandler;
    address admin = address(0xABCD);

    function setUp() public {
        ibcStore = new MockIBCStore();
        ibcHandler = address(ibcStore);

        // Deploy the implementation
        StateLensIcs23SmtClient implementation = new StateLensIcs23SmtClient();
        // Deploy proxy
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeWithSelector(
                StateLensIcs23SmtClient.initialize.selector, ibcHandler, admin
            )
        );
        // Cast proxy -> client
        client = StateLensIcs23SmtClient(address(proxy));

        // Create our mock L1 client and store it in the mock IBCStore
        lightClient = new MockLightClient();
        ibcStore.setClient(address(lightClient));
    }

    function test_initialize_ok() public {
        assertEq(client.owner(), admin);
    }

    function test_createClient_success(
        address caller,
        address relayer
    ) public {
        uint32 clientId = 99;

        ClientState memory cState = ClientState({
            l2ChainId: "fake-l2",
            l1ClientId: 10,
            l2ClientId: 20,
            l2LatestHeight: 100,
            timestampOffset: 0,
            stateRootOffset: 32
        });
        bytes memory clientStateBytes = StateLensIcs23SmtLib.encode(cState);

        ConsensusState memory consState = ConsensusState({
            timestamp: 12345,
            stateRoot: keccak256("fake-root")
        });
        bytes memory consStateBytes = StateLensIcs23SmtLib.encode(consState);

        vm.prank(ibcHandler);

        client.createClient(
            caller, clientId, clientStateBytes, consStateBytes, relayer
        );

        bytes memory storedClientState = client.getClientState(clientId);
        assertEq(
            keccak256(storedClientState),
            keccak256(clientStateBytes),
            "Stored clientState mismatch"
        );

        bytes memory storedConsState = client.getConsensusState(clientId, 100);
        assertEq(
            keccak256(storedConsState),
            keccak256(consStateBytes),
            "Stored consensusState mismatch"
        );
    }

    function test_createClient_revert_initialState(
        address caller,
        address relayer
    ) public {
        uint32 clientId = 1;

        ClientState memory cState = ClientState({
            l2ChainId: "bad-l2",
            l1ClientId: 11,
            l2ClientId: 22,
            l2LatestHeight: 0,
            timestampOffset: 0,
            stateRootOffset: 32
        });
        bytes memory cStateBytes = StateLensIcs23SmtLib.encode(cState);

        ConsensusState memory consState =
            ConsensusState({timestamp: 1234, stateRoot: keccak256("x")});
        bytes memory consStateBytes = StateLensIcs23SmtLib.encode(consState);

        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                StateLensIcs23SmtLib.ErrInvalidInitialConsensusState.selector
            )
        );
        client.createClient(
            caller, clientId, cStateBytes, consStateBytes, relayer
        );
    }

    function test_updateClient_success(
        address caller,
        address relayer
    ) public {
        ClientState memory cState = ClientState({
            l2ChainId: "fake-l2",
            l1ClientId: 10,
            l2ClientId: 20,
            l2LatestHeight: 100,
            timestampOffset: 0,
            stateRootOffset: 32
        });
        ConsensusState memory cs =
            ConsensusState({timestamp: 9999, stateRoot: keccak256("old-root")});

        vm.startPrank(ibcHandler);
        client.createClient(
            caller,
            1,
            StateLensIcs23SmtLib.encode(cState),
            StateLensIcs23SmtLib.encode(cs),
            relayer
        );

        Header memory header = Header({
            l1Height: 500,
            l2Height: 101,
            l2InclusionProof: hex"DEADBEEF",
            l2ConsensusState: abi.encode(
                uint64(8888), keccak256("new-root"), keccak256("new-storage")
            )
        });
        bytes memory headerBytes = abi.encode(
            header.l1Height,
            header.l2Height,
            header.l2InclusionProof,
            header.l2ConsensusState
        );

        lightClient.setVerifyMembershipReturn(true);

        client.updateClient(caller, 1, headerBytes, relayer);

        bytes memory stored = client.getConsensusState(1, 101);
        ConsensusState memory dec = decodeConsensusState(stored);

        assertEq(dec.timestamp, 8888, "timestamp mismatch");
        assertEq(dec.stateRoot, keccak256("new-root"), "stateRoot mismatch");

        vm.stopPrank();
    }

    function test_updateClient_revert_invalidProof(
        address caller,
        address relayer
    ) public {
        ClientState memory cState = ClientState({
            l2ChainId: "fake-l2",
            l1ClientId: 10,
            l2ClientId: 20,
            l2LatestHeight: 100,
            timestampOffset: 0,
            stateRootOffset: 32
        });
        ConsensusState memory cs =
            ConsensusState({timestamp: 9999, stateRoot: keccak256("old-root")});

        vm.startPrank(ibcHandler);
        client.createClient(
            caller,
            123,
            StateLensIcs23SmtLib.encode(cState),
            StateLensIcs23SmtLib.encode(cs),
            relayer
        );

        Header memory header = Header({
            l1Height: 500,
            l2Height: 101,
            l2InclusionProof: hex"BADDBEEF",
            l2ConsensusState: abi.encodePacked(
                uint64(8888), keccak256("new-root"), keccak256("new-storage")
            )
        });
        bytes memory headerBytes = abi.encode(
            header.l1Height,
            header.l2Height,
            header.l2InclusionProof,
            header.l2ConsensusState
        );

        lightClient.setVerifyMembershipReturn(false);

        vm.expectRevert(
            abi.encodeWithSelector(
                StateLensIcs23SmtLib.ErrInvalidL1Proof.selector
            )
        );
        client.updateClient(caller, 123, headerBytes, relayer);
        vm.stopPrank();
    }

    function test_misbehaviour_reverts(
        address caller,
        address relayer
    ) public {
        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(StateLensIcs23SmtLib.ErrUnsupported.selector)
        );
        client.misbehaviour(caller, 1, bytes(""), relayer);
    }

    function test_isFrozenImpl(address caller, address relayer) public {
        ClientState memory cState = ClientState({
            l2ChainId: "fake-l2",
            l1ClientId: 10,
            l2ClientId: 20,
            l2LatestHeight: 100,
            timestampOffset: 0,
            stateRootOffset: 32
        });
        ConsensusState memory cs =
            ConsensusState({timestamp: 9999, stateRoot: keccak256("old-root")});
        vm.startPrank(ibcHandler);
        client.createClient(
            caller,
            999,
            StateLensIcs23SmtLib.encode(cState),
            StateLensIcs23SmtLib.encode(cs),
            relayer
        );

        lightClient.setIsFrozenReturn(true);

        bool frozen = client.isFrozen(999);
        assertTrue(frozen, "expected client to be frozen");
        vm.stopPrank();
    }

    function decodeConsensusState(
        bytes memory bz
    ) internal pure returns (ConsensusState memory) {
        ConsensusState memory consensusState;
        (uint64 timestamp, bytes32 stateRoot) =
            abi.decode(bz, (uint64, bytes32));
        consensusState.timestamp = timestamp;
        consensusState.stateRoot = stateRoot;
        return consensusState;
    }
}
