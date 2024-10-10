pragma solidity ^0.8.27;

import "./ILightClient.sol";
import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCStore.sol";
import "../24-host/IBCCommitment.sol";
import "../02-client/IIBCClient.sol";

library IBCClientLib {
    event ClientRegistered(string clientType, address clientAddress);
    event ClientCreated(string clientType, uint32 clientId);
    event ClientUpdated(uint32 clientId, uint64 height);
}

/**
 * @dev IBCClient is a contract that implements [ICS-2](https://github.com/cosmos/ibc/tree/main/spec/core/ics-002-client-semantics).
 */
abstract contract IBCClient is IBCStore, IIBCClient {
    /**
     * @dev registerClient registers a new client type into the client registry
     */
    function registerClient(
        string calldata clientType,
        ILightClient client
    ) external override {
        if (address(clientRegistry[clientType]) != address(0)) {
            revert IBCErrors.ErrClientTypeAlreadyExists();
        }
        clientRegistry[clientType] = address(client);
        emit IBCClientLib.ClientRegistered(clientType, address(client));
    }

    /**
     * @dev createClient creates a new client state and populates it with a given consensus state
     */
    function createClient(
        IBCMsgs.MsgCreateClient calldata msg_
    ) external override returns (uint32) {
        address clientImpl = clientRegistry[msg_.clientType];
        if (clientImpl == address(0)) {
            revert IBCErrors.ErrClientTypeNotFound();
        }
        uint32 clientId = generateClientIdentifier();
        clientTypes[clientId] = msg_.clientType;
        clientImpls[clientId] = clientImpl;
        ConsensusStateUpdate memory update = ILightClient(clientImpl)
            .createClient(clientId, msg_.clientStateBytes, msg_.consensusStateBytes);
        commitments[IBCCommitment.clientStateCommitmentKey(clientId)] =
            update.clientStateCommitment;
        commitments[IBCCommitment.consensusStateCommitmentKey(
            clientId, update.height
        )] = update.consensusStateCommitment;
        emit IBCClientLib.ClientCreated(msg_.clientType, clientId);
        return clientId;
    }

    /**
     * @dev updateClient updates the consensus state and the state root from a provided header
     */
    function updateClient(
        IBCMsgs.MsgUpdateClient calldata msg_
    ) external override {
        ConsensusStateUpdate memory update = getClientInternal(msg_.clientId)
            .updateClient(msg_.clientId, msg_.clientMessage);
        commitments[IBCCommitment.clientStateCommitmentKey(msg_.clientId)] =
            update.clientStateCommitment;
        commitments[IBCCommitment.consensusStateCommitmentKey(
            msg_.clientId, update.height
        )] = update.consensusStateCommitment;
        emit IBCClientLib.ClientUpdated(msg_.clientId, update.height);
    }

    function generateClientIdentifier() internal returns (uint32) {
        uint32 nextClientSequence =
            uint32(uint256(commitments[nextClientSequencePath]));
        commitments[nextClientSequencePath] =
            bytes32(uint256(nextClientSequence + 1));
        return nextClientSequence;
    }
}
