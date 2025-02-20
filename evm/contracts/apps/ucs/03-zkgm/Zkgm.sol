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
    uint32 channelId;
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

struct FungibleAssetOrderV0 {
    bytes sender;
    bytes receiver;
    bytes baseToken;
    uint256 baseAmount;
    string baseTokenSymbol;
    string baseTokenName;
    uint256 baseTokenPath;
    bytes quoteToken;
    uint256 quoteAmount;
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
    bytes public constant ACK_EMPTY = hex"";

    uint256 public constant ACK_FAILURE = 0x00;
    uint256 public constant ACK_SUCCESS = 0x01;

    bytes public constant ACK_ERR_ONLYMAKER = hex"DEADC0DE";

    uint256 public constant FILL_TYPE_PROTOCOL = 0xB0CAD0;
    uint256 public constant FILL_TYPE_MARKETMAKER = 0xD1CEC45E;

    uint8 public constant OP_FORWARD = 0x00;
    uint8 public constant OP_MULTIPLEX = 0x01;
    uint8 public constant OP_BATCH = 0x02;
    uint8 public constant OP_FUNGIBLE_ASSET_ORDER = 0x03;

    uint8 public constant INSTR_VERSION_0 = 0x00;
    uint8 public constant INSTR_VERSION_1 = 0x01;

    bytes32 public constant IBC_VERSION = keccak256("ucs03-zkgm-0");

    error ErrUnsupportedVersion();
    error ErrUnimplemented();
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
    error ErrInvalidMultiplexSender();

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

