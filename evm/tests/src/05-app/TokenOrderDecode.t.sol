pragma solidity ^0.8.27;

import "forge-std/Test.sol";
import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";
import "../../../contracts/apps/ucs/03-zkgm/TokenOrder.sol";
import "../../../contracts/apps/ucs/03-zkgm/ZkgmERC20.sol";
import "../../../contracts/apps/ucs/03-zkgm/IWETH.sol";
import "../../../contracts/core/05-port/IIBCModule.sol";

contract TokenOrderDecodeTest is Test {
    UCS03ZkgmTokenOrderImpl public tokenOrder;
    ZkgmERC20 public erc20Impl;

    function setUp() public {
        erc20Impl = new ZkgmERC20();
        tokenOrder =
            new UCS03ZkgmTokenOrderImpl(IWETH(address(0)), erc20Impl, false);
    }

    function testDecodeZkgmERC20InitializeCall_invalidSelector(
        bytes4 selector,
        bytes calldata args
    ) public {
        vm.assume(selector != ZkgmERC20.initialize.selector);
        vm.expectRevert();
        tokenOrder.decodeZkgmERC20InitializeCall(
            abi.encodePacked(selector, args)
        );
    }

    function testDecodeZkgmERC20InitializeCall_ok(
        address fuzzAuthority,
        address fuzzMinter,
        string memory fuzzName,
        string memory fuzzSymbol,
        uint8 fuzzDecimals
    ) public view {
        bytes memory initializeCall = abi.encodeCall(
            ZkgmERC20.initialize,
            (fuzzAuthority, fuzzMinter, fuzzName, fuzzSymbol, fuzzDecimals)
        );

        (
            address decodedAuthority,
            address decodedMinter,
            string memory decodedName,
            string memory decodedSymbol,
            uint8 decodedDecimals
        ) = tokenOrder.decodeZkgmERC20InitializeCall(initializeCall);

        assertEq(decodedAuthority, fuzzAuthority);
        assertEq(decodedMinter, fuzzMinter);
        assertEq(decodedName, fuzzName);
        assertEq(decodedSymbol, fuzzSymbol);
        assertEq(decodedDecimals, fuzzDecimals);
    }
}
