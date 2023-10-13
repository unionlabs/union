pragma solidity ^0.8.21;

import "@openzeppelin/contracts/utils/Context.sol";
import "../../proto/ibc/core/client/v1/client.sol";
import "../02-client/ILightClient.sol";
import "../24-host/IBCStore.sol";
import "../05-port/ModuleManager.sol";

function _getRevertMsg(bytes memory _returnData) pure returns (string memory) {
    // If the _res length is less than 68, then the transaction failed silently (without a revert message)
    if (_returnData.length < 68) return "Transaction reverted silently";
    assembly {
        // Slice the sighash.
        _returnData := add(_returnData, 0x04)
    }
    return abi.decode(_returnData, (string)); // All that remains is the revert string
}

abstract contract IBCHost is IBCStore, Context, ModuleManager {
    /**
     * @dev claimCapability allows the IBC app module to claim a capability that core IBC passes to it
     */
    function claimCapability(
        bytes memory name,
        address addr
    ) internal override {
        for (uint32 i = 0; i < capabilities[name].length; i++) {
            require(capabilities[name][i] != addr);
        }
        capabilities[name].push(addr);
    }

    /**
     * @dev authenticateCapability attempts to authenticate a given name from a caller.
     * It allows for a caller to check that a capability does in fact correspond to a particular name.
     */
    function authenticateCapability(
        bytes memory name
    ) internal view override returns (bool) {
        address caller = _msgSender();
        for (uint32 i = 0; i < capabilities[name].length; i++) {
            if (capabilities[name][i] == caller) {
                return true;
            }
        }
        return false;
    }

    /**
     * @dev lookupModules will return the IBCModule addresses bound to a given name.
     */
    function lookupModules(
        bytes memory name
    ) internal view override returns (address[] storage, bool) {
        return (capabilities[name], capabilities[name].length > 0);
    }

    /**
     * @dev setExpectedTimePerBlock sets expected time per block.
     */
    function setExpectedTimePerBlock(
        uint64 expectedTimePerBlock_
    ) public virtual {
        expectedTimePerBlock = expectedTimePerBlock_;
    }
}
