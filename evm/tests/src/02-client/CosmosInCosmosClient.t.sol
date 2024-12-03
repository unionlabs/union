// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../core/IBCHandler.sol";
import "../core/Relay.sol";
import "../../../contracts/clients/CosmosInCosmosClient.sol";
import "@openzeppelin/proxy/ERC1967/ERC1967Proxy.sol";
import "solady/utils/LibString.sol";
import "@openzeppelin/token/ERC20/ERC20.sol";
import "solidity-stringutils/strings.sol";
import "solidity-bytes-utils/BytesLib.sol";

contract MockLightClient is ILightClient {
    bool isFrozenVar = false;
    bool globalVerifyMembership = true;

    function verifyMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path,
        bytes calldata value
    ) public view override returns (bool) {
        return globalVerifyMembership;
    }

    function createClient(
        uint32 clientId,
        bytes calldata clientStateBytes,
        bytes calldata consensusStateBytes
    ) external returns (ConsensusStateUpdate memory update) {
        return ConsensusStateUpdate({
            clientStateCommitment: bytes32(0),
            consensusStateCommitment: bytes32(0),
            height: 0
        });
    }

    function getClientState(
        uint32 clientId
    ) external view returns (bytes memory) {
        return abi.encodePacked("test");
    }

    function getTimestampAtHeight(
        uint32 clientId,
        uint64 height
    ) external view returns (uint64) {
        return 0;
    }

    function getLatestHeight(
        uint32 clientId
    ) external view returns (uint64 height) {
        return 0;
    }

    function updateClient(
        uint32 clientId,
        bytes calldata clientMessageBytes
    ) external returns (ConsensusStateUpdate memory update) {
        return ConsensusStateUpdate({
            clientStateCommitment: bytes32(0),
            consensusStateCommitment: bytes32(0),
            height: 0
        });
    }

    function verifyNonMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path
    ) external pure override returns (bool) {
        return true;
    }

    function getConsensusState(
        uint32 clientId,
        uint64 height
    ) external view returns (bytes memory) {
        return abi.encodePacked(uint64(0), keccak256("app"));
    }

    function isFrozen(
        uint32 clientId
    ) external view returns (bool) {
        return isFrozenVar;
    }

    function setIsFrozenReturn(
        bool isFrozen
    ) public {
        isFrozenVar = isFrozen;
    }

    function setVerifyMembership(
        bool verify_membership
    ) public {
        globalVerifyMembership = verify_membership;
    }

    function misbehaviour(
        uint32 clientId,
        bytes calldata clientMessageBytes
    ) external {}
}

contract MockIbcStore {
    address public client;

    function getClient(
        uint32 clientId
    ) external view returns (ILightClient) {
        return ILightClient(client);
    }

    function setClient(
        address _client
    ) public {
        client = _client;
    }
}

