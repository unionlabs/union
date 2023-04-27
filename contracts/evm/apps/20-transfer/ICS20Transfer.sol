// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "./IICS20Transfer.sol";
import "../../proto/Channel.sol";
import "../../core/05-port/IIBCModule.sol";
import "../../core/25-handler/IBCHandler.sol";
import "../../proto/FungibleTokenPacketData.sol";
import "solidity-stringutils/src/strings.sol";
import "solidity-bytes-utils/contracts/BytesLib.sol";
import "@openzeppelin/contracts/utils/Context.sol";

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
            FungibleTokenPacketData.Data({
                denom: denom,
                amount: amount,
                sender: abi.encodePacked(_msgSender()),
                receiver: abi.encodePacked(receiver)
            }),
            sourcePort,
            sourceChannel,
            timeoutHeight
        );
    }

    /// Module callbacks ///

    function onRecvPacket(Packet.Data calldata packet, address relayer)
        external
        virtual
        override
        returns (bytes memory acknowledgement)
    {
        FungibleTokenPacketData.Data memory data = FungibleTokenPacketData.decode(packet.data);
        strings.slice memory denom = data.denom.toSlice();
        strings.slice memory trimedDenom =
            data.denom.toSlice().beyond(_makeDenomPrefix(packet.source_port, packet.source_channel));
        if (!denom.equals(trimedDenom)) {
            // receiver is source chain
            return _newAcknowledgement(
                _transferFrom(
                    _getEscrowAddress(packet.destination_channel),
                    data.receiver.toAddress(0),
                    trimedDenom.toString(),
                    data.amount
                )
            );
        } else {
            string memory prefixedDenom =
                _makeDenomPrefix(packet.destination_port, packet.destination_channel).concat(denom);
            return _newAcknowledgement(_mint(data.receiver.toAddress(0), prefixedDenom, data.amount));
        }
    }

    function onAcknowledgementPacket(Packet.Data calldata packet, bytes calldata acknowledgement, address relayer)
        external
        virtual
        override
    {
        if (!_isSuccessAcknowledgement(acknowledgement)) {
            _refundTokens(FungibleTokenPacketData.decode(packet.data), packet.source_port, packet.source_channel);
        }
    }

    function onChanOpenInit(
        Channel.Order,
        string[] calldata,
        string calldata,
        string calldata channelId,
        ChannelCounterparty.Data calldata,
        string calldata
    ) external virtual override {
        // TODO authenticate a capability
        channelEscrowAddresses[channelId] = address(this);
    }

    function onChanOpenTry(
        Channel.Order,
        string[] calldata,
        string calldata,
        string calldata channelId,
        ChannelCounterparty.Data calldata,
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
        FungibleTokenPacketData.Data memory data,
        string memory sourcePort,
        string memory sourceChannel,
        uint64 timeoutHeight
    ) internal virtual {
        (Channel.Data memory channel, bool found) = ibcHandler.getChannel(sourcePort, sourceChannel);
        require(found, "channel not found");
        ibcHandler.sendPacket(
            Packet.Data({
                sequence: ibcHandler.getNextSequenceSend(sourcePort, sourceChannel),
                source_port: sourcePort,
                source_channel: sourceChannel,
                destination_port: channel.counterparty.port_id,
                destination_channel: channel.counterparty.channel_id,
                data: FungibleTokenPacketData.encode(data),
                timeout_height: Height.Data({revision_number: 0, revision_height: timeoutHeight}),
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
        FungibleTokenPacketData.Data memory data,
        string memory sourcePort,
        string memory sourceChannel
    ) internal virtual {
        if (!data.denom.toSlice().startsWith(_makeDenomPrefix(sourcePort, sourceChannel))) {
            // sender was source chain
            require(_transferFrom(_getEscrowAddress(sourceChannel), data.sender.toAddress(0), data.denom, data.amount));
        } else {
            require(_mint(data.sender.toAddress(0), data.denom, data.amount));
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
