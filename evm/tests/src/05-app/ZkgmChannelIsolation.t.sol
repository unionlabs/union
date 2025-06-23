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

    function test_channel_specific_token_isolation(
        uint32 channelA,
        uint32 channelB,
        uint256 pathA,
        uint256 pathB,
        uint256 amount,
        uint256 decreaseAmount1,
        uint256 decreaseAmount2
    ) public {
        vm.assume(channelA > 0);
        vm.assume(channelB > 0);
        vm.assume(channelA != channelB);

        vm.assume(pathA < type(uint192).max);
        vm.assume(pathB < type(uint192).max);

        vm.assume(amount > 0 && amount < type(uint128).max);
        vm.assume(decreaseAmount1 > 0 && decreaseAmount1 <= amount);
        vm.assume(
            decreaseAmount2 > 0 && decreaseAmount2 <= (amount - decreaseAmount1)
        );

        address baseToken = address(erc20);

        zkgm.doIncreaseOutstanding(channelA, pathA, baseToken, amount);

        zkgm.doIncreaseOutstanding(channelB, pathB, baseToken, amount);

        assertEq(
            zkgm.channelBalance(channelA, pathA, baseToken),
            amount,
            "Channel A balance incorrect"
        );
        assertEq(
            zkgm.channelBalance(channelB, pathB, baseToken),
            amount,
            "Channel B balance incorrect"
        );

        zkgm.doDecreaseOutstanding(channelA, pathA, baseToken, decreaseAmount1);

        assertEq(
            zkgm.channelBalance(channelA, pathA, baseToken),
            amount - decreaseAmount1,
            "Channel A balance not decreased correctly"
        );
        assertEq(
            zkgm.channelBalance(channelB, pathB, baseToken),
            amount,
            "Channel B balance incorrectly affected"
        );

        zkgm.doDecreaseOutstanding(channelB, pathB, baseToken, decreaseAmount1);

        assertEq(
            zkgm.channelBalance(channelA, pathA, baseToken),
            amount - decreaseAmount1,
            "Channel A balance incorrectly affected"
        );
        assertEq(
            zkgm.channelBalance(channelB, pathB, baseToken),
            amount - decreaseAmount1,
            "Channel B balance not decreased correctly"
        );

        zkgm.doDecreaseOutstanding(channelA, pathA, baseToken, decreaseAmount2);

        assertEq(
            zkgm.channelBalance(channelA, pathA, baseToken),
            amount - decreaseAmount1 - decreaseAmount2,
            "Channel A final balance incorrect"
        );
        assertEq(
            zkgm.channelBalance(channelB, pathB, baseToken),
            amount - decreaseAmount1,
            "Channel B final balance incorrect"
        );

        uint256 remainingBalanceA = amount - decreaseAmount1 - decreaseAmount2;
        vm.expectRevert(); // Should revert due to underflow
        zkgm.doDecreaseOutstanding(
            channelA, pathA, baseToken, remainingBalanceA + 1
        );

        zkgm.doDecreaseOutstanding(
            channelB, pathB, baseToken, amount - decreaseAmount1
        );
        assertEq(
            zkgm.channelBalance(channelB, pathB, baseToken),
            0,
            "Channel B balance not fully decreased"
        );

        assertEq(
            zkgm.channelBalance(channelA, pathA, baseToken),
            remainingBalanceA,
            "Channel A balance incorrectly changed"
        );
    }

    function test_channel_specific_token_isolation_v2(
        uint32 channelA,
        uint32 channelB,
        uint256 pathA,
        uint256 pathB,
        bytes32 metadataImage,
        uint256 amount,
        uint256 decreaseAmount1,
        uint256 decreaseAmount2
    ) public {
        vm.assume(channelA > 0);
        vm.assume(channelB > 0);
        vm.assume(channelA != channelB);

        vm.assume(pathA < type(uint192).max);
        vm.assume(pathB < type(uint192).max);

        vm.assume(metadataImage != bytes32(0));

        vm.assume(amount > 0 && amount < type(uint128).max);
        vm.assume(decreaseAmount1 > 0 && decreaseAmount1 <= amount);
        vm.assume(
            decreaseAmount2 > 0 && decreaseAmount2 <= (amount - decreaseAmount1)
        );

        address baseToken = address(erc20);

        zkgm.doIncreaseOutstandingV2(
            channelA, pathA, baseToken, metadataImage, amount
        );

        zkgm.doIncreaseOutstandingV2(
            channelB, pathB, baseToken, metadataImage, amount
        );

        assertEq(
            zkgm.channelBalanceV2(channelA, pathA, baseToken, metadataImage),
            amount,
            "Channel A V2 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(channelB, pathB, baseToken, metadataImage),
            amount,
            "Channel B V2 balance incorrect"
        );

        zkgm.doDecreaseOutstandingV2(
            channelA, pathA, baseToken, metadataImage, decreaseAmount1
        );

        assertEq(
            zkgm.channelBalanceV2(channelA, pathA, baseToken, metadataImage),
            amount - decreaseAmount1,
            "Channel A V2 balance not decreased correctly"
        );
        assertEq(
            zkgm.channelBalanceV2(channelB, pathB, baseToken, metadataImage),
            amount,
            "Channel B V2 balance incorrectly affected"
        );

        zkgm.doDecreaseOutstandingV2(
            channelB, pathB, baseToken, metadataImage, decreaseAmount1
        );

        assertEq(
            zkgm.channelBalanceV2(channelA, pathA, baseToken, metadataImage),
            amount - decreaseAmount1,
            "Channel A V2 balance incorrectly affected"
        );
        assertEq(
            zkgm.channelBalanceV2(channelB, pathB, baseToken, metadataImage),
            amount - decreaseAmount1,
            "Channel B V2 balance not decreased correctly"
        );

        zkgm.doDecreaseOutstandingV2(
            channelA, pathA, baseToken, metadataImage, decreaseAmount2
        );

        assertEq(
            zkgm.channelBalanceV2(channelA, pathA, baseToken, metadataImage),
            amount - decreaseAmount1 - decreaseAmount2,
            "Channel A final V2 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(channelB, pathB, baseToken, metadataImage),
            amount - decreaseAmount1,
            "Channel B final V2 balance incorrect"
        );

        uint256 remainingBalanceA = amount - decreaseAmount1 - decreaseAmount2;
        vm.expectRevert(); // Should revert due to underflow
        zkgm.doDecreaseOutstandingV2(
            channelA, pathA, baseToken, metadataImage, remainingBalanceA + 1
        );

        zkgm.doDecreaseOutstandingV2(
            channelB, pathB, baseToken, metadataImage, amount - decreaseAmount1
        );
        assertEq(
            zkgm.channelBalanceV2(channelB, pathB, baseToken, metadataImage),
            0,
            "Channel B V2 balance not fully decreased"
        );

        assertEq(
            zkgm.channelBalanceV2(channelA, pathA, baseToken, metadataImage),
            remainingBalanceA,
            "Channel A V2 balance incorrectly changed"
        );
    }

    // This test checks for the even more complex case where we use the same token address
    // with different metadata images across different channels
    function test_channel_and_metadata_isolation(
        uint32 channelA,
        uint32 channelB,
        uint256 pathA,
        uint256 pathB,
        bytes32 metadataImage1,
        bytes32 metadataImage2,
        uint256 amount,
        uint256 decreaseAmount
    ) public {
        vm.assume(channelA > 0);
        vm.assume(channelB > 0);
        vm.assume(channelA != channelB);

        vm.assume(pathA < type(uint192).max);
        vm.assume(pathB < type(uint192).max);

        vm.assume(metadataImage1 != bytes32(0));
        vm.assume(metadataImage2 != bytes32(0));
        vm.assume(metadataImage1 != metadataImage2);

        vm.assume(amount > 0 && amount < type(uint128).max);
        vm.assume(decreaseAmount > 0 && decreaseAmount <= amount / 2); // Ensure we can decrease twice

        address baseToken = address(erc20);

        zkgm.doIncreaseOutstandingV2(
            channelA, pathA, baseToken, metadataImage1, amount
        );
        zkgm.doIncreaseOutstandingV2(
            channelA, pathA, baseToken, metadataImage2, amount
        );
        zkgm.doIncreaseOutstandingV2(
            channelB, pathB, baseToken, metadataImage1, amount
        );
        zkgm.doIncreaseOutstandingV2(
            channelB, pathB, baseToken, metadataImage2, amount
        );

        assertEq(
            zkgm.channelBalanceV2(channelA, pathA, baseToken, metadataImage1),
            amount,
            "Channel A metadata1 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(channelA, pathA, baseToken, metadataImage2),
            amount,
            "Channel A metadata2 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(channelB, pathB, baseToken, metadataImage1),
            amount,
            "Channel B metadata1 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(channelB, pathB, baseToken, metadataImage2),
            amount,
            "Channel B metadata2 balance incorrect"
        );

        zkgm.doDecreaseOutstandingV2(
            channelA, pathA, baseToken, metadataImage1, decreaseAmount
        );

        assertEq(
            zkgm.channelBalanceV2(channelA, pathA, baseToken, metadataImage1),
            amount - decreaseAmount,
            "Target balance not decreased correctly"
        );
        assertEq(
            zkgm.channelBalanceV2(channelA, pathA, baseToken, metadataImage2),
            amount,
            "Wrong metadata balance affected"
        );
        assertEq(
            zkgm.channelBalanceV2(channelB, pathB, baseToken, metadataImage1),
            amount,
            "Wrong channel balance affected"
        );
        assertEq(
            zkgm.channelBalanceV2(channelB, pathB, baseToken, metadataImage2),
            amount,
            "Wrong channel and metadata balance affected"
        );

        vm.expectRevert(); // Should revert due to underflow
        zkgm.doDecreaseOutstandingV2(
            channelA, pathA, baseToken, metadataImage1, amount
        );

        zkgm.doDecreaseOutstandingV2(
            channelA, pathA, baseToken, metadataImage2, decreaseAmount
        );
        zkgm.doDecreaseOutstandingV2(
            channelB, pathB, baseToken, metadataImage1, decreaseAmount
        );
        zkgm.doDecreaseOutstandingV2(
            channelB, pathB, baseToken, metadataImage2, decreaseAmount
        );

        assertEq(
            zkgm.channelBalanceV2(channelA, pathA, baseToken, metadataImage1),
            amount - decreaseAmount,
            "Final Channel A metadata1 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(channelA, pathA, baseToken, metadataImage2),
            amount - decreaseAmount,
            "Final Channel A metadata2 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(channelB, pathB, baseToken, metadataImage1),
            amount - decreaseAmount,
            "Final Channel B metadata1 balance incorrect"
        );
        assertEq(
            zkgm.channelBalanceV2(channelB, pathB, baseToken, metadataImage2),
            amount - decreaseAmount,
            "Final Channel B metadata2 balance incorrect"
        );

        zkgm.doDecreaseOutstandingV2(
            channelA, pathA, baseToken, metadataImage1, amount - decreaseAmount
        );
        assertEq(
            zkgm.channelBalanceV2(channelA, pathA, baseToken, metadataImage1),
            0,
            "Failed to zero out first combination"
        );

        assertEq(
            zkgm.channelBalanceV2(channelA, pathA, baseToken, metadataImage2),
            amount - decreaseAmount,
            "Other balances incorrectly changed"
        );
        assertEq(
            zkgm.channelBalanceV2(channelB, pathB, baseToken, metadataImage1),
            amount - decreaseAmount,
            "Other balances incorrectly changed"
        );
        assertEq(
            zkgm.channelBalanceV2(channelB, pathB, baseToken, metadataImage2),
            amount - decreaseAmount,
            "Other balances incorrectly changed"
        );
    }

    // Test extreme cases with very large and very small values
    function test_channel_isolation_edge_cases(
        uint32 channelA,
        uint32 channelB,
        uint256 smallAmount,
        uint256 largeAmount
    ) public {
        vm.assume(channelA > 0);
        vm.assume(channelB > 0);
        vm.assume(channelA != channelB);

        vm.assume(smallAmount > 0 && smallAmount < 10);

        vm.assume(
            largeAmount > type(uint128).max
                && largeAmount < type(uint256).max - 1000
        );

        address baseToken = address(erc20);

        zkgm.doIncreaseOutstanding(channelA, 0, baseToken, smallAmount);
        zkgm.doIncreaseOutstanding(channelB, 0, baseToken, largeAmount);

        assertEq(
            zkgm.channelBalance(channelA, 0, baseToken),
            smallAmount,
            "Channel A small balance incorrect"
        );
        assertEq(
            zkgm.channelBalance(channelB, 0, baseToken),
            largeAmount,
            "Channel B large balance incorrect"
        );

        zkgm.doDecreaseOutstanding(channelA, 0, baseToken, smallAmount);

        assertEq(
            zkgm.channelBalance(channelA, 0, baseToken),
            0,
            "Channel A balance not zeroed"
        );
        assertEq(
            zkgm.channelBalance(channelB, 0, baseToken),
            largeAmount,
            "Channel B balance incorrectly affected"
        );

        zkgm.doDecreaseOutstanding(channelB, 0, baseToken, smallAmount);

        assertEq(
            zkgm.channelBalance(channelB, 0, baseToken),
            largeAmount - smallAmount,
            "Channel B balance not decreased correctly"
        );

        vm.expectRevert();
        zkgm.doDecreaseOutstanding(channelA, 0, baseToken, 1);
    }

    receive() external payable {}
}
