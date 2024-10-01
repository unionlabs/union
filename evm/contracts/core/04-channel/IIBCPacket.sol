pragma solidity ^0.8.27;

import "../25-handler/IBCMsgs.sol";

interface IIBCPacket {
    /**
     * @dev sendPacket is called by a module in order to send an IBC packet on a channel.
     * The packet sequence generated for the packet to be sent is returned. An error
     * is returned if one occurs.
     */
    function sendPacket(
        uint32 sourceChannel,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes calldata data
    ) external returns (uint64);

    /**
     * @dev recvPacket is called by a module in order to receive & process an IBC packet
     * sent on the corresponding channel end on the counterparty chain.
     */
    function recvPacket(
        IBCMsgs.MsgPacketRecv calldata msg_
    ) external;

    /**
     * @dev recvIntentPacket is called by a module in order to receive & process an IBC intent packet
     * for an IBC packet sent on the corresponding channel end on the counterparty chain.
     * Note that no verification is done by the handler, the protocol must ensure that the market maker fullfilling the intent executes the expected effects.
     */
    function recvIntentPacket(
        IBCMsgs.MsgIntentPacketRecv calldata msg_
    ) external;

    /**
     * @dev writeAcknowledgement writes the packet execution acknowledgement to the state,
     * which will be verified by the counterparty chain using AcknowledgePacket.
     */
    function writeAcknowledgement(
        IBCPacket calldata packet,
        bytes memory acknowledgement
    ) external;

    /**
     * @dev AcknowledgePacket is called by a module to process the acknowledgement of a
     * packet previously sent by the calling module on a channel to a counterparty
     * module on the counterparty chain. Its intended usage is within the ante
     * handler. AcknowledgePacket will clean up the packet commitment,
     * which is no longer necessary since the packet has been received and acted upon.
     * It will also increment NextSequenceAck in case of ORDERED channels.
     */
    function acknowledgePacket(
        IBCMsgs.MsgPacketAcknowledgement calldata msg_
    ) external;

    /**
     * @dev timeoutPacket is called by a module in order to receive & process an IBC packet
     * sent on the corresponding channel end on the counterparty chain.
     */
    function timeoutPacket(
        IBCMsgs.MsgPacketTimeout calldata msg_
    ) external;

    /**
     * @dev batchSend is called by a module in order to commit multiple IBC packets that have been previously sent.
     * An error occur if any of the packets wasn't sent.
     * If successful, a new commitment is registered for the batch.
     */
    function batchSend(
        IBCMsgs.MsgBatchSend calldata msg_
    ) external;

    /**
     * @dev batchAcks is called by a module in order to commit multiple IBC packets acknowledgements.
     * An error occur if any of the packets wasn't received.
     * If successful, a new commitment is registered for the batch.
     */
    function batchAcks(
        IBCMsgs.MsgBatchAcks calldata msg_
    ) external;
}
