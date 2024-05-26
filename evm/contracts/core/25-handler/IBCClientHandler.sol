pragma solidity ^0.8.23;

import "../24-host/IBCHost.sol";
import "../02-client/IIBCClient.sol";

/**
 * @dev IBCClientHandler is a contract that calls a contract that implements `IIBCClient` with delegatecall.
 */
abstract contract IBCClientHandler is IIBCClient {
    address ibcClient;

    /**
     * @dev registerClient registers a new client type into the client registry
     */
    function registerClient(string calldata, ILightClient) public virtual {
        passthrough(ibcClient);
    }

    /**
     * @dev createClient creates a new client state and populates it with a given consensus state
     */
    function createClient(IBCMsgs.MsgCreateClient calldata)
        external
        override
        returns (string memory)
    {
        passthrough(ibcClient);
    }

    /**
     * @dev updateClient updates the consensus state and the state root from a provided header
     */
    function updateClient(IBCMsgs.MsgUpdateClient calldata) external override {
        passthrough(ibcClient);
    }
}
