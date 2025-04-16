pragma solidity ^0.8.27;

import "./ILightClient.sol";
import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCStore.sol";
import "../24-host/IBCCommitment.sol";
import "../02-client/IIBCClient.sol";

library IBCClientLib {
    event RegisterClient(
        string indexed clientTypeIndex, string clientType, address clientAddress
    );
    event CreateClient(
        string indexed clientTypeIndex,
        string clientType,
        uint32 indexed clientId,
        string counterpartyChainId
    );
    event UpdateClient(uint32 indexed clientId, uint64 height);
    event Misbehaviour(uint32 indexed clientId);
}

/**
 * @dev IBCClient is a router contract that forward calls to clients implementing [ICS-2](https://github.com/cosmos/ibc/tree/main/spec/core/ics-002-client-semantics).
 */
abstract contract IBCClient is IBCStore, IIBCClient {
    /**
     * @dev registerClient registers a new client type into the client registry
     */
    function registerClient(
        string calldata clientType,
        ILightClient client
    ) external override restricted {
        if (address(clientRegistry[clientType]) != address(0)) {
            revert IBCErrors.ErrClientTypeAlreadyExists();
        }
        clientRegistry[clientType] = address(client);
        emit IBCClientLib.RegisterClient(
            clientType, clientType, address(client)
        );
    }

    /**
     * @dev createClient creates a new client state and populates it with a given consensus state
     */
    function createClient(
        IBCMsgs.MsgCreateClient calldata msg_
    ) external override restricted returns (uint32) {
        address clientImpl = clientRegistry[msg_.clientType];
        if (clientImpl == address(0)) {
            revert IBCErrors.ErrClientTypeNotFound();
        }
        uint32 clientId = generateClientIdentifier();
        clientTypes[clientId] = msg_.clientType;
        clientImpls[clientId] = clientImpl;
        (ConsensusStateUpdate memory update, string memory counterpartyChainId)
        = ILightClient(clientImpl).createClient(
            msg.sender,
            clientId,
            msg_.clientStateBytes,
            msg_.consensusStateBytes,
            msg_.relayer
        );
        commitments[IBCCommitment.clientStateCommitmentKey(clientId)] =
            update.clientStateCommitment;
        commitments[IBCCommitment.consensusStateCommitmentKey(
            clientId, update.height
        )] = update.consensusStateCommitment;
        emit IBCClientLib.CreateClient(
            msg_.clientType, msg_.clientType, clientId, counterpartyChainId
        );
        return clientId;
    }

    /**
     * @dev updateClient updates the consensus state and the state root from a provided header
     */
    function updateClient(
        IBCMsgs.MsgUpdateClient calldata msg_
    ) external override restricted {
        ConsensusStateUpdate memory update = getClientInternal(msg_.clientId)
            .updateClient(
            msg.sender, msg_.clientId, msg_.clientMessage, msg_.relayer
        );
        commitments[IBCCommitment.clientStateCommitmentKey(msg_.clientId)] =
            update.clientStateCommitment;
        commitments[IBCCommitment.consensusStateCommitmentKey(
            msg_.clientId, update.height
        )] = update.consensusStateCommitment;
        emit IBCClientLib.UpdateClient(msg_.clientId, update.height);
    }

    /**
     * @dev misbehaviour submits a misbehaviour to the client for it to take action if it is correct
     */
    function misbehaviour(
        IBCMsgs.MsgMisbehaviour calldata msg_
    ) external override restricted {
        getClientInternal(msg_.clientId).misbehaviour(
            msg.sender, msg_.clientId, msg_.clientMessage, msg_.relayer
        );
        emit IBCClientLib.Misbehaviour(msg_.clientId);
    }

    function generateClientIdentifier() internal returns (uint32) {
        uint32 nextClientSequence =
            uint32(uint256(commitments[nextClientSequencePath]));
        commitments[nextClientSequencePath] =
            bytes32(uint256(nextClientSequence + 1));
        return nextClientSequence;
    }
}
