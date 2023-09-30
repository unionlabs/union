pragma solidity ^0.8.18;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/utils/math/SafeMath.sol";
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
    string sender;
    string receiver;
    Token[] tokens;
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
            string memory sender,
            string memory receiver,
            Token[] memory tokens
        ) = abi.decode(packet, (string, string, Token[]));
        return
            RelayPacket({sender: sender, receiver: receiver, tokens: tokens});
    }
}

contract UCS01Relay is IBCAppBase {
    using RelayPacketLib for RelayPacket;
    using LibString for *;
    using strings for *;
    using SafeMath for uint256;

    bytes1 constant ACK_SUCCESS = 0x01;
    bytes1 constant ACK_FAILURE = 0x00;
    uint256 constant ACK_LENGTH = 1;

    IBCHandler private immutable ibcHandler;

    mapping(string => address) public denomToAddress;
    mapping(address => string) public addressToDenom;
    mapping(string => mapping(string => IbcCoreChannelV1Counterparty.Data))
        public counterpartyEndpoints;
    mapping(string => mapping(string => mapping(address => uint256)))
        public outstanding;
    mapping(string => mapping(string => mapping(address => uint256)))
        public inFlight;

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

    constructor(IBCHandler _ibcHandler) {
        ibcHandler = _ibcHandler;
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }

    // It expect 0x.. prefix
    function hexToAddress(
        string memory _a
    ) internal pure returns (address _parsedAddress) {
        bytes memory tmp = bytes(_a);
        uint160 iaddr = 0;
        uint160 b1;
        uint160 b2;
        for (uint256 i = 2; i < 2 + 2 * 20; i += 2) {
            iaddr *= 256;
            b1 = uint160(uint8(tmp[i]));
            b2 = uint160(uint8(tmp[i + 1]));
            if ((b1 >= 97) && (b1 <= 102)) {
                b1 -= 87;
            } else if ((b1 >= 65) && (b1 <= 70)) {
                b1 -= 55;
            } else if ((b1 >= 48) && (b1 <= 57)) {
                b1 -= 48;
            }
            if ((b2 >= 97) && (b2 <= 102)) {
                b2 -= 87;
            } else if ((b2 >= 65) && (b2 <= 70)) {
                b2 -= 55;
            } else if ((b2 >= 48) && (b2 <= 57)) {
                b2 -= 48;
            }
            iaddr += (b1 * 16 + b2);
        }
        return address(iaddr);
    }

    function makeDenomPrefix(
        string memory portId,
        string memory channelId
    ) public view returns (string memory) {
        return string(abi.encodePacked(portId, "/", channelId, "/"));
    }

    function makeForeignDenom(
        string memory portId,
        string memory channelId,
        string memory denom
    ) public view returns (string memory) {
        return
            string(abi.encodePacked(makeDenomPrefix(portId, channelId), denom));
    }

    function increaseOutstanding(
        string memory portId,
        string memory channelId,
        address token,
        uint256 amount
    ) internal {
        outstanding[portId][channelId][token] = outstanding[portId][channelId][
            token
        ].add(amount);
    }

    function decreaseOutstanding(
        string memory portId,
        string memory channelId,
        address token,
        uint256 amount
    ) internal {
        outstanding[portId][channelId][token] = outstanding[portId][channelId][
            token
        ].sub(amount);
    }

    function increaseInFlight(
        string memory portId,
        string memory channelId,
        address token,
        uint256 amount
    ) internal {
        inFlight[portId][channelId][token] = inFlight[portId][channelId][token]
            .add(amount);
    }

    function decreaseInFlight(
        string memory portId,
        string memory channelId,
        address token,
        uint256 amount
    ) internal {
        inFlight[portId][channelId][token] = inFlight[portId][channelId][token]
            .sub(amount);
    }

    function send(
        string calldata portId,
        string calldata channelId,
        string calldata receiver,
        LocalToken[] calldata tokens,
        uint64 counterpartyTimeoutRevisionNumber,
        uint64 counterpartyTimeoutRevisionHeight
    ) public {
        IbcCoreChannelV1Counterparty.Data
            memory counterparty = counterpartyEndpoints[portId][channelId];
        Token[] memory normalizedTokens = new Token[](tokens.length);
        // For each token, we transfer them locally then:
        // - if the token is locally native, keep it escrowed
        // - if the token is remote native, burn the wrapper
        for (uint256 i = 0; i < tokens.length; i++) {
            LocalToken calldata localToken = tokens[i];
            SafeERC20.safeTransferFrom(
                IERC20(localToken.denom),
                msg.sender,
                address(this),
                localToken.amount
            );
            string memory addressDenom = addressToDenom[localToken.denom];
            if (bytes(addressDenom).length != 0) {
                IERC20Denom(localToken.denom).burn(
                    address(this),
                    localToken.amount
                );
            } else {
                increaseInFlight(
                    portId,
                    channelId,
                    localToken.denom,
                    localToken.amount
                );
                addressDenom = localToken.denom.toHexString();
            }
            normalizedTokens[i].denom = addressDenom;
            normalizedTokens[i].amount = uint256(localToken.amount);
            emit Sent(
                msg.sender,
                receiver,
                addressDenom,
                localToken.denom,
                uint256(localToken.amount)
            );
        }
        RelayPacket memory packet = RelayPacket({
            sender: string(abi.encodePacked(msg.sender)),
            receiver: receiver,
            tokens: normalizedTokens
        });
        ibcHandler.sendPacket(
            portId,
            channelId,
            IbcCoreClientV1Height.Data({
                revision_number: counterpartyTimeoutRevisionNumber,
                revision_height: counterpartyTimeoutRevisionHeight
            }),
            0,
            packet.encode()
        );
    }

    function refundTokens(
        string memory portId,
        string memory channelId,
        RelayPacket memory packet
    ) internal {
        // We're going to refund, the receiver will be the sender.
        address receiver = hexToAddress(packet.sender);
        for (uint256 i = 0; i < packet.tokens.length; i++) {
            Token memory token = packet.tokens[i];
            // Either we tried to send back a remote native token
            // which we burnt, or a locally native token that we escrowed.
            address denomAddress = denomToAddress[token.denom];
            if (denomAddress != address(0)) {
                IERC20Denom(denomAddress).mint(receiver, token.amount);
            } else {
                // It must be in the form 0x...
                denomAddress = hexToAddress(token.denom);
                // The token must be in-flight
                decreaseInFlight(portId, channelId, denomAddress, token.amount);
                IERC20(denomAddress).transfer(receiver, token.amount);
            }
        }
    }

    // We received a successful ack, move tokens from in-flight to outstanding.
    function tokensLanded(
        string memory portId,
        string memory channelId,
        RelayPacket memory packet
    ) internal {
        for (uint256 i = 0; i < packet.tokens.length; i++) {
            Token memory token = packet.tokens[i];
            // For local tokens only as remote tokens are burnt.
            if (token.denom.toSlice().startsWith("0x".toSlice())) {
                address denomAddress = hexToAddress(token.denom);
                decreaseInFlight(portId, channelId, denomAddress, token.amount);
                increaseOutstanding(
                    portId,
                    channelId,
                    denomAddress,
                    token.amount
                );
            }
        }
    }

    function onRecvPacketProcessing(
        IbcCoreChannelV1Packet.Data calldata ibcPacket,
        address relayer
    ) public {
        require(msg.sender == address(this));
        RelayPacket memory packet = RelayPacketLib.decode(ibcPacket.data);
        string memory prefix = makeDenomPrefix(
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
            address receiver = hexToAddress(packet.receiver);
            address denomAddress;
            string memory denom;
            if (!denomSlice.equals(trimedDenom)) {
                denom = trimedDenom.toString();
                denomAddress = hexToAddress(denom);
                // The token must be outstanding.
                decreaseOutstanding(
                    ibcPacket.destination_port,
                    ibcPacket.destination_channel,
                    denomAddress,
                    token.amount
                );
                IERC20(denomAddress).transfer(receiver, token.amount);
            } else {
                denom = makeForeignDenom(
                    ibcPacket.source_port,
                    ibcPacket.source_channel,
                    token.denom
                );
                denomAddress = denomToAddress[denom];
                if (denomAddress == address(0)) {
                    denomAddress = address(new ERC20Denom(denom));
                    denomToAddress[denom] = denomAddress;
                    addressToDenom[denomAddress] = denom;
                    emit DenomCreated(denom, denomAddress);
                }
                IERC20Denom(denomAddress).mint(receiver, token.amount);
            }
            emit Received(
                packet.sender,
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
    ) external virtual override onlyIBC returns (bytes memory acknowledgement) {
        // TODO: maybe consider threading _res in the failure ack
        (bool success, bytes memory _res) = address(this).call(
            abi.encodeWithSelector(
                this.onRecvPacketProcessing.selector,
                ibcPacket,
                relayer
            )
        );
        if (success) {
            return abi.encodePacked(ACK_SUCCESS);
        } else {
            return abi.encodePacked(ACK_FAILURE);
        }
    }

    function onAcknowledgementPacket(
        IbcCoreChannelV1Packet.Data calldata ibcPacket,
        bytes calldata acknowledgement,
        address _relayer
    ) external virtual override onlyIBC {
        require(
            acknowledgement.length == ACK_LENGTH,
            "ucs01-relay: single byte ack"
        );
        RelayPacket memory packet = RelayPacketLib.decode(ibcPacket.data);
        if (acknowledgement[0] == ACK_SUCCESS) {
            tokensLanded(
                ibcPacket.source_port,
                ibcPacket.source_channel,
                packet
            );
        } else {
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
        IbcCoreChannelV1GlobalEnums.Order _order,
        string[] calldata _connectionHops,
        string calldata portId,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata counterpartyEndpoint,
        string calldata _version
    ) external virtual override onlyIBC {
        counterpartyEndpoints[portId][channelId] = counterpartyEndpoint;
    }

    function onChanOpenTry(
        IbcCoreChannelV1GlobalEnums.Order _order,
        string[] calldata _connectionHops,
        string calldata portId,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata counterpartyEndpoint,
        string calldata _version,
        string calldata _counterpartyVersion
    ) external virtual override onlyIBC {
        counterpartyEndpoints[portId][channelId] = counterpartyEndpoint;
    }

    function onChanOpenAck(
        string calldata _portId,
        string calldata _channelId,
        string calldata _counterpartyVersion
    ) external virtual override onlyIBC {}

    function onChanOpenConfirm(
        string calldata _portId,
        string calldata _channelId
    ) external virtual override onlyIBC {}

    function onChanCloseInit(
        string calldata _portId,
        string calldata _channelId
    ) external virtual override onlyIBC {
        revert("ucs01-relay: closing a channel is not supported");
    }

    function onChanCloseConfirm(
        string calldata _portId,
        string calldata _channelId
    ) external virtual override onlyIBC {
        revert("ucs01-relay: closing a channel is not supported");
    }
}
