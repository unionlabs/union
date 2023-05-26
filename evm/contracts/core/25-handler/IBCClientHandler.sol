pragma solidity ^0.8.18;

import "../24-host/IBCHost.sol";
import "../02-client/IIBCClient.sol";

/**
 * @dev IBCClientHandler is a contract that calls a contract that implements `IIBCClient` with delegatecall.
 */
abstract contract IBCClientHandler {
    // IBC Client contract address
    address immutable ibcClientAddress;

    event GeneratedClientIdentifier(string);

    constructor(address ibcClient) {
        ibcClientAddress = ibcClient;
    }

    /**
     * @dev registerClient registers a new client type into the client registry
     */
    function registerClient(string calldata clientType, ILightClient client) public virtual {
        (bool success, bytes memory res) = ibcClientAddress.delegatecall(
            abi.encodeWithSelector(IIBCClient.registerClient.selector, clientType, client)
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
    }

    /**
     * @dev createClient creates a new client state and populates it with a given consensus state
     */
    function createClient(IBCMsgs.MsgCreateClient calldata msg_) external returns (string memory clientId) {
        (bool success, bytes memory res) =
            ibcClientAddress.delegatecall(abi.encodeWithSelector(IIBCClient.createClient.selector, msg_));
        if (!success) {
            revert(_getRevertMsg(res));
        }
        clientId = abi.decode(res, (string));
        emit GeneratedClientIdentifier(clientId);
        return clientId;
    }

    /**
     * @dev updateClient updates the consensus state and the state root from a provided header
     */
    function updateClient(IBCMsgs.MsgUpdateClient calldata msg_) external {
        (bool success, bytes memory res) = ibcClientAddress.delegatecall(abi.encodeWithSelector(IIBCClient.updateClient.selector, msg_));
        if (!success) {
            revert(_getRevertMsg(res));
        }
    }
}
