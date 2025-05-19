pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../core/IBCHandler.sol";
import "../../../contracts/clients/StateLensIcs23MptClient.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

contract StateLensIcs23MptClientTest is Test {
    StateLensIcs23MptClient client;
    address admin = address(0xABCD);
    address ibcHandler;

    function setUp() public {
        ibcHandler = address(0xC0DE);
        StateLensIcs23MptClient implementation =
            new StateLensIcs23MptClient(ibcHandler);
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeWithSelector(
                StateLensIcs23MptClient.initialize.selector, admin
            )
        );
        client = StateLensIcs23MptClient(address(proxy));
    }

    function test_initialize_ok() public {
        assertEq(client.authority(), admin);
    }

    // ethereum consensus state
    // (slot, state root, storage root, timestamp)
    function test_extractConsensusStateEthereum() public {
        bytes memory rawL2ConsensusState =
            hex"00000000000000000000000000000000000000000000000000000000003e7d4056013ecdec1cff8ae46d82d3a021ebe40b327a0a6b915383af7756c3fc53797f624400e324ba4121a89b5b8fbec1c76837af1f834201969d321386383136d3f20000000000000000000000000000000000000000000000001837a60c35b18000";

        this.do_verifyExtractConsensusState(
            rawL2ConsensusState,
            120,
            32,
            64,
            1745045952000000000,
            hex"56013ecdec1cff8ae46d82d3a021ebe40b327a0a6b915383af7756c3fc53797f",
            hex"624400e324ba4121a89b5b8fbec1c76837af1f834201969d321386383136d3f2"
        );
    }

    // trusted/evm consensus state
    // (state root, storage root, timestamp)
    function test_extractConsensusStateTrustedEvm() public {
        bytes memory rawL2ConsensusState =
            hex"f8898b8f591dd568a5313faa75362bf588f1d38f1e5ef6c0bad8deeb329a15190f71a1b65c7d1a5216c48107062a0fca0a7bec2872f88201f8fb7ed5a4196c59000000000000000000000000000000000000000000000000183fce950c9b6000";

        this.do_verifyExtractConsensusState(
            rawL2ConsensusState,
            88,
            0,
            32,
            1747342320000000000,
            hex"f8898b8f591dd568a5313faa75362bf588f1d38f1e5ef6c0bad8deeb329a1519",
            hex"0f71a1b65c7d1a5216c48107062a0fca0a7bec2872f88201f8fb7ed5a4196c59"
        );
    }

    function do_verifyExtractConsensusState(
        bytes calldata rawL2ConsensusState,
        uint16 timestampOffset,
        uint16 stateRootOffset,
        uint16 storageRootOffset,
        uint64 expectedL2Timestamp,
        bytes32 expectedL2StateRoot,
        bytes32 expectedL2StorageRoot
    ) external {
        (uint64 l2Timestamp, bytes32 l2StateRoot, bytes32 l2StorageRoot) =
        StateLensIcs23MptLib.extractL2ConsensusState(
            rawL2ConsensusState,
            timestampOffset,
            stateRootOffset,
            storageRootOffset
        );

        assertEq(l2Timestamp, expectedL2Timestamp);
        assertEq(l2StateRoot, expectedL2StateRoot);
        assertEq(l2StorageRoot, expectedL2StorageRoot);
    }
}
