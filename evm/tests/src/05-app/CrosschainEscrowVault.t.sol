pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/access/manager/AccessManager.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";

import "../../../contracts/CrosschainEscrowVault.sol";
import "../../../contracts/core/Types.sol";
import "../../../contracts/apps/ucs/03-zkgm/Types.sol";
import "../../../contracts/core/04-channel/IBCPacket.sol";

contract MockERC20 is ERC20 {
    constructor(string memory name, string memory symbol) ERC20(name, symbol) {}

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }
}

contract CrosschainEscrowVaultTest is Test {
    CrosschainEscrowVault public vault;
    MockERC20 public token;
    AccessManager public accessManager;

    address public admin = address(0x1);
    address public zkgm = address(0x2);
    address public relayer = address(0x3);
    address public receiver = address(0x4);
    address public user = address(0x5);

    uint256 constant PATH = 1;
    uint32 constant CHANNEL_ID = 1;
    bytes constant BASE_TOKEN = "base_token";
    bytes constant COUNTERPARTY_BENEFICIARY = "counterparty_beneficiary";

    uint256 constant INITIAL_TOKEN_BALANCE = 1000 ether;
    uint256 constant INITIAL_ETH_BALANCE = 100 ether;

    function setUp() public {
        accessManager = new AccessManager(admin);

        vm.startPrank(admin);

        uint64 adminRole = accessManager.ADMIN_ROLE();
        accessManager.grantRole(adminRole, admin, 0);

        token = new MockERC20("Test Token", "TEST");
        vault = _deployVault();

        _setupPermissions(address(vault), adminRole);
        _setupBalances();

        vm.stopPrank();
    }

    function _deployVault() internal returns (CrosschainEscrowVault) {
        CrosschainEscrowVault implementation = new CrosschainEscrowVault();
        bytes memory initData = abi.encodeCall(
            CrosschainEscrowVault.initialize, (address(accessManager), zkgm)
        );
        ERC1967Proxy proxy = new ERC1967Proxy(address(implementation), initData);
        return CrosschainEscrowVault(payable(proxy));
    }

    function _setupPermissions(
        address vaultAddress,
        uint64 adminRole
    ) internal {
        bytes4[] memory selectors = new bytes4[](3);
        selectors[0] = CrosschainEscrowVault.setFungibleCounterparty.selector;
        selectors[1] = CrosschainEscrowVault.whitelistIntent.selector;
        selectors[2] = UUPSUpgradeable.upgradeToAndCall.selector;

        accessManager.setTargetFunctionRole(vaultAddress, selectors, adminRole);
    }

    function _setupBalances() internal {
        token.mint(address(vault), INITIAL_TOKEN_BALANCE);
        token.mint(user, INITIAL_TOKEN_BALANCE);

        vm.deal(address(vault), INITIAL_ETH_BALANCE);
        vm.deal(user, INITIAL_ETH_BALANCE);
    }

    function _createPacket() internal pure returns (IBCPacket memory) {
        return IBCPacket({
            sourceChannelId: 0,
            destinationChannelId: CHANNEL_ID,
            data: "",
            timeoutHeight: 0,
            timeoutTimestamp: 0
        });
    }

    function _createTokenOrder(
        address quoteTokenAddr,
        uint256 baseAmount,
        uint256 quoteAmount
    ) internal view returns (TokenOrderV2 memory) {
        return TokenOrderV2({
            sender: abi.encodePacked(user),
            receiver: abi.encodePacked(receiver),
            baseToken: BASE_TOKEN,
            baseAmount: baseAmount,
            quoteToken: abi.encodePacked(quoteTokenAddr),
            quoteAmount: quoteAmount,
            kind: 0,
            metadata: ""
        });
    }

    function _setupCounterparty(
        address escrowedToken
    ) internal {
        vm.prank(admin);
        vault.setFungibleCounterparty(
            PATH,
            CHANNEL_ID,
            BASE_TOKEN,
            CrosschainEscrowVault.FungibleCounterparty({
                beneficiary: COUNTERPARTY_BENEFICIARY,
                escrowedToken: escrowedToken
            })
        );
    }

    function _deployNewVault() internal returns (CrosschainEscrowVault) {
        CrosschainEscrowVault implementation = new CrosschainEscrowVault();
        bytes memory initData = abi.encodeCall(
            CrosschainEscrowVault.initialize, (address(accessManager), zkgm)
        );
        ERC1967Proxy proxy = new ERC1967Proxy(address(implementation), initData);
        CrosschainEscrowVault newVault = CrosschainEscrowVault(payable(proxy));

        vm.startPrank(admin);
        _setupPermissions(address(newVault), accessManager.ADMIN_ROLE());
        vm.stopPrank();

        return newVault;
    }

    function test_Initialize() public view {
        assertEq(vault.zkgm(), zkgm);
        assertEq(vault.authority(), address(accessManager));
    }

    function test_Initialize_Twice_Reverts() public {
        vm.expectRevert();
        vault.initialize(address(accessManager), zkgm);
    }

    function test_Constants() public view {
        assertEq(
            vault.NATIVE_TOKEN(), 0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE
        );
    }

    function test_Receive_ETH() public {
        uint256 balanceBefore = address(vault).balance;

        (bool success,) = address(vault).call{value: 1 ether}("");
        assertTrue(success);

        assertEq(address(vault).balance, balanceBefore + 1 ether);
    }

    function test_SetFungibleCounterparty_ERC20() public {
        vm.prank(admin);
        vault.setFungibleCounterparty(
            PATH,
            CHANNEL_ID,
            BASE_TOKEN,
            CrosschainEscrowVault.FungibleCounterparty({
                beneficiary: COUNTERPARTY_BENEFICIARY,
                escrowedToken: address(token)
            })
        );

        CrosschainEscrowVault.FungibleCounterparty memory counterparty =
            vault.fungibleCounterparty(PATH, CHANNEL_ID, BASE_TOKEN);

        assertEq(counterparty.beneficiary, COUNTERPARTY_BENEFICIARY);
        assertEq(counterparty.escrowedToken, address(token));
    }

    function test_SetFungibleCounterparty_NativeToken() public {
        vm.startPrank(admin);
        vault.setFungibleCounterparty(
            PATH,
            CHANNEL_ID,
            BASE_TOKEN,
            CrosschainEscrowVault.FungibleCounterparty({
                beneficiary: COUNTERPARTY_BENEFICIARY,
                escrowedToken: vault.NATIVE_TOKEN()
            })
        );
        vm.stopPrank();

        CrosschainEscrowVault.FungibleCounterparty memory counterparty =
            vault.fungibleCounterparty(PATH, CHANNEL_ID, BASE_TOKEN);

        assertEq(counterparty.escrowedToken, vault.NATIVE_TOKEN());
    }

    function test_SetFungibleCounterparty_OnlyRestricted() public {
        vm.prank(user);
        vm.expectRevert();
        vault.setFungibleCounterparty(
            PATH,
            CHANNEL_ID,
            BASE_TOKEN,
            CrosschainEscrowVault.FungibleCounterparty({
                beneficiary: COUNTERPARTY_BENEFICIARY,
                escrowedToken: address(token)
            })
        );
    }

    function test_WhitelistIntent_Single() public {
        bytes32[] memory hashes = new bytes32[](1);
        hashes[0] = keccak256("test_packet_hash");

        vm.prank(admin);
        vault.whitelistIntent(hashes, true);

        assertTrue(vault.intentWhitelist(hashes[0]));
    }

    function test_WhitelistIntent_Multiple() public {
        bytes32[] memory hashes = new bytes32[](3);
        hashes[0] = keccak256("hash1");
        hashes[1] = keccak256("hash2");
        hashes[2] = keccak256("hash3");

        vm.prank(admin);
        vault.whitelistIntent(hashes, true);

        for (uint256 i = 0; i < hashes.length; i++) {
            assertTrue(vault.intentWhitelist(hashes[i]));
        }
    }

    function test_WhitelistIntent_Remove() public {
        bytes32[] memory hashes = new bytes32[](1);
        hashes[0] = keccak256("test_packet_hash");

        vm.prank(admin);
        vault.whitelistIntent(hashes, true);
        assertTrue(vault.intentWhitelist(hashes[0]));

        vm.prank(admin);
        vault.whitelistIntent(hashes, false);
        assertFalse(vault.intentWhitelist(hashes[0]));
    }

    function test_WhitelistIntent_OnlyRestricted() public {
        bytes32[] memory hashes = new bytes32[](1);
        hashes[0] = keccak256("test_packet_hash");

        vm.prank(user);
        vm.expectRevert();
        vault.whitelistIntent(hashes, true);
    }

    function test_Solve_ERC20_Success() public {
        _setupCounterparty(address(token));

        IBCPacket memory packet = _createPacket();
        TokenOrderV2 memory order =
            _createTokenOrder(address(token), 100 ether, 90 ether);

        uint256 receiverBalanceBefore = token.balanceOf(receiver);
        uint256 relayerBalanceBefore = token.balanceOf(relayer);
        uint256 vaultBalanceBefore = token.balanceOf(address(vault));

        vm.prank(zkgm);
        bytes memory result =
            vault.solve(packet, order, PATH, user, relayer, "", false);

        assertEq(result, COUNTERPARTY_BENEFICIARY);
        assertEq(token.balanceOf(receiver), receiverBalanceBefore + 90 ether);
        assertEq(token.balanceOf(relayer), relayerBalanceBefore + 10 ether);
        assertEq(
            token.balanceOf(address(vault)), vaultBalanceBefore - 100 ether
        );
    }

    function test_Solve_ETH_Success() public {
        _setupCounterparty(vault.NATIVE_TOKEN());

        IBCPacket memory packet = _createPacket();
        TokenOrderV2 memory order =
            _createTokenOrder(vault.NATIVE_TOKEN(), 5 ether, 4 ether);

        uint256 receiverBalanceBefore = receiver.balance;
        uint256 relayerBalanceBefore = relayer.balance;
        uint256 vaultBalanceBefore = address(vault).balance;

        vm.prank(zkgm);
        bytes memory result =
            vault.solve(packet, order, PATH, user, relayer, "", false);

        assertEq(result, COUNTERPARTY_BENEFICIARY);
        assertEq(receiver.balance, receiverBalanceBefore + 4 ether);
        assertEq(relayer.balance, relayerBalanceBefore + 1 ether);
        assertEq(address(vault).balance, vaultBalanceBefore - 5 ether);
    }

    function test_Solve_NoRelayerFee() public {
        _setupCounterparty(address(token));

        IBCPacket memory packet = _createPacket();
        TokenOrderV2 memory order =
            _createTokenOrder(address(token), 100 ether, 100 ether);

        uint256 relayerBalanceBefore = token.balanceOf(relayer);

        vm.prank(zkgm);
        vault.solve(packet, order, PATH, user, relayer, "", false);

        assertEq(token.balanceOf(relayer), relayerBalanceBefore);
    }

    function test_Solve_WithIntent_Whitelisted() public {
        _setupCounterparty(address(token));

        IBCPacket memory packet = _createPacket();
        bytes32 packetHash = IBCPacketLib.commitPacket(packet);

        bytes32[] memory hashes = new bytes32[](1);
        hashes[0] = packetHash;

        vm.prank(admin);
        vault.whitelistIntent(hashes, true);

        TokenOrderV2 memory order =
            _createTokenOrder(address(token), 100 ether, 90 ether);

        vm.prank(zkgm);
        bytes memory result =
            vault.solve(packet, order, PATH, user, relayer, "", true);
        assertEq(result, COUNTERPARTY_BENEFICIARY);
    }

    function test_Solve_OnlyZkgm() public {
        _setupCounterparty(address(token));

        IBCPacket memory packet = _createPacket();
        TokenOrderV2 memory order =
            _createTokenOrder(address(token), 100 ether, 90 ether);

        vm.prank(user);
        vm.expectRevert(
            CrosschainEscrowVault.CrosschainEscrowVault_OnlyZkgm.selector
        );
        vault.solve(packet, order, PATH, user, relayer, "", false);
    }

    function test_Solve_Intent_NotWhitelisted() public {
        _setupCounterparty(address(token));

        IBCPacket memory packet = _createPacket();
        TokenOrderV2 memory order =
            _createTokenOrder(address(token), 100 ether, 90 ether);

        vm.prank(zkgm);
        vm.expectRevert(
            CrosschainEscrowVault
                .CrosschainEscrowVault_IntentWhitelistedOnly
                .selector
        );
        vault.solve(packet, order, PATH, user, relayer, "", true);
    }

    function test_Solve_CounterpartyNotConfigured() public {
        IBCPacket memory packet = _createPacket();
        TokenOrderV2 memory order =
            _createTokenOrder(address(token), 100 ether, 90 ether);

        vm.prank(zkgm);
        vm.expectRevert(
            CrosschainEscrowVault
                .CrosschainEscrowVault_CounterpartyIsNotFungible
                .selector
        );
        vault.solve(packet, order, PATH, user, relayer, "", false);
    }

    function test_Solve_InvalidQuoteToken() public {
        _setupCounterparty(address(token));

        IBCPacket memory packet = _createPacket();
        TokenOrderV2 memory order =
            _createTokenOrder(address(0x999), 100 ether, 90 ether);

        vm.prank(zkgm);
        vm.expectRevert(
            CrosschainEscrowVault
                .CrosschainEscrowVault_InvalidQuoteToken
                .selector
        );
        vault.solve(packet, order, PATH, user, relayer, "", false);
    }

    function test_Solve_BaseAmountTooLow() public {
        _setupCounterparty(address(token));

        IBCPacket memory packet = _createPacket();
        TokenOrderV2 memory order =
            _createTokenOrder(address(token), 90 ether, 100 ether);

        vm.prank(zkgm);
        vm.expectRevert(
            CrosschainEscrowVault
                .CrosschainEscrowVault_BaseAmountMustCoverQuoteAmount
                .selector
        );
        vault.solve(packet, order, PATH, user, relayer, "", false);
    }

    function test_Solve_InsufficientERC20Balance() public {
        CrosschainEscrowVault emptyVault = _deployNewVault();

        vm.startPrank(admin);
        emptyVault.setFungibleCounterparty(
            PATH,
            CHANNEL_ID,
            BASE_TOKEN,
            CrosschainEscrowVault.FungibleCounterparty({
                beneficiary: COUNTERPARTY_BENEFICIARY,
                escrowedToken: address(token)
            })
        );
        vm.stopPrank();

        IBCPacket memory packet = _createPacket();
        TokenOrderV2 memory order =
            _createTokenOrder(address(token), 100 ether, 90 ether);

        vm.prank(zkgm);
        vm.expectRevert();
        emptyVault.solve(packet, order, PATH, user, relayer, "", false);
    }

    function test_Solve_InsufficientETHBalance() public {
        CrosschainEscrowVault emptyVault = _deployNewVault();

        vm.startPrank(admin);
        emptyVault.setFungibleCounterparty(
            PATH,
            CHANNEL_ID,
            BASE_TOKEN,
            CrosschainEscrowVault.FungibleCounterparty({
                beneficiary: COUNTERPARTY_BENEFICIARY,
                escrowedToken: vault.NATIVE_TOKEN()
            })
        );
        vm.stopPrank();

        IBCPacket memory packet = _createPacket();
        TokenOrderV2 memory order =
            _createTokenOrder(vault.NATIVE_TOKEN(), 5 ether, 4 ether);

        vm.prank(zkgm);
        vm.expectRevert(
            CrosschainEscrowVault.CrosschainEscrowVault_TransferFailed.selector
        );
        emptyVault.solve(packet, order, PATH, user, relayer, "", false);
    }

    function testFuzz_Solve_ValidAmounts(
        uint128 baseAmount,
        uint128 quoteAmount
    ) public {
        vm.assume(baseAmount >= quoteAmount);
        vm.assume(quoteAmount > 0);
        vm.assume(baseAmount <= 1000 ether);

        if (baseAmount > token.balanceOf(address(vault))) {
            token.mint(address(vault), baseAmount);
        }

        _setupCounterparty(address(token));

        IBCPacket memory packet = _createPacket();
        TokenOrderV2 memory order =
            _createTokenOrder(address(token), baseAmount, quoteAmount);

        uint256 receiverBalanceBefore = token.balanceOf(receiver);
        uint256 relayerBalanceBefore = token.balanceOf(relayer);

        vm.prank(zkgm);
        bytes memory result =
            vault.solve(packet, order, PATH, user, relayer, "", false);

        assertEq(result, COUNTERPARTY_BENEFICIARY);
        assertEq(token.balanceOf(receiver), receiverBalanceBefore + quoteAmount);
        assertEq(
            token.balanceOf(relayer),
            relayerBalanceBefore + (baseAmount - quoteAmount)
        );
    }

    function test_Solve_MaxValues() public {
        uint256 maxAmount = type(uint128).max;

        token.mint(address(vault), maxAmount);
        _setupCounterparty(address(token));

        IBCPacket memory packet = _createPacket();
        TokenOrderV2 memory order =
            _createTokenOrder(address(token), maxAmount, maxAmount - 1);

        vm.prank(zkgm);
        bytes memory result =
            vault.solve(packet, order, PATH, user, relayer, "", false);

        assertEq(result, COUNTERPARTY_BENEFICIARY);
    }

    function test_Upgrade_OnlyRestricted() public {
        CrosschainEscrowVault newImpl = new CrosschainEscrowVault();

        vm.prank(user);
        vm.expectRevert();
        vault.upgradeToAndCall(address(newImpl), "");
    }

    function test_Upgrade_Success() public {
        CrosschainEscrowVault newImpl = new CrosschainEscrowVault();

        vm.prank(admin);
        vault.upgradeToAndCall(address(newImpl), "");

        assertEq(vault.zkgm(), zkgm);
    }
}
