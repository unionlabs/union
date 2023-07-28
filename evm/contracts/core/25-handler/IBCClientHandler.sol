pragma solidity ^0.8.18;

import "../24-host/IBCHost.sol";
import "../02-client/IIBCClient.sol";

/**
 * @dev IBCClientHandler is a contract that calls a contract that implements `IIBCClient` with delegatecall.
 */
abstract contract IBCClientHandler {
    // IBC Client contract address
    address immutable ibcClientAddress;

    constructor(address ibcClient) {
        ibcClientAddress = ibcClient;
    }

    /**
     * @dev registerClient registers a new client type into the client registry
     */
    function registerClient(string calldata clientType, ILightClient client) public virtual {
        passthrough();
    }

    /**
     * @dev createClient creates a new client state and populates it with a given consensus state
     */
    function createClient(IBCMsgs.MsgCreateClient calldata msg_) external returns (string memory clientId) {
        passthrough();
    }

    /**
     * @dev updateClient updates the consensus state and the state root from a provided header
     */
    function updateClient(IBCMsgs.MsgUpdateClient calldata msg_) external {
        passthrough();
    }

    function passthrough() public {
        address dest = ibcClientAddress;
        assembly {
            // copy function selector and any arguments
            calldatacopy(0, 0, calldatasize())
            // execute function call using the facet
            let result := delegatecall(gas(), dest, 0, calldatasize(), 0, 0)
            // get any return value
            returndatacopy(0, 0, returndatasize())
            // return any return value or error back to the caller
            switch result
            case 0 { revert(0, returndatasize()) }
            default { return(0, returndatasize()) }
        }
    }
}
