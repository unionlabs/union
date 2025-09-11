pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "forge-std/console.sol";

import "solady/utils/LibBytes.sol";

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/access/manager/AccessManager.sol";
import "@openzeppelin/contracts/access/manager/IAccessManager.sol";

import "../../../contracts/CrosschainVault.sol";
import "../../../contracts/core/Types.sol";
import "../../../contracts/core/04-channel/IBCPacket.sol";
import "../../../contracts/apps/ucs/03-zkgm/Types.sol";
import "../../../contracts/apps/ucs/03-zkgm/ISolver.sol";

contract MockERC20 is ERC20 {
    uint8 private immutable _decimals;

    constructor(
        string memory name,
        string memory symbol,
        uint8 decimals_
    ) ERC20(name, symbol) {
        _decimals = decimals_;
    }

    function mint(address to, uint256 amount) external {
        _mint(to, amount);
    }

    function decimals() public view override returns (uint8) {
        return _decimals;
    }
}

contract MockZkgm {}

abstract contract CrosschainVaultTestBase is Test {
    using LibBytes for *;

    CrosschainVault public vault;
    MockZkgm public zkgm;
    MockERC20 public quoteToken;
    AccessManager public accessManager;

    address public constant ADMIN = address(0x2);
    address public constant RELAYER = address(0x3);
    address public constant USER = address(0x4);
    address public constant PROTOCOL_FEE_BENEFICIARY = address(0x5);
    address public constant DEPOSITOR = address(0x100);
    address public constant REPAYER = address(0x99);

    uint256 constant BPS_SCALE = 1_000_000;
    uint256 constant DEFAULT_PATH = 1;
    uint32 constant DEFAULT_CHANNEL_ID = 100;
    uint256 constant DEFAULT_VAULT_FEE_BPS = 10000;
    uint256 constant DEFAULT_PROTOCOL_FEE_BPS = 5000;

    bytes constant BASE_TOKEN = hex"1234567890123456789012345678901234567890";
    bytes constant DEBT_ACCOUNT = hex"deadbeef";

    function setUp() public virtual {
        _deployContracts();
        _configureAccessControl();
    }

    function _deployContracts() internal {
        accessManager = new AccessManager(ADMIN);
        quoteToken = new MockERC20("USD Coin", "USDC", 6);

        CrosschainVault vaultImpl = new CrosschainVault();
        zkgm = new MockZkgm();

        bytes memory initData = abi.encodeWithSelector(
            CrosschainVault.initialize.selector,
            address(accessManager),
            address(zkgm),
            address(quoteToken)
        );
        ERC1967Proxy vaultProxy = new ERC1967Proxy(address(vaultImpl), initData);
        vault = CrosschainVault(address(vaultProxy));
    }

    function _configureAccessControl() internal {
        vm.startPrank(ADMIN);

        bytes4[] memory selectors = new bytes4[](2);
        selectors[0] = vault.setFungibleCounterparty.selector;
        selectors[1] = vault.whitelistIntent.selector;

        accessManager.setTargetFunctionRole(address(vault), selectors, 0);
        accessManager.grantRole(0, ADMIN, 0);

        vm.stopPrank();
    }

    function _createDefaultCounterparty()
        internal
        pure
        returns (CrosschainVault.FungibleCounterparty memory)
    {
        return CrosschainVault.FungibleCounterparty({
            bpsFee: DEFAULT_VAULT_FEE_BPS,
            bpsProtocolFee: DEFAULT_PROTOCOL_FEE_BPS,
            protocolFeeBeneficiary: PROTOCOL_FEE_BENEFICIARY,
            debt: 0,
            debtAccount: DEBT_ACCOUNT
        });
    }

    function _createTokenOrder(
        uint256 baseAmount,
        uint256 quoteAmount
    ) internal view returns (TokenOrderV2 memory) {
        return TokenOrderV2({
            sender: abi.encodePacked(USER),
            receiver: abi.encodePacked(USER),
            baseToken: BASE_TOKEN,
            baseAmount: baseAmount,
            quoteToken: abi.encodePacked(address(quoteToken)),
            quoteAmount: quoteAmount,
            kind: 0,
            metadata: ""
        });
    }

    function _createPacket() internal pure returns (IBCPacket memory) {
        return _createPacket(DEFAULT_CHANNEL_ID, "", 0, 0);
    }

    function _createPacket(
        uint32 destChannelId,
        bytes memory data,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) internal pure returns (IBCPacket memory) {
        return IBCPacket({
            sourceChannelId: 1,
            destinationChannelId: destChannelId,
            data: data,
            timeoutHeight: timeoutHeight,
            timeoutTimestamp: timeoutTimestamp
        });
    }

    function _setupCounterparty() internal {
        _setupCounterparty(_createDefaultCounterparty());
    }

    function _setupCounterparty(
        CrosschainVault.FungibleCounterparty memory counterparty
    ) internal {
        vm.prank(ADMIN);
        vault.setFungibleCounterparty(
            DEFAULT_PATH, DEFAULT_CHANNEL_ID, BASE_TOKEN, counterparty
        );
    }

    function _fundVault(
        uint256 amount
    ) internal {
        quoteToken.mint(address(vault), amount);
    }

    function _calculateFees(
        uint256 baseAmount,
        uint256 vaultFeeBps,
        uint256 protocolFeeBps
    )
        internal
        pure
        returns (uint256 vaultFee, uint256 protocolFee, uint256 totalFee)
    {
        vaultFee = (baseAmount * vaultFeeBps) / BPS_SCALE;
        protocolFee = (baseAmount * protocolFeeBps) / BPS_SCALE;
        totalFee = vaultFee + protocolFee;
    }
}

