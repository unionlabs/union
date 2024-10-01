pragma solidity ^0.8.27;

import "../../../contracts/core/25-handler/IBCHandler.sol";
import "../../../contracts/apps/Base.sol";

library TestModuleLib {
    bytes public constant ACKNOWLEDGEMENT = hex"BEEF";
}

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

    function onRecvPacket(
        IBCPacket calldata,
        address,
        bytes calldata
    ) external virtual override onlyIBC returns (bytes memory) {
        return TestModuleLib.ACKNOWLEDGEMENT;
    }

    function onRecvIntentPacket(
        IBCPacket calldata,
        address,
        bytes calldata
    ) external virtual override onlyIBC returns (bytes memory) {
        return TestModuleLib.ACKNOWLEDGEMENT;
    }
}
