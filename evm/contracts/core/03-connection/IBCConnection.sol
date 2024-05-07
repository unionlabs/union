pragma solidity ^0.8.23;

import "solady/utils/LibString.sol";
import "@openzeppelin/utils/Strings.sol";
import "../../proto/ibc/core/client/v1/client.sol";
import "../../proto/ibc/core/connection/v1/connection.sol";
import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCStore.sol";
import "../24-host/IBCCommitment.sol";
import "../03-connection/IIBCConnection.sol";

library IBCConnectionLib {
    event ConnectionOpenInit(string connectionId);
    event ConnectionOpenTry(string connectionId);
    event ConnectionOpenAck(string connectionId);
    event ConnectionOpenConfirm(string connectionId);

    error ErrConnectionAlreadyExists();
    error ErrValidateSelfClient();
    error ErrNoCounterpartyVersion();
    error ErrUnsupportedVersion();
    error ErrInvalidProof();
    error ErrInvalidConnectionState();
}

/**
 * @dev IBCConnection is a contract that implements [ICS-3](https://github.com/cosmos/ibc/tree/main/spec/core/ics-003-connection-semantics).
 */
contract IBCConnection is IBCStore, IIBCConnectionHandshake {
    using LibString for *;

    /* Handshake functions */

    /**
     * @dev connectionOpenInit initialises a connection attempt on chain A. The generated connection identifier
     * is returned.
     */
    function connectionOpenInit(IBCMsgs.MsgConnectionOpenInit calldata msg_)
        external
        override
        returns (string memory)
    {
        string memory connectionId = generateConnectionIdentifier();
        IbcCoreConnectionV1ConnectionEnd.Data storage connection =
            connections[connectionId];
        if (
            connection.state
                != IbcCoreConnectionV1GlobalEnums
                    .State
                    .STATE_UNINITIALIZED_UNSPECIFIED
        ) {
            revert IBCConnectionLib.ErrConnectionAlreadyExists();
        }
        connection.client_id = msg_.clientId;
        setSupportedVersions(connection.versions);
        connection.state = IbcCoreConnectionV1GlobalEnums.State.STATE_INIT;
        connection.delay_period = msg_.delayPeriod;
        connection.counterparty = msg_.counterparty;
        updateConnectionCommitment(connectionId);

        emit IBCConnectionLib.ConnectionOpenInit(connectionId);

        return connectionId;
    }

    /**
     * @dev connectionOpenTry relays notice of a connection attempt on chain A to chain B (this
     * code is executed on chain B).
     */
    function connectionOpenTry(IBCMsgs.MsgConnectionOpenTry calldata msg_)
        external
        override
        returns (string memory)
    {
        if (!validateSelfClient(msg_.clientStateBytes)) {
            revert IBCConnectionLib.ErrValidateSelfClient();
        }
        if (msg_.counterpartyVersions.length == 0) {
            revert IBCConnectionLib.ErrNoCounterpartyVersion();
        }
        if (!isSupportedVersion(msg_.counterpartyVersions[0])) {
            revert IBCConnectionLib.ErrUnsupportedVersion();
        }

        string memory connectionId = generateConnectionIdentifier();
        IbcCoreConnectionV1ConnectionEnd.Data storage connection =
            connections[connectionId];
        if (
            connection.state
                != IbcCoreConnectionV1GlobalEnums
                    .State
                    .STATE_UNINITIALIZED_UNSPECIFIED
        ) {
            revert IBCConnectionLib.ErrConnectionAlreadyExists();
        }
        connection.client_id = msg_.clientId;
        setSupportedVersions(connection.versions);
        connection.state = IbcCoreConnectionV1GlobalEnums.State.STATE_TRYOPEN;
        connection.delay_period = msg_.delayPeriod;
        connection.counterparty = msg_.counterparty;

        IbcCoreConnectionV1ConnectionEnd.Data memory expectedConnection =
        IbcCoreConnectionV1ConnectionEnd.Data({
            client_id: msg_.counterparty.client_id,
            versions: msg_.counterpartyVersions,
            state: IbcCoreConnectionV1GlobalEnums.State.STATE_INIT,
            delay_period: msg_.delayPeriod,
            counterparty: IbcCoreConnectionV1Counterparty.Data({
                client_id: msg_.clientId,
                connection_id: "",
                prefix: IbcCoreCommitmentV1MerklePrefix.Data({
                    key_prefix: bytes(COMMITMENT_PREFIX)
                })
            })
        });

        if (
            !verifyConnectionState(
                connection,
                msg_.proofHeight,
                msg_.proofInit,
                msg_.counterparty.connection_id,
                expectedConnection
            )
        ) {
            revert IBCConnectionLib.ErrInvalidProof();
        }
        if (
            !verifyClientState(
                connection,
                msg_.proofHeight,
                IBCCommitment.clientStatePath(connection.counterparty.client_id),
                msg_.proofClient,
                msg_.clientStateBytes
            )
        ) {
            revert IBCConnectionLib.ErrInvalidProof();
        }

        updateConnectionCommitment(connectionId);

        emit IBCConnectionLib.ConnectionOpenTry(connectionId);

        return connectionId;
    }

    /**
     * @dev connectionOpenAck relays acceptance of a connection open attempt from chain B back
     * to chain A (this code is executed on chain A).
     */
    function connectionOpenAck(IBCMsgs.MsgConnectionOpenAck calldata msg_)
        external
        override
    {
        IbcCoreConnectionV1ConnectionEnd.Data storage connection =
            connections[msg_.connectionId];
        if (connection.state != IbcCoreConnectionV1GlobalEnums.State.STATE_INIT)
        {
            revert IBCConnectionLib.ErrInvalidConnectionState();
        }

        if (!isSupportedVersion(msg_.version)) {
            revert IBCConnectionLib.ErrUnsupportedVersion();
        }
        if (!validateSelfClient(msg_.clientStateBytes)) {
            revert IBCConnectionLib.ErrValidateSelfClient();
        }

        IbcCoreConnectionV1Counterparty.Data memory expectedCounterparty =
        IbcCoreConnectionV1Counterparty.Data({
            client_id: connection.client_id,
            connection_id: msg_.connectionId,
            prefix: IbcCoreCommitmentV1MerklePrefix.Data({
                key_prefix: bytes(COMMITMENT_PREFIX)
            })
        });

        IbcCoreConnectionV1ConnectionEnd.Data memory expectedConnection =
        IbcCoreConnectionV1ConnectionEnd.Data({
            client_id: connection.counterparty.client_id,
            versions: makeVersionArray(msg_.version),
            state: IbcCoreConnectionV1GlobalEnums.State.STATE_TRYOPEN,
            delay_period: connection.delay_period,
            counterparty: expectedCounterparty
        });

        if (
            !verifyConnectionState(
                connection,
                msg_.proofHeight,
                msg_.proofTry,
                msg_.counterpartyConnectionID,
                expectedConnection
            )
        ) {
            revert IBCConnectionLib.ErrInvalidProof();
        }
        if (
            !verifyClientState(
                connection,
                msg_.proofHeight,
                IBCCommitment.clientStatePath(connection.counterparty.client_id),
                msg_.proofClient,
                msg_.clientStateBytes
            )
        ) {
            revert IBCConnectionLib.ErrInvalidProof();
        }

        connection.state = IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN;
        copyVersions(expectedConnection.versions, connection.versions);
        connection.counterparty.connection_id = msg_.counterpartyConnectionID;
        updateConnectionCommitment(msg_.connectionId);

        emit IBCConnectionLib.ConnectionOpenAck(msg_.connectionId);
    }

    /**
     * @dev connectionOpenConfirm confirms opening of a connection on chain A to chain B, after
     * which the connection is open on both chains (this code is executed on chain B).
     */
    function connectionOpenConfirm(
        IBCMsgs.MsgConnectionOpenConfirm calldata msg_
    ) external override {
        IbcCoreConnectionV1ConnectionEnd.Data storage connection =
            connections[msg_.connectionId];
        if (
            connection.state
                != IbcCoreConnectionV1GlobalEnums.State.STATE_TRYOPEN
        ) {
            revert IBCConnectionLib.ErrInvalidConnectionState();
        }

        IbcCoreConnectionV1Counterparty.Data memory expectedCounterparty =
        IbcCoreConnectionV1Counterparty.Data({
            client_id: connection.client_id,
            connection_id: msg_.connectionId,
            prefix: IbcCoreCommitmentV1MerklePrefix.Data({
                key_prefix: bytes(COMMITMENT_PREFIX)
            })
        });

        IbcCoreConnectionV1ConnectionEnd.Data memory expectedConnection =
        IbcCoreConnectionV1ConnectionEnd.Data({
            client_id: connection.counterparty.client_id,
            versions: connection.versions,
            state: IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN,
            delay_period: connection.delay_period,
            counterparty: expectedCounterparty
        });

        if (
            !verifyConnectionState(
                connection,
                msg_.proofHeight,
                msg_.proofAck,
                connection.counterparty.connection_id,
                expectedConnection
            )
        ) {
            revert IBCConnectionLib.ErrInvalidProof();
        }

        connection.state = IbcCoreConnectionV1GlobalEnums.State.STATE_OPEN;
        updateConnectionCommitment(msg_.connectionId);

        emit IBCConnectionLib.ConnectionOpenConfirm(msg_.connectionId);
    }

    function updateConnectionCommitment(string memory connectionId) private {
        commitments[IBCCommitment.connectionCommitmentKey(connectionId)] =
        keccak256(
            IbcCoreConnectionV1ConnectionEnd.encode(connections[connectionId])
        );
    }

    /* Verification functions */

    function verifyClientState(
        IbcCoreConnectionV1ConnectionEnd.Data storage connection,
        IbcCoreClientV1Height.Data memory height,
        bytes memory path,
        bytes memory proof,
        bytes memory clientStateBytes
    ) private returns (bool) {
        return getClient(connection.client_id).verifyMembership(
            connection.client_id,
            height,
            0,
            0,
            proof,
            connection.counterparty.prefix.key_prefix,
            path,
            clientStateBytes
        );
    }

    function verifyConnectionState(
        IbcCoreConnectionV1ConnectionEnd.Data storage connection,
        IbcCoreClientV1Height.Data memory height,
        bytes memory proof,
        string memory connectionId,
        IbcCoreConnectionV1ConnectionEnd.Data memory counterpartyConnection
    ) private returns (bool) {
        return getClient(connection.client_id).verifyMembership(
            connection.client_id,
            height,
            0,
            0,
            proof,
            connection.counterparty.prefix.key_prefix,
            IBCCommitment.connectionPath(connectionId),
            IbcCoreConnectionV1ConnectionEnd.encode(counterpartyConnection)
        );
    }

    /* Internal functions */

    function generateConnectionIdentifier() private returns (string memory) {
        string memory identifier = string(
            abi.encodePacked(
                "connection-", Strings.toString(nextConnectionSequence)
            )
        );
        nextConnectionSequence++;
        return identifier;
    }

    /**
     * @dev validateSelfClient validates the client parameters for a client of the host chain.
     *
     * NOTE: Developers can override this function to support an arbitrary EVM chain.
     */
    function validateSelfClient(bytes memory)
        internal
        view
        virtual
        returns (bool)
    {
        return true;
    }

    /**
     * @dev setSupportedVersions sets the supported versions to a given array.
     *
     * NOTE: `versions` must be an empty array
     */
    function setSupportedVersions(
        IbcCoreConnectionV1Version.Data[] storage versions
    ) internal {
        require(
            versions.length == 0, "setSupportedVersions: versions must be empty"
        );
        versions.push(
            IbcCoreConnectionV1Version.Data({
                identifier: "1",
                features: new string[](2)
            })
        );
        IbcCoreConnectionV1Version.Data storage version = versions[0];
        version.features[0] = "ORDER_ORDERED";
        version.features[1] = "ORDER_UNORDERED";
    }

    function isSupportedVersion(IbcCoreConnectionV1Version.Data memory version)
        internal
        pure
        returns (bool)
    {
        IbcCoreConnectionV1Version.Data memory expectedVersion =
        IbcCoreConnectionV1Version.Data({
            identifier: "1",
            features: new string[](2)
        });
        expectedVersion.features[0] = "ORDER_ORDERED";
        expectedVersion.features[1] = "ORDER_UNORDERED";
        return isEqualVersion(version, expectedVersion);
    }

    function isEqualVersion(
        IbcCoreConnectionV1Version.Data memory a,
        IbcCoreConnectionV1Version.Data memory b
    ) internal pure returns (bool) {
        return keccak256(IbcCoreConnectionV1Version.encode(a))
            == keccak256(IbcCoreConnectionV1Version.encode(b));
    }

    function makeVersionArray(IbcCoreConnectionV1Version.Data memory version)
        internal
        pure
        returns (IbcCoreConnectionV1Version.Data[] memory ret)
    {
        ret = new IbcCoreConnectionV1Version.Data[](1);
        ret[0] = version;
    }

    function copyVersions(
        IbcCoreConnectionV1Version.Data[] memory src,
        IbcCoreConnectionV1Version.Data[] storage dst
    ) internal {
        for (uint256 i = 0; i < src.length; i++) {
            copyVersion(src[i], dst[i]);
        }
    }

    function copyVersion(
        IbcCoreConnectionV1Version.Data memory src,
        IbcCoreConnectionV1Version.Data storage dst
    ) internal {
        dst.identifier = src.identifier;
        for (uint256 i = 0; i < src.features.length; i++) {
            dst.features[i] = src.features[i];
        }
    }
}