contract CrosschainVaultInitializationTest is CrosschainVaultTestBase {
    function testInitialization() public view {
        assertEq(vault.zkgm(), address(zkgm), "Invalid zkgm address");
        assertEq(vault.deployedCapital(), 0, "Deployed capital should be zero");
        assertEq(vault.asset(), address(quoteToken), "Invalid ERC4626 asset");
    }

    function testCannotReinitialize() public {
        vm.expectRevert();
        vault.initialize(
            address(accessManager), address(zkgm), address(quoteToken)
        );
    }
}

contract CrosschainVaultConfigurationTest is CrosschainVaultTestBase {
    function testSetFungibleCounterparty() public {
        CrosschainVault.FungibleCounterparty memory counterparty =
            _createDefaultCounterparty();

        vm.prank(ADMIN);
        vault.setFungibleCounterparty(
            DEFAULT_PATH, DEFAULT_CHANNEL_ID, BASE_TOKEN, counterparty
        );

        CrosschainVault.FungibleCounterparty memory stored = vault
            .fungibleCounterparty(DEFAULT_PATH, DEFAULT_CHANNEL_ID, BASE_TOKEN);

        assertEq(stored.bpsFee, counterparty.bpsFee, "Invalid vault fee");
        assertEq(
            stored.bpsProtocolFee,
            counterparty.bpsProtocolFee,
            "Invalid protocol fee"
        );
        assertEq(
            stored.protocolFeeBeneficiary,
            counterparty.protocolFeeBeneficiary,
            "Invalid beneficiary"
        );
        assertEq(stored.debt, 0, "Debt should be zero");
        assertEq(
            stored.debtAccount, counterparty.debtAccount, "Invalid debt account"
        );
    }

    function testWhitelistIntent() public {
        bytes32[] memory hashes = new bytes32[](2);
        hashes[0] = keccak256("packet1");
        hashes[1] = keccak256("packet2");

        vm.prank(ADMIN);
        vault.whitelistIntent(hashes, true);

        assertTrue(
            vault.intentWhitelist(hashes[0]), "Hash 0 should be whitelisted"
        );
        assertTrue(
            vault.intentWhitelist(hashes[1]), "Hash 1 should be whitelisted"
        );

        vm.prank(ADMIN);
        vault.whitelistIntent(hashes, false);

        assertFalse(
            vault.intentWhitelist(hashes[0]), "Hash 0 should not be whitelisted"
        );
        assertFalse(
            vault.intentWhitelist(hashes[1]), "Hash 1 should not be whitelisted"
        );
    }

    function testFeeCalculation() public {
        _setupCounterparty();

        uint256 amount = 1000000;
        (,, uint256 totalFee) = _calculateFees(
            amount, DEFAULT_VAULT_FEE_BPS, DEFAULT_PROTOCOL_FEE_BPS
        );

        assertEq(
            vault.fee(DEFAULT_PATH, DEFAULT_CHANNEL_ID, BASE_TOKEN, amount),
            totalFee
        );
    }
}

