pragma solidity ^0.8.18;

import "./TransferPacket.sol";
import "./IICS20Transfer.sol";
import "../../proto/ibc/core/channel/v1/channel.sol";
import "../../proto/ibc/applications/transfer/v2/packet.sol";
import "../../core/05-port/IIBCModule.sol";
import "../../core/25-handler/IBCHandler.sol";
import "solidity-stringutils/strings.sol";
import "solidity-bytes-utils/BytesLib.sol";
import "@openzeppelin/contracts/utils/Strings.sol";
import "../Base.sol";

abstract contract ICS20Transfer is IBCAppBase {
    using strings for *;
    using BytesLib for bytes;

    mapping(string => address) channelEscrowAddresses;

    function onRecvPacket(
        IbcCoreChannelV1Packet.Data calldata packet,
        address relayer
    ) external virtual override onlyIBC returns (bytes memory acknowledgement) {
        TransferPacket memory transferPacket = TransferPacketHelp.decode(
            packet.data
        );
        strings.slice memory denom = transferPacket.denom.toSlice();
        strings.slice memory trimedDenom = transferPacket
            .denom
            .toSlice()
            .beyond(
                _makeDenomPrefix(packet.source_port, packet.source_channel)
            );
        if (!denom.equals(trimedDenom)) {
            // receiver is source chain
            return
                _newAcknowledgement(
                    _transferFrom(
                        _getEscrowAddress(packet.destination_channel),
                        fromHex(transferPacket.receiver).toAddress(0),
                        trimedDenom.toString(),
                        transferPacket.amount
                    )
                );
        } else {
            string memory prefixedDenom = _makeDenomPrefix(
                packet.destination_port,
                packet.destination_channel
            ).concat(denom);
            return
                _newAcknowledgement(
                    _mint(
                        fromHex(transferPacket.receiver).toAddress(0),
                        prefixedDenom,
                        transferPacket.amount
                    )
                );
        }
    }

    
   // Convert an hexadecimal character to their value
   function fromHexChar(uint8 c) public pure returns (uint8) {
       if (bytes1(c) >= bytes1('0') && bytes1(c) <= bytes1('9')) {
           return c - uint8(bytes1('0'));
       }
       if (bytes1(c) >= bytes1('a') && bytes1(c) <= bytes1('f')) {
           return 10 + c - uint8(bytes1('a'));
       }
       if (bytes1(c) >= bytes1('A') && bytes1(c) <= bytes1('F')) {
           return 10 + c - uint8(bytes1('A'));
       }
       revert("fail");
   }

   // Convert an hexadecimal string to raw bytes
   function fromHex(string memory s) public pure returns (bytes memory) {
       bytes memory ss = bytes(s);
       require(ss.length%2 == 0); // length must be even
       bytes memory r = new bytes(ss.length/2);
       for (uint i=0; i<ss.length/2; ++i) {
           r[i] = bytes1(fromHexChar(uint8(ss[2*i])) * 16 +
                       fromHexChar(uint8(ss[2*i+1])));
       }
       return r;
   }

    function onAcknowledgementPacket(
        IbcCoreChannelV1Packet.Data calldata packet,
        bytes calldata acknowledgement,
        address relayer
    ) external virtual override onlyIBC {
        if (!_isSuccessAcknowledgement(acknowledgement)) {
            _refundTokens(
                TransferPacketHelp.decode(packet.data),
                packet.source_port,
                packet.source_channel
            );
        }
    }

    function onChanOpenInit(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata,
        string calldata,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata,
        string calldata
    ) external virtual override onlyIBC {
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
    ) external virtual override onlyIBC {
        // TODO authenticate a capability
        channelEscrowAddresses[channelId] = address(this);
    }

    function onChanOpenAck(
        string calldata portId,
        string calldata channelId,
        string calldata counterpartyVersion
    ) external virtual override onlyIBC {}

    function onChanOpenConfirm(
        string calldata portId,
        string calldata channelId
    ) external virtual override onlyIBC {}

    function onChanCloseInit(
        string calldata portId,
        string calldata channelId
    ) external virtual override onlyIBC {}

    function onChanCloseConfirm(
        string calldata portId,
        string calldata channelId
    ) external virtual override onlyIBC {}

    /// Internal functions ///

    function _transferFrom(
        address sender,
        address receiver,
        string memory denom,
        uint256 amount
    ) internal virtual returns (bool);

    function _mint(
        address account,
        string memory denom,
        uint256 amount
    ) internal virtual returns (bool);

    function _burn(
        address account,
        string memory denom,
        uint256 amount
    ) internal virtual returns (bool);

    function _sendPacket(
        TransferPacket memory packet,
        string memory sourcePort,
        string memory sourceChannel,
        uint64 timeoutRevisionNumber,
        uint64 timeoutRevisionHeight
    ) internal virtual {
        IBCHandler(ibcAddress()).sendPacket(
            sourcePort,
            sourceChannel,
            IbcCoreClientV1Height.Data({
                revision_number: timeoutRevisionNumber,
                revision_height: timeoutRevisionHeight
            }),
            0,
            TransferPacketHelp.encode(packet)
        );
    }

    function _getEscrowAddress(
        string memory sourceChannel
    ) internal view virtual returns (address) {
        address escrow = channelEscrowAddresses[sourceChannel];
        require(escrow != address(0), "escrow address must exist");
        return escrow;
    }

    function _newAcknowledgement(
        bool success
    ) internal pure virtual returns (bytes memory) {
        bytes memory acknowledgement = new bytes(1);
        if (success) {
            acknowledgement[0] = 0x01;
        } else {
            acknowledgement[0] = 0x00;
        }
        return acknowledgement;
    }

    function _isSuccessAcknowledgement(
        bytes memory acknowledgement
    ) internal pure virtual returns (bool) {
        require(
            acknowledgement.length == 1,
            "acknowledgement must be a single byte"
        );
        return acknowledgement[0] == 0x01;
    }

    function _refundTokens(
        TransferPacket memory data,
        string memory sourcePort,
        string memory sourceChannel
    ) internal virtual {
        if (
            !data.denom.toSlice().startsWith(
                _makeDenomPrefix(sourcePort, sourceChannel)
            )
        ) {
            // sender was source chain
            require(
                _transferFrom(
                    _getEscrowAddress(sourceChannel),
                    bytes(data.sender).toAddress(0),
                    data.denom,
                    data.amount
                )
            );
        } else {
            require(
                _mint(bytes(data.sender).toAddress(0), data.denom, data.amount)
            );
        }
    }

    /// Helper functions ///

    function _makeDenomPrefix(
        string memory port,
        string memory channel
    ) internal pure virtual returns (strings.slice memory) {
        return
            port
                .toSlice()
                .concat("/".toSlice())
                .toSlice()
                .concat(channel.toSlice())
                .toSlice()
                .concat("/".toSlice())
                .toSlice();
    }
}
