pragma solidity ^0.8.21;

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
    address immutable ibcPacket;

    // Events
    event SendPacket(
        uint64 sequence,
        string sourcePort,
        string sourceChannel,
        IbcCoreClientV1Height.Data timeoutHeight,
        uint64 timeoutTimestamp,
        bytes data
    );

    event RecvPacket(IbcCoreChannelV1Packet.Data packet);

    event WriteAcknowledgement(
        string destinationPortId,
        string destinationChannel,
        uint64 sequence,
        bytes acknowledgement
    );

    event AcknowledgePacket(
        IbcCoreChannelV1Packet.Data packet,
        bytes acknowledgement
    );

    event TimeoutPacket(IbcCoreChannelV1Packet.Data packet);

    constructor(address _ibcPacket) {
        ibcPacket = _ibcPacket;
    }

    function sendPacket(
        string calldata sourcePort,
        string calldata sourceChannel,
        IbcCoreClientV1Height.Data calldata timeoutHeight,
        uint64 timeoutTimestamp,
        bytes calldata data
    ) external {
        require(
            authenticateCapability(
                channelCapabilityPath(sourcePort, sourceChannel)
            )
        );
        (bool success, bytes memory res) = ibcPacket.delegatecall(
            abi.encodeWithSelector(
                IIBCPacket.sendPacket.selector,
                sourcePort,
                sourceChannel,
                timeoutHeight,
                timeoutTimestamp,
                data
            )
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
        emit SendPacket(
            abi.decode(res, (uint64)),
            sourcePort,
            sourceChannel,
            timeoutHeight,
            timeoutTimestamp,
            data
        );
    }

    function recvPacket(IBCMsgs.MsgPacketRecv calldata msg_) external {
        (bool success, bytes memory res) = ibcPacket.delegatecall(
            abi.encodeWithSelector(IIBCPacket.recvPacket.selector, msg_)
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
        IIBCModule module = lookupModuleByChannel(
            msg_.packet.destination_port,
            msg_.packet.destination_channel
        );
        bytes memory acknowledgement = module.onRecvPacket(
            msg_.packet,
            _msgSender()
        );
        if (acknowledgement.length > 0) {
            (success, res) = ibcPacket.delegatecall(
                abi.encodeWithSelector(
                    IIBCPacket.writeAcknowledgement.selector,
                    msg_.packet.destination_port,
                    msg_.packet.destination_channel,
                    msg_.packet.sequence,
                    acknowledgement
                )
            );
            if (!success) {
                revert(_getRevertMsg(res));
            }
            emit WriteAcknowledgement(
                msg_.packet.destination_port,
                msg_.packet.destination_channel,
                msg_.packet.sequence,
                acknowledgement
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
        require(
            authenticateCapability(
                channelCapabilityPath(destinationPortId, destinationChannel)
            )
        );
        (bool success, bytes memory res) = ibcPacket.delegatecall(
            abi.encodeWithSelector(
                IIBCPacket.writeAcknowledgement.selector,
                destinationPortId,
                destinationChannel,
                sequence,
                acknowledgement
            )
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
        emit WriteAcknowledgement(
            destinationPortId,
            destinationChannel,
            sequence,
            acknowledgement
        );
    }

    function acknowledgePacket(
        IBCMsgs.MsgPacketAcknowledgement calldata msg_
    ) external {
        IIBCModule module = lookupModuleByChannel(
            msg_.packet.source_port,
            msg_.packet.source_channel
        );
        module.onAcknowledgementPacket(
            msg_.packet,
            msg_.acknowledgement,
            _msgSender()
        );
        (bool success, bytes memory res) = ibcPacket.delegatecall(
            abi.encodeWithSelector(IIBCPacket.acknowledgePacket.selector, msg_)
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
        emit AcknowledgePacket(msg_.packet, msg_.acknowledgement);
    }

    function timeoutPacket(IBCMsgs.MsgPacketTimeout calldata msg_) external {
        IIBCModule module = lookupModuleByChannel(
            msg_.packet.source_port,
            msg_.packet.source_channel
        );
        module.onTimeoutPacket(msg_.packet, _msgSender());
        (bool success, bytes memory res) = ibcPacket.delegatecall(
            abi.encodeWithSelector(IIBCPacket.timeoutPacket.selector, msg_)
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
        emit TimeoutPacket(msg_.packet);
    }
}
