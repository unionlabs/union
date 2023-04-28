// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "@openzeppelin/contracts/utils/Context.sol";
import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCHost.sol";
import "../04-channel/IIBCChannel.sol";
import "../05-port/ModuleManager.sol";
import "../05-port/IIBCModule.sol";

/**
 * @dev IBCPacketHandler is a contract that calls a contract that implements `IIBCPacket` with delegatecall.
 */
abstract contract IBCPacketHandler is Context, ModuleManager {
    // IBC Packet contract address
    address immutable ibcChannelPacketAddress;

    // Events
    event SendPacket(Packet.Data packet);
    event RecvPacket(Packet.Data packet);
    event WriteAcknowledgement(
        string destinationPortId, string destinationChannel, uint64 sequence, bytes acknowledgement
    );
    event AcknowledgePacket(Packet.Data packet, bytes acknowledgement);

    constructor(address ibcChannelPacket) {
        ibcChannelPacketAddress = ibcChannelPacket;
    }

    function sendPacket(Packet.Data calldata packet) external {
        require(authenticateCapability(channelCapabilityPath(packet.source_port, packet.source_channel)));
        (bool success,) =
            ibcChannelPacketAddress.delegatecall(abi.encodeWithSelector(IIBCPacket.sendPacket.selector, packet));
        require(success);
        emit SendPacket(packet);
    }

    function recvPacket(IBCMsgs.MsgPacketRecv calldata msg_) external {
        IIBCModule module = lookupModuleByChannel(msg_.packet.destination_port, msg_.packet.destination_channel);
        bytes memory acknowledgement = module.onRecvPacket(msg_.packet, _msgSender());
        (bool success,) =
            ibcChannelPacketAddress.delegatecall(abi.encodeWithSelector(IIBCPacket.recvPacket.selector, msg_));
        require(success);
        if (acknowledgement.length > 0) {
            (success,) = ibcChannelPacketAddress.delegatecall(
                abi.encodeWithSelector(
                    IIBCPacket.writeAcknowledgement.selector,
                    msg_.packet.destination_port,
                    msg_.packet.destination_channel,
                    msg_.packet.sequence,
                    acknowledgement
                )
            );
            require(success);
            emit WriteAcknowledgement(
                msg_.packet.destination_port, msg_.packet.destination_channel, msg_.packet.sequence, acknowledgement
            );
        }
        emit RecvPacket(msg_.packet);
    }

    function writeAcknowledgement(
        string calldata destinationPortId,
        string calldata destinationChannel,
        uint64 sequence,
        bytes calldata acknowledgement
    ) external {
        require(authenticateCapability(channelCapabilityPath(destinationPortId, destinationChannel)));
        (bool success,) = ibcChannelPacketAddress.delegatecall(
            abi.encodeWithSelector(
                IIBCPacket.writeAcknowledgement.selector,
                destinationPortId,
                destinationChannel,
                sequence,
                acknowledgement
            )
        );
        require(success);
        emit WriteAcknowledgement(destinationPortId, destinationChannel, sequence, acknowledgement);
    }

    function acknowledgePacket(IBCMsgs.MsgPacketAcknowledgement calldata msg_) external {
        IIBCModule module = lookupModuleByChannel(msg_.packet.source_port, msg_.packet.source_channel);
        module.onAcknowledgementPacket(msg_.packet, msg_.acknowledgement, _msgSender());
        (bool success,) =
            ibcChannelPacketAddress.delegatecall(abi.encodeWithSelector(IIBCPacket.acknowledgePacket.selector, msg_));
        require(success);
        emit AcknowledgePacket(msg_.packet, msg_.acknowledgement);
    }
}
