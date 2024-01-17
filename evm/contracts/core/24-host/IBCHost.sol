pragma solidity ^0.8.23;

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
        string memory name,
        address addr
    ) internal override {
        require(
            capabilities[name] == address(0),
            "IBCHost: capability already claimed"
        );
        capabilities[name] = addr;
    }

    /**
     * @dev authenticateCapability attempts to authenticate a given name from a caller.
     * It allows for a caller to check that a capability does in fact correspond to a particular name.
     */
    function authenticateCapability(
        string memory name
    ) internal view override returns (bool) {
        return _msgSender() == capabilities[name];
    }

    /**
     * @dev lookupModule will return the IBCModule address bound to a given name.
     */
    function lookupModule(
        string memory name
    ) internal view override returns (address) {
        return capabilities[name];
    }
}
