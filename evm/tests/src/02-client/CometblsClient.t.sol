// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../core/IBCHandler.sol";
import "../../../contracts/clients/CometblsClient.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "solady/utils/LibString.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "solidity-stringutils/strings.sol";
import "solidity-bytes-utils/BytesLib.sol";

contract MockCometblsClient is CometblsClient {
    bool private zkpVerificationResult = true;

    function setZKPVerificationResult(
        bool result
    ) external {
        zkpVerificationResult = result;
    }

    function internalVerifyZKP(
        bytes calldata zkpBytes,
        bytes31 chainId,
        bytes32 trustedValidatorsHash,
        SignedHeader calldata header
    ) internal view override returns (bool) {
        // You can add additional logic to inspect the inputs if needed
        return zkpVerificationResult;
    }
}

contract CometblsClientTest is Test {
    MockCometblsClient cometblsClient;
    address admin = address(0xABcD);
    address ibcHandler = address(0x1234);

    function setUp() public {
        // Deploy the MockCometblsClient implementation
        MockCometblsClient implementation = new MockCometblsClient();

        // Deploy the proxy and initialize it with the implementation
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeWithSelector(
                CometblsClient.initialize.selector, ibcHandler, admin
            )
        );

        // Cast the proxy as the CometblsClient
        cometblsClient = MockCometblsClient(address(proxy));
    }

    function test_initialize_ok() public {
        // Verify the ibcHandler address
        // assertEq(cometblsClient.ibcHandler(), ibcHandler);

        // Verify the admin address
        assertEq(cometblsClient.owner(), admin);
    }

    function test_createClient_success(
        address caller,
        address relayer
    ) public {
        uint32 clientId = 1;

        // Encode the client state
        ClientState memory clientState = ClientState({
            chainId: bytes31("test-chain"),
            trustingPeriod: 86400, // 1 day in seconds
            maxClockDrift: 300, // 5 minutes
            frozenHeight: 0,
            latestHeight: 100,
            contractAddress: keccak256("test")
        });
        bytes memory clientStateBytes = abi.encode(clientState);

        // Encode the consensus state
        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: keccak256("app"),
            nextValidatorsHash: keccak256("validators")
        });
        bytes memory consensusStateBytes = abi.encode(consensusState);

        vm.prank(ibcHandler); // Simulate call from the IBC handler
        cometblsClient.createClient(
            caller, clientId, clientStateBytes, consensusStateBytes, relayer
        );

        // Verify the client state was stored
        bytes memory storedClientState = cometblsClient.getClientState(clientId);
        assertEq(
            keccak256(storedClientState),
            keccak256(clientStateBytes),
            "Client state mismatch"
        );

        // Verify the consensus state was stored
        bytes memory storedConsensusState =
            cometblsClient.getConsensusState(clientId, 100);
        assertEq(
            keccak256(storedConsensusState),
            keccak256(consensusStateBytes),
            "Consensus state mismatch"
        );
    }

    function misbehaviour_common(
        address caller,
        address relayer,
        uint256 vm_warp,
        uint64 trustingPeriod,
        uint64 maxClockDrift
    ) public {
        uint32 clientId = 1;

        vm.warp(vm_warp);

        cometblsClient.setZKPVerificationResult(true);
        // Mock client and consensus state
        ClientState memory clientState = ClientState({
            chainId: bytes31("test-chain"),
            trustingPeriod: trustingPeriod,
            maxClockDrift: maxClockDrift,
            frozenHeight: 0,
            latestHeight: 99,
            contractAddress: keccak256("test")
        });
        bytes memory clientStateBytes = abi.encode(clientState);

        ConsensusState memory consensusState = ConsensusState({
            timestamp: uint64(block.timestamp),
            appHash: keccak256("app"),
            nextValidatorsHash: keccak256("validatorsA")
        });
        bytes memory consensusStateBytes = abi.encode(consensusState);

        vm.prank(ibcHandler);
        cometblsClient.createClient(
            caller, clientId, clientStateBytes, consensusStateBytes, relayer
        );
        vm.stopPrank();
        clientState.latestHeight = 100;
        clientStateBytes = abi.encode(clientState);
        vm.prank(ibcHandler);
        cometblsClient.createClient(
            caller, clientId, clientStateBytes, consensusStateBytes, relayer
        );
        vm.stopPrank();
    }

    function test_misbehaviour_freezesClient(
        address caller,
        address relayer
    ) public {
        misbehaviour_common(
            caller, relayer, 1000000, 8640000000000000000, 8640000000000000000
        );
        uint32 clientId = 1;

        // Mock headers for misbehavior
        Header memory headerA = Header({
            signedHeader: SignedHeader({
                height: 101,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appA")
            }),
            trustedHeight: 100,
            zeroKnowledgeProof: bytes("proofA")
        });

        Header memory headerB = Header({
            signedHeader: SignedHeader({
                height: 100,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appB")
            }),
            trustedHeight: 99,
            zeroKnowledgeProof: bytes("proofA")
        });

        vm.prank(ibcHandler);
        cometblsClient.misbehaviour(
            caller, clientId, abi.encode(headerA, headerB), relayer
        );
        vm.stopPrank();

        // Verify the client is frozen
        bytes memory storedClientState = cometblsClient.getClientState(clientId);
        ClientState memory frozenState =
            abi.decode(storedClientState, (ClientState));
        assertEq(frozenState.frozenHeight, 1, "Client was not frozen");
    }

    function test_misbehaviour_freezesClient_fraud(
        address caller,
        address relayer
    ) public {
        misbehaviour_common(
            caller, relayer, 1000000, 8640000000000000000, 8640000000000000000
        );
        uint32 clientId = 1;

        // Mock headers for misbehavior
        Header memory headerA = Header({
            signedHeader: SignedHeader({
                height: 101,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appA")
            }),
            trustedHeight: 100,
            zeroKnowledgeProof: bytes("proofA")
        });

        Header memory headerB = Header({
            signedHeader: SignedHeader({
                height: 100,
                secs: uint64(block.timestamp - 1),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appB")
            }),
            trustedHeight: 99,
            zeroKnowledgeProof: bytes("proofA")
        });

        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                CometblsClientLib.ErrInvalidMisbehaviour.selector
            )
        );
        cometblsClient.misbehaviour(
            caller, clientId, abi.encode(headerA, headerB), relayer
        );
        vm.stopPrank();
    }

    function test_misbehaviour_freezesClient_fraud_different_hash(
        address caller,
        address relayer
    ) public {
        misbehaviour_common(
            caller, relayer, 1000000, 8640000000000000000, 8640000000000000000
        );
        uint32 clientId = 1;

        // Mock headers for misbehavior
        Header memory headerA = Header({
            signedHeader: SignedHeader({
                height: 100,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appA")
            }),
            trustedHeight: 99,
            zeroKnowledgeProof: bytes("proofA")
        });

        Header memory headerB = Header({
            signedHeader: SignedHeader({
                height: 100,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appA")
            }),
            trustedHeight: 99,
            zeroKnowledgeProof: bytes("proofA")
        });

        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                CometblsClientLib.ErrInvalidMisbehaviour.selector
            )
        );
        cometblsClient.misbehaviour(
            caller, clientId, abi.encode(headerA, headerB), relayer
        );
        vm.stopPrank();
    }

    function test_misbehaviour_freezesClient_headers_seq(
        address caller,
        address relayer
    ) public {
        misbehaviour_common(
            caller, relayer, 1000000, 8640000000000000000, 8640000000000000000
        );
        uint32 clientId = 1;

        // Mock headers for misbehavior
        Header memory headerA = Header({
            signedHeader: SignedHeader({
                height: 99,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appA")
            }),
            trustedHeight: 100,
            zeroKnowledgeProof: bytes("proofA")
        });

        Header memory headerB = Header({
            signedHeader: SignedHeader({
                height: 100,
                secs: uint64(block.timestamp - 1),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appB")
            }),
            trustedHeight: 99,
            zeroKnowledgeProof: bytes("proofA")
        });

        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                CometblsClientLib.ErrInvalidMisbehaviourHeadersSequence.selector
            )
        );
        cometblsClient.misbehaviour(
            caller, clientId, abi.encode(headerA, headerB), relayer
        );
        vm.stopPrank();
    }

    function test_misbehaviour_freezesClient_ErrInvalidMisbehaviourHeadersSequence(
        address caller,
        address relayer
    ) public {
        misbehaviour_common(
            caller, relayer, 1000000, 8640000000000000000, 8640000000000000000
        );
        uint32 clientId = 1;

        // Mock headers for misbehavior
        Header memory headerA = Header({
            signedHeader: SignedHeader({
                height: 99,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appA")
            }),
            trustedHeight: 200,
            zeroKnowledgeProof: bytes("proofA")
        });

        Header memory headerB = Header({
            signedHeader: SignedHeader({
                height: 100,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appB")
            }),
            trustedHeight: 99,
            zeroKnowledgeProof: bytes("proofA")
        });

        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                CometblsClientLib.ErrInvalidMisbehaviourHeadersSequence.selector
            )
        );
        cometblsClient.misbehaviour(
            caller, clientId, abi.encode(headerA, headerB), relayer
        );
        vm.stopPrank();
    }

    function test_misbehaviour_freezesClient_ErrUntrustedHeightLTETrustedHeight(
        address caller,
        address relayer
    ) public {
        misbehaviour_common(
            caller, relayer, 1000000, 8640000000000000000, 8640000000000000000
        );
        uint32 clientId = 1;

        // Mock headers for misbehavior
        Header memory headerA = Header({
            signedHeader: SignedHeader({
                height: 99,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appA")
            }),
            trustedHeight: 100,
            zeroKnowledgeProof: bytes("proofA")
        });

        Header memory headerB = Header({
            signedHeader: SignedHeader({
                height: 98,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appB")
            }),
            trustedHeight: 99,
            zeroKnowledgeProof: bytes("proofA")
        });

        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                CometblsClientLib.ErrUntrustedHeightLTETrustedHeight.selector
            )
        );
        cometblsClient.misbehaviour(
            caller, clientId, abi.encode(headerA, headerB), relayer
        );
        vm.stopPrank();
    }

    function test_misbehaviour_freezesClient_ErrUntrustedTimestampLTETrustedTimestamp(
        address caller,
        address relayer
    ) public {
        misbehaviour_common(caller, relayer, 1000000000000000, 0, 0);
        uint32 clientId = 1;

        // Mock headers for misbehavior
        Header memory headerA = Header({
            signedHeader: SignedHeader({
                height: 101,
                secs: uint64(900000),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appA")
            }),
            trustedHeight: 100,
            zeroKnowledgeProof: bytes("proofA")
        });

        Header memory headerB = Header({
            signedHeader: SignedHeader({
                height: 100,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appB")
            }),
            trustedHeight: 99,
            zeroKnowledgeProof: bytes("proofA")
        });

        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                CometblsClientLib
                    .ErrUntrustedTimestampLTETrustedTimestamp
                    .selector
            )
        );
        cometblsClient.misbehaviour(
            caller, clientId, abi.encode(headerA, headerB), relayer
        );
        vm.stopPrank();
    }

    function test_misbehaviour_freezesClient_ErrHeaderExpired(
        address caller,
        address relayer
    ) public {
        misbehaviour_common(caller, relayer, 1000000000000000, 0, 0);
        uint32 clientId = 1;

        // Mock headers for misbehavior
        Header memory headerA = Header({
            signedHeader: SignedHeader({
                height: 101,
                secs: uint64(10000000),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appA")
            }),
            trustedHeight: 100,
            zeroKnowledgeProof: bytes("proofA")
        });

        Header memory headerB = Header({
            signedHeader: SignedHeader({
                height: 100,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appB")
            }),
            trustedHeight: 99,
            zeroKnowledgeProof: bytes("proofA")
        });

        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(CometblsClientLib.ErrHeaderExpired.selector)
        );
        cometblsClient.misbehaviour(
            caller, clientId, abi.encode(headerA, headerB), relayer
        );
        vm.stopPrank();
    }

    function test_misbehaviour_freezesClient_ErrMaxClockDriftExceeded(
        address caller,
        address relayer
    ) public {
        misbehaviour_common(caller, relayer, 1000000, 8640000000000000000, 0);
        uint32 clientId = 1;

        // Mock headers for misbehavior
        Header memory headerA = Header({
            signedHeader: SignedHeader({
                height: 101,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appA")
            }),
            trustedHeight: 100,
            zeroKnowledgeProof: bytes("proofA")
        });

        Header memory headerB = Header({
            signedHeader: SignedHeader({
                height: 100,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appB")
            }),
            trustedHeight: 99,
            zeroKnowledgeProof: bytes("proofA")
        });

        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                CometblsClientLib.ErrMaxClockDriftExceeded.selector
            )
        );
        cometblsClient.misbehaviour(
            caller, clientId, abi.encode(headerA, headerB), relayer
        );
        vm.stopPrank();
    }

    function test_misbehaviour_freezesClient_ErrInvalidUntrustedValidatorsHash(
        address caller,
        address relayer
    ) public {
        misbehaviour_common(
            caller, relayer, 1000000, 8640000000000000000, 8640000000000000000
        );
        uint32 clientId = 1;

        // Mock headers for misbehavior
        Header memory headerA = Header({
            signedHeader: SignedHeader({
                height: 101,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsB"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appA")
            }),
            trustedHeight: 100,
            zeroKnowledgeProof: bytes("proofA")
        });

        Header memory headerB = Header({
            signedHeader: SignedHeader({
                height: 100,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsB"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appB")
            }),
            trustedHeight: 99,
            zeroKnowledgeProof: bytes("proofA")
        });

        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(
                CometblsClientLib.ErrInvalidUntrustedValidatorsHash.selector
            )
        );
        cometblsClient.misbehaviour(
            caller, clientId, abi.encode(headerA, headerB), relayer
        );
        vm.stopPrank();
    }

    function test_misbehaviour_freezesClient_ErrInvalidZKP(
        address caller,
        address relayer
    ) public {
        misbehaviour_common(
            caller, relayer, 1000000, 8640000000000000000, 8640000000000000000
        );
        uint32 clientId = 1;

        // Mock headers for misbehavior
        Header memory headerA = Header({
            signedHeader: SignedHeader({
                height: 101,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appA")
            }),
            trustedHeight: 100,
            zeroKnowledgeProof: bytes("proofA")
        });

        Header memory headerB = Header({
            signedHeader: SignedHeader({
                height: 100,
                secs: uint64(block.timestamp),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("validatorsB"),
                appHash: keccak256("appB")
            }),
            trustedHeight: 99,
            zeroKnowledgeProof: bytes("proofA")
        });

        cometblsClient.setZKPVerificationResult(false);
        vm.prank(ibcHandler);
        vm.expectRevert(
            abi.encodeWithSelector(CometblsClientLib.ErrInvalidZKP.selector)
        );
        cometblsClient.misbehaviour(
            caller, clientId, abi.encode(headerA, headerB), relayer
        );
        vm.stopPrank();
    }

    function encodeMemory(
        ConsensusState memory consensusState
    ) internal pure returns (bytes memory) {
        return abi.encode(
            consensusState.timestamp,
            consensusState.appHash,
            consensusState.nextValidatorsHash
        );
    }

    function encodeMemory(
        ClientState memory clientState
    ) internal pure returns (bytes memory) {
        return abi.encode(
            clientState.chainId,
            clientState.trustingPeriod,
            clientState.maxClockDrift,
            clientState.frozenHeight,
            clientState.latestHeight,
            clientState.contractAddress
        );
    }

    function commit(
        ConsensusState memory consensusState
    ) internal pure returns (bytes32) {
        return keccak256(encodeMemory(consensusState));
    }

    function commit(
        ClientState memory clientState
    ) public pure returns (bytes32) {
        return keccak256(encodeMemory(clientState));
    }

    function test_updateClient_success(
        address caller,
        address relayer
    ) public {
        misbehaviour_common(
            caller, relayer, 1000000, 8640000000000000000, 8640000000000000000
        );
        uint32 clientId = 1;

        ClientState memory clientState = ClientState({
            chainId: bytes31("test-chain"),
            trustingPeriod: 8640000000000000000,
            maxClockDrift: 8640000000000000000,
            frozenHeight: 0,
            latestHeight: 99,
            contractAddress: keccak256("test")
        });

        bytes memory clientMessageBytes = abi.encode(
            SignedHeader({
                height: 101,
                secs: uint64(block.timestamp + 1),
                nanos: 0,
                validatorsHash: keccak256("validatorsA"),
                nextValidatorsHash: keccak256("newValidators"),
                appHash: keccak256("newApp")
            }),
            100,
            bytes("proof")
        );

        // Step 4: Update the client
        vm.prank(ibcHandler);
        ConsensusStateUpdate memory update = cometblsClient.updateClient(
            caller, clientId, clientMessageBytes, relayer
        );

        // Step 5: Verify the updates
        // Ensure the latest height is updated
        assertEq(
            cometblsClient.getLatestHeight(clientId),
            101,
            "Latest height mismatch"
        );

        // Ensure the consensus state is updated
        bytes memory storedConsensusState =
            cometblsClient.getConsensusState(clientId, 101);
        ConsensusState memory updatedConsensusState =
            CometblsClientLib.decodeConsensusStateMemory(storedConsensusState);

        assertEq(
            updatedConsensusState.timestamp,
            uint64((block.timestamp + 1) * 1e9),
            "Consensus state timestamp mismatch"
        );
        assertEq(
            updatedConsensusState.appHash,
            keccak256("newApp"),
            "Consensus state appHash mismatch"
        );
        assertEq(
            updatedConsensusState.nextValidatorsHash,
            keccak256("newValidators"),
            "Consensus state nextValidatorsHash mismatch"
        );

        // Ensure the commitments are correct
        bytes32 expectedClientStateCommitment = commit(clientState);
        bytes32 expectedConsensusStateCommitment = commit(updatedConsensusState);

        assertEq(
            update.consensusStateCommitment,
            expectedConsensusStateCommitment,
            "Consensus state commitment mismatch"
        );
        assertEq(update.height, 101, "Height mismatch");
    }

    function verifyMembership(
        bytes32 root,
        bytes calldata proof,
        bytes memory prefix,
        bytes memory path,
        bytes calldata value
    ) public pure returns (bool) {
        return true;
    }
}
