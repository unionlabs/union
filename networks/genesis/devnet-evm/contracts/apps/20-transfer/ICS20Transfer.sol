// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.18;

import "./IICS20Transfer.sol";
import "../../proto/ibc/core/channel/v1/channel.sol";
import "../../proto/ibc/applications/transfer/v2/packet.sol";
import "../../core/05-port/IIBCModule.sol";
import "../../core/25-handler/IBCHandler.sol";
import "solidity-stringutils/src/strings.sol";
import "solidity-bytes-utils/contracts/BytesLib.sol";
import "@openzeppelin/contracts/utils/Context.sol";
import "@openzeppelin/contracts/utils/Strings.sol";

abstract contract ICS20Transfer is Context, IICS20Transfer {
    using strings for *;
    using BytesLib for bytes;

    IBCHandler ibcHandler;

    mapping(string => address) channelEscrowAddresses;

    constructor(IBCHandler ibcHandler_) {
        ibcHandler = ibcHandler_;
    }

    function sendTransfer(
        string calldata denom,
        uint64 amount,
        address receiver,
        string calldata sourcePort,
        string calldata sourceChannel,
        uint64 timeoutHeight
    ) external virtual override {
        if (!denom.toSlice().startsWith(_makeDenomPrefix(sourcePort, sourceChannel))) {
            // sender is source chain
            require(_transferFrom(_msgSender(), _getEscrowAddress(sourceChannel), denom, amount));
        } else {
            require(_burn(_msgSender(), denom, amount));
        }

        _sendPacket(
            IbcApplicationsTransferV2FungibleTokenPacketData.Data({
                denom: denom,
                amount: Strings.toString(amount),
                sender: string(abi.encodePacked(_msgSender())),
                receiver: string(abi.encodePacked(receiver)),
                // TODO: allow for users to dispatch memo?
                memo: ""
            }),
            sourcePort,
            sourceChannel,
            timeoutHeight
        );
    }

    /// Module callbacks ///

    function onRecvPacket(IbcCoreChannelV1Packet.Data calldata packet, address relayer)
        external
        virtual
        override
        returns (bytes memory acknowledgement)
    {
        IbcApplicationsTransferV2FungibleTokenPacketData.Data memory data = IbcApplicationsTransferV2FungibleTokenPacketData.decode(packet.data);
        strings.slice memory denom = data.denom.toSlice();
        strings.slice memory trimedDenom =
            data.denom.toSlice().beyond(_makeDenomPrefix(packet.source_port, packet.source_channel));
        if (!denom.equals(trimedDenom)) {
            // receiver is source chain
            return _newAcknowledgement(
                _transferFrom(
                    _getEscrowAddress(packet.destination_channel),
                    bytes(data.receiver).toAddress(0),
                    trimedDenom.toString(),
                    bytes(data.amount).toUint256(0)
                )
            );
        } else {
            string memory prefixedDenom =
                _makeDenomPrefix(packet.destination_port, packet.destination_channel).concat(denom);
            return _newAcknowledgement(_mint(bytes(data.receiver).toAddress(0), prefixedDenom, bytes(data.amount).toUint256(0)));
        }
    }

    function onAcknowledgementPacket(IbcCoreChannelV1Packet.Data calldata packet, bytes calldata acknowledgement, address relayer)
        external
        virtual
        override
    {
        if (!_isSuccessAcknowledgement(acknowledgement)) {
            _refundTokens(IbcApplicationsTransferV2FungibleTokenPacketData.decode(packet.data), packet.source_port, packet.source_channel);
        }
    }

    function onChanOpenInit(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata,
        string calldata,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata,
        string calldata
    ) external virtual override {
        // TODO authenticate a capability
        channelEscrowAddresses[channelId] = address(this);
    }

    function onChanOpenTry(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata,
        string calldata,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata,
        string calldata,
        string calldata
    ) external virtual override {
        // TODO authenticate a capability
        channelEscrowAddresses[channelId] = address(this);
    }

    function onChanOpenAck(string calldata portId, string calldata channelId, string calldata counterpartyVersion)
        external
        virtual
        override
    {}

    function onChanOpenConfirm(string calldata portId, string calldata channelId) external virtual override {}

    function onChanCloseInit(string calldata portId, string calldata channelId) external virtual override {}

    function onChanCloseConfirm(string calldata portId, string calldata channelId) external virtual override {}

    /// Internal functions ///

    function _transferFrom(address sender, address receiver, string memory denom, uint256 amount)
        internal
        virtual
        returns (bool);

    function _mint(address account, string memory denom, uint256 amount) internal virtual returns (bool);

    function _burn(address account, string memory denom, uint256 amount) internal virtual returns (bool);

    function _sendPacket(
        IbcApplicationsTransferV2FungibleTokenPacketData.Data memory data,
        string memory sourcePort,
        string memory sourceChannel,
        uint64 timeoutHeight
    ) internal virtual {
        (IbcCoreChannelV1Channel.Data memory channel, bool found) = ibcHandler.getChannel(sourcePort, sourceChannel);
        require(found, "channel not found");
        ibcHandler.sendPacket(
            IbcCoreChannelV1Packet.Data({
                sequence: ibcHandler.getNextSequenceSend(sourcePort, sourceChannel),
                source_port: sourcePort,
                source_channel: sourceChannel,
                destination_port: channel.counterparty.port_id,
                destination_channel: channel.counterparty.channel_id,
                data: IbcApplicationsTransferV2FungibleTokenPacketData.encode(data),
                timeout_height: IbcCoreClientV1Height.Data({revision_number: 0, revision_height: timeoutHeight}),
                timeout_timestamp: 0
            })
        );
    }

    function _getEscrowAddress(string memory sourceChannel) internal view virtual returns (address) {
        address escrow = channelEscrowAddresses[sourceChannel];
        require(escrow != address(0));
        return escrow;
    }

    function _newAcknowledgement(bool success) internal pure virtual returns (bytes memory) {
        bytes memory acknowledgement = new bytes(1);
        if (success) {
            acknowledgement[0] = 0x01;
        } else {
            acknowledgement[0] = 0x00;
        }
        return acknowledgement;
    }

    function _isSuccessAcknowledgement(bytes memory acknowledgement) internal pure virtual returns (bool) {
        require(acknowledgement.length == 1);
        return acknowledgement[0] == 0x01;
    }

    function _refundTokens(
        IbcApplicationsTransferV2FungibleTokenPacketData.Data memory data,
        string memory sourcePort,
        string memory sourceChannel
    ) internal virtual {
        if (!data.denom.toSlice().startsWith(_makeDenomPrefix(sourcePort, sourceChannel))) {
            // sender was source chain
            require(_transferFrom(_getEscrowAddress(sourceChannel), bytes(data.sender).toAddress(0), data.denom, bytes(data.amount).toUint256(0)));
        } else {
            require(_mint(bytes(data.sender).toAddress(0), data.denom, bytes(data.amount).toUint256(0)));
        }
    }

    /// Helper functions ///

    function _makeDenomPrefix(string memory port, string memory channel)
        internal
        pure
        virtual
        returns (strings.slice memory)
    {
        return port.toSlice().concat("/".toSlice()).toSlice().concat(channel.toSlice()).toSlice().concat("/".toSlice())
            .toSlice();
    }
}
