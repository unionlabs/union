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

    using StateLensIcs23Ics23Lib for *;

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
            version: uint256(1),
            state: ExtraV1({
                storeKey: bytes("wasm"),
                keyPrefixStorage: abi.encodePacked(
                    bytes1(0x03), keccak256("test"), bytes1(0x00)
                )
            }).encode()
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            clientState.version,
            clientState.state
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
            version: uint256(1),
            state: ExtraV1({
                storeKey: bytes("wasm"),
                keyPrefixStorage: abi.encodePacked(
                    bytes1(0x03), keccak256("test"), bytes1(0x00)
                )
            }).encode()
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            clientState.version,
            clientState.state
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
            version: uint256(1),
            state: ExtraV1({
                storeKey: bytes("wasm"),
                keyPrefixStorage: abi.encodePacked(
                    bytes1(0x03), keccak256("test"), bytes1(0x00)
                )
            }).encode()
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            clientState.version,
            clientState.state
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
            version: uint256(1),
            state: ExtraV1({
                storeKey: bytes("wasm"),
                keyPrefixStorage: abi.encodePacked(
                    bytes1(0x03), keccak256("test"), bytes1(0x00)
                )
            }).encode()
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            clientState.version,
            clientState.state
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
            version: uint256(1),
            state: ExtraV1({
                storeKey: bytes("wasm"),
                keyPrefixStorage: abi.encodePacked(
                    bytes1(0x03), keccak256("test"), bytes1(0x00)
                )
            }).encode()
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            clientState.version,
            clientState.state
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
            version: uint256(1),
            state: ExtraV1({
                storeKey: bytes("wasm"),
                keyPrefixStorage: abi.encodePacked(
                    bytes1(0x03), keccak256("test"), bytes1(0x00)
                )
            }).encode()
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            clientState.version,
            clientState.state
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
            version: uint256(1),
            state: ExtraV1({
                storeKey: bytes("wasm"),
                keyPrefixStorage: abi.encodePacked(
                    bytes1(0x03), keccak256("test"), bytes1(0x00)
                )
            }).encode()
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            clientState.version,
            clientState.state
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

    function test_verifyMembershipV2(address caller, address relayer) public {
        uint32 clientId = 1;

        // Mock the initial client and consensus states
        ClientState memory clientState = ClientState({
            l2ChainId: "atlantic-2",
            l1ClientId: 2,
            l2ClientId: 3,
            l2LatestHeight: 173218977,
            version: uint256(2),
            state: ExtraV2({
                storeKey: bytes("evm"),
                keyPrefixStorage: abi.encodePacked(
                    bytes1(0x03), hex"ee4ea8d358473f0fcebf0329feed95d56e8c04d7"
                )
            }).encode()
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            clientState.version,
            clientState.state
        );

        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: 0xD0A81718CC318725C92E633106070CAF16E51A447DED433F34C7C53DF6BADA71
        });
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(address(ibcHandler));
        client.createClient(
            caller, clientId, clientStateBytes, consensusStateBytes, relayer
        );

        // voyager rpc ibc-proof 1328 '{"client_state":{"client_id":4}}' --encode --ibc-interface ibc-solidity --client-type state-lens/ics23/ics23 --height 173218977
        bytes memory proof =
            hex"00000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000001aa0000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000000000003503ee4ea8d358473f0fcebf0329feed95d56e8c04d7b5c1de0ab73c11497798f429ec187ad784c3261f62637108ef73835a93ab9614000000000000000000000000000000000000000000000000000000000000000000000000000000000000209a0d99845a85f92af893fcc4f90db7fb76d262dfc144459c6e90642c0e4bb43700000000000000000000000000000000000000000000000000000000000000070002ea8f94a50100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b00000000000000000000000000000000000000000000000000000000000003600000000000000000000000000000000000000000000000000000000000000440000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000005e000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000007a0000000000000000000000000000000000000000000000000000000000000086000000000000000000000000000000000000000000000000000000000000009200000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000ac00000000000000000000000000000000000000000000000000000000000000b800000000000000000000000000000000000000000000000000000000000000c400000000000000000000000000000000000000000000000000000000000000d200000000000000000000000000000000000000000000000000000000000000de00000000000000000000000000000000000000000000000000000000000000ea00000000000000000000000000000000000000000000000000000000000000f800000000000000000000000000000000000000000000000000000000000001060000000000000000000000000000000000000000000000000000000000000112000000000000000000000000000000000000000000000000000000000000011e000000000000000000000000000000000000000000000000000000000000012a000000000000000000000000000000000000000000000000000000000000013600000000000000000000000000000000000000000000000000000000000001440000000000000000000000000000000000000000000000000000000000000152000000000000000000000000000000000000000000000000000000000000015e000000000000000000000000000000000000000000000000000000000000016a0000000000000000000000000000000000000000000000000000000000000176000000000000000000000000000000000000000000000000000000000000018200000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000080204ea8f94a501200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000021207912be8970359113fbb865562835e76480fa28b5f35d346faae7fb6ad96bb37700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000290408ea8f94a501205a26dbccd29242a010a0922b2a02b03ce7e8ea8f426185ea94ecdb1c75ba3a442000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000008060cea8f94a5012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000212058f4602044ae86d9f9e9fce2e04fb9af9d869169089f01157e71319f6567a35c00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000008081aea8f94a501200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000021207afacadec7f780ef84faed570a47c6e90593b3e5648224477ac21a90c4de6c0c000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000080a2aea8f94a50120000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120a807233f4d3c83321ab80ee8ed08a2ac703793b61a612351dcfc92939a164afb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000290c46ea8f94a501206a66a24db521b718e40e22df3afdabc4af0204dee2b8587b34c192bb3cd88ba32000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a0e8801ea8f94a50120cab16c18f017be51a65f473d5e7bdbd1318cf81665b9878b0cfa3880b76accd82000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000910c801ea8f94a5012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000021202e7118befb4cc724ca9022aaf8cd29de1b065972387f5f4b0b95429545268f5d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a12a602ea8f94a5012027808a27bf3490a5c9f4839ccc23f037c3119b0a502d858a4887bc4a69b61ceb20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a14bc04ea8f94a501203f5dc44e6e016ee2cf63117652a41e798da305e2ecafe56cd47ee8efb3d283b020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a168008ea8f94a50120a93bd924367779b7fd85f74b015aaa32c4fc36a11f253308454f615476e2aa7a200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000091ac418ea8f94a501200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120d2c575d2bb0dbc8ea59ed1cf8dce802d8170f0514f06f0c1ecad3101016f4a4600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a1eac5aea8f94a50120a93fcfca145b9e58c455e590ccf7bc76d7826558dbbfa8adaf2eac32971b6da320000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002b2096a601ea8f94a50120389e517541f67acc55abf7ee1945f9e080a5fa7a8002616e472788af81b53a8720000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000a22b68702ea8f94a5012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120cd6795fabe891156a6b60d70bff6b033dc2d35312327246b4b4a490691b1d2b20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000a24d88f03ea8f94a5012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120377732534efcee7126a74ca30ab3643c363e6c5f9a0be74479f6866caebf26b100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002b26a8a606ea8f94a50120513587dd3eb217b8e231cc6d43822a046628e088a57e77c35d6215673d2af6d4200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002b28b6930cea8f94a501205e4f42b1b0cf57bb1b2b98772eae4714501fc1f735c1c5ce90fbad647e10427c200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002b2ab2f112ea8f94a501208a504d1ba4611b1526146d14e613676d8bebf8564186676d327cc5a6ff5a5cfb200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002b2cfcdb20ea8f94a50120f1749362b059ed5688a52f32b5f0bd835f74f5002994786e53f75d407384d7ea20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000a2ea0e74af6f098a5012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120253279b9158fea6749bb98ec21ce999e8cfaf0b0647886f02eaa397daf308b980000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000b30f6b78e01c0f298a501200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000021207bb1ce6f31e87674ccc92d2210ff321615395ab7e838c06e19d8d3109aa9c9e700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002c3280bab302c0f298a50120a97902da9180c12af3f5f65cad3921ccd5099a3242319abe335ee4777fc1181a2000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002c34c2938104c0f298a5012043bc803a052999e33a56459f26e0852beec39d7d29be134d12288030bff01e152000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002c36b0908407c0f298a501209c2be38907dde07409e917e8aeadcd80bd7086ded9f2d7416c1f55710cfd92d92000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002c38b895db0bc0f298a501205999211d6315e7d3f3929e894fda0019ef9767c018b600682eb10885e8ee4b252000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002c3a9cafb617c0f298a5012047f136e2ddcd0ffac9345d8f47c8af2f36a6866dac9315fb532dae42872913e32000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000000365766d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002007250ac934a747a63f163bc4e64c0a4985e4e3297228c1082045fb3302d75b0d00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000002e000000000000000000000000000000000000000000000000000000000000003a0000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002101337fb01fe848a98e4d7e4f97edd06baae1ab103533b6345917aa8a82d164603200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000101000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020ce1207acd784dfaf0b5007e567e4c925ed0b25b76ebeb4635e0beb937fbe9144000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000001010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000204b666b98c59cf57436bcce3e618780d0d3898d55ba578f196ad058453b507880000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002101bb7c790383755ebb5c54aa138254059ecf09d6fd4f72bd54456a37946d4fab76000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000001010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000201b11d8cf47f215548b4bf10d7d8bf34cb0aca55faa54d3ded6f7d2675677bfc7";

        vm.prank(address(ibcHandler));
        bool ok = client.verifyMembership(
            clientId,
            clientState.l2LatestHeight,
            proof,
            // u path client-state 4
            hex"17ef568e3e12ab5b9c7254a8d58478811de00f9e6eb34345acd53bf8fd09d3ec",
            hex"9a0d99845a85f92af893fcc4f90db7fb76d262dfc144459c6e90642c0e4bb437"
        );
        assert(ok);
    }

    function test_verifyNonMembershipV2(
        address caller,
        address relayer
    ) public {
        uint32 clientId = 1;

        // Mock the initial client and consensus states
        ClientState memory clientState = ClientState({
            l2ChainId: "atlantic-2",
            l1ClientId: 2,
            l2ClientId: 3,
            l2LatestHeight: 173218977,
            version: uint256(2),
            state: ExtraV2({
                storeKey: bytes("evm"),
                keyPrefixStorage: abi.encodePacked(
                    bytes1(0x03), hex"ee4ea8d358473f0fcebf0329feed95d56e8c04d7"
                )
            }).encode()
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            clientState.version,
            clientState.state
        );

        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: 0xD0A81718CC318725C92E633106070CAF16E51A447DED433F34C7C53DF6BADA71
        });
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(address(ibcHandler));
        client.createClient(
            caller, clientId, clientStateBytes, consensusStateBytes, relayer
        );

        // voyager rpc ibc-proof 1328 '{"client_state":{"client_id":5}}' --encode --ibc-interface ibc-solidity --client-type state-lens/ics23/ics23 --height 173218977
        // client 5 does not exist at this height
        bytes memory proof =
            hex"000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000035a0000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000c00000000000000000000000000000000000000000000000000000000000001b20000000000000000000000000000000000000000000000000000000000000003503ee4ea8d358473f0fcebf0329feed95d56e8c04d7cb1c2d00091ce3f9ab6ac80192f98042c72a89aa2ae548025e96800b1514cd1e0000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000000000003503ee4ea8d358473f0fcebf0329feed95d56e8c04d7c7d3544292610d3ca89fdfd96140e78409e8fdfe2b997677676357195b032d4e00000000000000000000000000000000000000000000000000000000000000000000000000000000000020e54cea436bde55599dbdc4c4a0ba576a24de7e9d49d5d73d54265c2d49f9717c00000000000000000000000000000000000000000000000000000000000000070002c6a694a20100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b000000000000000000000000000000000000000000000000000000000000036000000000000000000000000000000000000000000000000000000000000004400000000000000000000000000000000000000000000000000000000000000520000000000000000000000000000000000000000000000000000000000000060000000000000000000000000000000000000000000000000000000000000006c000000000000000000000000000000000000000000000000000000000000007a0000000000000000000000000000000000000000000000000000000000000086000000000000000000000000000000000000000000000000000000000000009200000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000ac00000000000000000000000000000000000000000000000000000000000000b800000000000000000000000000000000000000000000000000000000000000c400000000000000000000000000000000000000000000000000000000000000d200000000000000000000000000000000000000000000000000000000000000de00000000000000000000000000000000000000000000000000000000000000ea00000000000000000000000000000000000000000000000000000000000000f800000000000000000000000000000000000000000000000000000000000001060000000000000000000000000000000000000000000000000000000000000112000000000000000000000000000000000000000000000000000000000000011e000000000000000000000000000000000000000000000000000000000000012a000000000000000000000000000000000000000000000000000000000000013600000000000000000000000000000000000000000000000000000000000001440000000000000000000000000000000000000000000000000000000000000152000000000000000000000000000000000000000000000000000000000000015e000000000000000000000000000000000000000000000000000000000000016a0000000000000000000000000000000000000000000000000000000000000176000000000000000000000000000000000000000000000000000000000000018200000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000080204ea8f94a50120000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120e917b661cfe4dd7ae97fae52f8ee7e190b94879c5c7c86b82c636ca821130197000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000080408ea8f94a50120000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120b70c5b0c267f1fa9e245af7dd065f9c0eaf466889665119ff62082b00f73919300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000008060eea8f94a5012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000212005639b1ead1a1e9aae79b0d5452c43a6b1764203c22f7c1dfb6714841c2c815100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000029081aea8f94a501201332cc18faa250d45ecad10d1a64e3fce9ba31198ebdaa7bf16f383c7312e41620000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000080a2aea8f94a50120000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120a807233f4d3c83321ab80ee8ed08a2ac703793b61a612351dcfc92939a164afb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000290c46ea8f94a501206a66a24db521b718e40e22df3afdabc4af0204dee2b8587b34c192bb3cd88ba32000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a0e8801ea8f94a50120cab16c18f017be51a65f473d5e7bdbd1318cf81665b9878b0cfa3880b76accd82000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000910c801ea8f94a5012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000021202e7118befb4cc724ca9022aaf8cd29de1b065972387f5f4b0b95429545268f5d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a12a602ea8f94a5012027808a27bf3490a5c9f4839ccc23f037c3119b0a502d858a4887bc4a69b61ceb20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a14bc04ea8f94a501203f5dc44e6e016ee2cf63117652a41e798da305e2ecafe56cd47ee8efb3d283b020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a168008ea8f94a50120a93bd924367779b7fd85f74b015aaa32c4fc36a11f253308454f615476e2aa7a200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000091ac418ea8f94a501200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120d2c575d2bb0dbc8ea59ed1cf8dce802d8170f0514f06f0c1ecad3101016f4a4600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a1eac5aea8f94a50120a93fcfca145b9e58c455e590ccf7bc76d7826558dbbfa8adaf2eac32971b6da320000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002b2096a601ea8f94a50120389e517541f67acc55abf7ee1945f9e080a5fa7a8002616e472788af81b53a8720000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000a22b68702ea8f94a5012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120cd6795fabe891156a6b60d70bff6b033dc2d35312327246b4b4a490691b1d2b20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000a24d88f03ea8f94a5012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120377732534efcee7126a74ca30ab3643c363e6c5f9a0be74479f6866caebf26b100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002b26a8a606ea8f94a50120513587dd3eb217b8e231cc6d43822a046628e088a57e77c35d6215673d2af6d4200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002b28b6930cea8f94a501205e4f42b1b0cf57bb1b2b98772eae4714501fc1f735c1c5ce90fbad647e10427c200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002b2ab2f112ea8f94a501208a504d1ba4611b1526146d14e613676d8bebf8564186676d327cc5a6ff5a5cfb200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002b2cfcdb20ea8f94a50120f1749362b059ed5688a52f32b5f0bd835f74f5002994786e53f75d407384d7ea20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000a2ea0e74af6f098a5012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120253279b9158fea6749bb98ec21ce999e8cfaf0b0647886f02eaa397daf308b980000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000b30f6b78e01c0f298a501200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000021207bb1ce6f31e87674ccc92d2210ff321615395ab7e838c06e19d8d3109aa9c9e700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002c3280bab302c0f298a50120a97902da9180c12af3f5f65cad3921ccd5099a3242319abe335ee4777fc1181a2000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002c34c2938104c0f298a5012043bc803a052999e33a56459f26e0852beec39d7d29be134d12288030bff01e152000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002c36b0908407c0f298a501209c2be38907dde07409e917e8aeadcd80bd7086ded9f2d7416c1f55710cfd92d92000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002c38b895db0bc0f298a501205999211d6315e7d3f3929e894fda0019ef9767c018b600682eb10885e8ee4b252000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002c3a9cafb617c0f298a5012047f136e2ddcd0ffac9345d8f47c8af2f36a6866dac9315fb532dae42872913e32000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000e000000000000000000000000000000000000000000000000000000000000001200000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000000000003503ee4ea8d358473f0fcebf0329feed95d56e8c04d7cbc4e5fb02c3d1de23a9f1e014b4d2ee5aeaea9505df5e855c9210bf472495af000000000000000000000000000000000000000000000000000000000000000000000000000000000000200000000000000000000000007700ea295f62989d2c98721389a521c96fe3c4c000000000000000000000000000000000000000000000000000000000000000070002a09795a20100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001b00000000000000000000000000000000000000000000000000000000000003600000000000000000000000000000000000000000000000000000000000000420000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000000000000000000005e000000000000000000000000000000000000000000000000000000000000006a000000000000000000000000000000000000000000000000000000000000007800000000000000000000000000000000000000000000000000000000000000840000000000000000000000000000000000000000000000000000000000000090000000000000000000000000000000000000000000000000000000000000009e00000000000000000000000000000000000000000000000000000000000000aa00000000000000000000000000000000000000000000000000000000000000b600000000000000000000000000000000000000000000000000000000000000c200000000000000000000000000000000000000000000000000000000000000d000000000000000000000000000000000000000000000000000000000000000dc00000000000000000000000000000000000000000000000000000000000000e800000000000000000000000000000000000000000000000000000000000000f600000000000000000000000000000000000000000000000000000000000001040000000000000000000000000000000000000000000000000000000000000110000000000000000000000000000000000000000000000000000000000000011c0000000000000000000000000000000000000000000000000000000000000128000000000000000000000000000000000000000000000000000000000000013400000000000000000000000000000000000000000000000000000000000001420000000000000000000000000000000000000000000000000000000000000150000000000000000000000000000000000000000000000000000000000000015c0000000000000000000000000000000000000000000000000000000000000168000000000000000000000000000000000000000000000000000000000000017400000000000000000000000000000000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000290204ea8f94a501207a6af60154738618bb335c98500b898f3ebfc9a6ddb26e5906fbc28c9d9b3d5520000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000080408ea8f94a50120000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120b70c5b0c267f1fa9e245af7dd065f9c0eaf466889665119ff62082b00f73919300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000008060eea8f94a5012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000212005639b1ead1a1e9aae79b0d5452c43a6b1764203c22f7c1dfb6714841c2c815100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000029081aea8f94a501201332cc18faa250d45ecad10d1a64e3fce9ba31198ebdaa7bf16f383c7312e41620000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000080a2aea8f94a50120000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120a807233f4d3c83321ab80ee8ed08a2ac703793b61a612351dcfc92939a164afb00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a000000000000000000000000000000000000000000000000000000000000000290c46ea8f94a501206a66a24db521b718e40e22df3afdabc4af0204dee2b8587b34c192bb3cd88ba32000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a0e8801ea8f94a50120cab16c18f017be51a65f473d5e7bdbd1318cf81665b9878b0cfa3880b76accd82000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000910c801ea8f94a5012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000021202e7118befb4cc724ca9022aaf8cd29de1b065972387f5f4b0b95429545268f5d00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a12a602ea8f94a5012027808a27bf3490a5c9f4839ccc23f037c3119b0a502d858a4887bc4a69b61ceb20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a14bc04ea8f94a501203f5dc44e6e016ee2cf63117652a41e798da305e2ecafe56cd47ee8efb3d283b020000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a168008ea8f94a50120a93bd924367779b7fd85f74b015aaa32c4fc36a11f253308454f615476e2aa7a200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000091ac418ea8f94a501200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120d2c575d2bb0dbc8ea59ed1cf8dce802d8170f0514f06f0c1ecad3101016f4a4600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002a1eac5aea8f94a50120a93fcfca145b9e58c455e590ccf7bc76d7826558dbbfa8adaf2eac32971b6da320000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002b2096a601ea8f94a50120389e517541f67acc55abf7ee1945f9e080a5fa7a8002616e472788af81b53a8720000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000a22b68702ea8f94a5012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120cd6795fabe891156a6b60d70bff6b033dc2d35312327246b4b4a490691b1d2b20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000a24d88f03ea8f94a5012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120377732534efcee7126a74ca30ab3643c363e6c5f9a0be74479f6866caebf26b100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002b26a8a606ea8f94a50120513587dd3eb217b8e231cc6d43822a046628e088a57e77c35d6215673d2af6d4200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002b28b6930cea8f94a501205e4f42b1b0cf57bb1b2b98772eae4714501fc1f735c1c5ce90fbad647e10427c200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002b2ab2f112ea8f94a501208a504d1ba4611b1526146d14e613676d8bebf8564186676d327cc5a6ff5a5cfb200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002b2cfcdb20ea8f94a50120f1749362b059ed5688a52f32b5f0bd835f74f5002994786e53f75d407384d7ea20000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000a2ea0e74af6f098a5012000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002120253279b9158fea6749bb98ec21ce999e8cfaf0b0647886f02eaa397daf308b980000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000b30f6b78e01c0f298a501200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000021207bb1ce6f31e87674ccc92d2210ff321615395ab7e838c06e19d8d3109aa9c9e700000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002c3280bab302c0f298a50120a97902da9180c12af3f5f65cad3921ccd5099a3242319abe335ee4777fc1181a2000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002c34c2938104c0f298a5012043bc803a052999e33a56459f26e0852beec39d7d29be134d12288030bff01e152000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002c36b0908407c0f298a501209c2be38907dde07409e917e8aeadcd80bd7086ded9f2d7416c1f55710cfd92d92000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002c38b895db0bc0f298a501205999211d6315e7d3f3929e894fda0019ef9767c018b600682eb10885e8ee4b252000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002c3a9cafb617c0f298a5012047f136e2ddcd0ffac9345d8f47c8af2f36a6866dac9315fb532dae42872913e32000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000000140000000000000000000000000000000000000000000000000000000000000000365766d0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002007250ac934a747a63f163bc4e64c0a4985e4e3297228c1082045fb3302d75b0d00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000000000022000000000000000000000000000000000000000000000000000000000000002e000000000000000000000000000000000000000000000000000000000000003a0000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002101337fb01fe848a98e4d7e4f97edd06baae1ab103533b6345917aa8a82d164603200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000400000000000000000000000000000000000000000000000000000000000000080000000000000000000000000000000000000000000000000000000000000000101000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000020ce1207acd784dfaf0b5007e567e4c925ed0b25b76ebeb4635e0beb937fbe9144000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000001010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000204b666b98c59cf57436bcce3e618780d0d3898d55ba578f196ad058453b507880000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000a0000000000000000000000000000000000000000000000000000000000000002101bb7c790383755ebb5c54aa138254059ecf09d6fd4f72bd54456a37946d4fab76000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000800000000000000000000000000000000000000000000000000000000000000001010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000201b11d8cf47f215548b4bf10d7d8bf34cb0aca55faa54d3ded6f7d2675677bfc7";

        vm.prank(address(ibcHandler));
        bool ok = client.verifyNonMembership(
            clientId,
            clientState.l2LatestHeight,
            proof,
            // u path client-state 5
            hex"05b8ccbb9d4d8fb16ea74ce3c29a41f1b461fbdaff4714a0d9a8eb05499746bc"
        );
        assert(ok);
    }
}
