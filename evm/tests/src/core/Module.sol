pragma solidity ^0.8.23;

import "../../../contracts/apps/Base.sol";

contract TestModule is IBCAppBase {
    address private immutable ibcHandler;

    constructor(
        address ibcHandler_
    ) {
        ibcHandler = ibcHandler_;
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }
}
