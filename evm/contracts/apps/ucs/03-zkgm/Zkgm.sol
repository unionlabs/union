pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin-upgradeable/utils/PausableUpgradeable.sol";

import "@openzeppelin/token/ERC20/IERC20.sol";
import "@openzeppelin/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/token/ERC20/extensions/IERC20Metadata.sol";

import "solady/utils/CREATE3.sol";
import "solady/utils/LibBit.sol";
import "solady/utils/LibString.sol";
import "solady/utils/LibBytes.sol";
import "solady/utils/EfficientHashLib.sol";

import "../../Base.sol";
import "../../../core/04-channel/IBCPacket.sol";
import "../../../core/05-port/IIBCModule.sol";

import "./IEurekaModule.sol";
import "./IZkgmERC20.sol";
import "./ZkgmERC20.sol";
import "./IWETH.sol";

struct ZkgmPacket {
    bytes32 salt;
    uint256 path;
    Instruction instruction;
}

struct Instruction {
    uint8 version;
    uint8 opcode;
    bytes operand;
}

struct Forward {
    uint256 path;
    uint64 timeoutHeight;
    uint64 timeoutTimestamp;
    Instruction instruction;
}

struct Multiplex {
    bytes sender;
    bool eureka;
    bytes contractAddress;
    bytes contractCalldata;
}

struct Batch {
    Instruction[] instructions;
}

struct FungibleAssetOrder {
    bytes sender;
    bytes receiver;
    bytes baseToken;
    uint256 baseAmount;
    string baseTokenSymbol;
    string baseTokenName;
    uint8 baseTokenDecimals;
    uint256 baseTokenPath;
    bytes quoteToken;
    uint256 quoteAmount;
}

struct Ack {
    uint256 tag;
    bytes innerAck;
}

struct BatchAck {
    bytes[] acknowledgements;
}

