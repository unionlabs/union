pragma solidity ^0.8.27;

import "../25-handler/IBCMsgs.sol";

interface IIBCConnection {
    /* Handshake functions */

    /**
     * @dev connectionOpenInit initialises a connection attempt on chain A. The generated connection identifier
     * is returned.
     */
    function connectionOpenInit(
        IBCMsgs.MsgConnectionOpenInit calldata msg_
    ) external returns (uint32);

    /**
     * @dev connectionOpenTry relays notice of a connection attempt on chain A to chain B (this
     * code is executed on chain B).
     */
    function connectionOpenTry(
        IBCMsgs.MsgConnectionOpenTry calldata msg_
    ) external returns (uint32);

    /**
     * @dev connectionOpenAck relays acceptance of a connection open attempt from chain B back
     * to chain A (this code is executed on chain A).
     */
    function connectionOpenAck(
        IBCMsgs.MsgConnectionOpenAck calldata msg_
    ) external;

    /**
     * @dev connectionOpenConfirm confirms opening of a connection on chain A to chain B, after
     * which the connection is open on both chains (this code is executed on chain B).
     */
    function connectionOpenConfirm(
        IBCMsgs.MsgConnectionOpenConfirm calldata msg_
    ) external;
}
