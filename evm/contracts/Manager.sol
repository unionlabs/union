pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/contracts/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/contracts/proxy/utils/UUPSUpgradeable.sol";
import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagerUpgradeable.sol";

library Roles {
    uint64 public constant RELAYER = 1;
    uint64 public constant PAUSER = 2;
    uint64 public constant UNPAUSER = 2;
    uint64 public constant RATE_LIMITER = 3;
}

contract Manager is Initializable, UUPSUpgradeable, AccessManagerUpgradeable {
    constructor() {
        _disableInitializers();
    }

    function initialize(
        address admin
    ) public override initializer {
        __AccessManager_init(admin);
    }

    function setTargetSingleFunctionRole(
        address target,
        bytes4 selector,
        uint64 roleId
    ) public onlyAuthorized {
        _setTargetFunctionRole(target, selector, roleId);
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyAuthorized {}
}
