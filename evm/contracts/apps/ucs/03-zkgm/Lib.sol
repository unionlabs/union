pragma solidity ^0.8.27;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/extensions/IERC20Metadata.sol";
import "solady/utils/LibBit.sol";
import "solady/utils/LibString.sol";
import "solady/utils/LibBytes.sol";
import "solady/utils/EfficientHashLib.sol";
import "./IZkgm.sol";

library ZkgmLib {
    using LibBytes for *;

    bytes public constant ACK_EMPTY = hex"";

    uint256 public constant ACK_FAILURE = 0x00;
    uint256 public constant ACK_SUCCESS = 0x01;

    bytes public constant ACK_ERR_ONLYMAKER = hex"DEADC0DE";

    bytes32 public constant ACK_ERR_ONLYMAKER_HASH =
        keccak256(ACK_ERR_ONLYMAKER);

    uint256 public constant FILL_TYPE_PROTOCOL = 0xB0CAD0;
    uint256 public constant FILL_TYPE_MARKETMAKER = 0xD1CEC45E;

    uint8 public constant OP_FORWARD = 0x00;
    uint8 public constant OP_MULTIPLEX = 0x01;
    uint8 public constant OP_BATCH = 0x02;
    uint8 public constant OP_FUNGIBLE_ASSET_ORDER = 0x03;

    uint8 public constant INSTR_VERSION_0 = 0x00;
    uint8 public constant INSTR_VERSION_1 = 0x01;

    bytes32 public constant FORWARD_SALT_MAGIC =
        0xC0DE00000000000000000000000000000000000000000000000000000000BABE;

    address public constant NATIVE_ETH_MAGIC =
        0xF000000000000000000000000000000000000001;

    string public constant IBC_VERSION_STR = "ucs03-zkgm-0";
    bytes32 public constant IBC_VERSION = keccak256(bytes(IBC_VERSION_STR));

    error ErrUnsupportedVersion();
    error ErrAsyncMultiplexUnsupported();
    error ErrBatchMustBeSync();
    error ErrUnknownOpcode();
    error ErrInfiniteGame();
    error ErrUnauthorized();
    error ErrInvalidAmount();
    error ErrOnlyMaker();
    error ErrInvalidFillType();
    error ErrInvalidIBCVersion();
    error ErrInvalidHops();
    error ErrInvalidAssetOrigin();
    error ErrInvalidAssetSymbol();
    error ErrInvalidAssetDecimals();
    error ErrInvalidAssetName();
    error ErrInvalidBatchInstruction();
    error ErrInvalidForwardInstruction();
    error ErrInvalidMultiplexSender();
    error ErrInvalidForwardDestinationChannelId();

    function encodeFungibleAssetOrderAck(
        FungibleAssetOrderAck memory ack
    ) internal pure returns (bytes memory) {
        return abi.encode(ack.fillType, ack.marketMaker);
    }

    function decodeFungibleAssetOrderAck(
        bytes calldata stream
    ) internal pure returns (FungibleAssetOrderAck calldata) {
        FungibleAssetOrderAck calldata ack;
        assembly {
            ack := stream.offset
        }
        return ack;
    }

    function encodeBatchAck(
        BatchAck memory ack
    ) internal pure returns (bytes memory) {
        return abi.encode(ack.acknowledgements);
    }

    function decodeBatchAck(
        bytes calldata stream
    ) internal pure returns (BatchAck calldata) {
        BatchAck calldata acks;
        assembly {
            acks := stream.offset
        }
        return acks;
    }

    function encodeAck(
        Ack memory ack
    ) internal pure returns (bytes memory) {
        return abi.encode(ack.tag, ack.innerAck);
    }

    function decodeAck(
        bytes calldata stream
    ) internal pure returns (Ack calldata) {
        Ack calldata operand;
        assembly {
            operand := stream.offset
        }
        return operand;
    }

    function encode(
        ZkgmPacket memory operand
    ) internal pure returns (bytes memory) {
        return abi.encode(operand.salt, operand.path, operand.instruction);
    }

    function decode(
        bytes calldata stream
    ) internal pure returns (ZkgmPacket calldata) {
        ZkgmPacket calldata operand;
        assembly {
            operand := stream.offset
        }
        return operand;
    }

    function encodeBatch(
        Batch memory batch
    ) internal pure returns (bytes memory) {
        return abi.encode(batch.instructions);
    }

    function decodeBatch(
        bytes calldata stream
    ) internal pure returns (Batch calldata) {
        Batch calldata operand;
        assembly {
            operand := stream.offset
        }
        return operand;
    }

    function encodeForward(
        Forward memory forward
    ) internal pure returns (bytes memory) {
        return abi.encode(
            forward.path,
            forward.timeoutHeight,
            forward.timeoutTimestamp,
            forward.instruction
        );
    }

    function decodeForward(
        bytes calldata stream
    ) internal pure returns (Forward calldata) {
        Forward calldata operand;
        assembly {
            operand := stream.offset
        }
        return operand;
    }

    function encodeMultiplex(
        Multiplex memory multiplex
    ) internal pure returns (bytes memory) {
        return abi.encode(
            multiplex.sender,
            multiplex.eureka,
            multiplex.contractAddress,
            multiplex.contractCalldata
        );
    }

    function decodeMultiplex(
        bytes calldata stream
    ) internal pure returns (Multiplex calldata) {
        Multiplex calldata operand;
        assembly {
            operand := stream.offset
        }
        return operand;
    }

    function encodeFungibleAssetOrder(
        FungibleAssetOrder memory order
    ) internal pure returns (bytes memory) {
        return abi.encode(
            order.sender,
            order.receiver,
            order.baseToken,
            order.baseAmount,
            order.baseTokenSymbol,
            order.baseTokenName,
            order.baseTokenDecimals,
            order.baseTokenPath,
            order.quoteToken,
            order.quoteAmount
        );
    }

    function decodeFungibleAssetOrder(
        bytes calldata stream
    ) internal pure returns (FungibleAssetOrder calldata) {
        FungibleAssetOrder calldata operand;
        assembly {
            operand := stream.offset
        }
        return operand;
    }

    function isDeployed(
        address addr
    ) internal returns (bool) {
        uint32 size = 0;
        assembly {
            size := extcodesize(addr)
        }
        return size > 0;
    }

    // Append a channel to a path, injecting the channel u32 to the next available index.
    function updateChannelPath(
        uint256 path,
        uint32 nextChannelId
    ) internal pure returns (uint256) {
        if (path == 0) {
            return uint256(nextChannelId);
        }
        uint256 nextHopIndex = LibBit.fls(path) / 32 + 1;
        if (nextHopIndex > 7) {
            revert ErrInvalidHops();
        }
        return (uint256(nextChannelId) << 32 * nextHopIndex) | path;
    }

    function popChannelFromPath(
        uint256 path
    ) internal pure returns (uint256, uint32) {
        if (path == 0) {
            return (0, 0);
        }
        uint256 currentHopIndex = LibBit.fls(path) / 32;
        uint256 clearShift = (8 - currentHopIndex) * 32;
        return (
            (path << clearShift) >> clearShift,
            uint32(path >> currentHopIndex * 32)
        );
    }

    function dequeueChannelFromPath(
        uint256 path
    ) internal pure returns (uint256, uint32) {
        return (path >> 32, uint32(path));
    }

    // Reverse a channel path consisting of [a, b, c, ...] to [..., c, b, a]
    function reverseChannelPath(
        uint256 path
    ) internal pure returns (uint256) {
        uint256 reversedPath = 0;
        do {
            (uint256 tail, uint32 head) = popChannelFromPath(path);
            reversedPath = updateChannelPath(reversedPath, head);
            path = tail;
        } while (path != 0);
        return reversedPath;
    }

    function isAllowedBatchInstruction(
        uint8 opcode
    ) internal pure returns (bool) {
        return opcode == OP_MULTIPLEX || opcode == OP_FUNGIBLE_ASSET_ORDER;
    }

    function isAllowedForwardInstruction(
        uint8 opcode
    ) internal pure returns (bool) {
        return opcode == OP_MULTIPLEX || opcode == OP_FUNGIBLE_ASSET_ORDER
            || opcode == OP_BATCH;
    }

    function tintForwardSalt(
        bytes32 salt
    ) internal pure returns (bytes32) {
        return FORWARD_SALT_MAGIC | (salt & ~FORWARD_SALT_MAGIC);
    }

    function isForwardedPacket(
        bytes32 salt
    ) internal pure returns (bool) {
        return (salt & FORWARD_SALT_MAGIC) == FORWARD_SALT_MAGIC;
    }

    function deriveForwardSalt(
        bytes32 salt
    ) internal pure returns (bytes32) {
        return tintForwardSalt(EfficientHashLib.hash(salt));
    }

    function deriveBatchSalt(
        uint256 index,
        bytes32 salt
    ) internal pure returns (bytes32) {
        return EfficientHashLib.hash(bytes32(index), salt);
    }

    function encodeMultiplexCalldata(
        uint256 path,
        bytes calldata sender,
        bytes calldata contractCalldata
    ) internal pure returns (bytes memory) {
        return abi.encode(path, sender, contractCalldata);
    }

    function encodeMultiplexCalldataMemory(
        uint256 path,
        bytes memory sender,
        bytes memory contractCalldata
    ) internal pure returns (bytes memory) {
        return abi.encode(path, sender, contractCalldata);
    }

    function makeFungibleAssetOrder(
        IZkgm zkgm,
        uint256 path,
        uint32 channelId,
        address sender,
        bytes memory receiver,
        address baseToken,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) internal returns (Instruction memory) {
        (address wrappedToken,) = zkgm.predictWrappedToken(
            ZkgmLib.reverseChannelPath(path), channelId, quoteToken
        );
        uint256 origin = zkgm.tokenOrigin(baseToken);
        (uint256 baseOrigin, uint32 finalChannelId) =
            ZkgmLib.popChannelFromPath(origin);
        uint256 baseTokenPath = finalChannelId == channelId
            && abi.encodePacked(baseToken).eq(abi.encodePacked(wrappedToken))
            ? origin
            : 0;
        IERC20Metadata sentTokenMeta = IERC20Metadata(baseToken);
        string memory symbol = sentTokenMeta.symbol();
        string memory name = sentTokenMeta.name();
        uint8 decimals = sentTokenMeta.decimals();
        FungibleAssetOrder memory order = FungibleAssetOrder({
            sender: abi.encodePacked(sender),
            receiver: receiver,
            baseToken: abi.encodePacked(baseToken),
            baseTokenPath: baseTokenPath,
            baseTokenSymbol: symbol,
            baseTokenName: name,
            baseTokenDecimals: decimals,
            baseAmount: baseAmount,
            quoteToken: quoteToken,
            quoteAmount: quoteAmount
        });
        return Instruction({
            version: INSTR_VERSION_1,
            opcode: OP_FUNGIBLE_ASSET_ORDER,
            operand: encodeFungibleAssetOrder(order)
        });
    }

    function makeMultiplexCall(
        address sender,
        bool eureka,
        bytes memory contractAddress,
        bytes memory contractCalldata
    ) internal returns (Instruction memory) {
        return Instruction({
            version: INSTR_VERSION_0,
            opcode: OP_FUNGIBLE_ASSET_ORDER,
            operand: encodeMultiplex(
                Multiplex({
                    sender: abi.encodePacked(sender),
                    eureka: eureka,
                    contractAddress: contractAddress,
                    contractCalldata: contractCalldata
                })
            )
        });
    }

    function makeBatch(
        Instruction[] memory instructions
    ) internal returns (Instruction memory) {
        return Instruction({
            version: INSTR_VERSION_0,
            opcode: OP_BATCH,
            operand: encodeBatch(Batch({instructions: instructions}))
        });
    }
}
