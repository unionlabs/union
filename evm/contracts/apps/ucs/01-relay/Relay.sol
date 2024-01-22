pragma solidity ^0.8.23;

import "@openzeppelin/token/ERC20/ERC20.sol";
import "@openzeppelin/token/ERC20/IERC20.sol";
import "@openzeppelin/token/ERC20/utils/SafeERC20.sol";
import "solady/utils/LibString.sol";
import "solidity-stringutils/strings.sol";
import "../../../core/25-handler/IBCHandler.sol";
import "../../Base.sol";
import "./IERC20Denom.sol";
import "./ERC20Denom.sol";

// NOTE: uint128 limitation from cosmwasm_std Coin type for transfers.
struct LocalToken {
    address denom;
    uint128 amount;
}

struct Token {
    string denom;
    uint256 amount;
}

struct RelayPacket {
    bytes sender;
    bytes receiver;
    Token[] tokens;
}

library RelayLib {
    using LibString for *;

    error ErrInvalidHexAddress();
    error ErrInvalidBytesAddress();
    error ErrUnauthorized();
    error ErrInvalidAcknowledgement();
    error ErrInvalidProtocolVersion();
    error ErrInvalidProtocolOrdering();
    error ErrInvalidCounterpartyProtocolVersion();
    error ErrUnstoppable();

    IbcCoreChannelV1GlobalEnums.Order public constant ORDER =
        IbcCoreChannelV1GlobalEnums.Order.ORDER_UNORDERED;
    string public constant VERSION = "ucs01-0";
    bytes1 public constant ACK_SUCCESS = 0x01;
    bytes1 public constant ACK_FAILURE = 0x00;
    uint256 public constant ACK_LENGTH = 1;

    event DenomCreated(string denom, address token);
    event Received(
        string sender,
        address receiver,
        string denom,
        address token,
        uint256 amount
    );
    event Sent(
        address sender,
        string receiver,
        string denom,
        address token,
        uint256 amount
    );
    event Timeout(
        address sender,
        string receiver,
        string denom,
        address token,
        uint256 amount
    );

    function isValidVersion(
        string memory version
    ) internal pure returns (bool) {
        return version.eq(VERSION);
    }

    function isFromChannel(
        string memory portId,
        string memory channelId,
        string memory denom
    ) internal pure returns (bool) {
        return
            bytes(denom).length > 0 &&
            denom.startsWith(makeDenomPrefix(portId, channelId));
    }

    function makeDenomPrefix(
        string memory portId,
        string memory channelId
    ) internal pure returns (string memory) {
        return string(abi.encodePacked(portId, "/", channelId, "/"));
    }

    function makeForeignDenom(
        string memory portId,
        string memory channelId,
        string memory denom
    ) internal pure returns (string memory) {
        return
            string(abi.encodePacked(makeDenomPrefix(portId, channelId), denom));
    }

    // Convert 32 hexadecimal digits into 16 bytes.
    function hexToBytes16(bytes32 h) internal pure returns (bytes16 b) {
        unchecked {
            // Ensure all chars below 128
            if (
                h &
                    0x8080808080808080808080808080808080808080808080808080808080808080 !=
                0
            ) {
                revert ErrInvalidHexAddress();
            }
            // Subtract '0' from every char
            h = bytes32(
                uint256(h) -
                    0x3030303030303030303030303030303030303030303030303030303030303030
            );
            // Ensure all chars still below 128, i.e. no underflow in the previous line
            if (
                h &
                    0x8080808080808080808080808080808080808080808080808080808080808080 !=
                0
            ) {
                revert ErrInvalidHexAddress();
            }
            // Calculate mask for chars that originally were above '9'
            bytes32 ndm = bytes32(
                (((uint256(h) +
                    0x7676767676767676767676767676767676767676767676767676767676767676) &
                    0x8080808080808080808080808080808080808080808080808080808080808080) >>
                    7) * 0xFF
            );
            // Subtract 7 ('A' - '0') from every char that originally was above '9'
            h = bytes32(
                uint256(h) -
                    uint256(
                        ndm &
                            0x0707070707070707070707070707070707070707070707070707070707070707
                    )
            );
            // Ensure all chars still below 128, i.e. no underflow in the previous line
            if (
                h &
                    0x8080808080808080808080808080808080808080808080808080808080808080 !=
                0
            ) {
                revert ErrInvalidHexAddress();
            }
            // Ensure chars that originally were above '9' are now above 9
            if (
                (uint256(h) -
                    uint256(
                        ndm &
                            0x0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A
                    )) &
                    0x8080808080808080808080808080808080808080808080808080808080808080 !=
                0
            ) {
                revert ErrInvalidHexAddress();
            }
            // Calculate Mask for chars that originally were above 'F'
            bytes32 lcm = bytes32(
                (((uint256(h) +
                    0x7070707070707070707070707070707070707070707070707070707070707070) &
                    0x8080808080808080808080808080808080808080808080808080808080808080) >>
                    7) * 0xFF
            );
            // Subtract 32 ('a' - 'A') from all chars that oroginally were above 'F'
            h = bytes32(
                uint256(h) -
                    uint256(
                        lcm &
                            0x2020202020202020202020202020202020202020202020202020202020202020
                    )
            );
            // Ensure chars that originally were above 'F' are now above 9
            if (
                (uint256(h) -
                    uint256(
                        lcm &
                            0x0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A0A
                    )) &
                    0x8080808080808080808080808080808080808080808080808080808080808080 !=
                0
            ) {
                revert ErrInvalidHexAddress();
            }
            // Ensure all chars are below 16
            if (
                h &
                    0xF0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0F0 !=
                0
            ) {
                revert ErrInvalidHexAddress();
            }
            // 0x0A0B0C0D... -> 0xAB00CD00...
            h =
                ((h &
                    0x0F000F000F000F000F000F000F000F000F000F000F000F000F000F000F000F00) <<
                    4) |
                ((h &
                    0x000F000F000F000F000F000F000F000F000F000F000F000F000F000F000F000F) <<
                    8);
            // 0xAA00BB00CC00DD00... -> 0xAABB0000CCDD0000...
            h =
                (h &
                    0xFF000000FF000000FF000000FF000000FF000000FF000000FF000000FF000000) |
                ((h &
                    0x0000FF000000FF000000FF000000FF000000FF000000FF000000FF000000FF00) <<
                    8);
            // 0xAAAA0000BBBB0000CCCC0000DDDD0000... -> 0xAAAABBBB00000000CCCCDDDD00000000...
            h =
                (h &
                    0xFFFF000000000000FFFF000000000000FFFF000000000000FFFF000000000000) |
                ((h &
                    0x00000000FFFF000000000000FFFF000000000000FFFF000000000000FFFF0000) <<
                    16);
            // 0xAAAAAAAA00000000BBBBBBBB00000000CCCCCCCC00000000DDDDDDDD00000000 -> 0xAAAAAAAABBBBBBBB0000000000000000CCCCCCCCDDDDDDDD0000000000000000
            h =
                (h &
                    0xFFFFFFFF000000000000000000000000FFFFFFFF000000000000000000000000) |
                ((h &
                    0x0000000000000000FFFFFFFF000000000000000000000000FFFFFFFF00000000) <<
                    32);
            // 0xAAAAAAAAAAAAAAAA0000000000000000BBBBBBBBBBBBBBBB0000000000000000 -> 0xAAAAAAAAAAAAAAAABBBBBBBBBBBBBBBB00000000000000000000000000000000
            h =
                (h &
                    0xFFFFFFFFFFFFFFFF000000000000000000000000000000000000000000000000) |
                ((h &
                    0x00000000000000000000000000000000FFFFFFFFFFFFFFFF0000000000000000) <<
                    64);
            // Trim to 16 bytes
            b = bytes16(h);
        }
    }

    function hexToAddress(string memory s) public pure returns (address) {
        if (bytes(s).length != 42) {
            revert ErrInvalidHexAddress();
        }
        bytes2 prefix;
        bytes32 leftHex;
        bytes32 rightHex;
        assembly {
            prefix := mload(add(s, 0x20))
            leftHex := mload(add(s, 0x22))
            rightHex := mload(add(s, 0x2A))
        }
        if (prefix != "0x") {
            revert ErrInvalidHexAddress();
        }
        bytes16 left = hexToBytes16(leftHex);
        bytes16 right = hexToBytes16(rightHex);
        return address(bytes20(left) | (bytes20(right) >> 32));
    }

    function bytesToAddress(bytes memory b) internal pure returns (address) {
        if (b.length != 20) {
            revert ErrInvalidBytesAddress();
        }
        return address(uint160(bytes20(b)));
    }
}

