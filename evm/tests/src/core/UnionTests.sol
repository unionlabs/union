pragma solidity ^0.8.27;

import "forge-std/Test.sol";

import "@openzeppelin/contracts/proxy/ERC1967/ERC1967Proxy.sol";

import "../../../contracts/Manager.sol";

import "./IBCHandler.sol";

abstract contract UnionTests is Test {
    function setupHandler() internal returns (Manager, TestIBCHandler) {
        Manager manager = Manager(
            address(
                new ERC1967Proxy(
                    address(new Manager()),
                    abi.encodeCall(Manager.initialize, (address(this)))
                )
            )
        );
        TestIBCHandler handler = TestIBCHandler(
            address(
                new ERC1967Proxy(
                    address(new TestIBCHandler()),
                    abi.encodeCall(IBCHandler.initialize, (address(manager)))
                )
            )
        );
        manager.setTargetSingleFunctionRole(
            address(handler), IBCClient.registerClient.selector, Roles.RELAYER
        );
        manager.setTargetSingleFunctionRole(
            address(handler), IBCClient.createClient.selector, Roles.RELAYER
        );
        manager.setTargetSingleFunctionRole(
            address(handler), IBCClient.updateClient.selector, Roles.RELAYER
        );
        manager.setTargetSingleFunctionRole(
            address(handler), IBCClient.misbehaviour.selector, Roles.RELAYER
        );
        manager.grantRole(Roles.RELAYER, address(this), 0);
        return (manager, handler);
    }
}
