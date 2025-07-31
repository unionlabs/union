pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "./Zkgm.t.sol";

contract ZkgmChannelTokenIsolationTest is Test {
    using LibString for *;

    TestIBCHandler handler;
    TestERC20 erc20;
    ZkgmERC20 erc20Impl;
    TestWETH weth;
    TestZkgm zkgm;
    Manager manager;

    function setUp() public {
        weth = new TestWETH();
        erc20 = new TestERC20("Test", "T", 18);
        handler = new TestIBCHandler();
        erc20Impl = new ZkgmERC20();
        manager = Manager(
            address(
                new ERC1967Proxy(
                    address(new Manager()),
                    abi.encodeCall(Manager.initialize, (address(this)))
                )
            )
        );
        ERC1967Proxy proxy = new ERC1967Proxy(
            address(new TestZkgm(handler, weth, erc20Impl)),
            abi.encodeCall(UCS03Zkgm.initialize, (address(manager)))
        );
        zkgm = TestZkgm(payable(address(proxy)));
        zkgm.doCreateStakeNFTManager();

        erc20.approve(address(zkgm), type(uint256).max);
    }

    function test_channel_specific_token_isolation_v2(
        uint32 channelA,
        uint32 channelB,
        uint256 pathA,
        uint256 pathB,
        address quoteTokenA,
        address quoteTokenB,
        uint256 amount,
        uint256 decreaseAmount1,
        uint256 decreaseAmount2
    ) public {
        vm.assume(channelA > 0);
        vm.assume(channelB > 0);
        vm.assume(channelA != channelB);

        vm.assume(pathA < type(uint192).max);
        vm.assume(pathB < type(uint192).max);

        vm.assume(quoteTokenA != address(0));
        vm.assume(quoteTokenB != address(0));
        vm.assume(quoteTokenA != quoteTokenB);

        vm.assume(amount > 0 && amount < type(uint128).max);
        vm.assume(decreaseAmount1 > 0 && decreaseAmount1 <= amount);
        vm.assume(
            decreaseAmount2 > 0 && decreaseAmount2 <= (amount - decreaseAmount1)
        );

        address baseToken = address(erc20);

        zkgm.doIncreaseOutstandingV2(
            channelA, pathA, baseToken, abi.encodePacked(quoteTokenA), amount
        );

        zkgm.doIncreaseOutstandingV2(
            channelB, pathB, baseToken, abi.encodePacked(quoteTokenB), amount
        );

        assertEq(
            zkgm.channelBalanceV2(
                channelA, pathA, baseToken, abi.encodePacked(quoteTokenA)
            ),
            amount,
            "Channel A V2 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelB, pathB, baseToken, abi.encodePacked(quoteTokenB)
            ),
            amount,
            "Channel B V2 balance incorrect"
        );

        zkgm.doDecreaseOutstandingV2(
            channelA,
            pathA,
            baseToken,
            abi.encodePacked(quoteTokenA),
            decreaseAmount1
        );

        assertEq(
            zkgm.channelBalanceV2(
                channelA, pathA, baseToken, abi.encodePacked(quoteTokenA)
            ),
            amount - decreaseAmount1,
            "Channel A V2 balance not decreased correctly"
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelB, pathB, baseToken, abi.encodePacked(quoteTokenB)
            ),
            amount,
            "Channel B V2 balance incorrectly affected"
        );

        zkgm.doDecreaseOutstandingV2(
            channelB,
            pathB,
            baseToken,
            abi.encodePacked(quoteTokenB),
            decreaseAmount1
        );

        assertEq(
            zkgm.channelBalanceV2(
                channelA, pathA, baseToken, abi.encodePacked(quoteTokenA)
            ),
            amount - decreaseAmount1,
            "Channel A V2 balance incorrectly affected"
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelB, pathB, baseToken, abi.encodePacked(quoteTokenB)
            ),
            amount - decreaseAmount1,
            "Channel B V2 balance not decreased correctly"
        );

        zkgm.doDecreaseOutstandingV2(
            channelA,
            pathA,
            baseToken,
            abi.encodePacked(quoteTokenA),
            decreaseAmount2
        );

        assertEq(
            zkgm.channelBalanceV2(
                channelA, pathA, baseToken, abi.encodePacked(quoteTokenA)
            ),
            amount - decreaseAmount1 - decreaseAmount2,
            "Channel A final V2 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelB, pathB, baseToken, abi.encodePacked(quoteTokenB)
            ),
            amount - decreaseAmount1,
            "Channel B final V2 balance incorrect"
        );

        uint256 remainingBalanceA = amount - decreaseAmount1 - decreaseAmount2;
        vm.expectRevert(); // Should revert due to underflow
        zkgm.doDecreaseOutstandingV2(
            channelA,
            pathA,
            baseToken,
            abi.encodePacked(quoteTokenA),
            remainingBalanceA + 1
        );

        zkgm.doDecreaseOutstandingV2(
            channelB,
            pathB,
            baseToken,
            abi.encodePacked(quoteTokenB),
            amount - decreaseAmount1
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelB, pathB, baseToken, abi.encodePacked(quoteTokenB)
            ),
            0,
            "Channel B V2 balance not fully decreased"
        );

        assertEq(
            zkgm.channelBalanceV2(
                channelA, pathA, baseToken, abi.encodePacked(quoteTokenA)
            ),
            remainingBalanceA,
            "Channel A V2 balance incorrectly changed"
        );
    }

    // This test checks for the even more complex case where we use the same base token
    // with different quote tokens across different channels
    function test_channel_and_quote_token_isolation(
        uint32 channelA,
        uint32 channelB,
        uint256 pathA,
        uint256 pathB,
        address quoteToken1,
        address quoteToken2,
        uint256 amount,
        uint256 decreaseAmount
    ) public {
        vm.assume(channelA > 0);
        vm.assume(channelB > 0);
        vm.assume(channelA != channelB);

        vm.assume(pathA < type(uint192).max);
        vm.assume(pathB < type(uint192).max);

        vm.assume(quoteToken1 != address(0));
        vm.assume(quoteToken2 != address(0));
        vm.assume(quoteToken1 != quoteToken2);

        vm.assume(amount > 0 && amount < type(uint128).max);
        vm.assume(decreaseAmount > 0 && decreaseAmount <= amount / 2); // Ensure we can decrease twice

        address baseToken = address(erc20);

        zkgm.doIncreaseOutstandingV2(
            channelA, pathA, baseToken, abi.encodePacked(quoteToken1), amount
        );
        zkgm.doIncreaseOutstandingV2(
            channelA, pathA, baseToken, abi.encodePacked(quoteToken2), amount
        );
        zkgm.doIncreaseOutstandingV2(
            channelB, pathB, baseToken, abi.encodePacked(quoteToken1), amount
        );
        zkgm.doIncreaseOutstandingV2(
            channelB, pathB, baseToken, abi.encodePacked(quoteToken2), amount
        );

        assertEq(
            zkgm.channelBalanceV2(
                channelA, pathA, baseToken, abi.encodePacked(quoteToken1)
            ),
            amount,
            "Channel A metadata1 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelA, pathA, baseToken, abi.encodePacked(quoteToken2)
            ),
            amount,
            "Channel A metadata2 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelB, pathB, baseToken, abi.encodePacked(quoteToken1)
            ),
            amount,
            "Channel B metadata1 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelB, pathB, baseToken, abi.encodePacked(quoteToken2)
            ),
            amount,
            "Channel B metadata2 balance incorrect"
        );

        zkgm.doDecreaseOutstandingV2(
            channelA,
            pathA,
            baseToken,
            abi.encodePacked(quoteToken1),
            decreaseAmount
        );

        assertEq(
            zkgm.channelBalanceV2(
                channelA, pathA, baseToken, abi.encodePacked(quoteToken1)
            ),
            amount - decreaseAmount,
            "Target balance not decreased correctly"
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelA, pathA, baseToken, abi.encodePacked(quoteToken2)
            ),
            amount,
            "Wrong metadata balance affected"
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelB, pathB, baseToken, abi.encodePacked(quoteToken1)
            ),
            amount,
            "Wrong channel balance affected"
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelB, pathB, baseToken, abi.encodePacked(quoteToken2)
            ),
            amount,
            "Wrong channel and metadata balance affected"
        );

        vm.expectRevert(); // Should revert due to underflow
        zkgm.doDecreaseOutstandingV2(
            channelA, pathA, baseToken, abi.encodePacked(quoteToken1), amount
        );

        zkgm.doDecreaseOutstandingV2(
            channelA,
            pathA,
            baseToken,
            abi.encodePacked(quoteToken2),
            decreaseAmount
        );
        zkgm.doDecreaseOutstandingV2(
            channelB,
            pathB,
            baseToken,
            abi.encodePacked(quoteToken1),
            decreaseAmount
        );
        zkgm.doDecreaseOutstandingV2(
            channelB,
            pathB,
            baseToken,
            abi.encodePacked(quoteToken2),
            decreaseAmount
        );

        assertEq(
            zkgm.channelBalanceV2(
                channelA, pathA, baseToken, abi.encodePacked(quoteToken1)
            ),
            amount - decreaseAmount,
            "Final Channel A metadata1 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelA, pathA, baseToken, abi.encodePacked(quoteToken2)
            ),
            amount - decreaseAmount,
            "Final Channel A metadata2 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelB, pathB, baseToken, abi.encodePacked(quoteToken1)
            ),
            amount - decreaseAmount,
            "Final Channel B metadata1 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelB, pathB, baseToken, abi.encodePacked(quoteToken2)
            ),
            amount - decreaseAmount,
            "Final Channel B metadata2 balance incorrect"
        );

        zkgm.doDecreaseOutstandingV2(
            channelA,
            pathA,
            baseToken,
            abi.encodePacked(quoteToken1),
            amount - decreaseAmount
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelA, pathA, baseToken, abi.encodePacked(quoteToken1)
            ),
            0,
            "Failed to zero out first combination"
        );

        assertEq(
            zkgm.channelBalanceV2(
                channelA, pathA, baseToken, abi.encodePacked(quoteToken2)
            ),
            amount - decreaseAmount,
            "Other balances incorrectly changed"
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelB, pathB, baseToken, abi.encodePacked(quoteToken1)
            ),
            amount - decreaseAmount,
            "Other balances incorrectly changed"
        );
        assertEq(
            zkgm.channelBalanceV2(
                channelB, pathB, baseToken, abi.encodePacked(quoteToken2)
            ),
            amount - decreaseAmount,
            "Other balances incorrectly changed"
        );
    }

    receive() external payable {}
}
