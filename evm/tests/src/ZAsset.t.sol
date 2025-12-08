pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/access/manager/AccessManager.sol";
import "../../contracts/ZAsset.sol";
import "../../contracts/core/02-client/ILightClient.sol";
import "../../contracts/core/24-host/IBCStore.sol";

contract MockERC1271Attestor {
    bytes4 internal constant MAGICVALUE = 0x1626ba7e;

    function isValidSignature(
        bytes32 _hash,
        bytes memory _signature
    ) public pure returns (bytes4) {
        return MAGICVALUE;
    }
}

contract MockERC1271RejectingAttestor {
    function isValidSignature(
        bytes32 _hash,
        bytes memory _signature
    ) public pure returns (bytes4) {
        return 0xffffffff;
    }
}

contract MockLightClient is ILightClient {
    bytes32 public constant STATE_ROOT_CLIENT_5 =
        0x968f5481a6f6d34f798a5145651b96b66b5d6a3b2a60ed301d3eb84fc0f23c32;
    bytes32 public constant STATE_ROOT_CLIENT_6 =
        0xf53670639dc5d41804cb2392f30a45f513b3c84231e13ddff2af56384db75287;
    bytes32 public mockAppHash = bytes32(
        uint256(
            0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321
        )
    );
    uint64 public mockTimestamp = 1234567890;

    function createClient(
        address caller,
        uint32 clientId,
        bytes calldata clientStateBytes,
        bytes calldata consensusStateBytes,
        address relayer
    )
        external
        pure
        returns (
            ConsensusStateUpdate memory update,
            string memory counterpartyChainId
        )
    {
        revert("Not implemented");
    }

    function updateClient(
        address caller,
        uint32 clientId,
        bytes calldata clientMessage,
        address relayer
    ) external pure returns (ConsensusStateUpdate memory update) {
        revert("Not implemented");
    }

    function misbehaviour(
        address caller,
        uint32 clientId,
        bytes calldata clientMessage,
        address relayer
    ) external pure {
        revert("Not implemented");
    }

    function verifyMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path,
        bytes calldata value
    ) external pure returns (bool) {
        revert("Not implemented");
    }

    function verifyNonMembership(
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        bytes calldata path
    ) external pure returns (bool) {
        revert("Not implemented");
    }

    function getClientState(
        uint32 clientId
    ) external pure returns (bytes memory) {
        revert("Not implemented");
    }

    function getTimestampAtHeight(
        uint32 clientId,
        uint64 height
    ) external view returns (uint64) {
        return mockTimestamp;
    }

    function getLatestHeight(
        uint32 clientId
    ) external pure returns (uint64) {
        return 100;
    }

    function getConsensusState(
        uint32 clientId,
        uint64 height
    ) external view returns (bytes memory) {
        // Return encoded MPT state lens consensus state (timestamp, stateRoot, appHash)
        bytes32 stateRoot;
        if (clientId == 5) {
            stateRoot = STATE_ROOT_CLIENT_5;
        } else if (clientId == 6) {
            stateRoot = STATE_ROOT_CLIENT_6;
        } else {
            revert("Unknown client ID");
        }
        return abi.encode(mockTimestamp, stateRoot, mockAppHash);
    }

    function isFrozen(
        uint32 clientId
    ) external pure returns (bool) {
        return false;
    }
}

contract MockIBCHandler {
    MockLightClient public lightClient;

    constructor(
        MockLightClient _lightClient
    ) {
        lightClient = _lightClient;
    }

    function getClient(
        uint32 clientId
    ) external view returns (ILightClient) {
        // Accept any client ID and return the same mock light client
        return ILightClient(address(lightClient));
    }
}

