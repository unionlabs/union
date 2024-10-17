pragma solidity ^0.8.27;

import "../24-host/IBCStore.sol";
import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCCommitment.sol";
import "../03-connection/IIBCConnection.sol";

library IBCConnectionLib {
    event ConnectionOpenInit(
        uint32 connectionId, uint32 clientId, uint32 counterpartyClientId
    );
    event ConnectionOpenTry(
        uint32 connectionId,
        uint32 clientId,
        uint32 counterpartyClientId,
        uint32 counterpartyConnectionId
    );
    event ConnectionOpenAck(
        uint32 connectionId,
        uint32 clientId,
        uint32 counterpartyClientId,
        uint32 counterpartyConnectionId
    );
    event ConnectionOpenConfirm(
        uint32 connectionId,
        uint32 clientId,
        uint32 counterpartyClientId,
        uint32 counterpartyConnectionId
    );
}

/**
 * @dev IBCConnection is a contract that implements [ICS-3](https://github.com/cosmos/ibc/tree/main/spec/core/ics-003-connection-semantics).
 */
abstract contract IBCConnectionImpl is IBCStore, IIBCConnection {
    /**
     * @dev connectionOpenInit initialises a connection attempt on chain A. The generated connection identifier
     * is returned.
     */
    function connectionOpenInit(
        IBCMsgs.MsgConnectionOpenInit calldata msg_
    ) external override returns (uint32) {
        uint32 connectionId = generateConnectionIdentifier();
        IBCConnection storage connection = connections[connectionId];
        connection.clientId = msg_.clientId;
        connection.state = IBCConnectionState.Init;
        connection.counterpartyClientId = msg_.counterpartyClientId;
        commitConnection(connectionId, connection);
        emit IBCConnectionLib.ConnectionOpenInit(
            connectionId, msg_.clientId, msg_.counterpartyClientId
        );
        return connectionId;
    }

    /**
     * @dev connectionOpenTry relays notice of a connection attempt on chain A to chain B (this
     * code is executed on chain B).
     */
    function connectionOpenTry(
        IBCMsgs.MsgConnectionOpenTry calldata msg_
    ) external override returns (uint32) {
        uint32 connectionId = generateConnectionIdentifier();
        IBCConnection storage connection = connections[connectionId];
        connection.clientId = msg_.clientId;
        connection.state = IBCConnectionState.TryOpen;
        connection.counterpartyClientId = msg_.counterpartyClientId;
        connection.counterpartyConnectionId = msg_.counterpartyConnectionId;
        IBCConnection memory expectedConnection = IBCConnection({
            state: IBCConnectionState.Init,
            clientId: msg_.counterpartyClientId,
            counterpartyClientId: msg_.clientId,
            counterpartyConnectionId: 0
        });
        if (
            !verifyConnectionState(
                connection,
                msg_.proofHeight,
                msg_.proofInit,
                msg_.counterpartyConnectionId,
                expectedConnection
            )
        ) {
            revert IBCErrors.ErrInvalidProof();
        }
        commitConnection(connectionId, connection);
        emit IBCConnectionLib.ConnectionOpenTry(
            connectionId,
            msg_.clientId,
            msg_.counterpartyClientId,
            msg_.counterpartyConnectionId
        );
        return connectionId;
    }

    /**
     * @dev connectionOpenAck relays acceptance of a connection open attempt from chain B back
     * to chain A (this code is executed on chain A).
     */
    function connectionOpenAck(
        IBCMsgs.MsgConnectionOpenAck calldata msg_
    ) external override {
        IBCConnection storage connection = connections[msg_.connectionId];
        if (connection.state != IBCConnectionState.Init) {
            revert IBCErrors.ErrInvalidConnectionState();
        }
        IBCConnection memory expectedConnection = IBCConnection({
            state: IBCConnectionState.TryOpen,
            clientId: connection.counterpartyClientId,
            counterpartyClientId: connection.clientId,
            counterpartyConnectionId: msg_.connectionId
        });
        if (
            !verifyConnectionState(
                connection,
                msg_.proofHeight,
                msg_.proofTry,
                msg_.counterpartyConnectionId,
                expectedConnection
            )
        ) {
            revert IBCErrors.ErrInvalidProof();
        }
        connection.state = IBCConnectionState.Open;
        connection.counterpartyConnectionId = msg_.counterpartyConnectionId;
        commitConnection(msg_.connectionId, connection);
        emit IBCConnectionLib.ConnectionOpenAck(
            msg_.connectionId,
            connection.clientId,
            connection.counterpartyClientId,
            connection.counterpartyConnectionId
        );
    }

    /**
     * @dev connectionOpenConfirm confirms opening of a connection on chain A to chain B, after
     * which the connection is open on both chains (this code is executed on chain B).
     */
    function connectionOpenConfirm(
        IBCMsgs.MsgConnectionOpenConfirm calldata msg_
    ) external override {
        IBCConnection storage connection = connections[msg_.connectionId];
        if (connection.state != IBCConnectionState.TryOpen) {
            revert IBCErrors.ErrInvalidConnectionState();
        }
        IBCConnection memory expectedConnection = IBCConnection({
            state: IBCConnectionState.Open,
            clientId: connection.counterpartyClientId,
            counterpartyClientId: connection.clientId,
            counterpartyConnectionId: msg_.connectionId
        });
        if (
            !verifyConnectionState(
                connection,
                msg_.proofHeight,
                msg_.proofAck,
                connection.counterpartyConnectionId,
                expectedConnection
            )
        ) {
            revert IBCErrors.ErrInvalidProof();
        }
        connection.state = IBCConnectionState.Open;
        commitConnection(msg_.connectionId, connection);
        emit IBCConnectionLib.ConnectionOpenConfirm(
            msg_.connectionId,
            connection.clientId,
            connection.counterpartyClientId,
            connection.counterpartyConnectionId
        );
    }

    function encodeConnection(
        IBCConnection memory connection
    ) internal pure returns (bytes32) {
        return keccak256(abi.encode(connection));
    }

    function encodeConnectionStorage(
        IBCConnection storage connection
    ) internal pure returns (bytes32) {
        return keccak256(abi.encode(connection));
    }

    function commitConnection(
        uint32 connectionId,
        IBCConnection storage connection
    ) internal {
        commitments[IBCCommitment.connectionCommitmentKey(connectionId)] =
            encodeConnectionStorage(connection);
    }

    function verifyConnectionState(
        IBCConnection storage connection,
        uint64 height,
        bytes calldata proof,
        uint32 connectionId,
        IBCConnection memory counterpartyConnection
    ) internal returns (bool) {
        return getClientInternal(connection.clientId).verifyMembership(
            connection.clientId,
            height,
            proof,
            abi.encodePacked(
                IBCCommitment.connectionCommitmentKey(connectionId)
            ),
            abi.encodePacked(encodeConnection(counterpartyConnection))
        );
    }

    function generateConnectionIdentifier() internal returns (uint32) {
        uint32 nextConnectionSequence =
            uint32(uint256(commitments[nextConnectionSequencePath]));
        commitments[nextConnectionSequencePath] =
            bytes32(uint256(nextConnectionSequence + 1));
        return nextConnectionSequence;
    }
}
