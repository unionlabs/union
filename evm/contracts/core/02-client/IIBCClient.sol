pragma solidity ^0.8.23;

import "./ILightClient.sol";
import "../25-handler/IBCMsgs.sol";

interface IIBCClient {
    /**
     * @dev registerClient registers a new client type into the client registry
     */
    function registerClient(
        string calldata clientType,
        ILightClient client
    ) external;

    /**
     * @dev createClient creates a new client state and populates it with a given consensus state
     */
    function createClient(IBCMsgs.MsgCreateClient calldata msg_)
        external
        returns (string memory clientId);

    /**
     * @dev updateClient updates the consensus state and the state root from a provided header
     */
    function updateClient(IBCMsgs.MsgUpdateClient calldata msg_) external;
}
