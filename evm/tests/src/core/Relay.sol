pragma solidity ^0.8.27;

import "../../../contracts/apps/ucs/01-relay/Relay.sol";
import "../../../contracts/core/04-channel/IIBCPacket.sol";

contract TestUCS01Relay is UCS01Relay {
    constructor(
        IIBCPacket _ibcHandler,
        address admin
    ) {
        __Ownable_init(admin);
        // ibcHandler = _ibcHandler;
    }
    // function assumePacketSent(
    //     uint32 channel,
    //     IBCPacket calldata packet
    // ) public {
    //     commitments[IBCCommitment.batchPacketsCommitmentKey(
    //         channel, IBCPacketLib.commitPacket(packet)
    //     )] = IBCPacketLib.COMMITMENT_MAGIC;
    // }
}
