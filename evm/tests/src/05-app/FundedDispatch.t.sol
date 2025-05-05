pragma solidity ^0.8.27;

import "forge-std/Test.sol";

import "solady/utils/LibBytes.sol";
import "solady/utils/LibString.sol";

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/interfaces/draft-IERC6093.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/Address.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagedUpgradeable.sol";

import "../../../contracts/apps/ucs/03-zkgm/Lib.sol";
import "../../../contracts/apps/ucs/06-funded-dispatch/FundedDispatch.sol";

contract TestERC20 is ERC20 {
    uint8 _decimals;

    constructor(
        string memory name,
        string memory symbol,
        uint8 d
    ) ERC20(name, symbol) {
        _decimals = d;
    }

    function decimals() public view override returns (uint8) {
        return _decimals;
    }

    function mint(address to, uint256 amount) public {
        _mint(to, amount);
    }
}

contract TestTarget {
    using SafeERC20 for *;
    using Address for *;

    event Executed(address token, uint256 amount);

    error Eureka();

    function execute(address token, uint256 amount) public {
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);

        emit Executed(token, amount);
    }

    function executeNative(
        uint256 amount
    ) public payable {
        if (msg.value != amount) {
            revert Eureka();
        }

        emit Executed(ZkgmLib.NATIVE_ETH_ERC_7528_ADDRESS, amount);
    }

    function explode(address token, uint256 amount) public {
        IERC20(token).safeTransferFrom(msg.sender, address(this), amount);

        revert Eureka();
    }

    function explodeNative(
        uint256 amount
    ) public payable {
        if (msg.value != amount) {
            revert("impossible");
        }

        revert Eureka();
    }

    receive() external payable {}
}

