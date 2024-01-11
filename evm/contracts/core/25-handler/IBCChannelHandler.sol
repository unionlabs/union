pragma solidity ^0.8.23;

import "../25-handler/IBCMsgs.sol";
import "../24-host/IBCHost.sol";
import "../04-channel/IIBCChannel.sol";
import "../05-port/IIBCModule.sol";
import "../05-port/ModuleManager.sol";

/**
 * @dev IBCChannelHandler is a contract that calls a contract that implements `IIBCChannelHandshake` with delegatecall.
 */
abstract contract IBCChannelHandler is ModuleManager {
    // IBC Channel contract address
    address immutable ibcChannelAddress;

    event ChannelOpenInit(
        string channelId,
        string connectionId,
        string portId,
        string counterpartyPortId
    );

    event ChannelOpenTry(
        string channelId,
        string connectionId,
        string portId,
        string counterpartyPortId,
        string version
    );

    event ChannelOpenAck(string channelId, string portId);

    event ChannelOpenConfirm(string channelId, string portId);

    event ChannelCloseInit(string channelId, string portId);

    event ChannelCloseConfirm(string channelId, string portId);

    constructor(address ibcChannel) {
        ibcChannelAddress = ibcChannel;
    }

    function channelOpenInit(
        IBCMsgs.MsgChannelOpenInit calldata msg_
    ) external returns (string memory channelId) {
        (bool success, bytes memory res) = ibcChannelAddress.delegatecall(
            abi.encodeWithSelector(
                IIBCChannelHandshake.channelOpenInit.selector,
                msg_
            )
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
        channelId = abi.decode(res, (string));

        IIBCModule module = lookupModuleByPort(msg_.portId);
        module.onChanOpenInit(
            msg_.channel.ordering,
            msg_.channel.connection_hops,
            msg_.portId,
            channelId,
            msg_.channel.counterparty,
            msg_.channel.version
        );
        claimCapability(
            channelCapabilityPath(msg_.portId, channelId),
            address(module)
        );

        string memory connectionId = msg_.channel.connection_hops[0];

        emit ChannelOpenInit(
            channelId,
            connectionId,
            msg_.portId,
            msg_.channel.counterparty.port_id
        );

        return channelId;
    }

    function channelOpenTry(
        IBCMsgs.MsgChannelOpenTry calldata msg_
    ) external returns (string memory channelId) {
        (bool success, bytes memory res) = ibcChannelAddress.delegatecall(
            abi.encodeWithSelector(
                IIBCChannelHandshake.channelOpenTry.selector,
                msg_
            )
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
        channelId = abi.decode(res, (string));
        IIBCModule module = lookupModuleByPort(msg_.portId);
        module.onChanOpenTry(
            msg_.channel.ordering,
            msg_.channel.connection_hops,
            msg_.portId,
            channelId,
            msg_.channel.counterparty,
            msg_.channel.version,
            msg_.counterpartyVersion
        );
        claimCapability(
            channelCapabilityPath(msg_.portId, channelId),
            address(module)
        );

        string memory connectionId = msg_.channel.connection_hops[0];

        emit ChannelOpenTry(
            channelId,
            connectionId,
            msg_.portId,
            msg_.channel.counterparty.port_id,
            msg_.counterpartyVersion
        );

        return channelId;
    }

    function channelOpenAck(IBCMsgs.MsgChannelOpenAck calldata msg_) external {
        (bool success, bytes memory res) = ibcChannelAddress.delegatecall(
            abi.encodeWithSelector(
                IIBCChannelHandshake.channelOpenAck.selector,
                msg_
            )
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
        lookupModuleByPort(msg_.portId).onChanOpenAck(
            msg_.portId,
            msg_.channelId,
            msg_.counterpartyChannelId,
            msg_.counterpartyVersion
        );

        emit ChannelOpenAck(msg_.channelId, msg_.portId);
    }

    function channelOpenConfirm(
        IBCMsgs.MsgChannelOpenConfirm calldata msg_
    ) external {
        (bool success, bytes memory res) = ibcChannelAddress.delegatecall(
            abi.encodeWithSelector(
                IIBCChannelHandshake.channelOpenConfirm.selector,
                msg_
            )
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
        lookupModuleByPort(msg_.portId).onChanOpenConfirm(
            msg_.portId,
            msg_.channelId
        );

        emit ChannelOpenConfirm(msg_.channelId, msg_.portId);
    }

    function channelCloseInit(
        IBCMsgs.MsgChannelCloseInit calldata msg_
    ) external {
        (bool success, bytes memory res) = ibcChannelAddress.delegatecall(
            abi.encodeWithSelector(
                IIBCChannelHandshake.channelCloseInit.selector,
                msg_
            )
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
        lookupModuleByPort(msg_.portId).onChanCloseInit(
            msg_.portId,
            msg_.channelId
        );

        emit ChannelCloseInit(msg_.channelId, msg_.portId);
    }

    function channelCloseConfirm(
        IBCMsgs.MsgChannelCloseConfirm calldata msg_
    ) external {
        (bool success, bytes memory res) = ibcChannelAddress.delegatecall(
            abi.encodeWithSelector(
                IIBCChannelHandshake.channelCloseConfirm.selector,
                msg_
            )
        );
        if (!success) {
            revert(_getRevertMsg(res));
        }
        lookupModuleByPort(msg_.portId).onChanCloseConfirm(
            msg_.portId,
            msg_.channelId
        );

        emit ChannelCloseConfirm(msg_.channelId, msg_.portId);
    }
}
