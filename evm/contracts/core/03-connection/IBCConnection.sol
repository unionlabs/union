pragma solidity ^0.8.27;

import "../24-host/IBCStore.sol";
import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCCommitment.sol";
import "../03-connection/IIBCConnection.sol";

library IBCConnectionLib {
    event ConnectionOpenInit(
        uint32 indexed connectionId,
        uint32 indexed clientId,
        uint32 counterpartyClientId
    );
    event ConnectionOpenTry(
        uint32 indexed connectionId,
        uint32 indexed clientId,
        uint32 counterpartyClientId,
        uint32 counterpartyConnectionId
    );
    event ConnectionOpenAck(
        uint32 indexed connectionId,
        uint32 indexed clientId,
        uint32 counterpartyClientId,
        uint32 counterpartyConnectionId
    );
    event ConnectionOpenConfirm(
        uint32 indexed connectionId,
        uint32 indexed clientId,
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

    function _connectionOpenTry(
        IBCMsgs.MsgConnectionOpenTry calldata msg_
    ) internal returns (uint32) {
        uint32 connectionId = generateConnectionIdentifier();
        IBCConnection storage connection = connections[connectionId];
        connection.clientId = msg_.clientId;
        connection.state = IBCConnectionState.TryOpen;
        connection.counterpartyClientId = msg_.counterpartyClientId;
        connection.counterpartyConnectionId = msg_.counterpartyConnectionId;
        commitConnection(connectionId, connection);
        emit IBCConnectionLib.ConnectionOpenTry(
            connectionId,
            msg_.clientId,
            msg_.counterpartyClientId,
            msg_.counterpartyConnectionId
        );
        return connectionId;
    }

    function forceConnectionOpenTry(
        IBCMsgs.MsgConnectionOpenTry calldata msg_
    ) public restricted returns (uint32) {
        return _connectionOpenTry(msg_);
    }

    /**
     * @dev connectionOpenTry relays notice of a connection attempt on chain A to chain B (this
     * code is executed on chain B).
     */
    function connectionOpenTry(
        IBCMsgs.MsgConnectionOpenTry calldata msg_
    ) external override returns (uint32) {
        IBCConnection memory expectedConnection = IBCConnection({
            state: IBCConnectionState.Init,
            clientId: msg_.counterpartyClientId,
            counterpartyClientId: msg_.clientId,
            counterpartyConnectionId: 0
        });
        if (
            !verifyConnectionState(
                msg_.clientId,
                msg_.proofHeight,
                msg_.proofInit,
                msg_.counterpartyConnectionId,
                expectedConnection
            )
        ) {
            revert IBCErrors.ErrInvalidProof();
        }
        return _connectionOpenTry(msg_);
    }

    function _connectionOpenAck(
        IBCMsgs.MsgConnectionOpenAck calldata msg_,
        IBCConnection storage connection
    ) internal {
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

    function forceConnectionOpenAck(
        IBCMsgs.MsgConnectionOpenAck calldata msg_
    ) public restricted {
        _connectionOpenAck(
            msg_,
            ensureHandshakeState(msg_.connectionId, IBCConnectionState.Init)
        );
    }

    /**
     * @dev connectionOpenAck relays acceptance of a connection open attempt from chain B back
     * to chain A (this code is executed on chain A).
     */
    function connectionOpenAck(
        IBCMsgs.MsgConnectionOpenAck calldata msg_
    ) external override {
        IBCConnection storage connection =
            ensureHandshakeState(msg_.connectionId, IBCConnectionState.Init);
        IBCConnection memory expectedConnection = IBCConnection({
            state: IBCConnectionState.TryOpen,
            clientId: connection.counterpartyClientId,
            counterpartyClientId: connection.clientId,
            counterpartyConnectionId: msg_.connectionId
        });
        if (
            !verifyConnectionState(
                connection.clientId,
                msg_.proofHeight,
                msg_.proofTry,
                msg_.counterpartyConnectionId,
                expectedConnection
            )
        ) {
            revert IBCErrors.ErrInvalidProof();
        }
        _connectionOpenAck(msg_, connection);
    }

    function _connectionOpenConfirm(
        IBCMsgs.MsgConnectionOpenConfirm calldata msg_,
        IBCConnection storage connection
    ) internal {
        connection.state = IBCConnectionState.Open;
        commitConnection(msg_.connectionId, connection);
        emit IBCConnectionLib.ConnectionOpenConfirm(
            msg_.connectionId,
            connection.clientId,
            connection.counterpartyClientId,
            connection.counterpartyConnectionId
        );
    }

    function forceConnectionOpenConfirm(
        IBCMsgs.MsgConnectionOpenConfirm calldata msg_
    ) public restricted {
        _connectionOpenConfirm(
            msg_,
            ensureHandshakeState(msg_.connectionId, IBCConnectionState.TryOpen)
        );
    }

    /**
     * @dev connectionOpenConfirm confirms opening of a connection on chain A to chain B, after
     * which the connection is open on both chains (this code is executed on chain B).
     */
    function connectionOpenConfirm(
        IBCMsgs.MsgConnectionOpenConfirm calldata msg_
    ) external override {
        IBCConnection storage connection =
            ensureHandshakeState(msg_.connectionId, IBCConnectionState.TryOpen);
        IBCConnection memory expectedConnection = IBCConnection({
            state: IBCConnectionState.Open,
            clientId: connection.counterpartyClientId,
            counterpartyClientId: connection.clientId,
            counterpartyConnectionId: msg_.connectionId
        });
        if (
            !verifyConnectionState(
                connection.clientId,
                msg_.proofHeight,
                msg_.proofAck,
                connection.counterpartyConnectionId,
                expectedConnection
            )
        ) {
            revert IBCErrors.ErrInvalidProof();
        }
        _connectionOpenConfirm(msg_, connection);
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
        uint32 clientId,
        uint64 height,
        bytes calldata proof,
        uint32 connectionId,
        IBCConnection memory counterpartyConnection
    ) internal returns (bool) {
        return getClientInternal(clientId).verifyMembership(
            clientId,
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

    function ensureHandshakeState(
        uint32 connectionId,
        IBCConnectionState state
    ) internal view returns (IBCConnection storage) {
        IBCConnection storage connection = connections[connectionId];
        if (connection.state != state) {
            revert IBCErrors.ErrInvalidConnectionState();
        }
        return connection;
    }
}
