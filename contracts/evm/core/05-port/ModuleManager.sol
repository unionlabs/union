// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "./IIBCModule.sol";

/**
 * @dev ModuleManager is an abstract contract that provides the functions defined in [ICS 5](https://github.com/cosmos/ibc/tree/main/spec/core/ics-005-port-allocation) and [ICS 26](https://github.com/cosmos/ibc/blob/main/spec/core/ics-005-port-module/README.md).
 */
abstract contract ModuleManager {
    /**
     * @dev bindPort binds to an unallocated port, failing if the port has already been allocated.
     */
    function bindPort(string calldata portId, address moduleAddress) public virtual {
        claimCapability(portCapabilityPath(portId), moduleAddress);
    }

    /**
     * @dev lookupModuleByPort will return the IBCModule along with the capability associated with a given portID
     */
    function lookupModuleByPort(string memory portId) internal view virtual returns (IIBCModule) {
        (address[] storage modules, bool found) = lookupModules(portCapabilityPath(portId));
        require(found);
        return IIBCModule(modules[0]);
    }

    /**
     * @dev lookupModuleByChannel will return the IBCModule along with the capability associated with a given channel defined by its portID and channelID
     */
    function lookupModuleByChannel(string memory portId, string memory channelId)
        internal
        view
        virtual
        returns (IIBCModule)
    {
        (address[] storage modules, bool found) = lookupModules(channelCapabilityPath(portId, channelId));
        require(found);
        return IIBCModule(modules[0]);
    }

    /**
     * @dev portCapabilityPath returns the path under which owner module address associated with a port should be stored.
     */
    function portCapabilityPath(string memory portId) public pure returns (bytes memory) {
        return abi.encodePacked(portId);
    }

    /**
     * @dev channelCapabilityPath returns the path under which module address associated with a port and channel should be stored.
     */
    function channelCapabilityPath(string memory portId, string memory channelId) public pure returns (bytes memory) {
        return abi.encodePacked(portId, "/", channelId);
    }

    /**
     * @dev claimCapability allows the IBC app module to claim a capability that core IBC passes to it
     */
    function claimCapability(bytes memory name, address addr) internal virtual;

    /**
     * @dev authenticateCapability attempts to authenticate a given name from a caller.
     * It allows for a caller to check that a capability does in fact correspond to a particular name.
     */
    function authenticateCapability(bytes memory name) internal view virtual returns (bool);

    /**
     * @dev lookupModule will return the IBCModule address bound to a given name.
     * Currently, the function returns only one module.
     */
    function lookupModules(bytes memory name) internal view virtual returns (address[] storage, bool);
}
