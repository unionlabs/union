// SPDX-License-Identifier: UNLICENSED
pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "../../../contracts/apps/ucs/01-relay/ERC20Denom.sol";

contract ERC20DenomTests is Test {
    ERC20Denom token;

    event smth(string tokname);

    address admin = address(0xABCD);
    address user = address(0x1234);

    function setUp() public {
        vm.startPrank(admin);
        token = new ERC20Denom("DenomToken");
        emit smth(token.name());
        vm.stopPrank();
    }

    function test_initialization() public {
        emit smth(token.name());
        assertEq(token.name(), "DenomToken");
        assertEq(token.symbol(), "DenomToken");
        assertEq(token.decimals(), 18); // Default value since `_decimals` is never updated
        assertEq(token.admin(), admin);
    }

    function test_mint_ok() public {
        uint256 mintAmount = 1000 ether;

        vm.startPrank(admin);
        token.mint(user, mintAmount);
        vm.stopPrank();

        assertEq(token.balanceOf(user), mintAmount);
    }

    function test_mint_revert_nonAdmin() public {
        uint256 mintAmount = 1000 ether;

        vm.startPrank(user);
        vm.expectRevert(ERC20Denom.ERC20Unauthorized.selector);
        token.mint(user, mintAmount);
        vm.stopPrank();
    }

    function test_burn_ok() public {
        uint256 mintAmount = 1000 ether;
        uint256 burnAmount = 500 ether;

        vm.startPrank(admin);
        token.mint(user, mintAmount);
        token.burn(user, burnAmount);
        vm.stopPrank();

        assertEq(token.balanceOf(user), mintAmount - burnAmount);
    }

    function test_burn_revert_nonAdmin() public {
        uint256 mintAmount = 1000 ether;
        uint256 burnAmount = 500 ether;

        vm.startPrank(admin);
        token.mint(user, mintAmount);
        vm.stopPrank();

        vm.startPrank(user);
        vm.expectRevert(ERC20Denom.ERC20Unauthorized.selector);
        token.burn(user, burnAmount);
        vm.stopPrank();
    }

    function test_update_ok() public {
        string memory newName = "UpdatedName";
        string memory newSymbol = "UPD";
        uint8 newDecimals = 6;

        vm.startPrank(admin);
        token.update(newName, newSymbol, newDecimals);
        vm.stopPrank();

        assertEq(token.name(), newName);
        assertEq(token.symbol(), newSymbol);
        assertEq(token.decimals(), newDecimals);
    }



    function test_update_revert_nonAdmin() public {
        string memory newName = "UpdatedName";
        string memory newSymbol = "UPD";
        uint8 newDecimals = 6;

        vm.startPrank(user);
        vm.expectRevert(ERC20Denom.ERC20Unauthorized.selector);
        token.update(newName, newSymbol, newDecimals);
        vm.stopPrank();
    }

}