contract CrosschainVaultSolveTest is CrosschainVaultTestBase {
    uint256 constant BASE_AMOUNT = 1000000;
    uint256 constant VAULT_BALANCE = 10000000;

    function setUp() public override {
        super.setUp();
        _setupCounterparty();
        _fundVault(VAULT_BALANCE);
    }

    function testSolveSuccessfulOrder() public {
        uint256 quoteAmount = 985000;
        TokenOrderV2 memory order = _createTokenOrder(BASE_AMOUNT, quoteAmount);
        IBCPacket memory packet = _createPacket();

        uint256 initialUserBalance = quoteToken.balanceOf(USER);
        uint256 initialProtocolBalance =
            quoteToken.balanceOf(PROTOCOL_FEE_BENEFICIARY);

        vm.prank(address(zkgm));
        bytes memory result = vault.solve(
            packet, order, DEFAULT_PATH, address(0), RELAYER, "", false
        );

        assertEq(result, DEBT_ACCOUNT, "Invalid market maker address");

        assertEq(
            quoteToken.balanceOf(USER),
            initialUserBalance + quoteAmount,
            "Invalid user balance"
        );
        assertEq(
            quoteToken.balanceOf(PROTOCOL_FEE_BENEFICIARY),
            initialProtocolBalance + 5000,
            "Invalid protocol fee"
        );
        assertEq(
            quoteToken.balanceOf(RELAYER),
            0,
            "Relayer should not receive excess fee"
        );

        assertEq(
            vault.fungibleCounterparty(
                DEFAULT_PATH, DEFAULT_CHANNEL_ID, BASE_TOKEN
            ).debt,
            BASE_AMOUNT,
            "Invalid debt amount"
        );
        uint256 expectedDeployed = 990000 + 10000; // deployed to users + vault fee
        assertEq(
            vault.deployedCapital(),
            expectedDeployed,
            "Invalid deployed capital"
        );
    }

    function testSolveWithExcessFee() public {
        uint256 quoteAmount = 980000;
        TokenOrderV2 memory order = _createTokenOrder(BASE_AMOUNT, quoteAmount);
        IBCPacket memory packet = _createPacket();

        uint256 initialRelayerBalance = quoteToken.balanceOf(RELAYER);

        vm.prank(address(zkgm));
        vault.solve(packet, order, DEFAULT_PATH, address(0), RELAYER, "", false);

        assertEq(
            quoteToken.balanceOf(RELAYER),
            initialRelayerBalance + 5000,
            "Relayer should receive excess fee"
        );
    }

    function testSolveWithIntent() public {
        IBCPacket memory packet =
            _createPacket(DEFAULT_CHANNEL_ID, abi.encode("test"), 100, 1000);
        TokenOrderV2 memory order = _createTokenOrder(BASE_AMOUNT, 985000);

        bytes32 packetHash = IBCPacketLib.commitPacket(packet);
        bytes32[] memory hashes = new bytes32[](1);
        hashes[0] = packetHash;

        vm.prank(ADMIN);
        vault.whitelistIntent(hashes, true);

        vm.prank(address(zkgm));
        bytes memory result = vault.solve(
            packet, order, DEFAULT_PATH, address(0), RELAYER, "", true
        );

        assertEq(result, DEBT_ACCOUNT, "Invalid result for intent solve");
    }

    function testFuzzSolveVariousFees(
        uint256 baseAmount,
        uint256 bpsFee,
        uint256 bpsProtocolFee
    ) public {
        baseAmount = bound(baseAmount, 1000, 10000000);
        bpsFee = bound(bpsFee, 0, 100000);
        bpsProtocolFee = bound(bpsProtocolFee, 0, 50000);

        CrosschainVault.FungibleCounterparty memory counterparty =
        CrosschainVault.FungibleCounterparty({
            bpsFee: bpsFee,
            bpsProtocolFee: bpsProtocolFee,
            protocolFeeBeneficiary: PROTOCOL_FEE_BENEFICIARY,
            debt: 0,
            debtAccount: DEBT_ACCOUNT
        });

        vm.prank(ADMIN);
        vault.setFungibleCounterparty(
            DEFAULT_PATH, DEFAULT_CHANNEL_ID, BASE_TOKEN, counterparty
        );

        quoteToken.mint(address(vault), baseAmount * 2);

        (, uint256 protocolFee, uint256 totalFee) =
            _calculateFees(baseAmount, bpsFee, bpsProtocolFee);
        uint256 quoteAmount = baseAmount - totalFee;

        TokenOrderV2 memory order = _createTokenOrder(baseAmount, quoteAmount);
        IBCPacket memory packet = _createPacket();

        uint256 initialUserBalance = quoteToken.balanceOf(USER);
        uint256 initialProtocolBalance =
            quoteToken.balanceOf(PROTOCOL_FEE_BENEFICIARY);

        vm.prank(address(zkgm));
        vault.solve(packet, order, DEFAULT_PATH, address(0), RELAYER, "", false);

        assertEq(quoteToken.balanceOf(USER), initialUserBalance + quoteAmount);
        assertEq(
            quoteToken.balanceOf(PROTOCOL_FEE_BENEFICIARY),
            initialProtocolBalance + protocolFee
        );
        assertEq(
            vault.fungibleCounterparty(
                DEFAULT_PATH, DEFAULT_CHANNEL_ID, BASE_TOKEN
            ).debt,
            baseAmount
        );
    }
}

