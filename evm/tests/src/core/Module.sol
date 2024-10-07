pragma solidity ^0.8.27;

import "../../../contracts/core/25-handler/IBCHandler.sol";
import "../../../contracts/apps/Base.sol";

library TestModuleLib {
    bytes public constant ACKNOWLEDGEMENT = hex"BEEF";
}

contract TestModule is IBCAppBase {
    IBCHandler private immutable ibcHandler;

    bool ack;
    bytes ackValue;

    constructor(
        IBCHandler ibcHandler_
    ) {
        ibcHandler = ibcHandler_;
        ack = true;
        ackValue = TestModuleLib.ACKNOWLEDGEMENT;
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }

    function pauseAck() public {
        ack = false;
    }

    function resumeAck() public {
        ack = true;
    }

    function setAck(
        bytes memory value
    ) public {
        ackValue = value;
    }

    function onRecvPacket(
        IBCPacket calldata,
        address,
        bytes calldata
    ) external virtual override onlyIBC returns (bytes memory) {
        if (!ack) {
            return hex"";
        } else {
            return ackValue;
        }
    }

    function onRecvIntentPacket(
        IBCPacket calldata,
        address,
        bytes calldata
    ) external virtual override onlyIBC returns (bytes memory) {
        return ackValue;
    }
}
