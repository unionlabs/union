// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../../../contracts/clients/StateLensIcs23MoveClient.sol";
import "../../../contracts/lib/SparseMerkleVerifier.sol"; // if needed
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
        uint32, /*clientId*/
        uint64, /*height*/
        bytes calldata, /*proof*/
        bytes calldata, /*path*/
        bytes calldata /*value*/
    ) external pure override returns (bool) {
        // For simplicity, just always return true
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
}

// Minimal mock IBCStore that returns our MockLightClient
contract MockIBCStore is IBCStore {
    address public client;

    function setClient(
        address _client
    ) external {
        client = _client;
    }

    function getClient(
        uint32 /*clientId*/
    ) external view override returns (ILightClient) {
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

    //-------------------------------------
    // 1. Setup
    //-------------------------------------
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

    //-------------------------------------
    // 2. Basic Admin Checks
    //-------------------------------------
    function test_initialize_ok() public {
        // Should be the admin set in initialize
        assertEq(client.owner(), admin);
    }

    //-------------------------------------
    // 3. createClient Tests
    //-------------------------------------
    function test_createClient_success() public {
        uint32 clientId = 99;

        // Build a valid clientState
        ClientState memory cState = ClientState({
            l2ChainId: "fake-l2",
            l1ClientId: 10,
            l2ClientId: 20,
            l2LatestHeight: 100,
            timestampOffset: 0,
            stateRootOffset: 32
        });
        bytes memory clientStateBytes = abi.encode(cState);

        // Build a valid consensusState
        ConsensusState memory consState = ConsensusState({
            timestamp: 12345,
            stateRoot: keccak256("fake-root"),
            storageRoot: keccak256("fake-storage")
        });
        bytes memory consStateBytes = abi.encode(consState);

        // We must call createClient from the ibcHandler
        vm.prank(ibcHandler);

        // Expect success
        client.createClient(clientId, clientStateBytes, consStateBytes);

        // Check that the clientState is stored
        bytes memory storedClientState = client.getClientState(clientId);
        assertEq(
            keccak256(storedClientState),
            keccak256(clientStateBytes),
            "Stored clientState mismatch"
        );

        // Check that the consensus state is stored
        bytes memory storedConsState = client.getConsensusState(clientId, 100);
        assertEq(
            keccak256(storedConsState),
            keccak256(consStateBytes),
            "Stored consensusState mismatch"
        );
    }

    function test_createClient_revert_initialState() public {
        uint32 clientId = 1;

        // l2LatestHeight=0 => triggers revert
        ClientState memory cState = ClientState({
            l2ChainId: "bad-l2",
            l1ClientId: 11,
            l2ClientId: 22,
            l2LatestHeight: 0,
            timestampOffset: 0,
            stateRootOffset: 32
        });
        bytes memory cStateBytes = abi.encode(cState);

        // consensusState with non-zero is fine, but the zero height triggers revert
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

    //-------------------------------------
    // 4. updateClient Tests
    //-------------------------------------
    function test_updateClient_success() public {
        // Step 1: create a valid client
        {
            ClientState memory cState = ClientState({
                l2ChainId: "fake-l2",
                l1ClientId: 10,
                l2ClientId: 20,
                l2LatestHeight: 100,
                timestampOffset: 0,
                stateRootOffset: 32
            });
            ConsensusState memory cs = ConsensusState({
                timestamp: 9999,
                stateRoot: keccak256("old-root"),
                storageRoot: keccak256("old-storage")
            });

            vm.prank(ibcHandler);
            client.createClient(1, abi.encode(cState), abi.encode(cs));
        }

        // Step 2: Build a fake header
        // The contract’s offset fields => timestampOffset=0, stateRootOffset=32 => we’ll store them at 0, 32, 64 in l2ConsensusState
        Header memory header = Header({
            l1Height: 500,
            l2Height: 101,
            l2InclusionProof: hex"DEADBEEF",
            l2ConsensusState: abi.encodePacked(
                // bytes at offset=0 => timestamp (uint64)
                uint64(8888),
                // bytes at offset=32 => stateRoot
                keccak256("new-root"),
                // bytes at offset=64 => storageRoot
                keccak256("new-storage")
            )
        });
        bytes memory headerBytes = abi.encode(header);

        // Step 3: Let the L1 client pass membership
        lightClient.setVerifyMembershipReturn(true);

        // Step 4: updateClient
        vm.prank(ibcHandler);
        client.updateClient(1, headerBytes);

        // Step 5: Verify the new consensus state at l2Height=101
        bytes memory stored = client.getConsensusState(1, 101);
        ConsensusState memory dec = abi.decode(stored, (ConsensusState));
        assertEq(dec.timestamp, 8888, "timestamp mismatch");
        assertEq(dec.stateRoot, keccak256("new-root"), "stateRoot mismatch");
        assertEq(
            dec.storageRoot, keccak256("new-storage"), "storageRoot mismatch"
        );
    }

    function test_updateClient_revert_invalidProof() public {
        // Step 1: create a valid client
        {
            ClientState memory cState = ClientState({
                l2ChainId: "fake-l2",
                l1ClientId: 10,
                l2ClientId: 20,
                l2LatestHeight: 100,
                timestampOffset: 0,
                stateRootOffset: 32
            });
            ConsensusState memory cs = ConsensusState({
                timestamp: 9999,
                stateRoot: keccak256("old-root"),
                storageRoot: keccak256("old-storage")
            });

            vm.prank(ibcHandler);
            client.createClient(123, abi.encode(cState), abi.encode(cs));
        }

        // Build a fake header
        Header memory header = Header({
            l1Height: 500,
            l2Height: 101,
            l2InclusionProof: hex"BADDBEEF",
            l2ConsensusState: abi.encodePacked(
                uint64(8888), keccak256("new-root"), keccak256("new-storage")
            )
        });
        bytes memory headerBytes = abi.encode(header);

        // Force membership check to fail
        lightClient.setVerifyMembershipReturn(false);

        // Expect revert with ErrInvalidL1Proof
        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                StateLensIcs23MoveLib.ErrInvalidL1Proof.selector
            )
        );
        client.updateClient(123, headerBytes);
    }

    //-------------------------------------
    // 5. misbehaviour Test
    //-------------------------------------
    function test_misbehaviour_reverts() public {
        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                StateLensIcs23MoveLib.ErrUnsupported.selector
            )
        );
        client.misbehaviour(1, bytes(""));
    }

    //-------------------------------------
    // 6. Frozen Client & Membership
    //-------------------------------------
    function test_isFrozenImpl() public {
        // 1. Create a client
        {
            ClientState memory cState = ClientState({
                l2ChainId: "fake-l2",
                l1ClientId: 10,
                l2ClientId: 20,
                l2LatestHeight: 100,
                timestampOffset: 0,
                stateRootOffset: 32
            });
            ConsensusState memory cs = ConsensusState({
                timestamp: 9999,
                stateRoot: keccak256("old-root"),
                storageRoot: keccak256("old-storage")
            });
            vm.prank(ibcHandler);
            client.createClient(999, abi.encode(cState), abi.encode(cs));
        }

        // 2. Freeze the mock L1 client
        lightClient.setIsFrozenReturn(true);

        // 3. Check isFrozen
        bool frozen = client.isFrozen(999);
        assertTrue(frozen, "expected client to be frozen");
    }

    function test_verifyMembership_isFrozen() public {
        // 1. Create a client
        {
            ClientState memory cState = ClientState({
                l2ChainId: "fake-l2",
                l1ClientId: 10,
                l2ClientId: 20,
                l2LatestHeight: 100,
                timestampOffset: 0,
                stateRootOffset: 32
            });
            ConsensusState memory cs = ConsensusState({
                timestamp: 9999,
                stateRoot: keccak256("old-root"),
                storageRoot: keccak256("old-storage")
            });
            vm.prank(ibcHandler);
            client.createClient(2, abi.encode(cState), abi.encode(cs));
        }

        // Freeze
        lightClient.setIsFrozenReturn(true);

        // 2. Attempt verifyMembership
        // Construct minimal proof => in real usage you'd pass a real SparseMerkleProof
        SparseMerkleVerifier.SparseMerkleProof memory proof;
        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                StateLensIcs23MoveLib.ErrClientFrozen.selector
            )
        );
        client.verifyMembership(2, 100, proof, bytes("path"), bytes("value"));
    }

    function test_verifyNonMembership_isFrozen() public {
        // 1. Create a client
        {
            ClientState memory cState = ClientState({
                l2ChainId: "fake-l2",
                l1ClientId: 10,
                l2ClientId: 20,
                l2LatestHeight: 100,
                timestampOffset: 0,
                stateRootOffset: 32
            });
            ConsensusState memory cs = ConsensusState({
                timestamp: 9999,
                stateRoot: keccak256("old-root"),
                storageRoot: keccak256("old-storage")
            });
            vm.prank(ibcHandler);
            client.createClient(3, abi.encode(cState), abi.encode(cs));
        }

        // Freeze
        lightClient.setIsFrozenReturn(true);

        // 2. Attempt verifyNonMembership => expect revert
        SparseMerkleVerifier.SparseMerkleProof memory proof;
        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                StateLensIcs23MoveLib.ErrClientFrozen.selector
            )
        );
        client.verifyNonMembership(3, 100, proof, bytes("path"), bytes("path"));
    }
}