    function decodeFungibleAssetOrderV0(
        bytes calldata stream
    ) internal pure returns (FungibleAssetOrderV0 calldata) {
        FungibleAssetOrderV0 calldata operand;
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

    function lastChannelFromPath(
        uint256 path
    ) internal pure returns (uint32) {
        if (path == 0) {
            return 0;
        }
        uint256 currentHopIndex = LibBit.fls(path) / 32;
        return uint32(path >> currentHopIndex * 32);
    }

    function isAllowedBatchInstruction(
        uint8 opcode
    ) internal returns (bool) {
        return opcode == OP_MULTIPLEX || opcode == OP_FUNGIBLE_ASSET_ORDER;
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

    IIBCPacket public ibcHandler;
    mapping(bytes32 => IBCPacket) public inFlightPacket;
    mapping(uint32 => mapping(address => uint256)) public channelBalance;
    mapping(address => uint256) public tokenOrigin;
    IWETH public weth;

    constructor() {
        _disableInitializers();
    }

    function initialize(
        IIBCPacket _ibcHandler,
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

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
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
        uint256 origin = tokenOrigin[baseToken];
        // Verify the unwrap
        (address wrappedToken,) =
            internalPredictWrappedTokenMemory(0, channelId, quoteToken);
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
            SafeERC20.safeTransferFrom(
                IERC20(baseToken), msg.sender, address(this), baseAmount
            );
            channelBalance[channelId][address(baseToken)] += baseAmount;
        }
        // TODO: make this non-failable as it's not guaranteed to exist
        Instruction[] memory instructions = new Instruction[](2);
        IERC20Metadata sentTokenMeta = IERC20Metadata(baseToken);
        string memory symbol = sentTokenMeta.symbol();
        string memory name = sentTokenMeta.name();
        uint8 decimals = sentTokenMeta.decimals();
        instructions[0] = Instruction({
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
        instructions[1] = Instruction({
            version: ZkgmLib.INSTR_VERSION_0,
            opcode: ZkgmLib.OP_MULTIPLEX,
            operand: ZkgmLib.encodeMultiplex(
                Multiplex({
                    sender: abi.encodePacked(msg.sender),
                    eureka: true,
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

    function call(
        uint32 channelId,
        bytes calldata contractAddress,
        bytes calldata contractCalldata,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes32 salt
    ) public {
        ibcHandler.sendPacket(
            channelId,
            timeoutHeight,
            timeoutTimestamp,
            ZkgmLib.encode(
                ZkgmPacket({
                    salt: keccak256(abi.encodePacked(msg.sender, salt)),
                    path: 0,
                    instruction: Instruction({
                        version: ZkgmLib.INSTR_VERSION_0,
                        opcode: ZkgmLib.OP_MULTIPLEX,
                        operand: ZkgmLib.encodeMultiplex(
                            Multiplex({
                                sender: abi.encodePacked(msg.sender),
                                eureka: true,
                                contractAddress: contractAddress,
                                contractCalldata: contractCalldata
                            })
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
        uint256 origin = tokenOrigin[baseToken];
        // Verify the unwrap
        (address wrappedToken,) =
            internalPredictWrappedTokenMemory(0, channelId, quoteToken);
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
            SafeERC20.safeTransferFrom(
                IERC20(baseToken), msg.sender, address(this), baseAmount
            );
            channelBalance[channelId][address(baseToken)] += baseAmount;
        }

        // TODO: make calls to sentTokenMeta non-failable as it's not guaranteed to exist
        IERC20Metadata sentTokenMeta = IERC20Metadata(baseToken);
        uint256 nbOfInstructions = 1;
        if (msg.value > 0) {
            nbOfInstructions = 2;
        }
        Instruction[] memory instructions = new Instruction[](nbOfInstructions);
        instructions[0] = Instruction({
            version: ZkgmLib.INSTR_VERSION_1,
            opcode: ZkgmLib.OP_FUNGIBLE_ASSET_ORDER,
            operand: ZkgmLib.encodeFungibleAssetOrder(
                FungibleAssetOrder({
                    sender: abi.encodePacked(msg.sender),
                    receiver: receiver,
                    baseToken: abi.encodePacked(baseToken),
                    baseTokenPath: origin,
                    baseTokenSymbol: sentTokenMeta.symbol(),
                    baseTokenName: sentTokenMeta.name(),
                    baseTokenDecimals: sentTokenMeta.decimals(),
                    baseAmount: baseAmount,
                    quoteToken: quoteToken,
                    quoteAmount: quoteAmount
                })
            )
        });
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
                        baseTokenName: "Wrapped ETH",
                        baseTokenDecimals: 18,
                        baseAmount: msg.value,
                        quoteToken: wethQuoteToken,
                        // means we pay the relayer on destination for 100% of the value
                        quoteAmount: 0
                    })
                )
            });
            channelBalance[channelId][address(wethBaseToken)] += msg.value;
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

    function transfer(
        uint32 channelId,
        bytes calldata receiver,
        address baseToken,
        uint256 baseAmount,
        bytes calldata quoteToken,
        uint256 quoteAmount,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes32 salt
    ) public {
        if (baseAmount == 0) {
            revert ZkgmLib.ErrInvalidAmount();
        }
        uint256 origin = tokenOrigin[baseToken];
        // Verify the unwrap
        (address wrappedToken,) =
            internalPredictWrappedTokenMemory(0, channelId, quoteToken);
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
            SafeERC20.safeTransferFrom(
                IERC20(baseToken), msg.sender, address(this), baseAmount
            );
            channelBalance[channelId][address(baseToken)] += baseAmount;
        }
        // TODO: make this non-failable as it's not guaranteed to exist
        IERC20Metadata sentTokenMeta = IERC20Metadata(baseToken);
        ibcHandler.sendPacket(
            channelId,
            timeoutHeight,
            timeoutTimestamp,
            ZkgmLib.encode(
                ZkgmPacket({
                    salt: salt,
                    path: 0,
                    instruction: Instruction({
                        version: ZkgmLib.INSTR_VERSION_1,
                        opcode: ZkgmLib.OP_FUNGIBLE_ASSET_ORDER,
                        operand: ZkgmLib.encodeFungibleAssetOrder(
                            FungibleAssetOrder({
                                sender: abi.encodePacked(msg.sender),
                                receiver: receiver,
                                baseToken: abi.encodePacked(baseToken),
                                baseTokenPath: origin,
                                baseTokenSymbol: sentTokenMeta.symbol(),
                                baseTokenName: sentTokenMeta.name(),
                                baseTokenDecimals: sentTokenMeta.decimals(),
                                baseAmount: baseAmount,
                                quoteToken: quoteToken,
                                quoteAmount: quoteAmount
                            })
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
                // TODO: change salt to string and then assert its prefixed with user address and keccak256 it
                ZkgmPacket({salt: salt, path: 0, instruction: instruction})
            )
        );
    }

    function verifyInternal(
        uint32 channelId,
        uint256 path,
        Instruction calldata instruction
    ) internal {
        if (instruction.opcode == ZkgmLib.OP_FUNGIBLE_ASSET_ORDER) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_1) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            if (instruction.version == ZkgmLib.INSTR_VERSION_0) {
                FungibleAssetOrderV0 calldata order =
                    ZkgmLib.decodeFungibleAssetOrderV0(instruction.operand);
                verifyFungibleAssetOrder(
                    channelId,
                    path,
                    order.baseToken,
                    order.baseAmount,
                    order.baseTokenSymbol,
                    order.baseTokenName,
                    order.baseTokenPath,
                    order.quoteToken,
                    order.quoteAmount
                );
            } else {
                FungibleAssetOrder calldata order =
                    ZkgmLib.decodeFungibleAssetOrder(instruction.operand);
                verifyFungibleAssetOrder(
                    channelId,
                    path,
                    order.baseToken,
                    order.baseAmount,
                    order.baseTokenSymbol,
                    order.baseTokenName,
                    order.baseTokenPath,
                    order.quoteToken,
                    order.quoteAmount
                );
            }
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
            verifyForward(
                channelId, path, ZkgmLib.decodeForward(instruction.operand)
            );
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
        bytes calldata orderBaseToken,
        uint256 orderBaseAmount,
        string calldata orderBaseTokenSymbol,
        string calldata orderBaseTokenName,
        uint256 orderBaseTokenPath,
        bytes calldata orderQuoteToken,
        uint256 orderQuoteAmount
    ) internal {
        if (orderBaseAmount == 0) {
            revert ZkgmLib.ErrInvalidAmount();
        }
        IERC20Metadata baseToken =
            IERC20Metadata(address(bytes20(orderBaseToken)));
        if (!orderBaseTokenName.eq(baseToken.name())) {
            revert ZkgmLib.ErrInvalidAssetName();
        }
        if (!orderBaseTokenSymbol.eq(baseToken.symbol())) {
            revert ZkgmLib.ErrInvalidAssetSymbol();
        }
        uint256 origin = tokenOrigin[address(baseToken)];
        (address wrappedToken,) =
            internalPredictWrappedTokenMemory(0, channelId, orderQuoteToken);
        if (
            ZkgmLib.lastChannelFromPath(origin) == channelId
                && abi.encodePacked(orderBaseToken).eq(
                    abi.encodePacked(wrappedToken)
                )
        ) {
            IZkgmERC20(address(baseToken)).burn(msg.sender, orderBaseAmount);
        } else {
            origin = 0;
            // TODO: extract this as a step before verifying to allow for ERC777
            // send hook
            SafeERC20.safeTransferFrom(
                baseToken, msg.sender, address(this), orderBaseAmount
            );
            channelBalance[channelId][address(baseToken)] += orderBaseAmount;
        }
        if (orderBaseTokenPath != origin) {
            revert ZkgmLib.ErrInvalidAssetOrigin();
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
        uint256 path,
        Forward calldata forward
    ) internal {
        verifyInternal(
            channelId,
            ZkgmLib.updateChannelPath(path, forward.channelId),
            forward.instruction
        );
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
        IBCPacket calldata operand,
        address relayer,
        bytes calldata relayerMsg
    ) external virtual override onlyIBC returns (bytes memory) {
        (bool success, bytes memory returnData) = address(this).call(
            abi.encodeCall(this.execute, (operand, relayer, relayerMsg))
        );
        bytes memory acknowledgement = abi.decode(returnData, (bytes));
        if (success) {
            // The acknowledgement may be asynchronous (forward/multiplex).
            if (acknowledgement.length == 0) {
                return ZkgmLib.ACK_EMPTY;
            }

            // Special case where we should avoid the packet from being
            // received entirely as it is only fillable by a market maker.
            if (
                keccak256(acknowledgement)
                    == keccak256(ZkgmLib.ACK_ERR_ONLYMAKER)
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
            if (instruction.version > ZkgmLib.INSTR_VERSION_1) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            if (instruction.version == ZkgmLib.INSTR_VERSION_0) {
                FungibleAssetOrderV0 calldata order =
                    ZkgmLib.decodeFungibleAssetOrderV0(instruction.operand);
                return executeFungibleAssetOrder(
                    ibcPacket,
                    relayer,
                    relayerMsg,
                    salt,
                    path,
                    order.sender,
                    order.receiver,
                    order.baseToken,
                    order.baseAmount,
                    order.baseTokenSymbol,
                    order.baseTokenName,
                    order.baseTokenPath,
                    order.quoteToken,
                    order.quoteAmount
                );
            } else {
                FungibleAssetOrder calldata order =
                    ZkgmLib.decodeFungibleAssetOrder(instruction.operand);
                return executeFungibleAssetOrder(
                    ibcPacket,
                    relayer,
                    relayerMsg,
                    salt,
                    path,
                    order.sender,
                    order.receiver,
                    order.baseToken,
                    order.baseAmount,
                    order.baseTokenSymbol,
                    order.baseTokenName,
                    order.baseTokenPath,
                    order.quoteToken,
                    order.quoteAmount
                );
            }
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
                keccak256(abi.encode(i, salt)),
                path,
                instruction
            );
            if (acks[i].length == 0) {
                revert ZkgmLib.ErrBatchMustBeSync();
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
        Forward calldata forward
    ) internal returns (bytes memory) {
        // TODO: consider using a magic value for few bytes of the salt in order
        // to know that it's a forwarded instruction in the acknowledgement, without
        // having to index in `inFlightPacket`, saving gas in the process.
        IBCPacket memory sentPacket = ibcHandler.sendPacket(
            forward.channelId,
            forward.timeoutHeight,
            forward.timeoutTimestamp,
            ZkgmLib.encode(
                ZkgmPacket({
                    salt: keccak256(abi.encode(salt)),
                    path: ZkgmLib.updateChannelPath(
                        path, ibcPacket.destinationChannelId
                    ),
                    instruction: forward.instruction
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
        bytes32 salt,
        Multiplex calldata multiplex
    ) internal returns (bytes memory) {
        address contractAddress = address(bytes20(multiplex.contractAddress));
        if (multiplex.eureka) {
            IEurekaModule(contractAddress).onZkgm(
                ibcPacket.destinationChannelId,
                multiplex.sender,
                multiplex.contractCalldata
            );
            return abi.encode(ZkgmLib.ACK_SUCCESS);
        } else {
            IBCPacket memory multiplexIbcPacket = IBCPacket({
                sourceChannelId: ibcPacket.sourceChannelId,
                destinationChannelId: ibcPacket.destinationChannelId,
                data: abi.encode(multiplex.sender, multiplex.contractCalldata),
                timeoutHeight: ibcPacket.timeoutHeight,
                timeoutTimestamp: ibcPacket.timeoutTimestamp
            });
            bytes memory acknowledgement = IIBCModule(contractAddress)
                .onRecvPacket(multiplexIbcPacket, relayer, relayerMsg);
            if (acknowledgement.length == 0) {
                /* TODO: store the packet to handle async acks on
                   multiplexing, we need to have a mapping from (receiver,
                   virtualPacket) => ibcPacket. Then the receiver will be the
                   only one able to acknowledge a virtual packet, resulting in
                   the origin ibc packet to be acknowledged itself.
                 */
                revert ZkgmLib.ErrUnimplemented();
            }
            return acknowledgement;
        }
    }

    function internalPredictWrappedToken(
        uint256 path,
        uint32 channel,
        bytes calldata token
    ) internal view returns (address, bytes32) {
        bytes32 wrappedTokenSalt = keccak256(abi.encode(path, channel, token));
        address wrappedToken =
            CREATE3.predictDeterministicAddress(wrappedTokenSalt);
        return (wrappedToken, wrappedTokenSalt);
    }

    function internalPredictWrappedTokenMemory(
        uint256 path,
        uint32 channel,
        bytes memory token
    ) internal view returns (address, bytes32) {
        bytes32 wrappedTokenSalt = keccak256(abi.encode(path, channel, token));
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

    function executeFungibleAssetOrder(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        uint256 path,
        bytes calldata orderSender,
        bytes calldata orderReceiver,
        bytes calldata orderBaseToken,
        uint256 orderBaseAmount,
        string calldata orderBaseTokenSymbol,
        string calldata orderBaseTokenName,
        uint256 orderBaseTokenPath,
        bytes calldata orderQuoteToken,
        uint256 orderQuoteAmount
    ) internal returns (bytes memory) {
        // The protocol can only wrap or unwrap an asset, hence 1:1 baked.
        // The fee is the difference, which can only be positive.
        if (orderQuoteAmount > orderBaseAmount) {
            return ZkgmLib.ACK_ERR_ONLYMAKER;
        }
        (address wrappedToken, bytes32 wrappedTokenSalt) =
        internalPredictWrappedToken(
            path, ibcPacket.destinationChannelId, orderBaseToken
        );
        address quoteToken = address(bytes20(orderQuoteToken));
        address receiver = address(bytes20(orderReceiver));
        // Previously asserted to be <=.
        uint256 fee = orderBaseAmount - orderQuoteAmount;
        if (quoteToken == wrappedToken) {
            if (!ZkgmLib.isDeployed(wrappedToken)) {
                CREATE3.deployDeterministic(
                    abi.encodePacked(
                        type(ZkgmERC20).creationCode,
                        abi.encode(
                            orderBaseTokenName,
                            orderBaseTokenSymbol,
                            address(this)
                        )
                    ),
                    wrappedTokenSalt
                );
                tokenOrigin[wrappedToken] = ZkgmLib.updateChannelPath(
                    path, ibcPacket.destinationChannelId
                );
            }
            IZkgmERC20(wrappedToken).mint(receiver, orderQuoteAmount);
            if (fee > 0) {
                IZkgmERC20(wrappedToken).mint(relayer, fee);
            }
        } else {
            // TODO: should be slightly more complicated, we have to check that
            // the path is the inverse of the baseTokenPath
            if (orderBaseTokenPath == ibcPacket.sourceChannelId) {
                channelBalance[ibcPacket.destinationChannelId][quoteToken] -=
                    orderBaseAmount;
                SafeERC20.safeTransfer(
                    IERC20(quoteToken), receiver, orderQuoteAmount
                );
                if (fee > 0) {
                    SafeERC20.safeTransfer(IERC20(quoteToken), relayer, fee);
                }
            } else {
                return ZkgmLib.ACK_ERR_ONLYMAKER;
            }
        }
        return ZkgmLib.encodeFungibleAssetOrderAck(
            FungibleAssetOrderAck({
                fillType: ZkgmLib.FILL_TYPE_PROTOCOL,
                marketMaker: ZkgmLib.ACK_EMPTY
            })
        );
    }

    function onAcknowledgementPacket(
        IBCPacket calldata ibcPacket,
        bytes calldata ack,
        address relayer
    ) external virtual override onlyIBC {
        bytes32 packetHash = IBCPacketLib.commitPacket(ibcPacket);
        IBCPacket memory parent = inFlightPacket[packetHash];
        // Specific case of forwarding where the ack is threaded back directly.
        if (parent.timeoutTimestamp != 0 || parent.timeoutHeight != 0) {
            ibcHandler.writeAcknowledgement(parent, ack);
            delete inFlightPacket[packetHash];
        } else {
            ZkgmPacket calldata zkgmPacket = ZkgmLib.decode(ibcPacket.data);
            Ack calldata zkgmAck = ZkgmLib.decodeAck(ack);
            acknowledgeInternal(
                ibcPacket,
                relayer,
                zkgmPacket.salt,
                zkgmPacket.instruction,
                zkgmAck.tag == ZkgmLib.ACK_SUCCESS,
                zkgmAck.innerAck
            );
        }
    }

    function acknowledgeInternal(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        Instruction calldata instruction,
        bool successful,
        bytes calldata ack
    ) internal {
        if (instruction.opcode == ZkgmLib.OP_FUNGIBLE_ASSET_ORDER) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_1) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            if (instruction.version == ZkgmLib.INSTR_VERSION_0) {
                FungibleAssetOrderV0 calldata order =
                    ZkgmLib.decodeFungibleAssetOrderV0(instruction.operand);
                acknowledgeFungibleAssetOrder(
                    ibcPacket,
                    relayer,
                    salt,
                    order.sender,
                    order.baseToken,
                    order.baseTokenPath,
                    order.baseAmount,
                    successful,
                    ack
                );
            } else {
                FungibleAssetOrder calldata order =
                    ZkgmLib.decodeFungibleAssetOrder(instruction.operand);
                acknowledgeFungibleAssetOrder(
                    ibcPacket,
                    relayer,
                    salt,
                    order.sender,
                    order.baseToken,
                    order.baseTokenPath,
                    order.baseAmount,
                    successful,
                    ack
                );
            }
        } else if (instruction.opcode == ZkgmLib.OP_BATCH) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            acknowledgeBatch(
                ibcPacket,
                relayer,
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
                keccak256(abi.encode(i, salt)),
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
    ) internal {}

    function acknowledgeMultiplex(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        Multiplex calldata multiplex,
        bool successful,
        bytes calldata ack
    ) internal {
        if (successful && !multiplex.eureka) {
            IBCPacket memory multiplexIbcPacket = IBCPacket({
                sourceChannelId: ibcPacket.sourceChannelId,
                destinationChannelId: ibcPacket.destinationChannelId,
                data: abi.encode(
                    multiplex.contractAddress, multiplex.contractCalldata
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
                if (
                    ZkgmLib.lastChannelFromPath(orderBaseTokenPath)
                        == ibcPacket.sourceChannelId
                ) {
                    IZkgmERC20(address(baseToken)).mint(
                        marketMaker, orderBaseAmount
                    );
                } else {
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
        IBCPacket memory parent = inFlightPacket[packetHash];
        // Specific case of forwarding where the failure is threaded back directly.
        if (parent.timeoutTimestamp != 0 || parent.timeoutHeight != 0) {
            ibcHandler.writeAcknowledgement(
                parent,
                ZkgmLib.encodeAck(
                    Ack({tag: ZkgmLib.ACK_FAILURE, innerAck: ZkgmLib.ACK_EMPTY})
                )
            );
            delete inFlightPacket[packetHash];
        } else {
            ZkgmPacket calldata zkgmPacket = ZkgmLib.decode(ibcPacket.data);
            timeoutInternal(
                ibcPacket, relayer, zkgmPacket.salt, zkgmPacket.instruction
            );
        }
    }

    function timeoutInternal(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        Instruction calldata instruction
    ) internal {
        if (instruction.opcode == ZkgmLib.OP_FUNGIBLE_ASSET_ORDER) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_1) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            if (instruction.version == ZkgmLib.INSTR_VERSION_0) {
                FungibleAssetOrderV0 calldata order =
                    ZkgmLib.decodeFungibleAssetOrderV0(instruction.operand);
                timeoutFungibleAssetOrder(
                    ibcPacket,
                    relayer,
                    salt,
                    order.sender,
                    order.baseToken,
                    order.baseTokenPath,
                    order.baseAmount
                );
            } else {
                FungibleAssetOrder calldata order =
                    ZkgmLib.decodeFungibleAssetOrder(instruction.operand);
                timeoutFungibleAssetOrder(
                    ibcPacket,
                    relayer,
                    salt,
                    order.sender,
                    order.baseToken,
                    order.baseTokenPath,
                    order.baseAmount
                );
            }
        } else if (instruction.opcode == ZkgmLib.OP_BATCH) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            timeoutBatch(
                ibcPacket,
                relayer,
                salt,
                ZkgmLib.decodeBatch(instruction.operand)
            );
        } else if (instruction.opcode == ZkgmLib.OP_FORWARD) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            timeoutForward(
                ibcPacket,
                relayer,
                salt,
                ZkgmLib.decodeForward(instruction.operand)
            );
        } else if (instruction.opcode == ZkgmLib.OP_MULTIPLEX) {
            if (instruction.version > ZkgmLib.INSTR_VERSION_0) {
                revert ZkgmLib.ErrUnsupportedVersion();
            }
            timeoutMultiplex(
                ibcPacket,
                relayer,
                salt,
                ZkgmLib.decodeMultiplex(instruction.operand)
            );
        } else {
            revert ZkgmLib.ErrUnknownOpcode();
        }
    }

    function timeoutBatch(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        Batch calldata batch
    ) internal {
        uint256 l = batch.instructions.length;
        for (uint256 i = 0; i < l; i++) {
            timeoutInternal(
                ibcPacket,
                relayer,
                keccak256(abi.encode(i, salt)),
                batch.instructions[i]
            );
        }
    }

    function timeoutForward(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        Forward calldata forward
    ) internal {}

    function timeoutMultiplex(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        Multiplex calldata multiplex
    ) internal {
        if (!multiplex.eureka) {
            IBCPacket memory multiplexIbcPacket = IBCPacket({
                sourceChannelId: ibcPacket.sourceChannelId,
                destinationChannelId: ibcPacket.destinationChannelId,
                data: abi.encode(
                    multiplex.contractAddress, multiplex.contractCalldata
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
        address relayer,
        bytes32 salt,
        bytes calldata orderSender,
        bytes calldata orderBaseToken,
        uint256 orderBaseTokenPath,
        uint256 orderBaseAmount
    ) internal {
        refund(
            ibcPacket.sourceChannelId,
            orderSender,
            orderBaseToken,
            orderBaseTokenPath,
            orderBaseAmount
        );
    }

    function refund(
        uint32 sourceChannelId,
        bytes calldata orderSender,
        bytes calldata orderBaseToken,
        uint256 orderBaseTokenPath,
        uint256 orderBaseAmount
    ) internal {
        address sender = address(bytes20(orderSender));
        address baseToken = address(bytes20(orderBaseToken));
        if (ZkgmLib.lastChannelFromPath(orderBaseTokenPath) == sourceChannelId)
        {
            IZkgmERC20(address(baseToken)).mint(sender, orderBaseAmount);
        } else {
            SafeERC20.safeTransfer(IERC20(baseToken), sender, orderBaseAmount);
        }
    }

    function onChanOpenInit(
        uint32,
        uint32,
        string calldata version,
        address
    ) external virtual override onlyIBC {
        if (keccak256(bytes(version)) != ZkgmLib.IBC_VERSION) {
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
        if (keccak256(bytes(version)) != ZkgmLib.IBC_VERSION) {
            revert ZkgmLib.ErrInvalidIBCVersion();
        }
        if (keccak256(bytes(counterpartyVersion)) != ZkgmLib.IBC_VERSION) {
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
