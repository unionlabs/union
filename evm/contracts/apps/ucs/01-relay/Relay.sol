pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin-upgradeable/utils/PausableUpgradeable.sol";

import "@openzeppelin/token/ERC20/ERC20.sol";
import "@openzeppelin/token/ERC20/IERC20.sol";
import "@openzeppelin/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/utils/math/Math.sol";

import "solady/utils/LibString.sol";

import "solidity-stringutils/strings.sol";

import "../../../core/04-channel/IIBCPacket.sol";
import "../../../lib/Hex.sol";
import "../../Base.sol";
import "./IERC20Denom.sol";
import "./ERC20Denom.sol";

// NOTE: uint128 limitation from cosmwasm_std Coin type for transfers.
struct LocalToken {
    address denom;
    uint128 amount;
    uint128 fee;
}

struct Token {
    string denom;
    uint128 amount;
    uint128 fee;
}

struct RelayPacket {
    bytes sender;
    bytes receiver;
    Token[] tokens;
    string extension;
}

interface IRelay is IIBCModule {
    function getDenomAddress(
        uint32 sourceChannel,
        string memory denom
    ) external view returns (address);

    function getOutstanding(
        uint32 sourceChannel,
        address token
    ) external view returns (uint256);

    function send(
        uint32 sourceChannel,
        bytes calldata receiver,
        LocalToken[] calldata tokens,
        string calldata extension,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) external;
}

library RelayLib {
    using LibString for *;

    error ErrInvalidBytesAddress();
    error ErrUnauthorized();
    error ErrInvalidAcknowledgement();
    error ErrInvalidProtocolVersion();
    error ErrInvalidProtocolOrdering();
    error ErrInvalidCounterpartyProtocolVersion();
    error ErrInvalidAmount();
    error ErrUnstoppable();

    IBCChannelOrder public constant ORDER = IBCChannelOrder.Unordered;

    string public constant VERSION = "ucs01-relay-1";
    bytes1 public constant ACK_SUCCESS = 0x01;
    bytes1 public constant ACK_FAILURE = 0x00;
    uint256 public constant ACK_LENGTH = 1;

    event DenomCreated(
        uint64 indexed packetSequence,
        uint32 channelId,
        string denom,
        address token
    );
    event Received(
        uint64 packetSequence,
        uint32 channelId,
        string sender,
        address indexed receiver,
        string denom,
        address indexed token,
        uint256 amount
    );
    event FeePaid(
        uint64 packetSequence,
        uint32 channelId,
        string sender,
        address indexed receiver,
        string denom,
        address indexed token,
        uint256 amount
    );
    event Sent(
        uint64 packetSequence,
        uint32 channelId,
        address indexed sender,
        string receiver,
        string denom,
        address indexed token,
        uint256 amount
    );
    event Refunded(
        uint64 packetSequence,
        uint32 channelId,
        address indexed sender,
        string receiver,
        string denom,
        address indexed token,
        uint256 amount
    );

    function isValidVersion(
        string calldata version
    ) internal pure returns (bool) {
        return version.eq(VERSION);
    }

    function isFromChannel(
        uint32 channelId,
        string memory denom
    ) internal pure returns (bool) {
        return bytes(denom).length > 0
            && denom.startsWith(makeDenomPrefix(channelId));
    }

    function makeDenomPrefix(
        uint32 channelId
    ) internal pure returns (string memory) {
        return string(abi.encodePacked(channelId, "/"));
    }

    function makeForeignDenom(
        uint32 channelId,
        string memory denom
    ) internal pure returns (string memory) {
        return string(abi.encodePacked(makeDenomPrefix(channelId), denom));
    }

    function bytesToAddress(
        bytes memory b
    ) internal pure returns (address) {
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
        return abi.encode(
            packet.sender, packet.receiver, packet.tokens, packet.extension
        );
    }

    function decode(
        bytes calldata stream
    ) internal pure returns (RelayPacket calldata) {
        RelayPacket calldata packet;
        assembly {
            packet := stream.offset
        }
        return packet;
    }
}

