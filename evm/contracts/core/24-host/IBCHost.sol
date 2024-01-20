pragma solidity ^0.8.23;

import "@openzeppelin/contracts/utils/Context.sol";
import "../../proto/ibc/core/client/v1/client.sol";
import "../02-client/ILightClient.sol";
import "../24-host/IBCStore.sol";
import "../05-port/ModuleManager.sol";

function passthrough(address impl) {
    assembly {
        // copy function selector and any arguments
        calldatacopy(0, 0, calldatasize())
        // execute function call using the facet
        let result := delegatecall(gas(), impl, 0, calldatasize(), 0, 0)
        // get any return value
        returndatacopy(0, 0, returndatasize())
        // return any return value or error back to the caller
        switch result
        case 0 {
            revert(0, returndatasize())
        }
        default {
            return(0, returndatasize())
        }
    }
}

abstract contract IBCHost is ModuleManager {}
