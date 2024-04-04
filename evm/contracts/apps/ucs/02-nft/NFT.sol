pragma solidity ^0.8.23;

import "@openzeppelin/token/ERC721/ERC721.sol";
import "@openzeppelin/token/ERC721/IERC721.sol";
import "solady/utils/LibString.sol";
import "solidity-stringutils/strings.sol";
import "solidity-bytes-utils/BytesLib.sol";
import "../../../core/25-handler/IBCHandler.sol";
import "../../../core/02-client/IBCHeight.sol";
import "../../../lib/Hex.sol";
import "../../Base.sol";
import "./IERC721Denom.sol";
import "./ERC721Denom.sol";

struct NFTPacket {
    string classId;
    string classUri;
    bytes classData;
    string[] tokenIds;
    string[] tokenUris;
    bytes[] tokenData;
    string sender;
    string receiver;
    string memo;
}

library NFTPacketLib {
    function encode(NFTPacket memory packet)
        internal
        pure
        returns (bytes memory)
    {
        return abi.encode(
            packet.classId,
            packet.classUri,
            packet.classData,
            packet.tokenIds,
            packet.tokenUris,
            packet.tokenData,
            packet.sender,
            packet.receiver,
            packet.memo
        );
    }

    function decode(bytes calldata stream)
        internal
        pure
        returns (NFTPacket calldata)
    {
        NFTPacket calldata packet;
        assembly {
            packet := stream.offset
        }
        return packet;
    }
}

library NFTLib {
    using LibString for *;

    error ErrInvalidBytesAddress();
    error ErrUnauthorized();
    error ErrInvalidAcknowledgement();
    error ErrInvalidProtocolVersion();
    error ErrInvalidProtocolOrdering();
    error ErrInvalidCounterpartyProtocolVersion();
    error ErrUnstoppable();

    IbcCoreChannelV1GlobalEnums.Order public constant ORDER =
        IbcCoreChannelV1GlobalEnums.Order.ORDER_UNORDERED;
    string public constant VERSION = "ucs02-nft-1";
    bytes1 public constant ACK_SUCCESS = 0x01;
    bytes1 public constant ACK_FAILURE = 0x00;
    uint256 public constant ACK_LENGTH = 1;

    event ClassCreated(
        uint64 packetSequence, string channelId, address nftClass
    );
    event Received(
        uint64 packetSequence,
        string channelId,
        string sender,
        address receiver,
        address nftClass,
        string[] tokenIds
    );
    event Sent(
        uint64 packetSequence,
        string channelId,
        address sender,
        string receiver,
        address nftClass,
        string[] tokenIds
    );
    event Refunded(
        uint64 packetSequence,
        string channelId,
        address sender,
        string receiver,
        address nftClass,
        string[] tokenIds
    );

    function isValidVersion(string memory version)
        internal
        pure
        returns (bool)
    {
        return version.eq(VERSION);
    }

    function isFromChannel(
        string memory portId,
        string memory channelId,
        string memory nftDenom
    ) internal pure returns (bool) {
        return bytes(nftDenom).length > 0
            && nftDenom.startsWith(makeDenomPrefix(portId, channelId));
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
        string memory nftDenom
    ) internal pure returns (string memory) {
        return string(
            abi.encodePacked(makeDenomPrefix(portId, channelId), nftDenom)
        );
    }

    function bytesToAddress(bytes memory b) internal pure returns (address) {
        if (b.length != 20) {
            revert ErrInvalidBytesAddress();
        }
        return address(uint160(bytes20(b)));
    }
}

