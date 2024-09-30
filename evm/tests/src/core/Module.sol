pragma solidity ^0.8.23;

import "../../../contracts/apps/Base.sol";

contract TestModule is IBCAppBase {
    address private immutable ibcHandler;

    constructor(address _ibcHandler) {
        ibcHandler = _ibcHandler;
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }
}