contract UCS01Relay is
    IBCAppBase,
    IRelay,
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    PausableUpgradeable
{
    using RelayPacketLib for RelayPacket;
    using LibString for *;
    using strings for *;

    IIBCPacket private ibcHandler;

    // A mapping from remote denom to local ERC20 wrapper.
    mapping(uint32 => mapping(string => address)) private denomToAddress;
    // A mapping from a local ERC20 wrapper to the remote denom.
    // Required to determine whether an ERC20 token is originating from a remote chain.
    mapping(uint32 => mapping(address => string)) private addressToDenom;
    mapping(uint32 => mapping(address => uint256)) private outstanding;

    constructor() {
        _disableInitializers();
    }

    function initialize(
        IIBCPacket _ibcHandler,
        address admin
    ) public initializer {
        __Ownable_init(admin);
        ibcHandler = _ibcHandler;
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }

    // Return the ERC20 wrapper for the given remote-native denom.
    function getDenomAddress(
        uint32 sourceChannel,
        string memory denom
    ) external view override returns (address) {
        return denomToAddress[sourceChannel][denom];
    }

    // Return the amount of tokens submitted through the given port/channel.
    function getOutstanding(
        uint32 sourceChannel,
        address token
    ) external view override returns (uint256) {
        return outstanding[sourceChannel][token];
    }

    // Increase the oustanding amount on the given port/channel.
    // Happens when we send the token.
    function increaseOutstanding(
        uint32 sourceChannel,
        address token,
        uint256 amount
    ) internal {
        outstanding[sourceChannel][token] += amount;
    }

    // Decrease the outstanding amount on the given port/channel.
    // Happens either when receiving previously sent tokens or when refunding.
    function decreaseOutstanding(
        uint32 sourceChannel,
        address token,
        uint256 amount
    ) internal {
        outstanding[sourceChannel][token] -= amount;
    }

    // TODO: temporary entrypoint until the protocol gets extended to support this via the counterparty.
    function updateMetadata(
        IERC20Denom denom,
        string calldata newName,
        string calldata newSymbol,
        uint8 newDecimals
    ) public onlyOwner {
        IERC20Denom(denom).update(newName, newSymbol, newDecimals);
    }

    // Internal function
    // Send the given token over the specified channel.
    // If token is native, we increase the oustanding amount and escrow it. Otherwise, we burn the amount.
    // The operation is symmetric with the counterparty, if we burn locally, the remote relay will unescrow. If we escrow locally, the remote relay will mint.
    function sendToken(
        uint32 sourceChannel,
        LocalToken calldata localToken
    ) internal returns (string memory) {
        if (localToken.amount == 0) {
            revert RelayLib.ErrInvalidAmount();
        }
        // If the token is originating from the counterparty channel, we must have saved it's denom.
        string memory addressDenom =
            addressToDenom[sourceChannel][localToken.denom];
        if (bytes(addressDenom).length != 0) {
            // Token originating from the remote chain, burn the amount.
            IERC20Denom(localToken.denom).burn(msg.sender, localToken.amount);
        } else {
            // Ensure the user properly fund us.
            SafeERC20.safeTransferFrom(
                IERC20(localToken.denom),
                msg.sender,
                address(this),
                localToken.amount
            );
            // Token originating from the local chain, increase outstanding and escrow the amount.
            increaseOutstanding(
                sourceChannel, localToken.denom, localToken.amount
            );
            addressDenom = localToken.denom.toHexString();
        }
        return addressDenom;
    }

    function send(
        uint32 sourceChannel,
        bytes calldata receiver,
        LocalToken[] calldata tokens,
        string calldata extension,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp
    ) external override {
        Token[] memory normalizedTokens = new Token[](tokens.length);
        uint256 tokensLength = tokens.length;
        for (uint256 i; i < tokensLength; i++) {
            LocalToken calldata localToken = tokens[i];
            normalizedTokens[i].denom = sendToken(sourceChannel, localToken);
            normalizedTokens[i].amount = localToken.amount;
            normalizedTokens[i].fee = localToken.fee;
        }
        RelayPacket memory packet = RelayPacket({
            sender: abi.encodePacked(msg.sender),
            receiver: receiver,
            tokens: normalizedTokens,
            extension: extension
        });
        uint64 packetSequence = ibcHandler.sendPacket(
            sourceChannel, timeoutHeight, timeoutTimestamp, packet.encode()
        );
        for (uint256 i; i < tokensLength; i++) {
            LocalToken calldata localToken = tokens[i];
            Token memory normalizedToken = normalizedTokens[i];
            emit RelayLib.Sent(
                packetSequence,
                sourceChannel,
                msg.sender,
                receiver.toHexString(),
                normalizedToken.denom,
                localToken.denom,
                uint256(localToken.amount)
            );
        }
    }

    function refundTokens(
        uint64 sequence,
        uint32 channelId,
        RelayPacket calldata packet
    ) internal {
        string memory receiver = packet.receiver.toHexString();
        // We're going to refund, the receiver will be the sender.
        address userToRefund = RelayLib.bytesToAddress(packet.sender);
        uint256 packetTokensLength = packet.tokens.length;
        for (uint256 i; i < packetTokensLength; i++) {
            Token memory token = packet.tokens[i];
            address denomAddress = denomToAddress[channelId][token.denom];
            if (denomAddress != address(0)) {
                // The token was originating from the remote chain, we burnt it.
                // Refund means minting in this case.
                IERC20Denom(denomAddress).mint(userToRefund, token.amount);
            } else {
                // The token was originating from the local chain, we escrowed
                // it. Refund means unescrowing.

                // It's an ERC20 string 0x prefixed hex address
                denomAddress = Hex.hexToAddress(token.denom);
                decreaseOutstanding(channelId, denomAddress, token.amount);
                IERC20(denomAddress).transfer(userToRefund, token.amount);
            }
            emit RelayLib.Refunded(
                sequence,
                channelId,
                userToRefund,
                receiver,
                token.denom,
                denomAddress,
                token.amount
            );
        }
    }

    function calculateFee(
        uint256 amount,
        uint128 fee
    ) internal pure returns (uint256, uint256) {
        uint256 feeAmount = Math.mulDiv(amount, fee, type(uint128).max);
        uint256 actualAmount = amount - feeAmount;
        return (actualAmount, feeAmount);
    }

    function onRecvLocalTransfer(
        uint32 destinationChannel,
        string memory denom,
        address receiver,
        uint256 amount,
        address relayer,
        uint256 feeAmount
    ) internal returns (address) {
        // It's an ERC20 string 0x prefixed hex address
        address denomAddress = Hex.hexToAddress(denom);
        // The token must be outstanding.
        decreaseOutstanding(destinationChannel, denomAddress, amount);
        IERC20(denomAddress).transfer(receiver, amount);
        if (feeAmount > 0) {
            IERC20(denomAddress).transfer(relayer, feeAmount);
        }
        return denomAddress;
    }

    function onRecvRemoteTransfer(
        uint64 sequence,
        uint32 destinationChannel,
        string memory denom,
        address receiver,
        uint256 amount,
        address relayer,
        uint256 feeAmount
    ) internal returns (address) {
        address denomAddress = denomToAddress[destinationChannel][denom];
        if (denomAddress == address(0)) {
            denomAddress =
                address(new ERC20Denom{salt: keccak256(bytes(denom))}(denom));
            denomToAddress[destinationChannel][denom] = denomAddress;
            addressToDenom[destinationChannel][denomAddress] = denom;
            emit RelayLib.DenomCreated(
                sequence, destinationChannel, denom, denomAddress
            );
        }
        IERC20Denom(denomAddress).mint(receiver, amount);
        if (feeAmount > 0) {
            IERC20Denom(denomAddress).mint(relayer, feeAmount);
        }
        return denomAddress;
    }

    function onRecvPacketProcessing(
        IBCPacket calldata ibcPacket,
        address relayer
    ) public {
        if (msg.sender != address(this)) {
            revert RelayLib.ErrUnauthorized();
        }
        RelayPacket calldata packet = RelayPacketLib.decode(ibcPacket.data);
        string memory prefix = RelayLib.makeDenomPrefix(ibcPacket.sourceChannel);
        uint256 packetTokensLength = packet.tokens.length;
        for (uint256 i; i < packetTokensLength; i++) {
            Token memory token = packet.tokens[i];
            if (token.amount == 0) {
                revert RelayLib.ErrInvalidAmount();
            }
            (uint256 actualAmount, uint256 feeAmount) =
                calculateFee(token.amount, token.fee);
            address receiver = RelayLib.bytesToAddress(packet.receiver);
            address denomAddress;
            string memory denom;
            if (token.denom.startsWith(prefix)) {
                // In this branch the token was originating from
                // this chain as it was prefixed by the remote channel/port.
                // We need to unescrow the amount.
                denom = token.denom.slice(bytes(prefix).length);
                denomAddress = onRecvLocalTransfer(
                    ibcPacket.destinationChannel,
                    denom,
                    receiver,
                    actualAmount,
                    relayer,
                    feeAmount
                );
            } else {
                // In this branch the token was originating from the
                // counterparty chain. We need to prefix the denom and mint the amount.
                denom = RelayLib.makeForeignDenom(
                    ibcPacket.destinationChannel, token.denom
                );
                denomAddress = onRecvRemoteTransfer(
                    ibcPacket.sequence,
                    ibcPacket.destinationChannel,
                    denom,
                    receiver,
                    actualAmount,
                    relayer,
                    feeAmount
                );
            }
            string memory senderAddress = packet.sender.toHexString();
            emit RelayLib.Received(
                ibcPacket.sequence,
                ibcPacket.destinationChannel,
                senderAddress,
                receiver,
                denom,
                denomAddress,
                actualAmount
            );
            if (feeAmount > 0) {
                emit RelayLib.FeePaid(
                    ibcPacket.sequence,
                    ibcPacket.destinationChannel,
                    senderAddress,
                    relayer,
                    denom,
                    denomAddress,
                    feeAmount
                );
            }
        }
    }

    function onRecvPacket(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata
    )
        external
        override(IBCAppBase, IIBCModule)
        onlyIBC
        returns (bytes memory)
    {
        // TODO: maybe consider threading _res in the failure ack
        (bool success,) = address(this).call(
            abi.encodeWithSelector(
                this.onRecvPacketProcessing.selector, ibcPacket, relayer
            )
        );
        // We make sure not to revert to allow the failure ack to be sent back,
        // resulting in a refund.
        if (success) {
            return abi.encodePacked(RelayLib.ACK_SUCCESS);
        } else {
            return abi.encodePacked(RelayLib.ACK_FAILURE);
        }
    }

    function onAcknowledgementPacket(
        IBCPacket calldata ibcPacket,
        bytes calldata acknowledgement,
        address
    ) external override(IBCAppBase, IIBCModule) onlyIBC {
        if (
            acknowledgement.length != RelayLib.ACK_LENGTH
                || (
                    acknowledgement[0] != RelayLib.ACK_FAILURE
                        && acknowledgement[0] != RelayLib.ACK_SUCCESS
                )
        ) {
            revert RelayLib.ErrInvalidAcknowledgement();
        }
        // Counterparty failed to execute the transfer, we refund.
        if (acknowledgement[0] == RelayLib.ACK_FAILURE) {
            refundTokens(
                ibcPacket.sequence,
                ibcPacket.sourceChannel,
                RelayPacketLib.decode(ibcPacket.data)
            );
        }
    }

    function onTimeoutPacket(
        IBCPacket calldata ibcPacket,
        address
    ) external override(IBCAppBase, IIBCModule) onlyIBC {
        refundTokens(
            ibcPacket.sequence,
            ibcPacket.sourceChannel,
            RelayPacketLib.decode(ibcPacket.data)
        );
    }

    function onChanOpenInit(
        IBCChannelOrder order,
        uint32,
        uint32,
        string calldata version,
        address
    ) external view override(IBCAppBase, IIBCModule) onlyIBC {
        if (!RelayLib.isValidVersion(version)) {
            revert RelayLib.ErrInvalidProtocolVersion();
        }
        if (order != RelayLib.ORDER) {
            revert RelayLib.ErrInvalidProtocolOrdering();
        }
    }

    function onChanOpenTry(
        IBCChannelOrder order,
        uint32,
        uint32,
        uint32,
        string calldata version,
        string calldata counterpartyVersion,
        address
    ) external view override(IBCAppBase, IIBCModule) onlyIBC {
        if (!RelayLib.isValidVersion(version)) {
            revert RelayLib.ErrInvalidProtocolVersion();
        }
        if (order != RelayLib.ORDER) {
            revert RelayLib.ErrInvalidProtocolOrdering();
        }
        if (!RelayLib.isValidVersion(counterpartyVersion)) {
            revert RelayLib.ErrInvalidCounterpartyProtocolVersion();
        }
    }

    function onChanOpenAck(
        uint32,
        uint32,
        string calldata counterpartyVersion,
        address
    ) external view override(IBCAppBase, IIBCModule) onlyIBC {
        if (!RelayLib.isValidVersion(counterpartyVersion)) {
            revert RelayLib.ErrInvalidCounterpartyProtocolVersion();
        }
    }

    function onChanCloseInit(
        uint32,
        address
    ) external view override(IBCAppBase, IIBCModule) onlyIBC {
        revert RelayLib.ErrUnstoppable();
    }

    function onChanCloseConfirm(
        uint32,
        address
    ) external view override(IBCAppBase, IIBCModule) onlyIBC {
        revert RelayLib.ErrUnstoppable();
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}
}
