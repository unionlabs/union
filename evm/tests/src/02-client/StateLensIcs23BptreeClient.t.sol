pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../../../contracts/clients/StateLensIcs23BptreeClient.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

contract MockLightClient is ILightClient {
    bool isFrozenVar = false;

    function verifyMembership(
        uint32,
        uint64,
        bytes calldata,
        bytes calldata,
        bytes calldata
    ) public view override returns (bool) {
        return true;
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
    ) external view returns (uint64) {
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

    function misbehaviour(address, uint32, bytes calldata, address) external {}
}

contract MockIbcStore {
    address public client;

    function getClient(
        uint32
    ) external view returns (ILightClient) {
        return ILightClient(client);
    }

    function setClient(
        address _client
    ) public {
        client = _client;
    }
}

contract StateLensIcs23BptreeClientTest is Test {
    StateLensIcs23BptreeClient client;
    address admin = address(0xABcD);
    address ibcHandler;
    MockIbcStore ibcStore;
    MockLightClient lightClient;

    using StateLensIcs23BptreeLib for *;

    function setUp() public {
        ibcStore = new MockIbcStore();
        ibcHandler = address(ibcStore);
        StateLensIcs23BptreeClient implementation =
            new StateLensIcs23BptreeClient(ibcHandler);
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(implementation),
            abi.encodeWithSelector(
                StateLensIcs23BptreeClient.initialize.selector, admin
            )
        );
        client = StateLensIcs23BptreeClient(address(proxy));
        lightClient = new MockLightClient();
        ibcStore.setClient(address(lightClient));
    }

    function test_initialize_ok() public {
        assertEq(client.authority(), admin);
    }

    // Self-consistent 2-layer bptree proof (L2 store proof + L1 "main"
    // commitment), built the same way gno.land's bptree chains them, rather
    // than a real chain capture: the real topaz.testnets.gno.land proof used
    // to validate GNO_SPECS in lib/ics23's Rust tests embeds a ~67KB value (a
    // whole VM package) and doesn't exercise the `empty_child` leftmost
    // fallback anyway (see lib/ics23/src/verify.rs,
    // verify_non_membership_bptree_topaz). This one does, by giving the
    // bottom-most inner op the branch-1 shape (33-byte prefix) with
    // BptreeIcs23's EMPTY_CHILD embedded in place of a real left sibling hash.
    //
    // Cross-checked against lib/ics23/src/verify.rs,
    // verify_non_membership_bptree_empty_child_fallback, which builds the
    // identical proof in Rust.
    function bptreeFixture()
        internal
        pure
        returns (UnionIcs23.ExistenceProof memory l2Proof, bytes32 root)
    {
        UnionIcs23.InnerOp[] memory path = new UnionIcs23.InnerOp[](5);
        path[0] = UnionIcs23.InnerOp({
            prefix: hex"01",
            suffix: hex"1111111111111111111111111111111111111111111111111111111111111111"
        });
        path[1] = UnionIcs23.InnerOp({
            prefix: hex"01",
            suffix: hex"2222222222222222222222222222222222222222222222222222222222222222"
        });
        path[2] = UnionIcs23.InnerOp({
            prefix: hex"01dbc1b4c900ffe48d575b5da5c638040125f65db0fe3e24494b76ea986457d986",
            suffix: hex""
        });
        path[3] = UnionIcs23.InnerOp({
            prefix: hex"01",
            suffix: hex"3333333333333333333333333333333333333333333333333333333333333333"
        });
        path[4] = UnionIcs23.InnerOp({
            prefix: hex"01",
            suffix: hex"4444444444444444444444444444444444444444444444444444444444444444"
        });

        l2Proof = UnionIcs23.ExistenceProof({
            key: "key-b",
            value: "value-b",
            leafPrefix: hex"00",
            path: path
        });

        root = 0x17bc030ec6c05b84393004b37eec6777040bd9a792147e528b413d35502687c0;
    }

    function bptreeL1Proof(
        bytes32 l2Root
    ) internal pure returns (UnionIcs23.ExistenceProof memory) {
        UnionIcs23.InnerOp[] memory path = new UnionIcs23.InnerOp[](1);
        path[0] = UnionIcs23.InnerOp({
            prefix: hex"01",
            suffix: hex"5555555555555555555555555555555555555555555555555555555555555555"
        });
        return UnionIcs23.ExistenceProof({
            key: "main",
            value: abi.encodePacked(l2Root),
            leafPrefix: hex"00",
            path: path
        });
    }

    function createTestClient(
        address caller,
        address relayer,
        bytes32 root,
        bytes memory keyPrefixStorage
    ) internal returns (uint32 clientId) {
        clientId = 1;
        ClientState memory clientState = ClientState({
            l2ChainId: "dev.ibc",
            l1ClientId: 2,
            l2ClientId: 3,
            l2LatestHeight: 1,
            storeKey: bytes("main"),
            keyPrefixStorage: keyPrefixStorage
        });
        bytes memory clientStateBytes = abi.encode(
            clientState.l2ChainId,
            clientState.l1ClientId,
            clientState.l2ClientId,
            clientState.l2LatestHeight,
            clientState.storeKey,
            clientState.keyPrefixStorage
        );
        ConsensusState memory consensusState =
            ConsensusState({timestamp: uint64(block.timestamp), appHash: root});
        bytes memory consensusStateBytes =
            abi.encode(consensusState.timestamp, consensusState.appHash);

        vm.prank(ibcHandler);
        client.createClient(
            caller, clientId, clientStateBytes, consensusStateBytes, relayer
        );
    }

    function test_verifyMembership(address caller, address relayer) public {
        (UnionIcs23.ExistenceProof memory l2Proof, bytes32 root) =
            bptreeFixture();
        bytes32 l2Root =
            0x3238d76bf3588f6bdb138f3d4fc1fc8097297f1693e62f3c62e4ef69d37c4980;
        UnionIcs23.ExistenceProof memory l1Proof = bptreeL1Proof(l2Root);

        // key = keyPrefixStorage ++ toHexStringNoPrefix(path); path empty and
        // keyPrefixStorage set to the literal key reconstructs "key-b".
        uint32 clientId =
            createTestClient(caller, relayer, root, bytes("key-b"));

        bool ok = client.verifyMembership(
            clientId, 1, abi.encode(l2Proof, l1Proof), hex"", "value-b"
        );
        assertTrue(ok);
    }

    function test_verifyNonMembership_emptyChildFallback(
        address caller,
        address relayer
    ) public {
        (UnionIcs23.ExistenceProof memory l2Proof, bytes32 root) =
            bptreeFixture();
        bytes32 l2Root =
            0x3238d76bf3588f6bdb138f3d4fc1fc8097297f1693e62f3c62e4ef69d37c4980;
        UnionIcs23.ExistenceProof memory l1Proof = bptreeL1Proof(l2Root);

        UnionIcs23.NonExistenceProof memory nonExistProof = UnionIcs23
            .NonExistenceProof({
            key: "",
            left: UnionIcs23.ExistenceProof({
                key: "",
                value: "",
                leafPrefix: "",
                path: new UnionIcs23.InnerOp[](0)
            }),
            right: l2Proof
        });

        // proven-absent key = hex(path) = "ab", < "key-b" since 'a' < 'k';
        // keyPrefixStorage is left empty since only the ordering matters
        // here, not the literal key.
        uint32 clientId = createTestClient(caller, relayer, root, "");

        bool ok = client.verifyNonMembership(
            clientId, 1, abi.encode(nonExistProof, l1Proof), hex"ab"
        );
        assertTrue(ok);
    }
}
