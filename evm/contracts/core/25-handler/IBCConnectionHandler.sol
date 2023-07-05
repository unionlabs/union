pragma solidity ^0.8.18;

import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCHost.sol";
import "../03-connection/IIBCConnection.sol";

/**
 * @dev IBCConnectionHandler is a contract that calls a contract that implements `IIBCConnectionHandshake` with delegatecall.
 */
abstract contract IBCConnectionHandler {
    // IBC Connection contract address
    address immutable ibcConnectionAddress;

    event GeneratedConnectionIdentifier(string);

    constructor(address ibcConnection) {
        ibcConnectionAddress = ibcConnection;
    }

    function connectionOpenInit(
        IBCMsgs.MsgConnectionOpenInit calldata msg_
    ) external returns (string memory connectionId) {
        (bool success, bytes memory res) = ibcConnectionAddress.delegatecall(
            abi.encodeWithSelector(
                IIBCConnectionHandshake.connectionOpenInit.selector,
                msg_
            )
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
        connectionId = abi.decode(res, (string));
        emit GeneratedConnectionIdentifier(connectionId);
        return connectionId;
    }

    function connectionOpenTry(
        IBCMsgs.MsgConnectionOpenTry calldata msg_
    ) external returns (string memory connectionId) {
        (bool success, bytes memory res) = ibcConnectionAddress.delegatecall(
            abi.encodeWithSelector(
                IIBCConnectionHandshake.connectionOpenTry.selector,
                msg_
            )
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
        connectionId = abi.decode(res, (string));
        emit GeneratedConnectionIdentifier(connectionId);
        return connectionId;
    }

    function connectionOpenAck(
        IBCMsgs.MsgConnectionOpenAck calldata msg_
    ) external {
        (bool success, bytes memory res) = ibcConnectionAddress.delegatecall(
            abi.encodeWithSelector(
                IIBCConnectionHandshake.connectionOpenAck.selector,
                msg_
            )
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
    }

    function connectionOpenConfirm(
        IBCMsgs.MsgConnectionOpenConfirm calldata msg_
    ) external {
        (bool success, bytes memory res) = ibcConnectionAddress.delegatecall(
            abi.encodeWithSelector(
                IIBCConnectionHandshake.connectionOpenConfirm.selector,
                msg_
            )
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
    }
}