contract FundedDispatchTests is Test {
    using SafeERC20 for *;

    UCS06FundedDispatch fundedDispatch;
    TestERC20 erc20;
    TestTarget target;

    function setUp() public {
        target = new TestTarget();
        erc20 = new TestERC20("Test", "TT", 18);
        fundedDispatch = UCS06FundedDispatch(
            address(
                new ERC1967Proxy(
                    address(new UCS06FundedDispatch()),
                    abi.encodeCall(
                        UCS06FundedDispatch.initialize, (address(this))
                    )
                )
            )
        );
        vm.deal(address(this), 0);
    }

    function test_execute_noMarketMaker(
        address sender,
        FundedDispatchFund[] calldata funds,
        address contractAddress,
        bytes calldata contractCalldata
    ) public {
        vm.expectRevert(FundedDispatchLib.FundedDispatch_NoMarketMaker.selector);
        vm.prank(sender);
        fundedDispatch.execute(
            FundedDispatchLib.encode(
                FundedDispatchParameters({
                    flags: FundedDispatchLib.FLAG_ALLOW_FAILURE,
                    funds: funds,
                    contractAddress: abi.encodePacked(contractAddress),
                    contractCalldata: contractCalldata,
                    beneficiary: abi.encodePacked(address(this))
                })
            ),
            true
        );
    }

    function test_execute_insufficientBalance(
        address sender,
        uint256 amount,
        bool intent
    ) public {
        vm.assume(sender != address(0));
        vm.assume(amount > 0);

        erc20.mint(sender, amount - 1);
        FundedDispatchFund[] memory funds = new FundedDispatchFund[](1);
        funds[0] = FundedDispatchFund({
            token: abi.encodePacked(address(erc20)),
            amount: amount
        });

        vm.expectRevert(
            abi.encodeWithSelector(
                IERC20Errors.ERC20InsufficientBalance.selector,
                address(fundedDispatch),
                0,
                amount
            )
        );
        vm.prank(sender);
        fundedDispatch.execute(
            FundedDispatchLib.encode(
                FundedDispatchParameters({
                    flags: FundedDispatchLib.FLAG_ALLOW_MARKET_MAKER,
                    funds: funds,
                    contractAddress: abi.encodePacked(address(target)),
                    contractCalldata: abi.encodeCall(
                        TestTarget.execute, (address(erc20), amount)
                    ),
                    beneficiary: abi.encodePacked(address(this))
                })
            ),
            intent
        );
    }

    function test_execute_disallowFailure_bubbleUpRevert(
        address sender,
        uint256 amount,
        bool intent
    ) public {
        vm.assume(sender != address(0));
        vm.assume(amount > 0);

        erc20.mint(sender, amount);
        vm.prank(sender);
        erc20.transfer(address(fundedDispatch), amount);
        FundedDispatchFund[] memory funds = new FundedDispatchFund[](1);
        funds[0] = FundedDispatchFund({
            token: abi.encodePacked(address(erc20)),
            amount: amount
        });

        vm.expectRevert(TestTarget.Eureka.selector);
        vm.prank(sender);
        fundedDispatch.execute(
            FundedDispatchLib.encode(
                FundedDispatchParameters({
                    flags: FundedDispatchLib.FLAG_ALLOW_MARKET_MAKER,
                    funds: funds,
                    contractAddress: abi.encodePacked(address(target)),
                    contractCalldata: abi.encodeCall(
                        TestTarget.explode, (address(erc20), amount)
                    ),
                    beneficiary: abi.encodePacked(address(this))
                })
            ),
            intent
        );
    }

    function test_execute_allowFailure_forwardToBeneficiary(
        address sender,
        uint256 amount,
        bool intent
    ) public {
        vm.assume(sender != address(this));
        vm.assume(sender != address(0));
        vm.assume(amount > 0);

        erc20.mint(sender, amount);
        vm.prank(sender);
        erc20.transfer(address(fundedDispatch), amount);
        FundedDispatchFund[] memory funds = new FundedDispatchFund[](1);
        funds[0] = FundedDispatchFund({
            token: abi.encodePacked(address(erc20)),
            amount: amount
        });

        vm.prank(sender);
        fundedDispatch.execute(
            FundedDispatchLib.encode(
                FundedDispatchParameters({
                    flags: FundedDispatchLib.FLAG_ALLOW_FAILURE
                        | FundedDispatchLib.FLAG_ALLOW_MARKET_MAKER,
                    funds: funds,
                    contractAddress: abi.encodePacked(address(target)),
                    contractCalldata: abi.encodeCall(
                        TestTarget.explode, (address(erc20), amount)
                    ),
                    beneficiary: abi.encodePacked(address(this))
                })
            ),
            intent
        );

        assertEq(erc20.balanceOf(sender), 0);
        assertEq(erc20.balanceOf(address(fundedDispatch)), 0);
        assertEq(erc20.balanceOf(address(this)), amount);
    }

    function test_executeNative_allowFailure_forwardToBeneficiary(
        address sender,
        uint256 amount,
        bool intent
    ) public {
        vm.assume(sender != address(target));
        vm.assume(sender != address(0));
        vm.assume(amount > 0);

        uint256 targetBalanceBefore = address(target).balance;
        uint256 fundedDispatchBalanceBefore = address(fundedDispatch).balance;
        uint256 beneficiaryBalanceBefore = address(this).balance;

        vm.deal(address(fundedDispatch), fundedDispatchBalanceBefore + amount);

        FundedDispatchFund[] memory funds = new FundedDispatchFund[](1);
        funds[0] = FundedDispatchFund({
            token: abi.encodePacked(ZkgmLib.NATIVE_ETH_ERC_7528_ADDRESS),
            amount: amount
        });

        vm.prank(sender);
        fundedDispatch.execute(
            FundedDispatchLib.encode(
                FundedDispatchParameters({
                    flags: FundedDispatchLib.FLAG_ALLOW_FAILURE
                        | FundedDispatchLib.FLAG_ALLOW_MARKET_MAKER,
                    funds: funds,
                    contractAddress: abi.encodePacked(address(target)),
                    contractCalldata: abi.encodeCall(
                        TestTarget.explodeNative, (amount)
                    ),
                    beneficiary: abi.encodePacked(address(this))
                })
            ),
            intent
        );

        assertEq(address(fundedDispatch).balance, fundedDispatchBalanceBefore);
        assertEq(address(target).balance, targetBalanceBefore);
        assertEq(address(this).balance, beneficiaryBalanceBefore + amount);
    }

    function test_execute_forwardToContract(
        address sender,
        uint256 amount,
        bool intent
    ) public {
        vm.assume(sender != address(target));
        vm.assume(sender != address(0));
        vm.assume(amount > 0);

        erc20.mint(sender, amount);
        vm.prank(sender);
        erc20.transfer(address(fundedDispatch), amount);
        FundedDispatchFund[] memory funds = new FundedDispatchFund[](1);
        funds[0] = FundedDispatchFund({
            token: abi.encodePacked(address(erc20)),
            amount: amount
        });

        vm.prank(sender);
        fundedDispatch.execute(
            FundedDispatchLib.encode(
                FundedDispatchParameters({
                    flags: FundedDispatchLib.FLAG_ALLOW_MARKET_MAKER,
                    funds: funds,
                    contractAddress: abi.encodePacked(address(target)),
                    contractCalldata: abi.encodeCall(
                        TestTarget.execute, (address(erc20), amount)
                    ),
                    beneficiary: abi.encodePacked(address(this))
                })
            ),
            false
        );

        assertEq(erc20.balanceOf(sender), 0);
        assertEq(erc20.balanceOf(address(fundedDispatch)), 0);
        assertEq(erc20.balanceOf(address(target)), amount);
    }

    function test_executeNative_forwardToContract(
        address sender,
        uint256 amount,
        bool intent
    ) public {
        vm.assume(sender != address(0));
        vm.assume(amount > 0);

        uint256 targetBalanceBefore = address(target).balance;
        uint256 fundedDispatchBalanceBefore = address(fundedDispatch).balance;

        vm.deal(address(fundedDispatch), fundedDispatchBalanceBefore + amount);
        FundedDispatchFund[] memory funds = new FundedDispatchFund[](1);
        funds[0] = FundedDispatchFund({
            token: abi.encodePacked(ZkgmLib.NATIVE_ETH_ERC_7528_ADDRESS),
            amount: amount
        });

        vm.prank(sender);
        fundedDispatch.execute(
            FundedDispatchLib.encode(
                FundedDispatchParameters({
                    flags: FundedDispatchLib.FLAG_ALLOW_MARKET_MAKER,
                    funds: funds,
                    contractAddress: abi.encodePacked(address(target)),
                    contractCalldata: abi.encodeCall(
                        TestTarget.executeNative, (amount)
                    ),
                    beneficiary: abi.encodePacked(address(this))
                })
            ),
            false
        );

        assertEq(address(fundedDispatch).balance, fundedDispatchBalanceBefore);
        assertEq(address(target).balance, targetBalanceBefore + amount);
    }

    receive() external payable {}
}