contract UCS02NFT is IBCAppBase {
    using LibString for *;
    using BytesLib for *;
    using strings for *;
    using NFTPacketLib for *;

    IBCHandler private immutable ibcHandler;

    // A mapping from remote denom to local ERC721 wrapper.
    mapping(string => mapping(string => mapping(string => address))) private
        denomToNft;
    // A mapping from a local ERC721 wrapper to the remote denom.
    // Required to determine whether an ERC721 token is originating from a remote chain.
    mapping(string => mapping(string => mapping(address => string))) private
        nftToDenom;
    mapping(
        string
            => mapping(string => mapping(address => mapping(uint256 => string)))
    ) private nftTokenToId;
    mapping(
        string
            => mapping(string => mapping(address => mapping(string => uint256)))
    ) private nftIdToToken;
    // A mapping from local port/channel to it's counterparty.
    // This is required to remap denoms.
    mapping(string => mapping(string => IbcCoreChannelV1Counterparty.Data))
        private counterpartyEndpoints;
    mapping(string => mapping(string => mapping(address => uint256))) private
        outstanding;

    constructor(IBCHandler _ibcHandler) {
        ibcHandler = _ibcHandler;
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }

    // Return a channel counterparty endpoint.
    // A counterparty will exist only if a channel has been previously opened.
    function getCounterpartyEndpoint(
        string memory sourcePort,
        string memory sourceChannel
    ) external view returns (IbcCoreChannelV1Counterparty.Data memory) {
        return counterpartyEndpoints[sourcePort][sourceChannel];
    }

    // Return the amount of tokens submitted through the given port/channel.
    function getOutstanding(
        string memory sourcePort,
        string memory sourceChannel,
        address token
    ) external view returns (uint256) {
        return outstanding[sourcePort][sourceChannel][token];
    }

    // Increase the oustanding amount on the given port/channel.
    // Happens when we send the token.
    function increaseOutstanding(
        string memory sourcePort,
        string memory sourceChannel,
        address nftClass,
        uint256 amount
    ) internal {
        outstanding[sourcePort][sourceChannel][nftClass] += amount;
    }

    // Decrease the outstanding amount on the given port/channel.
    // Happens either when receiving previously sent tokens or when refunding.
    function decreaseOutstanding(
        string memory sourcePort,
        string memory sourceChannel,
        address nftClass,
        uint256 amount
    ) internal {
        outstanding[sourcePort][sourceChannel][nftClass] -= amount;
    }

    function sendRemoteNative(
        string calldata sourcePort,
        string calldata sourceChannel,
        string calldata receiver,
        address nftClass,
        uint256[] calldata tokens
    ) internal returns (string[] memory) {
        uint256 tokensLength = tokens.length;
        string[] memory normalizedTokens = new string[](tokens.length);
        for (uint256 i = 0; i < tokensLength; i++) {
            uint256 tokenId = tokens[i];
            // we could avoid transferring before burning here?
            IERC721Denom(nftClass).safeTransferFrom(
                msg.sender, address(this), tokenId
            );
            IERC721Denom(nftClass).burn(tokenId);
            normalizedTokens[i] =
                nftTokenToId[sourcePort][sourceChannel][nftClass][tokenId];
        }
        return normalizedTokens;
    }

    function sendLocalNative(
        string calldata sourcePort,
        string calldata sourceChannel,
        string calldata receiver,
        address nftClass,
        uint256[] calldata tokens
    ) internal returns (string[] memory) {
        uint256 tokensLength = tokens.length;
        string[] memory normalizedTokens = new string[](tokens.length);
        increaseOutstanding(sourcePort, sourceChannel, nftClass, tokensLength);
        for (uint256 i = 0; i < tokensLength; i++) {
            uint256 tokenId = tokens[i];
            IERC721Denom(nftClass).safeTransferFrom(
                msg.sender, address(this), tokenId
            );
            normalizedTokens[i] = tokenId.toHexString();
        }
        return normalizedTokens;
    }

    function send(
        string calldata sourcePort,
        string calldata sourceChannel,
        string calldata receiver,
        address nftClass,
        uint256[] calldata tokens,
        uint64 timeoutTimestamp
    ) external {
        // If the token is originating from the counterparty channel, we must have saved it's denom.
        string memory nftDenom = nftToDenom[sourcePort][sourceChannel][nftClass];
        bool isSource = bytes(nftDenom).length == 0;
        string[] memory normalizedTokens;
        if (isSource) {
            nftDenom = nftClass.toHexString();
            normalizedTokens = sendLocalNative(
                sourcePort, sourceChannel, receiver, nftClass, tokens
            );
        } else {
            normalizedTokens = sendRemoteNative(
                sourcePort, sourceChannel, receiver, nftClass, tokens
            );
        }

        string memory sender = msg.sender.toHexString();

        uint64 packetSequence = ibcHandler.sendPacket(
            sourcePort,
            sourceChannel,
            IBCHeight.zero(),
            timeoutTimestamp,
            NFTPacket({
                classId: nftDenom,
                classUri: "",
                classData: hex"",
                tokenIds: normalizedTokens,
                tokenUris: new string[](0),
                tokenData: new bytes[](0),
                sender: sender,
                receiver: receiver,
                memo: ""
            }).encode()
        );

        emit NFTLib.Sent(
            packetSequence,
            sourceChannel,
            msg.sender,
            receiver,
            nftClass,
            normalizedTokens
        );
    }

    function onRecvPacket(
        IbcCoreChannelV1Packet.Data calldata ibcPacket,
        address relayer
    ) external override onlyIBC returns (bytes memory) {
        // TODO: maybe consider threading _res in the failure ack
        (bool success, bytes memory _res) = address(this).call(
            abi.encodeWithSelector(
                this.onRecvPacketProcessing.selector, ibcPacket, relayer
            )
        );
        // We make sure not to revert to allow the failure ack to be sent back,
        // resulting in a refund.
        if (success) {
            return abi.encodePacked(NFTLib.ACK_SUCCESS);
        } else {
            return abi.encodePacked(NFTLib.ACK_FAILURE);
        }
    }

    function receiveRemoteNative(
        IbcCoreChannelV1Packet.Data calldata ibcPacket,
        NFTPacket calldata packet,
        address receiver,
        string memory nftDenom
    ) internal returns (address) {
        address nftClass = denomToNft[ibcPacket.destination_port][ibcPacket
            .destination_channel][nftDenom];
        if (nftClass == address(0)) {
            nftClass = address(new ERC721Denom(nftDenom));
            denomToNft[ibcPacket.destination_port][ibcPacket.destination_channel][nftDenom]
            = nftClass;
            nftToDenom[ibcPacket.destination_port][ibcPacket.destination_channel][nftClass]
            = nftDenom;
            emit NFTLib.ClassCreated(
                ibcPacket.sequence, ibcPacket.source_channel, nftClass
            );
        }
        uint256 tokenIdsLength = packet.tokenIds.length;
        for (uint256 i = 0; i < tokenIdsLength; i++) {
            string memory tokenId = packet.tokenIds[i];
            uint256 localTokenId = uint256(keccak256(bytes(tokenId)));
            nftIdToToken[ibcPacket.destination_port][ibcPacket
                .destination_channel][nftClass][tokenId] = localTokenId;
            IERC721Denom(nftClass).mint(receiver, localTokenId);
        }
        return nftClass;
    }

    function receiveLocalNative(
        IbcCoreChannelV1Packet.Data calldata ibcPacket,
        NFTPacket calldata packet,
        address receiver,
        string memory nftDenom
    ) internal returns (address) {
        address nftClass = Hex.hexToAddress(nftDenom);
        uint256 tokenIdsLength = packet.tokenIds.length;
        decreaseOutstanding(
            ibcPacket.destination_port,
            ibcPacket.destination_channel,
            nftClass,
            tokenIdsLength
        );
        for (uint256 i = 0; i < tokenIdsLength; i++) {
            string memory tokenId = packet.tokenIds[i];
            uint256 localTokenId = Hex.hexToUint256(packet.tokenIds[i]);
            IERC721Denom(nftClass).safeTransferFrom(
                address(this), receiver, localTokenId
            );
        }
        return nftClass;
    }

    function onRecvPacketProcessing(
        IbcCoreChannelV1Packet.Data calldata ibcPacket,
        address relayer
    ) public {
        if (msg.sender != address(this)) {
            revert NFTLib.ErrUnauthorized();
        }
        NFTPacket calldata packet = NFTPacketLib.decode(ibcPacket.data);
        // {src_port}/{src_channel}/denom
        // This will trim the denom in-place IFF it is prefixed
        strings.slice memory trimedClassId = packet.classId.toSlice().beyond(
            NFTLib.makeDenomPrefix(
                ibcPacket.destination_port, ibcPacket.destination_channel
            ).toSlice()
        );
        address receiver = Hex.hexToAddress(packet.receiver);
        address nftClass;
        if (trimedClassId.equals(packet.classId.toSlice())) {
            // In this branch the token was originating from the
            // counterparty chain. We need to mint the amount.
            string memory nftDenom = NFTLib.makeForeignDenom(
                ibcPacket.source_port, ibcPacket.source_channel, packet.classId
            );
            receiveRemoteNative(ibcPacket, packet, receiver, nftDenom);
        } else {
            // The NFT was originating from the local chain, it's class is a hex representation of the ERC721 address.
            receiveLocalNative(
                ibcPacket, packet, receiver, trimedClassId.toString()
            );
        }
        emit NFTLib.Received(
            ibcPacket.sequence,
            ibcPacket.source_channel,
            packet.sender,
            receiver,
            nftClass,
            packet.tokenIds
        );
    }

    function onAcknowledgementPacket(
        IbcCoreChannelV1Packet.Data calldata ibcPacket,
        bytes calldata acknowledgement,
        address _relayer
    ) external override onlyIBC {
        if (
            acknowledgement.length != NFTLib.ACK_LENGTH
                || (
                    acknowledgement[0] != NFTLib.ACK_FAILURE
                        && acknowledgement[0] != NFTLib.ACK_SUCCESS
                )
        ) {
            revert NFTLib.ErrInvalidAcknowledgement();
        }
        // Counterparty failed to execute the transfer, we refund.
        if (acknowledgement[0] == NFTLib.ACK_FAILURE) {
            refundTokens(
                ibcPacket.sequence,
                ibcPacket.source_port,
                ibcPacket.source_channel,
                NFTPacketLib.decode(ibcPacket.data)
            );
        }
    }

    function refundTokens(
        uint64 sequence,
        string memory portId,
        string memory channelId,
        NFTPacket calldata packet
    ) internal {
        // We're going to refund, the receiver will be the sender.
        address userToRefund = Hex.hexToAddress(packet.sender);
        // The nft class must exist as we previously created it.
        // If it does not, it means it was a originating from the local chain.
        address nftClass = denomToNft[portId][channelId][packet.classId];
        if (nftClass != address(0)) {
            uint256 tokenIdsLength = packet.tokenIds.length;
            for (uint256 i = 0; i < tokenIdsLength; i++) {
                // The tokenId must exist as we previously created it along the nftClass.
                uint256 tokenId = nftIdToToken[portId][channelId][nftClass][packet
                    .tokenIds[i]];
                // The token was originating from the remote chain, we burnt it.
                // Refund means minting in this case.
                IERC721Denom(nftClass).mint(userToRefund, tokenId);
            }
        } else {
            uint256 tokenIdsLength = packet.tokenIds.length;
            decreaseOutstanding(portId, channelId, nftClass, tokenIdsLength);
            for (uint256 i = 0; i < tokenIdsLength; i++) {
                // The token was originating from the local chain, we escrowed
                // it. Refund means unescrowing.
                // It's an ERC721 tokenId
                uint256 tokenId = Hex.hexToUint256(packet.tokenIds[i]);
                IERC721(nftClass).safeTransferFrom(
                    address(this), userToRefund, tokenId
                );
            }
        }
        emit NFTLib.Refunded(
            sequence,
            channelId,
            userToRefund,
            packet.receiver,
            nftClass,
            packet.tokenIds
        );
    }

    function onTimeoutPacket(
        IbcCoreChannelV1Packet.Data calldata ibcPacket,
        address _relayer
    ) external override onlyIBC {
        refundTokens(
            ibcPacket.sequence,
            ibcPacket.source_port,
            ibcPacket.source_channel,
            NFTPacketLib.decode(ibcPacket.data)
        );
    }

    function onChanOpenInit(
        IbcCoreChannelV1GlobalEnums.Order order,
        string[] calldata _connectionHops,
        string calldata portId,
        string calldata channelId,
        IbcCoreChannelV1Counterparty.Data calldata counterpartyEndpoint,
        string calldata version
    ) external override onlyIBC {
        if (!NFTLib.isValidVersion(version)) {
            revert NFTLib.ErrInvalidProtocolVersion();
        }
        if (order != NFTLib.ORDER) {
            revert NFTLib.ErrInvalidProtocolOrdering();
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
    ) external override onlyIBC {
        if (!NFTLib.isValidVersion(version)) {
            revert NFTLib.ErrInvalidProtocolVersion();
        }
        if (order != NFTLib.ORDER) {
            revert NFTLib.ErrInvalidProtocolOrdering();
        }
        if (!NFTLib.isValidVersion(counterpartyVersion)) {
            revert NFTLib.ErrInvalidCounterpartyProtocolVersion();
        }
        counterpartyEndpoints[portId][channelId] = counterpartyEndpoint;
    }

    function onChanOpenAck(
        string calldata portId,
        string calldata channelId,
        string calldata counterpartyChannelId,
        string calldata counterpartyVersion
    ) external override onlyIBC {
        if (!NFTLib.isValidVersion(counterpartyVersion)) {
            revert NFTLib.ErrInvalidCounterpartyProtocolVersion();
        }
        // Counterparty channel was empty.
        counterpartyEndpoints[portId][channelId].channel_id =
            counterpartyChannelId;
    }

    function onChanOpenConfirm(
        string calldata _portId,
        string calldata _channelId
    ) external override onlyIBC {}

    function onChanCloseInit(
        string calldata _portId,
        string calldata _channelId
    ) external override onlyIBC {
        revert NFTLib.ErrUnstoppable();
    }

    function onChanCloseConfirm(
        string calldata _portId,
        string calldata _channelId
    ) external override onlyIBC {
        revert NFTLib.ErrUnstoppable();
    }
}
