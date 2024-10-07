pragma solidity ^0.8.27;

import "../../../contracts/core/25-handler/IBCHandler.sol";

contract TestIBCHandler is IBCHandler {
    function assumePacketSent(
        uint32 channel,
        IBCPacket calldata packet
    ) public {
        commitments[IBCCommitment.batchPacketsCommitmentKey(
            channel, IBCPacketLib.commitPacket(packet)
        )] = IBCPacketLib.COMMITMENT_MAGIC;
    }
}
