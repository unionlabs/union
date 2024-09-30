pragma solidity ^0.8.23;

import "../../../contracts/core/25-handler/IBCHandler.sol";
import "../../../contracts/apps/Base.sol";

contract TestModule is IBCAppBase {
    IBCHandler private immutable ibcHandler;

    constructor(
        IBCHandler ibcHandler_
    ) {
        ibcHandler = ibcHandler_;
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }
}