library RelayPacketLib {
    function encode(
        RelayPacket memory packet
    ) internal pure returns (bytes memory) {
        return abi.encode(packet.sender, packet.receiver, packet.tokens);
    }

    function decode(
        bytes memory packet
    ) internal pure returns (RelayPacket memory) {
        (
            bytes memory sender,
            bytes memory receiver,
            Token[] memory tokens
        ) = abi.decode(packet, (bytes, bytes, Token[]));
        return
            RelayPacket({sender: sender, receiver: receiver, tokens: tokens});
    }
}

contract UCS01Relay is IBCAppBase {
    using RelayPacketLib for RelayPacket;
    using LibString for *;
    using strings for *;

    IBCHandler private immutable ibcHandler;

    mapping(string => address) private denomToAddress;
    mapping(address => string) private addressToDenom;
    mapping(string => mapping(string => IbcCoreChannelV1Counterparty.Data))
        private counterpartyEndpoints;
    mapping(string => mapping(string => mapping(address => uint256)))
        private outstanding;

    constructor(IBCHandler _ibcHandler) {
        ibcHandler = _ibcHandler;
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }

    function getDenomAddress(
        string memory denom
    ) public view returns (address) {
        return denomToAddress[denom];
    }

    function getOutstanding(
        string memory sourcePort,
        string memory sourceChannel,
        address token
    ) public view returns (uint256) {
        return outstanding[sourcePort][sourceChannel][token];
    }

    function getCounterpartyEndpoint(
        string memory sourcePort,
        string memory sourceChannel
    ) public view returns (IbcCoreChannelV1Counterparty.Data memory) {
        return counterpartyEndpoints[sourcePort][sourceChannel];
    }

    function increaseOutstanding(
        string memory sourcePort,
        string memory sourceChannel,
        address token,
        uint256 amount
    ) internal {
        outstanding[sourcePort][sourceChannel][token] += amount;
    }

    function decreaseOutstanding(
        string memory sourcePort,
        string memory sourceChannel,
        address token,
        uint256 amount
    ) internal {
        outstanding[sourcePort][sourceChannel][token] -= amount;
    }

    function sendToken(
        string calldata sourcePort,
        string calldata sourceChannel,
        string memory counterpartyPortId,
        string memory counterpartyChannelId,
        LocalToken calldata localToken
    ) internal returns (string memory addressDenom) {
        SafeERC20.safeTransferFrom(
            IERC20(localToken.denom),
            msg.sender,
            address(this),
            localToken.amount
        );
        addressDenom = addressToDenom[localToken.denom];
        if (
            RelayLib.isFromChannel(
                counterpartyPortId,
                counterpartyChannelId,
                addressDenom
            )
        ) {
            IERC20Denom(localToken.denom).burn(
                address(this),
                localToken.amount
            );
        } else {
            increaseOutstanding(
                sourcePort,
                sourceChannel,
                localToken.denom,
                localToken.amount
            );
            addressDenom = localToken.denom.toHexString();
        }
    }

    function send(
        string calldata sourcePort,
        string calldata sourceChannel,
        bytes calldata receiver,
        LocalToken[] calldata tokens,
        uint64 counterpartyTimeoutRevisionNumber,
        uint64 counterpartyTimeoutRevisionHeight
    ) public {
        IbcCoreChannelV1Counterparty.Data
            memory counterparty = counterpartyEndpoints[sourcePort][
                sourceChannel
            ];
        Token[] memory normalizedTokens = new Token[](tokens.length);
        // For each token, we transfer them locally then:
        // - if the token is locally native, keep it escrowed
        // - if the token is remote native, burn the wrapper
        for (uint256 i = 0; i < tokens.length; i++) {
            LocalToken calldata localToken = tokens[i];
            string memory addressDenom = sendToken(
                sourcePort,
                sourceChannel,
                counterparty.port_id,
                counterparty.channel_id,
                localToken
            );
            normalizedTokens[i].denom = addressDenom;
            normalizedTokens[i].amount = uint256(localToken.amount);
            emit RelayLib.Sent(
                msg.sender,
                receiver.toHexString(),
                addressDenom,
                localToken.denom,
                uint256(localToken.amount)
            );
        }
        string memory sender = msg.sender.toHexString();
        RelayPacket memory packet = RelayPacket({
            sender: abi.encodePacked(msg.sender),
            receiver: receiver,
            tokens: normalizedTokens
        });
        IbcCoreClientV1Height.Data memory timeoutHeight = IbcCoreClientV1Height
            .Data({
                revision_number: counterpartyTimeoutRevisionNumber,
                revision_height: counterpartyTimeoutRevisionHeight
            });
        ibcHandler.sendPacket(
            sourcePort,
            sourceChannel,
            timeoutHeight,
            0,
            packet.encode()
        );
    }

    function refundTokens(
        string memory portId,
        string memory channelId,
        RelayPacket memory packet
    ) internal {
        string memory receiver = packet.receiver.toHexString();
        // We're going to refund, the receiver will be the sender.
        address userToRefund = RelayLib.bytesToAddress(packet.sender);
        for (uint256 i = 0; i < packet.tokens.length; i++) {
            Token memory token = packet.tokens[i];
            // Either we tried to send back a remote native token
            // which we burnt, or a locally native token that we escrowed.
            address denomAddress = denomToAddress[token.denom];
            if (denomAddress != address(0)) {
                IERC20Denom(denomAddress).mint(userToRefund, token.amount);
            } else {
                // It must be in the form 0x...
                denomAddress = RelayLib.hexToAddress(token.denom);
                decreaseOutstanding(
                    portId,
                    channelId,
                    denomAddress,
                    token.amount
                );
                IERC20(denomAddress).transfer(userToRefund, token.amount);
            }
            emit RelayLib.Timeout(
                userToRefund,
                receiver,
                token.denom,
                denomAddress,
                token.amount
            );
        }
    }

    function onRecvPacketProcessing(
        IbcCoreChannelV1Packet.Data calldata ibcPacket,
        address relayer
    ) public {
        if (msg.sender != address(this)) {
            revert RelayLib.ErrUnauthorized();
        }
        RelayPacket memory packet = RelayPacketLib.decode(ibcPacket.data);
        string memory prefix = RelayLib.makeDenomPrefix(
            ibcPacket.destination_port,
            ibcPacket.destination_channel
        );
        for (uint256 i = 0; i < packet.tokens.length; i++) {
            Token memory token = packet.tokens[i];
            strings.slice memory denomSlice = token.denom.toSlice();
            // This will trim the denom IFF it is prefixed
            strings.slice memory trimedDenom = denomSlice.beyond(
                prefix.toSlice()
            );
            address receiver = RelayLib.bytesToAddress(packet.receiver);
            address denomAddress;
            string memory denom;
            if (!denomSlice.equals(token.denom.toSlice())) {
                denom = trimedDenom.toString();
                denomAddress = RelayLib.hexToAddress(denom);
                // The token must be outstanding.
                decreaseOutstanding(
                    ibcPacket.destination_port,
                    ibcPacket.destination_channel,
                    denomAddress,
                    token.amount
                );
                IERC20(denomAddress).transfer(receiver, token.amount);
            } else {
                denom = RelayLib.makeForeignDenom(
                    ibcPacket.source_port,
                    ibcPacket.source_channel,
                    token.denom
                );
                denomAddress = denomToAddress[denom];
                if (denomAddress == address(0)) {
                    denomAddress = address(new ERC20Denom(denom));
                    denomToAddress[denom] = denomAddress;
                    addressToDenom[denomAddress] = denom;
                    emit RelayLib.DenomCreated(denom, denomAddress);
                }
                IERC20Denom(denomAddress).mint(receiver, token.amount);
            }
            emit RelayLib.Received(
                packet.sender.toHexString(),
                receiver,
                denom,
                denomAddress,
                token.amount
            );
        }
    }

    function onRecvPacket(
        IbcCoreChannelV1Packet.Data calldata ibcPacket,
        address relayer
    ) external virtual override onlyIBC returns (bytes memory) {
        // TODO: maybe consider threading _res in the failure ack
        (bool success, bytes memory _res) = address(this).call(
            abi.encodeWithSelector(
                this.onRecvPacketProcessing.selector,
                ibcPacket,
                relayer
            )
        );
        if (success) {
            return abi.encodePacked(RelayLib.ACK_SUCCESS);
        } else {
            return abi.encodePacked(RelayLib.ACK_FAILURE);
        }
    }

    function onAcknowledgementPacket(
        IbcCoreChannelV1Packet.Data calldata ibcPacket,
        bytes calldata acknowledgement,
        address _relayer
    ) external virtual override onlyIBC {
        if (
            acknowledgement.length != RelayLib.ACK_LENGTH ||
            (acknowledgement[0] != RelayLib.ACK_FAILURE &&
                acknowledgement[0] != RelayLib.ACK_SUCCESS)
        ) {
            revert RelayLib.ErrInvalidAcknowledgement();
        }
        RelayPacket memory packet = RelayPacketLib.decode(ibcPacket.data);
        if (acknowledgement[0] == RelayLib.ACK_FAILURE) {
            refundTokens(
                ibcPacket.source_port,
                ibcPacket.source_channel,
                packet
            );
        }
    }

    function onTimeoutPacket(
        IbcCoreChannelV1Packet.Data calldata ibcPacket,
        address _relayer
    ) external virtual override onlyIBC {
        refundTokens(
            ibcPacket.source_port,
            ibcPacket.source_channel,
            RelayPacketLib.decode(ibcPacket.data)
        );
    }

    function onChanOpenInit(
        IbcCoreChannelV1GlobalEnums.Order order,
        string[] calldata _connectionHops,
        string calldata portId,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata counterpartyEndpoint,
        string calldata version
    ) external virtual override onlyIBC {
        if (!RelayLib.isValidVersion(version)) {
            revert RelayLib.ErrInvalidProtocolVersion();
        }
        if (order != RelayLib.ORDER) {
            revert RelayLib.ErrInvalidProtocolOrdering();
        }
        counterpartyEndpoints[portId][channelId] = counterpartyEndpoint;
    }

    function onChanOpenTry(
        IbcCoreChannelV1GlobalEnums.Order order,
        string[] calldata _connectionHops,
        string calldata portId,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata counterpartyEndpoint,
        string calldata version,
        string calldata counterpartyVersion
    ) external virtual override onlyIBC {
        if (!RelayLib.isValidVersion(version)) {
            revert RelayLib.ErrInvalidProtocolVersion();
        }
        if (order != RelayLib.ORDER) {
            revert RelayLib.ErrInvalidProtocolOrdering();
        }
        if (!RelayLib.isValidVersion(counterpartyVersion)) {
            revert RelayLib.ErrInvalidCounterpartyProtocolVersion();
        }
        counterpartyEndpoints[portId][channelId] = counterpartyEndpoint;
    }

    function onChanOpenAck(
        string calldata portId,
        string calldata channelId,
        string calldata counterpartyChannelId,
        string calldata counterpartyVersion
    ) external virtual override onlyIBC {
        if (!RelayLib.isValidVersion(counterpartyVersion)) {
            revert RelayLib.ErrInvalidCounterpartyProtocolVersion();
        }
        // Counterparty channel was empty.
        counterpartyEndpoints[portId][channelId]
            .channel_id = counterpartyChannelId;
    }

    function onChanOpenConfirm(
        string calldata _portId,
        string calldata _channelId
    ) external virtual override onlyIBC {}

    function onChanCloseInit(
        string calldata _portId,
        string calldata _channelId
    ) external virtual override onlyIBC {
        revert RelayLib.ErrUnstoppable();
    }

    function onChanCloseConfirm(
        string calldata _portId,
        string calldata _channelId
    ) external virtual override onlyIBC {
        revert RelayLib.ErrUnstoppable();
    }
}
