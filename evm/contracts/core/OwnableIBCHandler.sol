pragma solidity ^0.8.27;

import "./25-handler/IBCHandler.sol";

/**
 * @dev OwnableIBCHandler is a contract that implements [ICS-25](https://github.com/cosmos/ibc/tree/main/spec/core/ics-025-handler-interface).
 */
contract OwnableIBCHandler is IBCHandler {
    constructor() {
        _disableInitializers();
    }

    function initialize(
        address admin
    ) public override initializer {
        IBCHandler.initialize(admin);
    }
}