contract CrosschainVaultSecurityTest is CrosschainVaultTestBase {
    function testSolveRevertOnlyZkgm() public {
        TokenOrderV2 memory order = _createTokenOrder(1000000, 985000);
        IBCPacket memory packet = _createPacket();

        vm.expectRevert(CrosschainVault.CrosschainVault_OnlyZkgm.selector);
        vault.solve(packet, order, DEFAULT_PATH, address(0), RELAYER, "", false);
    }

    function testSolveRevertIntentNotWhitelisted() public {
        _setupCounterparty();

        TokenOrderV2 memory order = _createTokenOrder(1000000, 985000);
        IBCPacket memory packet = _createPacket();

        vm.prank(address(zkgm));
        vm.expectRevert(
            CrosschainVault.CrosschainVault_IntentWhitelistedOnly.selector
        );
        vault.solve(packet, order, DEFAULT_PATH, address(0), RELAYER, "", true);
    }

    function testSolveRevertCounterpartyNotFungible() public {
        TokenOrderV2 memory order = _createTokenOrder(1000000, 985000);
        IBCPacket memory packet = _createPacket();

        vm.prank(address(zkgm));
        vm.expectRevert(
            CrosschainVault.CrosschainVault_CounterpartyIsNotFungible.selector
        );
        vault.solve(packet, order, DEFAULT_PATH, address(0), RELAYER, "", false);
    }

    function testSolveRevertWrongQuoteToken() public {
        _setupCounterparty();

        TokenOrderV2 memory order = TokenOrderV2({
            sender: abi.encodePacked(USER),
            receiver: abi.encodePacked(USER),
            baseToken: BASE_TOKEN,
            baseAmount: 1000000,
            quoteToken: abi.encodePacked(address(0xdead)),
            quoteAmount: 985000,
            kind: 0,
            metadata: ""
        });

        IBCPacket memory packet = _createPacket();

        vm.prank(address(zkgm));
        vm.expectRevert(CrosschainVault.CrosschainVault_Fool.selector);
        vault.solve(packet, order, DEFAULT_PATH, address(0), RELAYER, "", false);
    }

    function testSolveRevertQuoteAmountTooHigh() public {
        _setupCounterparty();

        TokenOrderV2 memory order = _createTokenOrder(1000000, 1000001);
        IBCPacket memory packet = _createPacket();

        vm.prank(address(zkgm));
        vm.expectRevert(
            CrosschainVault
                .CrosschainVault_BaseAmountMustCoverQuoteAmount
                .selector
        );
        vault.solve(packet, order, DEFAULT_PATH, address(0), RELAYER, "", false);
    }

    function testSolveRevertInsufficientFee() public {
        _setupCounterparty();

        TokenOrderV2 memory order = _createTokenOrder(1000000, 990000);
        IBCPacket memory packet = _createPacket();

        vm.prank(address(zkgm));
        vm.expectRevert(CrosschainVault.CrosschainVault_RespectMinFee.selector);
        vault.solve(packet, order, DEFAULT_PATH, address(0), RELAYER, "", false);
    }
}

