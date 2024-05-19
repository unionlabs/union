pragma solidity ^0.8.23;

import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCHost.sol";
import "../03-connection/IIBCConnection.sol";

/**
 * @dev IBCConnectionHandler is a contract that calls a contract that implements `IIBCConnectionHandshake` with delegatecall.
 */
abstract contract IBCConnectionHandler is IIBCConnectionHandshake {
    address ibcConnection;

    function connectionOpenInit(IBCMsgs.MsgConnectionOpenInit calldata)
        external
        override
        returns (string memory)
    {
        passthrough(ibcConnection);
    }

    function connectionOpenTry(IBCMsgs.MsgConnectionOpenTry calldata)
        external
        override
        returns (string memory)
    {
        passthrough(ibcConnection);
    }

    function connectionOpenAck(IBCMsgs.MsgConnectionOpenAck calldata)
        external
        override
    {
        passthrough(ibcConnection);
    }

    function connectionOpenConfirm(IBCMsgs.MsgConnectionOpenConfirm calldata)
        external
        override
    {
        passthrough(ibcConnection);
    }
}
