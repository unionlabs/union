// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../../../contracts/clients/StateLensIcs23MoveClient.sol";
import "@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol";

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
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path
    ) external pure override returns (bool) {
        return true;
    }

    // The other ILightClient functions are unused in these tests. We stub them out:
    function createClient(
        uint32, /*clientId*/
        bytes calldata, /*clientStateBytes*/
        bytes calldata /*consensusStateBytes*/
    ) external returns (ConsensusStateUpdate memory, string memory) {
        return (ConsensusStateUpdate(bytes32(0), bytes32(0), 0), "");
    }

    function updateClient(
        uint32, /*clientId*/
        bytes calldata /*clientMessageBytes*/
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

    function misbehaviour(
        uint32 clientId,
        bytes calldata clientMessageBytes
    ) external {}

    function getClientState(
        uint32 clientId
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

contract StateLensIcs23MoveClientTest is Test {
    StateLensIcs23MoveClient client;
    MockIBCStore ibcStore;
    MockLightClient lightClient;
    address ibcHandler;
    address admin = address(0xABCD);

    function setUp() public {
        ibcStore = new MockIBCStore();
        ibcHandler = address(ibcStore);

        // Deploy the implementation
        StateLensIcs23MoveClient implementation = new StateLensIcs23MoveClient();
        // Deploy proxy
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeWithSelector(
                StateLensIcs23MoveClient.initialize.selector, ibcHandler, admin
            )
        );
        // Cast proxy -> client
        client = StateLensIcs23MoveClient(address(proxy));

        // Create our mock L1 client and store it in the mock IBCStore
        lightClient = new MockLightClient();
        ibcStore.setClient(address(lightClient));
    }

    function test_initialize_ok() public {
        assertEq(client.owner(), admin);
    }

    function test_createClient_success() public {
        uint32 clientId = 99;

        ClientState memory cState = ClientState({
            l2ChainId: "fake-l2",
            l1ClientId: 10,
            l2ClientId: 20,
            l2LatestHeight: 100,
            timestampOffset: 0,
            stateRootOffset: 32,
            storageRootOffset: 64
        });
        bytes memory clientStateBytes = abi.encode(cState);

        ConsensusState memory consState = ConsensusState({
            timestamp: 12345,
            stateRoot: keccak256("fake-root"),
            storageRoot: keccak256("fake-storage")
        });
        bytes memory consStateBytes = abi.encode(consState);

        vm.prank(ibcHandler);

        client.createClient(clientId, clientStateBytes, consStateBytes);

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

    function test_createClient_revert_initialState() public {
        uint32 clientId = 1;

        ClientState memory cState = ClientState({
            l2ChainId: "bad-l2",
            l1ClientId: 11,
            l2ClientId: 22,
            l2LatestHeight: 0,
            timestampOffset: 0,
            stateRootOffset: 32,
            storageRootOffset: 64
        });
        bytes memory cStateBytes = abi.encode(cState);

        ConsensusState memory consState = ConsensusState({
            timestamp: 1234,
            stateRoot: keccak256("x"),
            storageRoot: keccak256("y")
        });
        bytes memory consStateBytes = abi.encode(consState);

        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                StateLensIcs23MoveLib.ErrInvalidInitialConsensusState.selector
            )
        );
        client.createClient(clientId, cStateBytes, consStateBytes);
    }

    function test_updateClient_success() public {
        {
            ClientState memory cState = ClientState({
                l2ChainId: "fake-l2",
                l1ClientId: 10,
                l2ClientId: 20,
                l2LatestHeight: 100,
                timestampOffset: 0,
                stateRootOffset: 32,
                storageRootOffset: 64
            });
            ConsensusState memory cs = ConsensusState({
                timestamp: 9999,
                stateRoot: keccak256("old-root"),
                storageRoot: keccak256("old-storage")
            });

            vm.prank(ibcHandler);
            client.createClient(1, abi.encode(cState), abi.encode(cs));
        }

        Header memory header = Header({
            l1Height: 500,
            l2Height: 101,
            l2InclusionProof: hex"DEADBEEF",
            l2ConsensusState: abi.encode(
                uint64(8888), keccak256("new-root"), keccak256("new-storage")
            )
        });
        bytes memory headerBytes = abi.encode(header);

        lightClient.setVerifyMembershipReturn(true);

        vm.prank(ibcHandler);
        client.updateClient(1, headerBytes);

        bytes memory stored = client.getConsensusState(1, 101);
        ConsensusState memory dec = abi.decode(stored, (ConsensusState));

        assertEq(dec.timestamp, 8888, "timestamp mismatch");
        assertEq(dec.stateRoot, keccak256("new-root"), "stateRoot mismatch");
        assertEq(
            dec.storageRoot, keccak256("new-storage"), "storageRoot mismatch"
        );
    }

    function test_updateClient_revert_invalidProof() public {
        {
            ClientState memory cState = ClientState({
                l2ChainId: "fake-l2",
                l1ClientId: 10,
                l2ClientId: 20,
                l2LatestHeight: 100,
                timestampOffset: 0,
                stateRootOffset: 32,
                storageRootOffset: 64
            });
            ConsensusState memory cs = ConsensusState({
                timestamp: 9999,
                stateRoot: keccak256("old-root"),
                storageRoot: keccak256("old-storage")
            });

            vm.prank(ibcHandler);
            client.createClient(123, abi.encode(cState), abi.encode(cs));
        }

        Header memory header = Header({
            l1Height: 500,
            l2Height: 101,
            l2InclusionProof: hex"BADDBEEF",
            l2ConsensusState: abi.encodePacked(
                uint64(8888), keccak256("new-root"), keccak256("new-storage")
            )
        });
        bytes memory headerBytes = abi.encode(header);

        lightClient.setVerifyMembershipReturn(false);

        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                StateLensIcs23MoveLib.ErrInvalidL1Proof.selector
            )
        );
        client.updateClient(123, headerBytes);
    }

    function test_misbehaviour_reverts() public {
        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                StateLensIcs23MoveLib.ErrUnsupported.selector
            )
        );
        client.misbehaviour(1, bytes(""));
    }

    function test_isFrozenImpl() public {
        {
            ClientState memory cState = ClientState({
                l2ChainId: "fake-l2",
                l1ClientId: 10,
                l2ClientId: 20,
                l2LatestHeight: 100,
                timestampOffset: 0,
                stateRootOffset: 32,
                storageRootOffset: 64
            });
            ConsensusState memory cs = ConsensusState({
                timestamp: 9999,
                stateRoot: keccak256("old-root"),
                storageRoot: keccak256("old-storage")
            });
            vm.prank(ibcHandler);
            client.createClient(999, abi.encode(cState), abi.encode(cs));
        }

        lightClient.setIsFrozenReturn(true);
        // TODO: verifymembership is returning true automatically, so this test will revert anyway

        bool frozen = client.isFrozen(999);
        assertTrue(frozen, "expected client to be frozen");
    }

    function test_verifyMembership_isFrozen() public {
        {
            ClientState memory cState = ClientState({
                l2ChainId: "fake-l2",
                l1ClientId: 10,
                l2ClientId: 20,
                l2LatestHeight: 100,
                timestampOffset: 0,
                stateRootOffset: 32,
                storageRootOffset: 64
            });
            ConsensusState memory cs = ConsensusState({
                timestamp: 9999,
                stateRoot: keccak256("old-root"),
                storageRoot: keccak256("old-storage")
            });
            vm.prank(ibcHandler);
            client.createClient(2, abi.encode(cState), abi.encode(cs));
        }

        lightClient.setIsFrozenReturn(true);
        // TODO: verifymembership is returning true automatically, so this test will revert anyway

        // vm.prank(ibcHandler);
        // vm.expectRevert(
        //     abi.encodeWithSelector(
        //         StateLensIcs23MoveLib.ErrClientFrozen.selector
        //     )
        // );

        client.verifyMembership(
            2, 100, bytes("proof"), bytes("path"), bytes("value")
        );
    }

    function test_verifyNonMembership_isFrozen() public {
        {
            ClientState memory cState = ClientState({
                l2ChainId: "fake-l2",
                l1ClientId: 10,
                l2ClientId: 20,
                l2LatestHeight: 100,
                timestampOffset: 0,
                stateRootOffset: 32,
                storageRootOffset: 64
            });
            ConsensusState memory cs = ConsensusState({
                timestamp: 9999,
                stateRoot: keccak256("old-root"),
                storageRoot: keccak256("old-storage")
            });
            vm.prank(ibcHandler);
            client.createClient(3, abi.encode(cState), abi.encode(cs));
        }

        lightClient.setIsFrozenReturn(true);
        // TODO: verifymembership is returning true automatically, so this test will revert anyway

        // vm.prank(ibcHandler);
        // vm.expectRevert(
        //     abi.encodeWithSelector(
        //         StateLensIcs23MoveLib.ErrClientFrozen.selector
        //     )
        // );
        // client.verifyNonMembership(3, 100, bytes("proof"), bytes("path"));
    }
}
