pragma solidity ^0.8.23;

import "@openzeppelin/utils/Context.sol";
import "./IIBCModule.sol";
import "../24-host/IBCStore.sol";
import "../../lib/Hex.sol";
import {IBCChannelLib} from "../04-channel/IBCChannelHandshake.sol";

library ModuleManagerLib {
    error ErrModuleNotFound();
    error ErrCapabilityAlreadyClaimed();
}

/**
 * @dev ModuleManager is an abstract contract that provides the functions defined in [ICS 5](https://github.com/cosmos/ibc/tree/main/spec/core/ics-005-port-allocation) and [ICS 26](https://github.com/cosmos/ibc/blob/main/spec/core/ics-005-port-module/README.md).
 */
abstract contract ModuleManager is IBCStore, Context {
    /**
     * @dev lookupModuleByPort will return the IBCModule along with the capability associated with a given portID
     */
    function lookupModuleByPort(string memory portId)
        internal
        view
        virtual
        returns (IIBCModule)
    {
        return IIBCModule(Hex.hexToAddress(portId));
    }

    /**
     * @dev lookupModuleByChannel will return the IBCModule along with the capability associated with a given channel defined by its portID and channelID
     */
    // REVIEW: Remove in favor of lookupModuleByPort?
    function lookupModuleByChannel(ChannelId channelId)
        internal
        view
        virtual
        returns (IIBCModule)
    {
        address module = lookupModule(channelId);
        if (module == address(0)) {
            revert ModuleManagerLib.ErrModuleNotFound();
        }
        return IIBCModule(module);
    }

    // /**
    //  * @dev channelCapabilityPath returns the path under which module address associated with a port and channel should be stored.
    //  */
    // function channelCapabilityPath(
    //     string memory portId,
    //     ChannelId channelId
    // ) public pure returns (string memory) {
    //     return string.concat(portId, "/", channelId);
    // }

    /**
     * @dev claimCapability allows the IBC app module to claim a capability that core IBC passes to it
     */
    function claimCapability(ChannelId channelId, address addr) internal {
        if (capabilities[channelId] != address(0)) {
            revert ModuleManagerLib.ErrCapabilityAlreadyClaimed();
        }
        capabilities[channelId] = addr;
    }

    /**
     * @dev authenticateCapability attempts to authenticate a given name from a caller.
     * It allows for a caller to check that a capability does in fact correspond to a particular name.
     */
    function authenticateCapability(ChannelId channelId)
        internal
        view
        returns (bool)
    {
        return _msgSender() == capabilities[channelId];
    }

    /**
     * @dev lookupModule will return the IBCModule address bound to a given name.
     */
    function lookupModule(ChannelId channelId)
        internal
        view
        returns (address)
    {
        return capabilities[channelId];
    }
}
