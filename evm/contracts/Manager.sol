pragma solidity ^0.8.27;

import
    "@openzeppelin-upgradeable/contracts/access/manager/AccessManagerUpgradeable.sol";

contract Manager is AccessManagerUpgradeable {
    uint64 constant RELAYER = 1;
    uint64 constant PAUSER = 2;
    uint64 constant RATE_LIMITER = 3;

    constructor() {
        _disableInitializers();
    }

    function initialize(
        address admin
    ) public override initializer {
        __AccessManager_init(admin);
    }
}
