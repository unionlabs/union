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
    event ConnectionOpenInit(
        string connectionId, string clientId, string counterpartyClientId
    );
    event ConnectionOpenTry(
        string connectionId, string clientId, string counterpartyClientId
    );
    event ConnectionOpenAck(
        string connectionId, string clientId, string counterpartyClientId
    );
    event ConnectionOpenConfirm(
        string connectionId, string clientId, string counterpartyClientId
    );

    error ErrConnectionAlreadyExists();
    error ErrValidateSelfClient();
    error ErrNoCounterpartyVersion();
    error ErrUnsupportedVersion();
    error ErrVersionMustBeUnset();
    error ErrInvalidProof();
    error ErrInvalidConnectionState();

    // yes, these are all defined as strings in the ibc spec
    string internal constant IBC_VERSION_IDENTIFIER = "1";
    string internal constant ORDER_ORDERED = "ORDER_ORDERED";
    string internal constant ORDER_UNORDERED = "ORDER_UNORDERED";

    /**
     * @dev defaultIBCVersion returns the latest supported version of IBC used in connection version negotiation
     */
    function defaultIBCVersion()
        internal
        pure
        returns (IbcCoreConnectionV1Version.Data memory)
    {
        IbcCoreConnectionV1Version.Data memory version =
        IbcCoreConnectionV1Version.Data({
            identifier: IBC_VERSION_IDENTIFIER,
            features: new string[](2)
        });
        version.features[0] = ORDER_ORDERED;
        version.features[1] = ORDER_UNORDERED;
        return version;
    }

    /**
     * @dev setSupportedVersions sets the supported versions to a given array.
     *
     * NOTE: `dst` must be an empty array
     */
    function setSupportedVersions(
        IbcCoreConnectionV1Version.Data[] memory supportedVersions,
        IbcCoreConnectionV1Version.Data[] storage dst
    ) internal {
        if (dst.length != 0) {
            revert ErrVersionMustBeUnset();
        }
        for (uint256 i = 0; i < supportedVersions.length; i++) {
            dst.push(supportedVersions[i]);
        }
    }

    /**
     * @dev isSupportedVersion returns true if the proposed version has a matching version
     * identifier and its entire feature set is supported or the version identifier
     * supports an empty feature set.
     */
    function isSupportedVersion(
        IbcCoreConnectionV1Version.Data[] memory supportedVersions,
        IbcCoreConnectionV1Version.Data memory version
    ) internal pure returns (bool) {
        (IbcCoreConnectionV1Version.Data memory supportedVersion, bool found) =
            findSupportedVersion(version, supportedVersions);
        if (!found) {
            return false;
        }
        return verifyProposedVersion(supportedVersion, version);
    }

    function isSupported(
        IbcCoreConnectionV1Version.Data[] storage supportedVersions,
        string memory feature
    ) internal view returns (bool) {
        for (uint256 i = 0; i < supportedVersions.length; i++) {
            if (verifySupportedFeature(supportedVersions[i], feature)) {
                return true;
            }
        }
        return false;
    }

    /**
     * @dev verifyProposedVersion verifies that the entire feature set in the
     * proposed version is supported by this chain. If the feature set is
     * empty it verifies that this is allowed for the specified version
     * identifier.
     */
    function verifyProposedVersion(
        IbcCoreConnectionV1Version.Data memory supportedVersion,
        IbcCoreConnectionV1Version.Data memory proposedVersion
    ) internal pure returns (bool) {
        if (
            keccak256(abi.encodePacked(proposedVersion.identifier))
                != keccak256(abi.encodePacked(supportedVersion.identifier))
        ) {
            return false;
        }
        if (proposedVersion.features.length == 0) {
            return false;
        }
        for (uint256 i = 0; i < proposedVersion.features.length; i++) {
            if (
                !contains(proposedVersion.features[i], supportedVersion.features)
            ) {
                return false;
            }
        }
        return true;
    }

    /**
     * @dev findSupportedVersion returns the version with a matching version identifier
     * if it exists. The returned boolean is true if the version is found and
     * false otherwise.
     */
    function findSupportedVersion(
        IbcCoreConnectionV1Version.Data memory version,
        IbcCoreConnectionV1Version.Data[] memory supportedVersions
    )
        internal
        pure
        returns (
            IbcCoreConnectionV1Version.Data memory supportedVersion,
            bool found
        )
    {
        for (uint256 i = 0; i < supportedVersions.length; i++) {
            supportedVersion = supportedVersions[i];
            if (
                keccak256(abi.encodePacked(supportedVersion.identifier))
                    == keccak256(abi.encodePacked(version.identifier))
            ) {
                return (supportedVersion, true);
            }
        }
        return (supportedVersion, false);
    }

    function pickVersion(
        IbcCoreConnectionV1Version.Data[] memory supportedVersions,
        IbcCoreConnectionV1Version.Data[] memory counterpartyVersions
    ) internal pure returns (IbcCoreConnectionV1Version.Data memory) {
        for (uint256 i = 0; i < supportedVersions.length; i++) {
            IbcCoreConnectionV1Version.Data memory supportedVersion =
                supportedVersions[i];
            (
                IbcCoreConnectionV1Version.Data memory counterpartyVersion,
                bool found
            ) = findSupportedVersion(supportedVersion, counterpartyVersions);
            if (!found) {
                continue;
            }
            string[] memory featureSet = getFeatureSetIntersection(
                supportedVersion.features, counterpartyVersion.features
            );
            if (featureSet.length > 0) {
                return IbcCoreConnectionV1Version.Data({
                    identifier: supportedVersion.identifier,
                    features: featureSet
                });
            }
        }
        revert ErrUnsupportedVersion();
    }

    /**
     * @dev copyVersions copies `src` to `dst`
     */
    function copyVersions(
        IbcCoreConnectionV1Version.Data[] memory src,
        IbcCoreConnectionV1Version.Data[] storage dst
    ) internal {
        uint256 srcLength = src.length;
        uint256 dstLength = dst.length;
        if (srcLength == dstLength) {
            for (uint256 i = 0; i < srcLength; i++) {
                copyVersion(src[i], dst[i]);
            }
        } else if (srcLength > dstLength) {
            for (uint256 i = 0; i < dstLength; i++) {
                copyVersion(src[i], dst[i]);
            }
            for (uint256 i = dstLength; i < srcLength; i++) {
                dst.push(src[i]);
            }
        } else {
            for (uint256 i = 0; i < srcLength; i++) {
                copyVersion(src[i], dst[i]);
            }
            for (uint256 i = srcLength; i < dstLength; i++) {
                dst.pop();
            }
        }
    }

    /**
     * @dev newVersions returns a new array with a given version
     */
    function newVersions(IbcCoreConnectionV1Version.Data memory version)
        internal
        pure
        returns (IbcCoreConnectionV1Version.Data[] memory ret)
    {
        ret = new IbcCoreConnectionV1Version.Data[](1);
        ret[0] = version;
    }

    /**
     * @dev verifySupportedFeature takes in a version and feature string and returns
     * true if the feature is supported by the version and false otherwise.
     */
    function verifySupportedFeature(
        IbcCoreConnectionV1Version.Data memory version,
        string memory feature
    ) internal pure returns (bool) {
        bytes32 hashedFeature = keccak256(bytes(feature));
        for (uint256 i = 0; i < version.features.length; i++) {
            if (keccak256(bytes(version.features[i])) == hashedFeature) {
                return true;
            }
        }
        return false;
    }

    function getFeatureSetIntersection(
        string[] memory sourceFeatureSet,
        string[] memory counterpartyFeatureSet
    ) private pure returns (string[] memory) {
        string[] memory featureSet = new string[](sourceFeatureSet.length);
        uint256 featureSetLength = 0;
        for (uint256 i = 0; i < sourceFeatureSet.length; i++) {
            if (contains(sourceFeatureSet[i], counterpartyFeatureSet)) {
                featureSet[featureSetLength] = sourceFeatureSet[i];
                featureSetLength++;
            }
        }
        string[] memory ret = new string[](featureSetLength);
        for (uint256 i = 0; i < featureSetLength; i++) {
            ret[i] = featureSet[i];
        }
        return ret;
    }

    function copyVersion(
        IbcCoreConnectionV1Version.Data memory src,
        IbcCoreConnectionV1Version.Data storage dst
    ) private {
        dst.identifier = src.identifier;
        uint256 srcLength = src.features.length;
        uint256 dstLength = dst.features.length;

        if (srcLength == dstLength) {
            for (uint256 i = 0; i < srcLength; i++) {
                dst.features[i] = src.features[i];
            }
        } else if (srcLength > dstLength) {
            for (uint256 i = 0; i < dstLength; i++) {
                dst.features[i] = src.features[i];
            }
            for (uint256 i = dstLength; i < srcLength; i++) {
                dst.features.push(src.features[i]);
            }
        } else {
            for (uint256 i = 0; i < srcLength; i++) {
                dst.features[i] = src.features[i];
            }
            for (uint256 i = srcLength; i < dstLength; i++) {
                dst.features.pop();
            }
        }
    }

    function contains(
        string memory elem,
        string[] memory set
    ) private pure returns (bool) {
        bytes32 hashedElem = keccak256(bytes(elem));
        for (uint256 i = 0; i < set.length; i++) {
            if (keccak256(bytes(set[i])) == hashedElem) {
                return true;
            }
        }
        return false;
    }
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

        if (msg_.version.features.length > 0) {
            if (
                !IBCConnectionLib.isSupportedVersion(
                    getCompatibleVersions(), msg_.version
                )
            ) {
                revert IBCConnectionLib.ErrUnsupportedVersion();
            }
            connection.versions.push(msg_.version);
        } else {
            IBCConnectionLib.setSupportedVersions(
                getCompatibleVersions(), connection.versions
            );
        }

        connection.state = IbcCoreConnectionV1GlobalEnums.State.STATE_INIT;
        connection.delay_period = msg_.delayPeriod;
        connection.counterparty = msg_.counterparty;
        updateConnectionCommitment(connectionId);

        emit IBCConnectionLib.ConnectionOpenInit(
            connectionId, msg_.clientId, msg_.counterparty.client_id
        );

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
        connection.versions.push(
            IBCConnectionLib.pickVersion(
                getCompatibleVersions(), msg_.counterpartyVersions
            )
        );
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

        emit IBCConnectionLib.ConnectionOpenTry(
            connectionId, msg_.clientId, msg_.counterparty.client_id
        );

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
        if (
            !IBCConnectionLib.isSupportedVersion(
                connection.versions, msg_.version
            )
        ) {
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
            versions: IBCConnectionLib.newVersions(msg_.version),
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
        IBCConnectionLib.copyVersions(
            expectedConnection.versions, connection.versions
        );
        connection.counterparty.connection_id = msg_.counterpartyConnectionID;
        updateConnectionCommitment(msg_.connectionId);

        emit IBCConnectionLib.ConnectionOpenAck(
            msg_.connectionId,
            connection.client_id,
            connection.counterparty.connection_id
        );
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

        emit IBCConnectionLib.ConnectionOpenConfirm(
            msg_.connectionId,
            connection.client_id,
            connection.counterparty.connection_id
        );
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
        uint256 nextConnectionSequence =
            uint256(commitments[nextConnectionSequencePath]);

        string memory identifier = string(
            abi.encodePacked(
                "connection-", Strings.toString(nextConnectionSequence)
            )
        );
        commitments[nextConnectionSequencePath] =
            bytes32(nextConnectionSequence + 1);
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
     * @dev getCompatibleVersions returns the supported versions of the host chain.
     */
    function getCompatibleVersions()
        public
        pure
        virtual
        returns (IbcCoreConnectionV1Version.Data[] memory)
    {
        IbcCoreConnectionV1Version.Data[] memory versions =
            new IbcCoreConnectionV1Version.Data[](1);
        versions[0] = IBCConnectionLib.defaultIBCVersion();
        return versions;
    }
}