contract CrosschainVaultDebtTest is CrosschainVaultTestBase {
    function testRepay() public {
        _setupCounterparty();
        _fundVault(10000000);

        TokenOrderV2 memory order = _createTokenOrder(1000000, 985000);
        IBCPacket memory packet = _createPacket();

        vm.prank(address(zkgm));
        vault.solve(packet, order, DEFAULT_PATH, address(0), RELAYER, "", false);

        assertEq(
            vault.fungibleCounterparty(
                DEFAULT_PATH, DEFAULT_CHANNEL_ID, BASE_TOKEN
            ).debt,
            1000000
        );
        assertEq(vault.deployedCapital(), 1000000); // 990000 deployed + 10000 vault fee

        uint256 repayAmount = 500000;
        quoteToken.mint(REPAYER, repayAmount);

        vm.startPrank(REPAYER);
        quoteToken.approve(address(vault), repayAmount);
        vault.repay(DEFAULT_PATH, DEFAULT_CHANNEL_ID, BASE_TOKEN, repayAmount);
        vm.stopPrank();

        assertEq(
            vault.fungibleCounterparty(
                DEFAULT_PATH, DEFAULT_CHANNEL_ID, BASE_TOKEN
            ).debt,
            500000,
            "Debt not properly reduced"
        );
        // After partial repayment, deployedCapital should be reduced by repayment amount
        // Original: 990000 (deployed) + 10000 (vault fee) = 1000000
        // After repaying 500000: 1000000 - 500000 = 500000
        assertEq(
            vault.deployedCapital(),
            500000,
            "Deployed capital not properly reduced"
        );
    }

    function testRepayRevertNotFungible() public {
        vm.expectRevert(
            CrosschainVault.CrosschainVault_CounterpartyIsNotFungible.selector
        );
        vault.repay(DEFAULT_PATH, DEFAULT_CHANNEL_ID, BASE_TOKEN, 100000);
    }

    function testRepayRevertTooMuch() public {
        CrosschainVault.FungibleCounterparty memory counterparty =
        CrosschainVault.FungibleCounterparty({
            bpsFee: 10000,
            bpsProtocolFee: 5000,
            protocolFeeBeneficiary: PROTOCOL_FEE_BENEFICIARY,
            debt: 100000,
            debtAccount: DEBT_ACCOUNT
        });

        vm.prank(ADMIN);
        vault.setFungibleCounterparty(
            DEFAULT_PATH, DEFAULT_CHANNEL_ID, BASE_TOKEN, counterparty
        );

        vm.expectRevert(
            CrosschainVault.CrosschainVault_RepayingTooMuch.selector
        );
        vault.repay(DEFAULT_PATH, DEFAULT_CHANNEL_ID, BASE_TOKEN, 100001);
    }
}