contract CosmosInCosmosClientTest is Test {
    CosmosInCosmosClient client;
    address admin = address(0xABcD);
    address ibcHandler; // = address(0x1234);
    MockIbcStore ibcStore;
    MockLightClient lightClient;

    function setUp() public {
        // Deploy and initialize the CosmosInCosmosClient contract
        ibcStore = new MockIbcStore();
        ibcHandler = address(ibcStore);
        CosmosInCosmosClient implementation = new CosmosInCosmosClient();
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeWithSelector(
                CosmosInCosmosClient.initialize.selector, ibcHandler, admin
            )
        );
        client = CosmosInCosmosClient(address(proxy));
        lightClient = new MockLightClient();
        ibcStore.setClient(address(lightClient));
    }

    function test_initialize_ok() public {
        // Verify initialization
        assertEq(client.owner(), admin);
    }

    function test_createClient_success() public {
        uint32 clientId = 1;

        // Encode the client state

        ClientState memory clientState = ClientState({
            l2ChainId: "test-chain",
            l1ClientId: 2,
            l2ClientId: 3,
            latestHeight: 100
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.latestHeight
        );

        // Encode the consensus state
        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: keccak256("app")
        });
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(ibcHandler); // Simulate call from the IBC handler
        client.createClient(clientId, clientStateBytes, consensusStateBytes);

        // Verify client state
        bytes memory storedClientState = client.getClientState(clientId);
        assertEq(
            keccak256(storedClientState),
            keccak256(clientStateBytes),
            "Client state mismatch"
        );
        uint64 latestHeight = client.getLatestHeight(clientId);
        assertEq(
            latestHeight, clientState.latestHeight, "Latest height mismatch"
        );

        // Verify consensus state
        bytes memory storedConsensusState =
            client.getConsensusState(clientId, 100);
        assertEq(
            keccak256(storedConsensusState),
            keccak256(consensusStateBytes),
            "Consensus state mismatch"
        );
        uint64 timestamp_at_height = client.getTimestampAtHeight(clientId, 100);
        assertEq(
            timestamp_at_height, consensusState.timestamp, "Timestamp mismatch"
        );
    }

    function test_createClient_ErrInvalidInitialConsensusState() public {
        uint32 clientId = 1;

        // Encode the client state

        ClientState memory clientState = ClientState({
            l2ChainId: "test-chain",
            l1ClientId: 2,
            l2ClientId: 3,
            latestHeight: 0
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.latestHeight
        );

        // Encode the consensus state
        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: keccak256("app")
        });
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(ibcHandler); // Simulate call from the IBC handler
        vm.expectRevert(
            abi.encodeWithSelector(
                CosmosInCosmosLib.ErrInvalidInitialConsensusState.selector
            )
        );

        client.createClient(clientId, clientStateBytes, consensusStateBytes);
    }

    function test_updateClient_success() public {
        uint32 clientId = 1;

        // Mock the initial client and consensus states
        ClientState memory clientState = ClientState({
            l2ChainId: "test-chain",
            l1ClientId: 2,
            l2ClientId: 3,
            latestHeight: 100
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.latestHeight
        );

        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: keccak256("app")
        });
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(address(ibcHandler));
        client.createClient(clientId, clientStateBytes, consensusStateBytes);

        // Prepare update client message
        Header memory header = Header({
            l1Height: 101,
            l2Height: 102,
            l2InclusionProof: bytes("proof"),
            l2ConsensusState: abi.encode(
                TendermintConsensusState({
                    timestamp: uint64(block.timestamp + 1),
                    appHash: keccak256("newApp"),
                    nextValidatorsHash: keccak256("newValidators")
                })
            )
        });
        bytes memory clientMessageBytes = abi.encode(
            header.l1Height,
            header.l2Height,
            header.l2InclusionProof,
            header.l2ConsensusState
        );

        lightClient.setVerifyMembership(true);
        // Update client
        vm.prank(address(ibcHandler));
        ConsensusStateUpdate memory update =
            client.updateClient(clientId, clientMessageBytes);

        // // Verify updated consensus state
        bytes memory updatedConsensusState =
            client.getConsensusState(clientId, 102);

        // Decode the updated consensus state
        ConsensusState memory decodedConsensusState =
            abi.decode(updatedConsensusState, (ConsensusState));

        assertEq(
            decodedConsensusState.timestamp,
            uint64(block.timestamp + 1),
            "Consensus state timestamp mismatch"
        );
        assertEq(
            decodedConsensusState.appHash,
            keccak256("newApp"),
            "Consensus state appHash mismatch"
        );
    }

    function test_updateClient_revert_invalid_proof() public {
        uint32 clientId = 1;

        // Mock the initial client and consensus states
        ClientState memory clientState = ClientState({
            l2ChainId: "test-chain",
            l1ClientId: 2,
            l2ClientId: 3,
            latestHeight: 100
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.latestHeight
        );

        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: keccak256("app")
        });
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(address(ibcHandler));
        client.createClient(clientId, clientStateBytes, consensusStateBytes);

        // Prepare update client message
        Header memory header = Header({
            l1Height: 101,
            l2Height: 102,
            l2InclusionProof: bytes("proof"),
            l2ConsensusState: abi.encode(
                TendermintConsensusState({
                    timestamp: uint64(block.timestamp + 1),
                    appHash: keccak256("newApp"),
                    nextValidatorsHash: keccak256("newValidators")
                })
            )
        });
        bytes memory clientMessageBytes = abi.encode(
            header.l1Height,
            header.l2Height,
            header.l2InclusionProof,
            header.l2ConsensusState
        );

        lightClient.setVerifyMembership(false);
        // Update client
        vm.prank(address(ibcHandler));
        vm.expectRevert(
            abi.encodeWithSelector(CosmosInCosmosLib.ErrInvalidL1Proof.selector)
        );
        client.updateClient(clientId, clientMessageBytes);
    }

    function test_misbehavuour_error() public {
        vm.prank(ibcHandler); // Simulate call from the IBC handler
        vm.expectRevert(
            abi.encodeWithSelector(
                CosmosInCosmosLib.ErrInvalidMisbehaviour.selector
            )
        );
        client.misbehaviour(1, bytes(""));
    }

    function test_isFrozenImpl() public {
        uint32 clientId = 1;

        // Mock the initial client and consensus states
        ClientState memory clientState = ClientState({
            l2ChainId: "test-chain",
            l1ClientId: 2,
            l2ClientId: 3,
            latestHeight: 100
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.latestHeight
        );

        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: keccak256("app")
        });
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(address(ibcHandler));
        client.createClient(clientId, clientStateBytes, consensusStateBytes);

        // Prepare update client message
        Header memory header = Header({
            l1Height: 101,
            l2Height: 102,
            l2InclusionProof: bytes("proof"),
            l2ConsensusState: abi.encode(
                TendermintConsensusState({
                    timestamp: uint64(block.timestamp + 1),
                    appHash: keccak256("newApp"),
                    nextValidatorsHash: keccak256("newValidators")
                })
            )
        });
        bytes memory clientMessageBytes = abi.encode(
            header.l1Height,
            header.l2Height,
            header.l2InclusionProof,
            header.l2ConsensusState
        );

        // Update client
        vm.prank(address(ibcHandler));
        lightClient.setIsFrozenReturn(true);
        bool isFrozen = client.isFrozen(clientId);
        assertTrue(isFrozen, "Client should be frozen");
    }

    function test_verifyMembershipIsFrozen() public {
        uint32 clientId = 1;

        // Mock the initial client and consensus states
        ClientState memory clientState = ClientState({
            l2ChainId: "test-chain",
            l1ClientId: 2,
            l2ClientId: 3,
            latestHeight: 100
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.latestHeight
        );

        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: keccak256("app")
        });
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(address(ibcHandler));
        client.createClient(clientId, clientStateBytes, consensusStateBytes);

        // Prepare update client message
        Header memory header = Header({
            l1Height: 101,
            l2Height: 102,
            l2InclusionProof: bytes("proof"),
            l2ConsensusState: abi.encode(
                TendermintConsensusState({
                    timestamp: uint64(block.timestamp + 1),
                    appHash: keccak256("newApp"),
                    nextValidatorsHash: keccak256("newValidators")
                })
            )
        });
        bytes memory clientMessageBytes = abi.encode(
            header.l1Height,
            header.l2Height,
            header.l2InclusionProof,
            header.l2ConsensusState
        );

        // Update client
        lightClient.setIsFrozenReturn(true);
        vm.prank(address(ibcHandler));
        vm.expectRevert(
            abi.encodeWithSelector(CosmosInCosmosLib.ErrClientFrozen.selector)
        );
        client.verifyMembership(1, 1, bytes(""), bytes(""), bytes(""));
    }

    function test_verifyNonMembershipIsFrozen() public {
        uint32 clientId = 1;

        // Mock the initial client and consensus states
        ClientState memory clientState = ClientState({
            l2ChainId: "test-chain",
            l1ClientId: 2,
            l2ClientId: 3,
            latestHeight: 100
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.latestHeight
        );

        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: keccak256("app")
        });
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(address(ibcHandler));
        client.createClient(clientId, clientStateBytes, consensusStateBytes);

        // Prepare update client message
        Header memory header = Header({
            l1Height: 101,
            l2Height: 102,
            l2InclusionProof: bytes("proof"),
            l2ConsensusState: abi.encode(
                TendermintConsensusState({
                    timestamp: uint64(block.timestamp + 1),
                    appHash: keccak256("newApp"),
                    nextValidatorsHash: keccak256("newValidators")
                })
            )
        });
        bytes memory clientMessageBytes = abi.encode(
            header.l1Height,
            header.l2Height,
            header.l2InclusionProof,
            header.l2ConsensusState
        );

        // Update client
        lightClient.setIsFrozenReturn(true);
        vm.prank(address(ibcHandler));
        vm.expectRevert(
            abi.encodeWithSelector(CosmosInCosmosLib.ErrClientFrozen.selector)
        );
        client.verifyNonMembership(1, 1, bytes(""), bytes(""));
    }
}
