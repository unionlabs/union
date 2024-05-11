pragma solidity ^0.8.23;

import {IBCHandler} from "../../../contracts/core/25-handler/IBCHandler.sol";

import "../../../contracts/proto/ibc/core/client/v1/client.sol";
import "../../../contracts/core/02-client/ILightClient.sol";
import "../../../contracts/core/24-host/IBCStore.sol";
import "../../../contracts/core/05-port/ModuleManager.sol";
import "../../../contracts/core/24-host/IBCCommitment.sol";

contract IBCHandler_Testable is IBCHandler {
    function getConnection(string calldata connectionId)
        external
        view
        returns (IbcCoreConnectionV1ConnectionEnd.Data memory, bool)
    {
        IbcCoreConnectionV1ConnectionEnd.Data storage connection =
            connections[connectionId];
        return (
            connection,
            connection.state
                != IbcCoreConnectionV1GlobalEnums
                    .State
                    .STATE_UNINITIALIZED_UNSPECIFIED
        );
    }
}
