pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../core/IBCHandler.sol";
import "../../../contracts/clients/StateLensIcs23Ics23Client.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "solady/utils/LibString.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "solidity-stringutils/strings.sol";
import "solidity-bytes-utils/BytesLib.sol";

contract MockLightClient is ILightClient {
    bool isFrozenVar = false;
    bool globalVerifyMembership = true;

    function verifyMembership(
        uint32,
        uint64,
        bytes calldata,
        bytes calldata,
        bytes calldata
    ) public view override returns (bool) {
        return globalVerifyMembership;
    }

    function createClient(
        address,
        uint32,
        bytes calldata,
        bytes calldata,
        address
    )
        external
        returns (
            ConsensusStateUpdate memory update,
            string memory counterpartyChainId
        )
    {
        return (
            ConsensusStateUpdate({
                clientStateCommitment: bytes32(0),
                consensusStateCommitment: bytes32(0),
                height: 0
            }),
            ""
        );
    }

    function getClientState(
        uint32
    ) external view returns (bytes memory) {
        return abi.encodePacked("test");
    }

    function getTimestampAtHeight(
        uint32,
        uint64
    ) external view returns (uint64) {
        return 0;
    }

    function getLatestHeight(
        uint32
    ) external view returns (uint64 height) {
        return 0;
    }

    function updateClient(
        address,
        uint32,
        bytes calldata,
        address
    ) external returns (ConsensusStateUpdate memory update) {
        return ConsensusStateUpdate({
            clientStateCommitment: bytes32(0),
            consensusStateCommitment: bytes32(0),
            height: 0
        });
    }

    function verifyNonMembership(
        uint32,
        uint64,
        bytes calldata,
        bytes calldata
    ) external pure override returns (bool) {
        return true;
    }

    function getConsensusState(
        uint32,
        uint64
    ) external view returns (bytes memory) {
        return abi.encodePacked(uint64(0), keccak256("app"));
    }

    function isFrozen(
        uint32
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

    function misbehaviour(address, uint32, bytes calldata, address) external {}
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

contract StateLensIcs23Ics23ClientTest is Test {
    StateLensIcs23Ics23Client client;
    address admin = address(0xABcD);
    address ibcHandler; // = address(0x1234);
    MockIbcStore ibcStore;
    MockLightClient lightClient;

    function setUp() public {
        // Deploy and initialize the StateLensIcs23Ics23Client contract
        ibcStore = new MockIbcStore();
        ibcHandler = address(ibcStore);
        StateLensIcs23Ics23Client implementation =
            new StateLensIcs23Ics23Client(ibcHandler);
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeWithSelector(
                StateLensIcs23Ics23Client.initialize.selector, admin
            )
        );
        client = StateLensIcs23Ics23Client(address(proxy));
        lightClient = new MockLightClient();
        ibcStore.setClient(address(lightClient));
    }

    function test_initialize_ok() public {
        // Verify initialization
        assertEq(client.authority(), admin);
    }

    function test_createClient_success(
        address caller,
        address relayer
    ) public {
        uint32 clientId = 1;

        // Encode the client state

        ClientState memory clientState = ClientState({
            l2ChainId: "test-chain",
            l1ClientId: 2,
            l2ClientId: 3,
            l2LatestHeight: 100,
            contractAddress: keccak256("test")
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            keccak256("test")
        );

        // Encode the consensus state
        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: keccak256("app")
        });
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(ibcHandler); // Simulate call from the IBC handler
        client.createClient(
            caller, clientId, clientStateBytes, consensusStateBytes, relayer
        );

        // Verify client state
        bytes memory storedClientState = client.getClientState(clientId);
        assertEq(
            keccak256(storedClientState),
            keccak256(clientStateBytes),
            "Client state mismatch"
        );
        uint64 latestHeight = client.getLatestHeight(clientId);
        assertEq(
            latestHeight, clientState.l2LatestHeight, "Latest height mismatch"
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

    function test_createClient_ErrInvalidInitialConsensusState(
        address caller,
        address relayer
    ) public {
        uint32 clientId = 1;

        // Encode the client state

        ClientState memory clientState = ClientState({
            l2ChainId: "test-chain",
            l1ClientId: 2,
            l2ClientId: 3,
            l2LatestHeight: 0,
            contractAddress: keccak256("test")
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            keccak256("test")
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
                StateLensIcs23Ics23Lib.ErrInvalidInitialConsensusState.selector
            )
        );

        client.createClient(
            caller, clientId, clientStateBytes, consensusStateBytes, relayer
        );
    }

    function test_updateClient_success(
        address caller,
        address relayer
    ) public {
        uint32 clientId = 1;

        // Mock the initial client and consensus states
        ClientState memory clientState = ClientState({
            l2ChainId: "test-chain",
            l1ClientId: 2,
            l2ClientId: 3,
            l2LatestHeight: 100,
            contractAddress: keccak256("test")
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            keccak256("test")
        );

        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: keccak256("app")
        });
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(address(ibcHandler));
        client.createClient(
            caller, clientId, clientStateBytes, consensusStateBytes, relayer
        );

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
            client.updateClient(caller, clientId, clientMessageBytes, relayer);

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

    function test_updateClient_revert_invalid_proof(
        address caller,
        address relayer
    ) public {
        uint32 clientId = 1;

        // Mock the initial client and consensus states
        ClientState memory clientState = ClientState({
            l2ChainId: "test-chain",
            l1ClientId: 2,
            l2ClientId: 3,
            l2LatestHeight: 100,
            contractAddress: keccak256("test")
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            keccak256("test")
        );

        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: keccak256("app")
        });
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(address(ibcHandler));
        client.createClient(
            caller, clientId, clientStateBytes, consensusStateBytes, relayer
        );

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
            abi.encodeWithSelector(
                StateLensIcs23Ics23Lib.ErrInvalidL1Proof.selector
            )
        );
        client.updateClient(caller, clientId, clientMessageBytes, relayer);
    }

    function test_misbehavuour_error(address caller, address relayer) public {
        vm.prank(ibcHandler); // Simulate call from the IBC handler
        vm.expectRevert(
            abi.encodeWithSelector(
                StateLensIcs23Ics23Lib.ErrInvalidMisbehaviour.selector
            )
        );
        client.misbehaviour(caller, 1, bytes(""), relayer);
    }

    function test_isFrozenImpl(address caller, address relayer) public {
        uint32 clientId = 1;

        // Mock the initial client and consensus states
        ClientState memory clientState = ClientState({
            l2ChainId: "test-chain",
            l1ClientId: 2,
            l2ClientId: 3,
            l2LatestHeight: 100,
            contractAddress: keccak256("test")
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            keccak256("test")
        );

        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: keccak256("app")
        });
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(address(ibcHandler));
        client.createClient(
            caller, clientId, clientStateBytes, consensusStateBytes, relayer
        );

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

    function test_verifyMembershipIsFrozen(
        address caller,
        address relayer
    ) public {
        uint32 clientId = 1;

        // Mock the initial client and consensus states
        ClientState memory clientState = ClientState({
            l2ChainId: "test-chain",
            l1ClientId: 2,
            l2ClientId: 3,
            l2LatestHeight: 100,
            contractAddress: keccak256("test")
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            keccak256("test")
        );

        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: keccak256("app")
        });
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(address(ibcHandler));
        client.createClient(
            caller, clientId, clientStateBytes, consensusStateBytes, relayer
        );

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
            abi.encodeWithSelector(
                StateLensIcs23Ics23Lib.ErrClientFrozen.selector
            )
        );
        client.verifyMembership(1, 1, bytes(""), bytes(""), bytes(""));
    }

    function test_verifyNonMembershipIsFrozen(
        address caller,
        address relayer
    ) public {
        uint32 clientId = 1;

        // Mock the initial client and consensus states
        ClientState memory clientState = ClientState({
            l2ChainId: "test-chain",
            l1ClientId: 2,
            l2ClientId: 3,
            l2LatestHeight: 100,
            contractAddress: keccak256("test")
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            keccak256("test")
        );

        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: keccak256("app")
        });
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(address(ibcHandler));
        client.createClient(
            caller, clientId, clientStateBytes, consensusStateBytes, relayer
        );

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
            abi.encodeWithSelector(
                StateLensIcs23Ics23Lib.ErrClientFrozen.selector
            )
        );
        client.verifyNonMembership(1, 1, bytes(""), bytes(""));
    }
}