contract ZAssetTest is Test {
    ZAsset public implementation;
    ZAsset public zAsset;
    MockERC1271Attestor public attestor;
    MockERC1271RejectingAttestor public rejectingAttestor;
    MockLightClient public lightClient;
    MockIBCHandler public ibcHandler;
    AccessManager public accessManager;

    address public constant ZKGM =
        address(0x1234567890123456789012345678901234567890);
    address public constant BENEFICIARY =
        address(0x259BDDc9E7C3DbD14C62293a3531dA8E2B5193aD);
    uint256 public constant SHADOW_REALM_ID = 1;
    uint32 public constant CLIENT_ID_1 = 5;
    uint32 public constant CLIENT_ID_2 = 6;

    uint256[8] public validProof = [
        0x2cf836f4da765258c4cdd04753ca26f7a3de01caa9219a8c4577a1bd08260bd0,
        0x04266681e1c495ed26def3fad2d990f86bef9c4501403d4c3f37fc4987f3e706,
        0x281a638b4f6b18a8f577f0c9687216f1e1bd1e2885db128b8bf34848a4098ea0,
        0x0f9ac4ad66899faf83e7be1920e22d4693903ae2ea70bf28d08c4aaca3bc61d2,
        0x2c2e5c30b28891fa3711b4673bc492b80d3738f88fe6c01319e6eafc0f4c58f6,
        0x2f5e19f479865aad81d3be56abd415f778b01d96d20d4d044fa3887ce5dcb705,
        0x2b6acfca02f4c0519446b91e461ede81f59ec09252e011260c22d773a32eac42,
        0x1947cd9583113727cc0213f2074a38eac4d249e7064863b4d4eaff58eb376c3d
    ];

    uint256[2] public validCommitments = [
        0x2b8c07bba86f5245a6fa45ff260ab61bba06c98b39837fad0ef354def9f3e846,
        0x2851eba15c5bb7084b73f028ef30f286d870d75b9bdf7a702cbac270bf6f8e7d
    ];

    uint256[2] public validCommitmentPok = [
        0x03db6916f198e167919ac2c79dc909baf4144a4529b34a62064dd281e561ff81,
        0x2ef836c0a93d46a00caf68349bd62385ff476348fda9554a2a72a68caee2de23
    ];

    uint256 public constant VALID_NULLIFIER =
        0x047927ab303472b02eba67223446fc49bdf78522d43fb1be631c5647df67cc76;
    uint256 public constant VALID_VALUE = 100000000000000; // 0.0001 in wei
    uint64 public constant VALID_HEIGHT_1 = 66202018;
    uint64 public constant VALID_HEIGHT_2 = 38607541;

    function setUp() public {
        attestor = new MockERC1271Attestor();
        rejectingAttestor = new MockERC1271RejectingAttestor();
        lightClient = new MockLightClient();
        ibcHandler = new MockIBCHandler(lightClient);

        accessManager = new AccessManager(address(this));

        implementation = new ZAsset();

        bytes memory initData = abi.encodeCall(
            ZAsset.initialize,
            (
                address(accessManager),
                ZKGM,
                "Test ZAsset",
                "TZAS",
                18,
                bytes("test-salt"),
                address(attestor),
                address(ibcHandler),
                SHADOW_REALM_ID,
                address(0) // Native mode - no underlying token
            )
        );

        ERC1967Proxy proxy = new ERC1967Proxy(address(implementation), initData);
        zAsset = ZAsset(address(proxy));

        ZAsset.Counterparty memory counterparty1 = ZAsset.Counterparty({
            tokenAddressKey: keccak256(
                abi.encodePacked(
                    address(0xba5eD44733953d79717F6269357C77718C8Ba5ed)
                )
            ),
            balanceSlot: 0x52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00
        });

        // Client 6 uses the same configuration (both reference the same token)
        ZAsset.Counterparty memory counterparty2 = ZAsset.Counterparty({
            tokenAddressKey: keccak256(
                abi.encodePacked(
                    address(0xba5eD44733953d79717F6269357C77718C8Ba5ed)
                )
            ),
            balanceSlot: 0x52c63247e1f47db19d5ce0460030c497f067ca4cebf71ba98eeadabe20bace00
        });

        uint64 adminRole = 0;
        accessManager.grantRole(adminRole, address(this), 0);

        zAsset.setConfidentialCounterparty(CLIENT_ID_1, counterparty1);
        zAsset.setConfidentialCounterparty(CLIENT_ID_2, counterparty2);
    }

    function test_redeem_InvalidSignature() public {
        bytes memory initData = abi.encodeCall(
            ZAsset.initialize,
            (
                address(accessManager),
                ZKGM,
                "Test ZAsset 2",
                "TZAS2",
                18,
                bytes("test-salt-2"),
                address(rejectingAttestor), // Use rejecting attestor
                address(ibcHandler),
                SHADOW_REALM_ID,
                address(0)
            )
        );
        ERC1967Proxy proxy2 =
            new ERC1967Proxy(address(implementation), initData);
        ZAsset zAsset2 = ZAsset(address(proxy2));
        ZAsset.Counterparty memory counterparty = ZAsset.Counterparty({
            tokenAddressKey: keccak256(abi.encodePacked(address(0xdead))),
            balanceSlot: bytes32(uint256(0x1))
        });
        zAsset2.setConfidentialCounterparty(CLIENT_ID_1, counterparty);
        uint256 nullifier = 12345;
        uint256 value = 1000 * 10 ** 18;
        bytes32 attestedMessage = bytes32(uint256(123456789));
        bytes memory signature = hex"1234";
        ZAsset.LightClientSource[] memory lightClientSources =
            new ZAsset.LightClientSource[](1);
        lightClientSources[0] =
            ZAsset.LightClientSource({clientId: CLIENT_ID_1, height: 100});
        vm.expectRevert(ZAsset.InvalidAttestationSig.selector);
        zAsset2.redeem(
            validProof,
            validCommitments,
            validCommitmentPok,
            lightClientSources,
            nullifier,
            value,
            BENEFICIARY,
            attestedMessage,
            signature,
            true
        );
    }

    function test_redeem_ValueExceedsScalarField() public {
        uint256 nullifier = 12345;
        // Value >= R (BN254 scalar field modulus)
        uint256 value =
            0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001;
        bytes32 attestedMessage = bytes32(uint256(123456789));
        bytes memory signature = hex"1234";

        ZAsset.LightClientSource[] memory lightClientSources =
            new ZAsset.LightClientSource[](1);
        lightClientSources[0] =
            ZAsset.LightClientSource({clientId: CLIENT_ID_1, height: 100});

        vm.expectRevert(ZAsset.ValueExceedScalarField.selector);
        zAsset.redeem(
            validProof,
            validCommitments,
            validCommitmentPok,
            lightClientSources,
            nullifier,
            value,
            BENEFICIARY,
            attestedMessage,
            signature,
            true
        );
    }

    function test_redeem_NullifierExceedsScalarField() public {
        // Nullifier >= R (BN254 scalar field modulus)
        uint256 nullifier =
            0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001;
        uint256 value = 1000 * 10 ** 18;
        bytes32 attestedMessage = bytes32(uint256(123456789));
        bytes memory signature = hex"1234";
        ZAsset.LightClientSource[] memory lightClientSources =
            new ZAsset.LightClientSource[](1);
        lightClientSources[0] =
            ZAsset.LightClientSource({clientId: CLIENT_ID_1, height: 100});
        vm.expectRevert(ZAsset.NullifierExceedScalarField.selector);
        zAsset.redeem(
            validProof,
            validCommitments,
            validCommitmentPok,
            lightClientSources,
            nullifier,
            value,
            BENEFICIARY,
            attestedMessage,
            signature,
            true
        );
    }

    function test_redeem_TooManyLightClients() public {
        uint256 nullifier = 12345;
        uint256 value = 1000 * 10 ** 18;
        bytes32 attestedMessage = bytes32(uint256(123456789));
        bytes memory signature = hex"1234";
        // Create 5 lightClient IDs (exceeds limit of 4)
        ZAsset.LightClientSource[] memory lightClientSources =
            new ZAsset.LightClientSource[](5);
        for (uint256 i = 0; i < 5; i++) {
            lightClientSources[i] =
                ZAsset.LightClientSource({clientId: CLIENT_ID_1, height: 100});
        }
        vm.expectRevert(ZAsset.TooManyLightClients.selector);
        zAsset.redeem(
            validProof,
            validCommitments,
            validCommitmentPok,
            lightClientSources,
            nullifier,
            value,
            BENEFICIARY,
            attestedMessage,
            signature,
            true
        );
    }

    function test_redeem_UnknownLightClient() public {
        uint256 nullifier = 12345;
        uint256 value = 1000 * 10 ** 18;
        bytes32 attestedMessage = bytes32(uint256(123456789));
        bytes memory signature = hex"1234";
        // Use an unconfigured client ID
        ZAsset.LightClientSource[] memory lightClientSources =
            new ZAsset.LightClientSource[](1);
        lightClientSources[0] = ZAsset.LightClientSource({
            clientId: 999, // Unconfigured client
            height: 100
        });
        vm.expectRevert(ZAsset.OnlyKnownLightClients.selector);
        zAsset.redeem(
            validProof,
            validCommitments,
            validCommitmentPok,
            lightClientSources,
            nullifier,
            value,
            BENEFICIARY,
            attestedMessage,
            signature,
            true
        );
    }

    function test_deposit_RevertsInNativeMode() public {
        vm.expectRevert(ZAsset.NotWrapped.selector);
        zAsset.deposit(1000);
    }

    function test_withdraw_RevertsInNativeMode() public {
        vm.expectRevert(ZAsset.NotWrapped.selector);
        zAsset.withdraw(1000);
    }
}