contract CrosschainVaultERC4626Test is CrosschainVaultTestBase {
    function testTotalAssets() public {
        quoteToken.mint(address(vault), 1000000);
        assertEq(vault.totalAssets(), 1000000);

        _setupCounterparty();

        TokenOrderV2 memory order = _createTokenOrder(100000, 98500);
        IBCPacket memory packet = _createPacket();

        vm.prank(address(zkgm));
        vault.solve(packet, order, DEFAULT_PATH, address(0), RELAYER, "", false);

        assertEq(
            vault.totalAssets(),
            1001000, // includes 1000 vault fee
            "Total assets should include deployed capital and fees"
        );
        uint256 expectedDeployed = 99000 + 1000; // deployed + vault fee (1% of 100k)
        assertEq(
            vault.deployedCapital(),
            expectedDeployed,
            "Invalid deployed capital"
        );
        assertEq(
            quoteToken.balanceOf(address(vault)),
            901000,
            "Invalid vault balance"
        );
    }

    function testDepositAndRedeem() public {
        quoteToken.mint(DEPOSITOR, 1000000);

        vm.startPrank(DEPOSITOR);
        quoteToken.approve(address(vault), 1000000);
        uint256 shares = vault.deposit(1000000, DEPOSITOR);
        vm.stopPrank();

        assertEq(shares, 1000000, "Invalid shares minted");
        assertEq(vault.balanceOf(DEPOSITOR), 1000000, "Invalid share balance");
        assertEq(vault.totalAssets(), 1000000, "Invalid total assets");

        _setupCounterparty();
        TokenOrderV2 memory order = _createTokenOrder(100000, 98500);
        IBCPacket memory packet = _createPacket();

        vm.prank(address(zkgm));
        vault.solve(packet, order, DEFAULT_PATH, address(0), RELAYER, "", false);

        assertEq(
            vault.totalAssets(),
            1001000,
            "Total assets should include vault fee"
        );
        assertEq(
            quoteToken.balanceOf(address(vault)),
            901000,
            "Invalid liquid balance"
        );

        // Calculate shares to redeem based on liquid balance vs total assets
        // Total assets = 1001000, liquid balance = 901000
        uint256 partialShares = (shares * 901000) / 1001000;

        vm.prank(DEPOSITOR);
        uint256 assetsOut = vault.redeem(partialShares, DEPOSITOR, DEPOSITOR);

        // Due to rounding in ERC4626 conversion, we expect approximately 901000
        assertApproxEqAbs(assetsOut, 901000, 1, "Invalid redemption amount");
    }

    function testMaxWithdraw() public {
        quoteToken.mint(DEPOSITOR, 1000000);

        vm.startPrank(DEPOSITOR);
        quoteToken.approve(address(vault), 1000000);
        vault.deposit(1000000, DEPOSITOR);
        vm.stopPrank();

        _setupCounterparty();
        TokenOrderV2 memory order = _createTokenOrder(100000, 98500);
        IBCPacket memory packet = _createPacket();

        vm.prank(address(zkgm));
        vault.solve(packet, order, DEFAULT_PATH, address(0), RELAYER, "", false);

        uint256 maxWithdrawAmount = vault.maxWithdraw(DEPOSITOR);
        uint256 liquidBalance = quoteToken.balanceOf(address(vault));

        assertEq(
            maxWithdrawAmount,
            1000999, // Share value increased due to vault fees
            "Max withdraw should be based on share value including fees"
        );
        assertEq(
            liquidBalance,
            901000,
            "Liquid balance should be reduced by deployed capital"
        );
    }
}
