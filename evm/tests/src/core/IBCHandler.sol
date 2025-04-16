pragma solidity ^0.8.27;

import "../../../contracts/core/25-handler/IBCHandler.sol";

contract TestIBCHandler is IBCHandler {
    function assumePacketSent(
        IBCPacket calldata packet
    ) public {
        commitments[IBCCommitment.batchPacketsCommitmentKey(
            IBCPacketLib.commitPacket(packet)
        )] = IBCPacketLib.COMMITMENT_MAGIC;
    }
}