struct FungibleAssetOrderAck {
    uint256 fillType;
    bytes marketMaker;
}

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

    // Extract the last channel from a path
    function lastChannelFromPath(
        uint256 path
    ) internal pure returns (uint32) {
        if (path == 0) {
            return 0;
        }
        uint256 currentHopIndex = LibBit.fls(path) / 32;
        return uint32(path >> currentHopIndex * 32);
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
        return uint256(uint32(path >> 0)) << 224
            | uint256(uint32(path >> 32)) << 192
            | uint256(uint32(path >> 64)) << 160
            | uint256(uint32(path >> 96)) << 128
            | uint256(uint32(path >> 128)) << 96
            | uint256(uint32(path >> 160)) << 64
            | uint256(uint32(path >> 192)) << 32 | uint256(uint32(path >> 224)) << 0;
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

    function makeFungibleAssetOrder(
        UCS03Zkgm zkgm,
        uint256 path,
        uint32 channelId,
        address sender,
        bytes memory receiver,
        address baseToken,
        uint256 baseAmount,
        bytes memory quoteToken,
        uint256 quoteAmount
    ) internal returns (FungibleAssetOrder memory) {
        (address wrappedToken,) = zkgm.predictWrappedToken(
            ZkgmLib.reverseChannelPath(path), channelId, quoteToken
        );
        uint256 origin = zkgm.tokenOrigin(baseToken);
        (uint256 baseOrigin, uint32 finalChannelId) =
            ZkgmLib.popChannelFromPath(origin);
        uint256 baseTokenPath = finalChannelId == channelId
            && abi.encodePacked(baseToken).eq(abi.encodePacked(wrappedToken))
            ? baseOrigin
            : 0;
        IERC20Metadata sentTokenMeta = IERC20Metadata(baseToken);
        string memory symbol = sentTokenMeta.symbol();
        string memory name = sentTokenMeta.name();
        uint8 decimals = sentTokenMeta.decimals();
        return FungibleAssetOrder({
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
    }

    function tintForwardSalt(
        bytes32 salt
    ) internal pure returns (bytes32) {
        return FORWARD_SALT_MAGIC | (salt & ~FORWARD_SALT_MAGIC);
    }

    function isSaltForwardTinted(
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
}

contract UCS03Zkgm is
    IBCAppBase,
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    PausableUpgradeable
{
    using ZkgmLib for *;
    using LibString for *;
    using LibBytes for *;

    IIBCModulePacket public ibcHandler;
    mapping(bytes32 => IBCPacket) public inFlightPacket;
    mapping(uint32 => mapping(address => uint256)) public channelBalance;
    mapping(address => uint256) public tokenOrigin;
    IWETH public weth;
    mapping(uint32 => mapping(uint256 => mapping(address => uint256))) public
        channelBalanceV2;

    constructor() {
        _disableInitializers();
    }

    function initialize(
        IIBCModulePacket _ibcHandler,
        address admin,
        IWETH _weth
    ) public initializer {
        __Ownable_init(admin);
        ibcHandler = _ibcHandler;
        weth = _weth;
    }

    function setWeth(
        IWETH _weth
    ) public onlyOwner {
        weth = _weth;
    }

    // Temporary function that we need to remove after testnet.
    function withdrawWeth(
        uint256 wad
    ) public onlyOwner {
        weth.transfer(msg.sender, wad);
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }

    function internalSendOverChannel(
        uint32 channelId,
        bytes calldata receiver,
        address baseToken,
        uint256 baseAmount,
        bytes calldata quoteToken,
        uint256 quoteAmount
    ) internal returns (Instruction memory) {
        uint256 origin = tokenOrigin[baseToken];
        // Verify the unwrap
        (address wrappedToken,) =
            internalPredictWrappedToken(0, channelId, quoteToken);
        // Only allow unwrapping if the quote asset is the unwrapped asset.
        if (
            ZkgmLib.lastChannelFromPath(origin) == channelId
                && abi.encodePacked(baseToken).eq(abi.encodePacked(wrappedToken))
        ) {
            IZkgmERC20(baseToken).burn(msg.sender, baseAmount);
        } else {
            // We reset the origin, the asset will not be unescrowed on the destination
            origin = 0;
            // TODO: extract this as a step before verifying to allow for ERC777
            // send hook
            increaseOutstanding(channelId, 0, baseToken, baseAmount);
            SafeERC20.safeTransferFrom(
                IERC20(baseToken), msg.sender, address(this), baseAmount
            );
        }

        // TODO: make this non-failable as it's not guaranteed to exist
        IERC20Metadata sentTokenMeta = IERC20Metadata(baseToken);
        string memory symbol = sentTokenMeta.symbol();
        string memory name = sentTokenMeta.name();
        uint8 decimals = sentTokenMeta.decimals();

        return Instruction({
            version: ZkgmLib.INSTR_VERSION_1,
            opcode: ZkgmLib.OP_FUNGIBLE_ASSET_ORDER,
            operand: ZkgmLib.encodeFungibleAssetOrder(
                FungibleAssetOrder({
                    sender: abi.encodePacked(msg.sender),
                    receiver: receiver,
                    baseToken: abi.encodePacked(baseToken),
                    baseTokenPath: origin,
                    baseTokenSymbol: symbol,
                    baseTokenName: name,
                    baseTokenDecimals: decimals,
                    baseAmount: baseAmount,
                    quoteToken: quoteToken,
                    quoteAmount: quoteAmount
                })
            )
        });
    }

    function transferAndCall(
        uint32 channelId,
        bytes calldata receiver,
        address baseToken,
        uint256 baseAmount,
        bytes calldata quoteToken,
        uint256 quoteAmount,
        bytes calldata contractAddress,
        bytes calldata contractCalldata,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes32 salt
    ) public {
        if (baseAmount == 0) {
            revert ZkgmLib.ErrInvalidAmount();
        }
        Instruction[] memory instructions = new Instruction[](2);
        instructions[0] = internalSendOverChannel(
            channelId, receiver, baseToken, baseAmount, quoteToken, quoteAmount
        );
        instructions[1] = Instruction({
            version: ZkgmLib.INSTR_VERSION_0,
            opcode: ZkgmLib.OP_MULTIPLEX,
            operand: ZkgmLib.encodeMultiplex(
                Multiplex({
                    sender: abi.encodePacked(msg.sender),
                    eureka: false,
                    contractAddress: contractAddress,
                    contractCalldata: contractCalldata
                })
            )
        });
        ibcHandler.sendPacket(
            channelId,
            timeoutHeight,
            timeoutTimestamp,
            ZkgmLib.encode(
                ZkgmPacket({
                    salt: salt,
                    path: 0,
                    instruction: Instruction({
                        version: ZkgmLib.INSTR_VERSION_0,
                        opcode: ZkgmLib.OP_BATCH,
                        operand: ZkgmLib.encodeBatch(
                            Batch({instructions: instructions})
                        )
                    })
                })
            )
        );
    }

    function transferV2(
        uint32 channelId,
        bytes calldata receiver,
        address baseToken,
        uint256 baseAmount,
        bytes calldata quoteToken,
        uint256 quoteAmount,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes32 salt,
        bytes calldata wethQuoteToken
    ) public payable {
        if (baseAmount == 0) {
            revert ZkgmLib.ErrInvalidAmount();
        }
        Instruction[] memory instructions =
            new Instruction[](msg.value > 0 ? 2 : 1);
        instructions[0] = internalSendOverChannel(
            channelId, receiver, baseToken, baseAmount, quoteToken, quoteAmount
        );
        if (msg.value > 0) {
            weth.deposit{value: msg.value}();
            address wethBaseToken = address(weth);
            instructions[1] = Instruction({
                version: ZkgmLib.INSTR_VERSION_1,
                opcode: ZkgmLib.OP_FUNGIBLE_ASSET_ORDER,
                operand: ZkgmLib.encodeFungibleAssetOrder(
                    FungibleAssetOrder({
                        sender: abi.encodePacked(msg.sender),
                        receiver: receiver,
                        baseToken: abi.encodePacked(wethBaseToken),
                        baseTokenPath: 0,
                        baseTokenSymbol: "WETH",
                        baseTokenName: "Wrapped Ether",
                        baseTokenDecimals: 18,
                        baseAmount: msg.value,
                        quoteToken: wethQuoteToken,
                        // means we pay the relayer on destination for 100% of the value
                        quoteAmount: 0
                    })
                )
            });
            increaseOutstanding(channelId, 0, wethBaseToken, msg.value);
        }
        ibcHandler.sendPacket(
            channelId,
            timeoutHeight,
            timeoutTimestamp,
            ZkgmLib.encode(
                ZkgmPacket({
                    salt: salt,
                    path: 0,
                    instruction: Instruction({
                        version: ZkgmLib.INSTR_VERSION_0,
                        opcode: ZkgmLib.OP_BATCH,
                        operand: ZkgmLib.encodeBatch(
                            Batch({instructions: instructions})
                        )
                    })
                })
            )
        );
    }

    function send(
        uint32 channelId,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes32 salt,
        Instruction calldata instruction
    ) public {
        verifyInternal(channelId, 0, instruction);
        ibcHandler.sendPacket(
            channelId,
            timeoutHeight,
            timeoutTimestamp,
            ZkgmLib.encode(
                ZkgmPacket({
                    salt: EfficientHashLib.hash(
                        abi.encodePacked(abi.encodePacked(msg.sender), salt)
                    ),
                    path: 0,
                    instruction: instruction
                })
            )
        );
    }

    function increaseOutstanding(
        uint32 sourceChannelId,
        uint256 path,
        address token,
        uint256 amount
    ) internal {
        channelBalanceV2[sourceChannelId][path][token] += amount;
    }

    // Decrease the outstanding balance of a (channel, path). We assume that the
    // function is called when receiving funds, hence, to decrease we need to
    // first inverse the path. If we increased the balance for (0, [1, 2, 3])
    // and funds are sent back over [3, 2, 1], this will only work if the path
    // is the inverse.
    function decreaseOutstanding(
        uint32 sourceChannelId,
        uint256 path,
        address token,
        uint256 amount
    ) internal {
        channelBalanceV2[sourceChannelId][path][token] -= amount;
    }

    function verifyInternal(
        uint32 channelId,
        uint256 path,
        Instruction calldata instruction
    ) internal {
        if (instruction.opcode == ZkgmLib.OP_FUNGIBLE_ASSET_ORDER) {
            if (instruction.version != ZkgmLib.INSTR_VERSION_1) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            FungibleAssetOrder calldata order =
                ZkgmLib.decodeFungibleAssetOrder(instruction.operand);
            verifyFungibleAssetOrder(channelId, path, order);
        } else if (instruction.opcode == ZkgmLib.OP_BATCH) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            verifyBatch(
                channelId, path, ZkgmLib.decodeBatch(instruction.operand)
            );
        } else if (instruction.opcode == ZkgmLib.OP_FORWARD) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            verifyForward(channelId, ZkgmLib.decodeForward(instruction.operand));
        } else if (instruction.opcode == ZkgmLib.OP_MULTIPLEX) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            verifyMultiplex(
                channelId, path, ZkgmLib.decodeMultiplex(instruction.operand)
            );
        } else {
            revert ZkgmLib.ErrUnknownOpcode();
        }
    }

    function verifyFungibleAssetOrder(
        uint32 channelId,
        uint256 path,
        FungibleAssetOrder calldata order
    ) internal {
        IERC20Metadata baseToken =
            IERC20Metadata(address(bytes20(order.baseToken)));
        if (!order.baseTokenName.eq(baseToken.name())) {
            revert ZkgmLib.ErrInvalidAssetName();
        }
        if (!order.baseTokenSymbol.eq(baseToken.symbol())) {
            revert ZkgmLib.ErrInvalidAssetSymbol();
        }
        if (order.baseTokenDecimals != baseToken.decimals()) {
            revert ZkgmLib.ErrInvalidAssetDecimals();
        }
        // The origin is the concatenation of (path, destinationChannelId) where
        // path are the intermediate channels hops, if we send from channel X on
        // A over channel Y on B to channel Z on C, the path would be
        // [(X.destinationChannelId, Y.sourceChannelId)].
        uint256 origin = tokenOrigin[address(baseToken)];
        // Split back the origin as the intermediate path and the destinationChannelId
        (uint256 intermediateChannelPath, uint32 destinationChannelId) =
            ZkgmLib.popChannelFromPath(origin);
        // We compute the wrapped token from the destination to the source. If
        // the base token matches the predicted wrapper, we want to unwrap only
        // if it's being sent back through the same channel/path.
        (address wrappedToken,) = internalPredictWrappedToken(
            intermediateChannelPath, channelId, order.quoteToken
        );
        bool isInverseIntermediatePath =
            path == ZkgmLib.reverseChannelPath(intermediateChannelPath);
        bool isSendingBackToSameChannel = destinationChannelId == channelId;
        bool isUnwrapping =
            abi.encodePacked(order.baseToken).eq(abi.encodePacked(wrappedToken));
        // If we take the same path starting from the same channel using the
        // wrapped asset, we unwrapp.
        if (
            isInverseIntermediatePath && isSendingBackToSameChannel
                && isUnwrapping
        ) {
            if (order.baseTokenPath != origin) {
                revert ZkgmLib.ErrInvalidAssetOrigin();
            }
            IZkgmERC20(address(baseToken)).burn(msg.sender, order.baseAmount);
        } else {
            if (order.baseTokenPath != 0) {
                revert ZkgmLib.ErrInvalidAssetOrigin();
            }
            increaseOutstanding(
                channelId, path, address(baseToken), order.baseAmount
            );
            SafeERC20.safeTransferFrom(
                baseToken, msg.sender, address(this), order.baseAmount
            );
        }
    }

    function verifyBatch(
        uint32 channelId,
        uint256 path,
        Batch calldata batch
    ) internal {
        uint256 l = batch.instructions.length;
        for (uint256 i = 0; i < l; i++) {
            if (
                !ZkgmLib.isAllowedBatchInstruction(batch.instructions[i].opcode)
            ) {
                revert ZkgmLib.ErrInvalidBatchInstruction();
            }
            verifyInternal(channelId, path, batch.instructions[i]);
        }
    }

    function verifyForward(
        uint32 channelId,
        Forward calldata forward
    ) internal {
        if (!ZkgmLib.isAllowedForwardInstruction(forward.instruction.opcode)) {
            revert ZkgmLib.ErrInvalidForwardInstruction();
        }
        verifyInternal(channelId, forward.path, forward.instruction);
    }

    function verifyMultiplex(
        uint32 channelId,
        uint256 path,
        Multiplex calldata multiplex
    ) internal {
        if (!multiplex.sender.eq(abi.encodePacked(msg.sender))) {
            revert ZkgmLib.ErrInvalidMultiplexSender();
        }
    }

    function onRecvPacket(
        IBCPacket calldata packet,
        address relayer,
        bytes calldata relayerMsg
    ) external virtual override onlyIBC returns (bytes memory) {
        (bool success, bytes memory returnData) = address(this).call(
            abi.encodeCall(this.execute, (packet, relayer, relayerMsg))
        );
        if (success) {
            bytes memory acknowledgement = abi.decode(returnData, (bytes));
            // The acknowledgement may be asynchronous (forward/multiplex).
            if (acknowledgement.length == 0) {
                return ZkgmLib.ACK_EMPTY;
            }

            // Special case where we should avoid the packet from being
            // received entirely as it is only fillable by a market maker.
            if (
                EfficientHashLib.hash(acknowledgement)
                    == ZkgmLib.ACK_ERR_ONLYMAKER_HASH
            ) {
                revert ZkgmLib.ErrOnlyMaker();
            }

            return ZkgmLib.encodeAck(
                Ack({tag: ZkgmLib.ACK_SUCCESS, innerAck: acknowledgement})
            );
        } else {
            return ZkgmLib.encodeAck(
                Ack({tag: ZkgmLib.ACK_FAILURE, innerAck: ZkgmLib.ACK_EMPTY})
            );
        }
    }

    function execute(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg
    ) public returns (bytes memory) {
        // Only callable through the onRecvPacket endpoint.
        if (msg.sender != address(this)) {
            revert ZkgmLib.ErrUnauthorized();
        }
        ZkgmPacket calldata zkgmPacket = ZkgmLib.decode(ibcPacket.data);
        return executeInternal(
            ibcPacket,
            relayer,
            relayerMsg,
            zkgmPacket.salt,
            zkgmPacket.path,
            zkgmPacket.instruction
        );
    }

    function executeInternal(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        uint256 path,
        Instruction calldata instruction
    ) internal returns (bytes memory) {
        if (instruction.opcode == ZkgmLib.OP_FUNGIBLE_ASSET_ORDER) {
            if (instruction.version != ZkgmLib.INSTR_VERSION_1) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            FungibleAssetOrder calldata order =
                ZkgmLib.decodeFungibleAssetOrder(instruction.operand);
            return executeFungibleAssetOrder(
                ibcPacket,
                relayer,
                path,
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
        } else if (instruction.opcode == ZkgmLib.OP_BATCH) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            return executeBatch(
                ibcPacket,
                relayer,
                relayerMsg,
                salt,
                path,
                ZkgmLib.decodeBatch(instruction.operand)
            );
        } else if (instruction.opcode == ZkgmLib.OP_FORWARD) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            return executeForward(
                ibcPacket,
                relayer,
                relayerMsg,
                salt,
                path,
                instruction.version,
                ZkgmLib.decodeForward(instruction.operand)
            );
        } else if (instruction.opcode == ZkgmLib.OP_MULTIPLEX) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            return executeMultiplex(
                ibcPacket,
                relayer,
                relayerMsg,
                path,
                salt,
                ZkgmLib.decodeMultiplex(instruction.operand)
            );
        } else {
            revert ZkgmLib.ErrUnknownOpcode();
        }
    }

    function executeBatch(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        uint256 path,
        Batch calldata batch
    ) internal returns (bytes memory) {
        uint256 l = batch.instructions.length;
        bytes[] memory acks = new bytes[](l);
        for (uint256 i = 0; i < l; i++) {
            Instruction calldata instruction = batch.instructions[i];
            acks[i] = executeInternal(
                ibcPacket,
                relayer,
                relayerMsg,
                ZkgmLib.deriveBatchSalt(i, salt),
                path,
                instruction
            );
            // We should have the guarantee that the acks are non empty because
            // the only instructions allowed in a batch are multiplex and
            // fungibleAssetOrder which returns non-empty acks only.
            if (acks[i].length == 0) {
                revert ZkgmLib.ErrBatchMustBeSync();
            } else if (
                EfficientHashLib.hash(acks[i]) == ZkgmLib.ACK_ERR_ONLYMAKER_HASH
            ) {
                return acks[i];
            }
        }
        return ZkgmLib.encodeBatchAck(BatchAck({acknowledgements: acks}));
    }

    function executeForward(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        uint256 path,
        uint8 version,
        Forward calldata forward
    ) internal returns (bytes memory) {
        (uint256 tailPath, uint32 previousDestinationChannelId) =
            ZkgmLib.dequeueChannelFromPath(forward.path);
        (uint256 continuationPath, uint32 nextSourceChannelId) =
            ZkgmLib.dequeueChannelFromPath(tailPath);
        if (ibcPacket.destinationChannelId != previousDestinationChannelId) {
            revert ZkgmLib.ErrInvalidForwardDestinationChannelId();
        }
        Instruction memory nextInstruction;
        if (continuationPath == 0) {
            // If we are done hopping, the sub-instruction is dispatched for execution.
            nextInstruction = forward.instruction;
        } else {
            // If we are not done, the continuation path is used and the forward is re-executed.
            nextInstruction = Instruction({
                version: version,
                opcode: ZkgmLib.OP_FORWARD,
                operand: ZkgmLib.encodeForward(
                    Forward({
                        path: continuationPath,
                        timeoutHeight: forward.timeoutHeight,
                        timeoutTimestamp: forward.timeoutTimestamp,
                        instruction: forward.instruction
                    })
                )
            });
        }
        IBCPacket memory sentPacket = ibcHandler.sendPacket(
            nextSourceChannelId,
            forward.timeoutHeight,
            forward.timeoutTimestamp,
            ZkgmLib.encode(
                ZkgmPacket({
                    salt: ZkgmLib.deriveForwardSalt(salt),
                    path: ZkgmLib.updateChannelPath(
                        ZkgmLib.updateChannelPath(
                            path, previousDestinationChannelId
                        ),
                        nextSourceChannelId
                    ),
                    instruction: nextInstruction
                })
            )
        );
        // Guaranteed to be unique by the above sendPacket
        bytes32 packetHash = IBCPacketLib.commitPacketMemory(sentPacket);
        inFlightPacket[packetHash] = ibcPacket;
        return ZkgmLib.ACK_EMPTY;
    }

    function executeMultiplex(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        uint256 path,
        bytes32 salt,
        Multiplex calldata multiplex
    ) internal returns (bytes memory) {
        address contractAddress = address(bytes20(multiplex.contractAddress));
        if (!multiplex.eureka) {
            IEurekaModule(contractAddress).onZkgm(
                path,
                ibcPacket.sourceChannelId,
                ibcPacket.destinationChannelId,
                multiplex.sender,
                multiplex.contractCalldata
            );
            return abi.encode(ZkgmLib.ACK_SUCCESS);
        } else {
            IBCPacket memory multiplexIbcPacket = IBCPacket({
                sourceChannelId: ibcPacket.sourceChannelId,
                destinationChannelId: ibcPacket.destinationChannelId,
                data: ZkgmLib.encodeMultiplexCalldata(
                    path, multiplex.sender, multiplex.contractCalldata
                ),
                timeoutHeight: ibcPacket.timeoutHeight,
                timeoutTimestamp: ibcPacket.timeoutTimestamp
            });
            bytes memory acknowledgement = IIBCModuleRecv(contractAddress)
                .onRecvPacket(multiplexIbcPacket, relayer, relayerMsg);
            if (acknowledgement.length == 0) {
                revert ZkgmLib.ErrAsyncMultiplexUnsupported();
            }
            return acknowledgement;
        }
    }

    function internalPredictWrappedToken(
        uint256 path,
        uint32 channel,
        bytes calldata token
    ) internal view returns (address, bytes32) {
        bytes32 wrappedTokenSalt =
            EfficientHashLib.hash(abi.encode(path, channel, token));
        address wrappedToken =
            CREATE3.predictDeterministicAddress(wrappedTokenSalt);
        return (wrappedToken, wrappedTokenSalt);
    }

    function predictWrappedToken(
        uint256 path,
        uint32 channel,
        bytes calldata token
    ) public view returns (address, bytes32) {
        return internalPredictWrappedToken(path, channel, token);
    }

    function internalProtocolFill(
        uint32 channelId,
        uint256 path,
        address wrappedToken,
        address quoteToken,
        address receiver,
        address relayer,
        uint256 baseAmount,
        uint256 quoteAmount,
        bool mint
    ) internal returns (bytes memory) {
        uint256 fee = baseAmount - quoteAmount;
        if (mint) {
            if (quoteAmount > 0) {
                IZkgmERC20(wrappedToken).mint(receiver, quoteAmount);
            }
            if (fee > 0) {
                IZkgmERC20(wrappedToken).mint(relayer, fee);
            }
        } else {
            // If the base token path is being unwrapped, it's going to be non-zero.
            decreaseOutstanding(
                channelId,
                ZkgmLib.reverseChannelPath(path),
                quoteToken,
                baseAmount
            );
            if (quoteAmount > 0) {
                SafeERC20.safeTransfer(
                    IERC20(quoteToken), receiver, quoteAmount
                );
            }
            if (fee > 0) {
                SafeERC20.safeTransfer(IERC20(quoteToken), relayer, fee);
            }
        }
        return ZkgmLib.encodeFungibleAssetOrderAck(
            FungibleAssetOrderAck({
                fillType: ZkgmLib.FILL_TYPE_PROTOCOL,
                marketMaker: ZkgmLib.ACK_EMPTY
            })
        );
    }

    function internalDeployWrappedToken(
        uint32 channelId,
        uint256 path,
        address wrappedToken,
        bytes32 wrappedTokenSalt,
        string calldata orderBaseTokenSymbol,
        string calldata orderBaseTokenName,
        uint8 orderBaseTokenDecimals
    ) internal {
        if (!ZkgmLib.isDeployed(wrappedToken)) {
            CREATE3.deployDeterministic(
                abi.encodePacked(
                    type(ZkgmERC20).creationCode,
                    abi.encode(
                        orderBaseTokenName,
                        orderBaseTokenSymbol,
                        orderBaseTokenDecimals,
                        address(this)
                    )
                ),
                wrappedTokenSalt
            );
            tokenOrigin[wrappedToken] =
                ZkgmLib.updateChannelPath(path, channelId);
        }
    }

    function executeFungibleAssetOrder(
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        bytes calldata orderReceiver,
        bytes calldata orderBaseToken,
        uint256 orderBaseAmount,
        string calldata orderBaseTokenSymbol,
        string calldata orderBaseTokenName,
        uint8 orderBaseTokenDecimals,
        uint256 orderBaseTokenPath,
        bytes calldata orderQuoteToken,
        uint256 orderQuoteAmount
    ) internal returns (bytes memory) {
        (address wrappedToken, bytes32 wrappedTokenSalt) =
        internalPredictWrappedToken(
            path, ibcPacket.destinationChannelId, orderBaseToken
        );
        address quoteToken = address(bytes20(orderQuoteToken));
        address receiver = address(bytes20(orderReceiver));
        bool baseAmountCoversQuoteAmount = orderBaseAmount >= orderQuoteAmount;
        if (quoteToken == wrappedToken && baseAmountCoversQuoteAmount) {
            internalDeployWrappedToken(
                ibcPacket.destinationChannelId,
                path,
                wrappedToken,
                wrappedTokenSalt,
                orderBaseTokenSymbol,
                orderBaseTokenName,
                orderBaseTokenDecimals
            );
            return internalProtocolFill(
                ibcPacket.destinationChannelId,
                path,
                wrappedToken,
                quoteToken,
                receiver,
                relayer,
                orderBaseAmount,
                orderQuoteAmount,
                true
            );
        } else if (orderBaseTokenPath != 0 && baseAmountCoversQuoteAmount) {
            return internalProtocolFill(
                ibcPacket.destinationChannelId,
                path,
                wrappedToken,
                quoteToken,
                receiver,
                relayer,
                orderBaseAmount,
                orderQuoteAmount,
                false
            );
        } else {
            // TODO: allow for MM filling after having added the caller to the
            // interface (from which we extract funds)
            return ZkgmLib.ACK_ERR_ONLYMAKER;
        }
    }

    function isInFlightPacket(
        bytes32 packetHash
    ) internal pure returns (bool) {
        return ZkgmLib.isSaltForwardTinted(packetHash);
    }

    function onAcknowledgementPacket(
        IBCPacket calldata ibcPacket,
        bytes calldata ack,
        address relayer
    ) external virtual override onlyIBC {
        bytes32 packetHash = IBCPacketLib.commitPacket(ibcPacket);
        if (isInFlightPacket(packetHash)) {
            IBCPacket memory parent = inFlightPacket[packetHash];
            if (parent.timeoutTimestamp != 0 || parent.timeoutHeight != 0) {
                ibcHandler.writeAcknowledgement(parent, ack);
                delete inFlightPacket[packetHash];
                return;
            }
        }
        ZkgmPacket calldata zkgmPacket = ZkgmLib.decode(ibcPacket.data);
        Ack calldata zkgmAck = ZkgmLib.decodeAck(ack);
        acknowledgeInternal(
            ibcPacket,
            relayer,
            zkgmPacket.path,
            zkgmPacket.salt,
            zkgmPacket.instruction,
            zkgmAck.tag == ZkgmLib.ACK_SUCCESS,
            zkgmAck.innerAck
        );
    }

    function acknowledgeInternal(
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        bytes32 salt,
        Instruction calldata instruction,
        bool successful,
        bytes calldata ack
    ) internal {
        if (instruction.opcode == ZkgmLib.OP_FUNGIBLE_ASSET_ORDER) {
            if (instruction.version != ZkgmLib.INSTR_VERSION_1) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            FungibleAssetOrder calldata order =
                ZkgmLib.decodeFungibleAssetOrder(instruction.operand);
            acknowledgeFungibleAssetOrder(
                ibcPacket,
                relayer,
                path,
                salt,
                order.sender,
                order.baseToken,
                order.baseTokenPath,
                order.baseAmount,
                successful,
                ack
            );
        } else if (instruction.opcode == ZkgmLib.OP_BATCH) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            acknowledgeBatch(
                ibcPacket,
                relayer,
                path,
                salt,
                ZkgmLib.decodeBatch(instruction.operand),
                successful,
                ack
            );
        } else if (instruction.opcode == ZkgmLib.OP_FORWARD) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            acknowledgeForward(
                ibcPacket,
                relayer,
                salt,
                ZkgmLib.decodeForward(instruction.operand),
                successful,
                ack
            );
        } else if (instruction.opcode == ZkgmLib.OP_MULTIPLEX) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            acknowledgeMultiplex(
                ibcPacket,
                relayer,
                path,
                salt,
                ZkgmLib.decodeMultiplex(instruction.operand),
                successful,
                ack
            );
        } else {
            revert ZkgmLib.ErrUnknownOpcode();
        }
    }

    function acknowledgeBatch(
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        bytes32 salt,
        Batch calldata batch,
        bool successful,
        bytes calldata ack
    ) internal {
        uint256 l = batch.instructions.length;
        BatchAck calldata batchAck = ZkgmLib.decodeBatchAck(ack);
        for (uint256 i = 0; i < l; i++) {
            // The syscallAck is set to the ack by default just to satisfy the
            // compiler. The failure branch will never read the ack, hence the
            // assignation has no effect in the recursive handling semantic.
            bytes calldata syscallAck = ack;
            if (successful) {
                syscallAck = batchAck.acknowledgements[i];
            }
            acknowledgeInternal(
                ibcPacket,
                relayer,
                path,
                ZkgmLib.deriveBatchSalt(i, salt),
                batch.instructions[i],
                successful,
                syscallAck
            );
        }
    }

    function acknowledgeForward(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        Forward calldata forward,
        bool successful,
        bytes calldata ack
    ) internal {
        acknowledgeInternal(
            ibcPacket,
            relayer,
            forward.path,
            salt,
            forward.instruction,
            successful,
            ack
        );
    }

    function acknowledgeMultiplex(
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        bytes32 salt,
        Multiplex calldata multiplex,
        bool successful,
        bytes calldata ack
    ) internal {
        if (successful && multiplex.eureka) {
            IBCPacket memory multiplexIbcPacket = IBCPacket({
                sourceChannelId: ibcPacket.sourceChannelId,
                destinationChannelId: ibcPacket.destinationChannelId,
                data: ZkgmLib.encodeMultiplexCalldata(
                    path, multiplex.sender, multiplex.contractCalldata
                ),
                timeoutHeight: ibcPacket.timeoutHeight,
                timeoutTimestamp: ibcPacket.timeoutTimestamp
            });
            IIBCModule(address(bytes20(multiplex.sender)))
                .onAcknowledgementPacket(multiplexIbcPacket, ack, relayer);
        }
    }

    function acknowledgeFungibleAssetOrder(
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        bytes32 salt,
        bytes calldata orderSender,
        bytes calldata orderBaseToken,
        uint256 orderBaseTokenPath,
        uint256 orderBaseAmount,
        bool successful,
        bytes calldata ack
    ) internal {
        if (successful) {
            FungibleAssetOrderAck calldata assetOrderAck =
                ZkgmLib.decodeFungibleAssetOrderAck(ack);
            if (assetOrderAck.fillType == ZkgmLib.FILL_TYPE_PROTOCOL) {
                // The protocol filled, fee was paid to relayer.
            } else if (assetOrderAck.fillType == ZkgmLib.FILL_TYPE_MARKETMAKER)
            {
                // A market maker filled, we pay with the sent asset.
                address marketMaker =
                    address(bytes20(assetOrderAck.marketMaker));
                address baseToken = address(bytes20(orderBaseToken));
                if (orderBaseTokenPath != 0) {
                    IZkgmERC20(address(baseToken)).mint(
                        marketMaker, orderBaseAmount
                    );
                } else {
                    decreaseOutstanding(
                        ibcPacket.sourceChannelId,
                        path,
                        baseToken,
                        orderBaseAmount
                    );
                    SafeERC20.safeTransfer(
                        IERC20(baseToken), marketMaker, orderBaseAmount
                    );
                }
            } else {
                revert ZkgmLib.ErrInvalidFillType();
            }
        } else {
            refund(
                ibcPacket.sourceChannelId,
                path,
                orderSender,
                orderBaseToken,
                orderBaseTokenPath,
                orderBaseAmount
            );
        }
    }

    function onTimeoutPacket(
        IBCPacket calldata ibcPacket,
        address relayer
    ) external virtual override onlyIBC {
        bytes32 packetHash = IBCPacketLib.commitPacket(ibcPacket);
        if (isInFlightPacket(packetHash)) {
            IBCPacket memory parent = inFlightPacket[packetHash];
            if (parent.timeoutTimestamp != 0 || parent.timeoutHeight != 0) {
                delete inFlightPacket[packetHash];
                return;
            }
        }
        ZkgmPacket calldata zkgmPacket = ZkgmLib.decode(ibcPacket.data);
        timeoutInternal(
            ibcPacket, relayer, zkgmPacket.path, zkgmPacket.instruction
        );
    }

    function timeoutInternal(
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        Instruction calldata instruction
    ) internal {
        if (instruction.opcode == ZkgmLib.OP_FUNGIBLE_ASSET_ORDER) {
            if (instruction.version != ZkgmLib.INSTR_VERSION_1) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            FungibleAssetOrder calldata order =
                ZkgmLib.decodeFungibleAssetOrder(instruction.operand);
            timeoutFungibleAssetOrder(
                ibcPacket,
                path,
                order.sender,
                order.baseToken,
                order.baseTokenPath,
                order.baseAmount
            );
        } else if (instruction.opcode == ZkgmLib.OP_BATCH) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            timeoutBatch(
                ibcPacket,
                relayer,
                path,
                ZkgmLib.decodeBatch(instruction.operand)
            );
        } else if (instruction.opcode == ZkgmLib.OP_FORWARD) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            timeoutForward(
                ibcPacket, relayer, ZkgmLib.decodeForward(instruction.operand)
            );
        } else if (instruction.opcode == ZkgmLib.OP_MULTIPLEX) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            timeoutMultiplex(
                ibcPacket,
                relayer,
                path,
                ZkgmLib.decodeMultiplex(instruction.operand)
            );
        } else {
            revert ZkgmLib.ErrUnknownOpcode();
        }
    }

    function timeoutBatch(
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        Batch calldata batch
    ) internal {
        uint256 l = batch.instructions.length;
        for (uint256 i = 0; i < l; i++) {
            timeoutInternal(ibcPacket, relayer, path, batch.instructions[i]);
        }
    }

    function timeoutForward(
        IBCPacket calldata ibcPacket,
        address relayer,
        Forward calldata forward
    ) internal {
        timeoutInternal(ibcPacket, relayer, forward.path, forward.instruction);
    }

    function timeoutMultiplex(
        IBCPacket calldata ibcPacket,
        address relayer,
        uint256 path,
        Multiplex calldata multiplex
    ) internal {
        if (multiplex.eureka) {
            IBCPacket memory multiplexIbcPacket = IBCPacket({
                sourceChannelId: ibcPacket.sourceChannelId,
                destinationChannelId: ibcPacket.destinationChannelId,
                data: ZkgmLib.encodeMultiplexCalldata(
                    path, multiplex.contractAddress, multiplex.contractCalldata
                ),
                timeoutHeight: ibcPacket.timeoutHeight,
                timeoutTimestamp: ibcPacket.timeoutTimestamp
            });
            IIBCModule(address(bytes20(multiplex.sender))).onTimeoutPacket(
                multiplexIbcPacket, relayer
            );
        }
    }

    function timeoutFungibleAssetOrder(
        IBCPacket calldata ibcPacket,
        uint256 path,
        bytes calldata orderSender,
        bytes calldata orderBaseToken,
        uint256 orderBaseTokenPath,
        uint256 orderBaseAmount
    ) internal {
        refund(
            ibcPacket.sourceChannelId,
            path,
            orderSender,
            orderBaseToken,
            orderBaseTokenPath,
            orderBaseAmount
        );
    }

    function refund(
        uint32 sourceChannelId,
        uint256 path,
        bytes calldata orderSender,
        bytes calldata orderBaseToken,
        uint256 orderBaseTokenPath,
        uint256 orderBaseAmount
    ) internal {
        address sender = address(bytes20(orderSender));
        address baseToken = address(bytes20(orderBaseToken));
        if (orderBaseTokenPath != 0) {
            IZkgmERC20(address(baseToken)).mint(sender, orderBaseAmount);
        } else {
            decreaseOutstanding(
                sourceChannelId, path, baseToken, orderBaseAmount
            );
            SafeERC20.safeTransfer(IERC20(baseToken), sender, orderBaseAmount);
        }
    }

    function onChanOpenInit(
        uint32,
        uint32,
        string calldata version,
        address
    ) external virtual override onlyIBC {
        if (EfficientHashLib.hash(bytes(version)) != ZkgmLib.IBC_VERSION) {
            revert ZkgmLib.ErrInvalidIBCVersion();
        }
    }

    function onChanOpenTry(
        uint32,
        uint32,
        uint32,
        string calldata version,
        string calldata counterpartyVersion,
        address
    ) external virtual override onlyIBC {
        if (EfficientHashLib.hash(bytes(version)) != ZkgmLib.IBC_VERSION) {
            revert ZkgmLib.ErrInvalidIBCVersion();
        }
        if (
            EfficientHashLib.hash(bytes(counterpartyVersion))
                != ZkgmLib.IBC_VERSION
        ) {
            revert ZkgmLib.ErrInvalidIBCVersion();
        }
    }

    function onChanOpenAck(
        uint32 channelId,
        uint32,
        string calldata,
        address
    ) external virtual override onlyIBC {}

    function onChanOpenConfirm(
        uint32 channelId,
        address
    ) external virtual override onlyIBC {}

    function onChanCloseInit(
        uint32,
        address
    ) external virtual override onlyIBC {
        revert ZkgmLib.ErrInfiniteGame();
    }

    function onChanCloseConfirm(
        uint32,
        address
    ) external virtual override onlyIBC {
        revert ZkgmLib.ErrInfiniteGame();
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}
}
