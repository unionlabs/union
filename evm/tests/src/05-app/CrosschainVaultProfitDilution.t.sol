pragma solidity ^0.8.27;

import "./CrosschainVault.t.sol";

contract CrosschainVaultProfitDilutionTest is CrosschainVaultTestBase {
    address public constant ATTACKER = address(0x666);
    address public constant EARLY_LP = address(0x777);
    address public constant LATE_LP = address(0x888);

    uint256 public constant INITIAL_DEPOSIT = 10_000_000;
    uint256 public constant LOAN_AMOUNT = 1_000_000;
    uint256 public constant ATTACKER_DEPOSIT = 100_000_000;

    uint256 public constant EXPECTED_VAULT_FEE = 10_000;
    uint256 public constant EXPECTED_PROTOCOL_FEE = 5_000;
    uint256 public constant EXPECTED_TOTAL_FEE = 15_000;

    function setUp() public override {
        super.setUp();
        _setupCounterparty();
    }

    function testImmediateFeeRecognition() public {
        _mintAndDeposit(EARLY_LP, INITIAL_DEPOSIT);

        assertEq(vault.deployedCapital(), 0);
        assertEq(vault.totalAssets(), INITIAL_DEPOSIT);

        TokenOrderV2 memory order = _createLoanOrder();
        _executeLoan(order);

        uint256 expectedDeployedPrincipal =
            order.quoteAmount + EXPECTED_PROTOCOL_FEE;
        uint256 expectedDeployedTotal =
            expectedDeployedPrincipal + EXPECTED_VAULT_FEE;

        assertEq(vault.deployedCapital(), expectedDeployedTotal);
        assertEq(vault.totalAssets(), INITIAL_DEPOSIT + EXPECTED_VAULT_FEE);
    }

    function testImmediateSharePriceAppreciation() public {
        _mintAndDeposit(EARLY_LP, INITIAL_DEPOSIT);
        uint256 initialSharePrice = vault.convertToAssets(1e18);

        _executeMultipleLoans(3);

        uint256 newSharePrice = vault.convertToAssets(1e18);
        assertGt(newSharePrice, initialSharePrice);

        _verifyLateDepositorPremium();
    }

    /**
     * @dev Verifies that an attacker cannot sandwich a repayment to steal profits
     */
    function testProfitDilutionAttackPrevented() public {
        _mintAndDeposit(EARLY_LP, INITIAL_DEPOSIT);
        uint256 earlyLpShares = vault.balanceOf(EARLY_LP);

        TokenOrderV2 memory order = _createLoanOrder();
        _executeLoan(order);

        uint256 totalAssetsWithProfit = vault.totalAssets();
        uint256 attackerShares =
            _performAttackerDeposit(totalAssetsWithProfit, earlyLpShares);

        _repayLoan();
        _verifyAttackPrevention(earlyLpShares, attackerShares);
    }

    function testSandwichAttackUnprofitable() public {
        _mintAndDeposit(EARLY_LP, INITIAL_DEPOSIT);

        TokenOrderV2 memory order = _createLoanOrder();
        _executeLoan(order);

        uint256 attackerShares = _executeSandwichAttack();
        _repayLoan();
        _verifySandwichAttackFailed(attackerShares);
    }

    function testPartialRepaymentAccounting() public {
        _mintAndDeposit(EARLY_LP, INITIAL_DEPOSIT);
        TokenOrderV2 memory order = _createLoanOrder();
        _executeLoan(order);

        uint256 initialDeployedCapital = vault.deployedCapital();
        uint256 initialTotalAssets = vault.totalAssets();

        uint256 partialRepayment = LOAN_AMOUNT / 2;
        _performRepayment(partialRepayment);

        assertEq(
            vault.deployedCapital(), initialDeployedCapital - partialRepayment
        );
        assertEq(vault.totalAssets(), initialTotalAssets);
    }

    function testFullRepaymentProfitRealization() public {
        _mintAndDeposit(EARLY_LP, INITIAL_DEPOSIT);
        TokenOrderV2 memory order = _createLoanOrder();
        _executeLoan(order);

        uint256 deployedBeforeRepay = vault.deployedCapital();
        assertEq(deployedBeforeRepay, LOAN_AMOUNT);

        _repayLoan();

        assertEq(vault.deployedCapital(), 0);
        assertEq(vault.totalAssets(), INITIAL_DEPOSIT + EXPECTED_VAULT_FEE);
    }

    function testMultipleCounterpartiesScenario() public {
        _mintAndDeposit(EARLY_LP, INITIAL_DEPOSIT * 10);
        bytes memory baseToken2 = _setupSecondCounterparty();
        _executeMultiCounterpartyLoans(baseToken2);
        _performMultiCounterpartyRepayments(baseToken2);
    }

    function _performAttackerDeposit(
        uint256 totalAssetsBeforeAttack,
        uint256 earlyLpShares
    ) internal returns (uint256 attackerShares) {
        _mintAndDeposit(ATTACKER, ATTACKER_DEPOSIT);
        attackerShares = vault.balanceOf(ATTACKER);

        uint256 expectedSharePrice =
            (totalAssetsBeforeAttack * 1e18) / earlyLpShares;
        uint256 attackerExpectedShares =
            (ATTACKER_DEPOSIT * 1e18) / expectedSharePrice;

        assertApproxEqRel(
            attackerShares,
            attackerExpectedShares,
            0.01e18,
            "Attacker should pay premium including unrealized profits"
        );
    }

    function _verifyAttackPrevention(
        uint256 earlyLpShares,
        uint256 attackerShares
    ) internal {
        uint256 earlyLpProfitShare = (EXPECTED_VAULT_FEE * earlyLpShares)
            / (earlyLpShares + attackerShares);
        assertGt(
            earlyLpProfitShare,
            900,
            "Early LP should maintain proportional profit share"
        );

        vm.prank(ATTACKER);
        uint256 attackerWithdrawal =
            vault.redeem(attackerShares, ATTACKER, ATTACKER);

        assertApproxEqRel(
            attackerWithdrawal,
            ATTACKER_DEPOSIT,
            0.01e18,
            "Attacker should not profit from the attack"
        );
    }

    function _executeSandwichAttack()
        internal
        returns (uint256 attackerShares)
    {
        _mintAndDeposit(ATTACKER, ATTACKER_DEPOSIT);
        return vault.balanceOf(ATTACKER);
    }

    function _verifySandwichAttackFailed(
        uint256 attackerShares
    ) internal {
        vm.prank(ATTACKER);
        uint256 withdrawn = vault.redeem(attackerShares, ATTACKER, ATTACKER);

        int256 attackerProfit = int256(withdrawn) - int256(ATTACKER_DEPOSIT);
        uint256 maxExpectedProfit = (EXPECTED_VAULT_FEE * 10) / 11;
        uint256 actualProfit = attackerProfit > 0 ? uint256(attackerProfit) : 0;

        assertLt(
            actualProfit,
            maxExpectedProfit,
            "Sandwich attack should not yield excessive profits"
        );
    }

    function _setupSecondCounterparty()
        internal
        returns (bytes memory baseToken2)
    {
        baseToken2 = hex"2222222222222222222222222222222222222222";

        CrosschainVault.FungibleCounterparty memory counterparty2 =
            _createDefaultCounterparty();
        counterparty2.bpsFee = 20000;

        vm.prank(ADMIN);
        vault.setFungibleCounterparty(
            DEFAULT_PATH, DEFAULT_CHANNEL_ID, baseToken2, counterparty2
        );
    }

    function _executeMultiCounterpartyLoans(
        bytes memory baseToken2
    ) internal {
        TokenOrderV2 memory order1 = _createLoanOrder();
        _executeLoan(order1);

        uint256 deployedAfterFirst = vault.deployedCapital();

        TokenOrderV2 memory order2 = _createHighFeeOrder(baseToken2);
        _executeLoan(order2);

        uint256 deployedAfterBoth = vault.deployedCapital();
        assertGt(
            deployedAfterBoth,
            deployedAfterFirst,
            "Deployed capital should accumulate"
        );
    }

    function _performMultiCounterpartyRepayments(
        bytes memory baseToken2
    ) internal {
        _performRepayment(LOAN_AMOUNT);
        _performRepayment(LOAN_AMOUNT * 2, baseToken2);

        assertEq(vault.deployedCapital(), 0, "All capital should be returned");

        uint256 expectedFees1 = EXPECTED_VAULT_FEE;
        uint256 expectedFees2 = (LOAN_AMOUNT * 2 * 20000) / BPS_SCALE;
        uint256 totalExpectedFees = expectedFees1 + expectedFees2;

        assertEq(vault.totalAssets(), INITIAL_DEPOSIT * 10 + totalExpectedFees);
    }

    function _executeMultipleLoans(
        uint256 count
    ) internal {
        for (uint256 i = 0; i < count; i++) {
            TokenOrderV2 memory order = _createLoanOrder();
            _executeLoan(order);
        }
    }

    function _verifyLateDepositorPremium() internal {
        _mintAndDeposit(LATE_LP, INITIAL_DEPOSIT);

        uint256 lateLpShares = vault.balanceOf(LATE_LP);
        uint256 earlyLpShares = vault.balanceOf(EARLY_LP);

        assertLt(
            lateLpShares,
            earlyLpShares,
            "Late depositor should receive fewer shares"
        );
    }

    function _createLoanOrder() internal view returns (TokenOrderV2 memory) {
        return _createTokenOrder(LOAN_AMOUNT, LOAN_AMOUNT - EXPECTED_TOTAL_FEE);
    }

    function _createHighFeeOrder(
        bytes memory baseToken
    ) internal view returns (TokenOrderV2 memory) {
        return TokenOrderV2({
            sender: abi.encodePacked(USER),
            receiver: abi.encodePacked(USER),
            baseToken: baseToken,
            baseAmount: LOAN_AMOUNT * 2,
            quoteToken: abi.encodePacked(address(quoteToken)),
            quoteAmount: 1_950_000,
            kind: 0,
            metadata: ""
        });
    }

    function _executeLoan(
        TokenOrderV2 memory order
    ) internal {
        IBCPacket memory packet = _createPacket();
        vm.prank(address(zkgm));
        vault.solve(packet, order, DEFAULT_PATH, address(0), RELAYER, "", false);
    }

    function _repayLoan() internal {
        _performRepayment(LOAN_AMOUNT);
    }

    function _performRepayment(
        uint256 amount
    ) internal {
        _performRepayment(amount, BASE_TOKEN);
    }

    function _performRepayment(
        uint256 amount,
        bytes memory baseToken
    ) internal {
        quoteToken.mint(REPAYER, amount);
        vm.startPrank(REPAYER);
        quoteToken.approve(address(vault), amount);
        vault.repay(DEFAULT_PATH, DEFAULT_CHANNEL_ID, baseToken, amount);
        vm.stopPrank();
    }

    function _mintAndDeposit(address depositor, uint256 amount) internal {
        quoteToken.mint(depositor, amount);
        vm.startPrank(depositor);
        quoteToken.approve(address(vault), amount);
        vault.deposit(amount, depositor);
        vm.stopPrank();
    }
}
