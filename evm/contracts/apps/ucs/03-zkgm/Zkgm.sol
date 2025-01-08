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

import "../../Base.sol";
import "../../../core/04-channel/IBCPacket.sol";
import "../../../core/05-port/IIBCModule.sol";

import "./IEurekaModule.sol";
import "./IZkgmERC20.sol";
import "./ZkgmERC20.sol";

struct ZkgmPacket {
    bytes32 salt;
    uint256 path;
    SyscallPacket syscall;
}

struct SyscallPacket {
    uint8 version;
    uint8 index;
    bytes packet;
}

struct ForwardPacket {
    uint32 channelId;
    uint64 timeoutHeight;
    uint64 timeoutTimestamp;
    SyscallPacket syscallPacket;
}

struct MultiplexPacket {
    bytes sender;
    bool eureka;
    bytes contractAddress;
    bytes contractCalldata;
}

struct BatchPacket {
    SyscallPacket[] syscallPackets;
}

struct FungibleAssetTransferPacket {
    bytes sender;
    bytes receiver;
    bytes sentToken;
    uint256 sentTokenPrefix;
    string sentSymbol;
    string sentName;
    uint256 sentAmount;
    bytes askToken;
    uint256 askAmount;
    bool onlyMaker;
}

struct Acknowledgement {
    uint256 tag;
    bytes innerAck;
}

struct BatchAcknowledgement {
    bytes[] acknowledgements;
}

struct AssetTransferAcknowledgement {
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

    uint8 public constant SYSCALL_FORWARD = 0x00;
    uint8 public constant SYSCALL_MULTIPLEX = 0x01;
    uint8 public constant SYSCALL_BATCH = 0x02;
    uint8 public constant SYSCALL_FUNGIBLE_ASSET_TRANSFER = 0x03;

    uint8 public constant ZKGM_VERSION_0 = 0x00;

    bytes32 public constant IBC_VERSION = keccak256("ucs03-zkgm-0");

    error ErrUnsupportedVersion();
    error ErrUnimplemented();
    error ErrBatchMustBeSync();
    error ErrUnknownSyscall();
    error ErrInfiniteGame();
    error ErrUnauthorized();
    error ErrInvalidAmount();
    error ErrOnlyMaker();
    error ErrInvalidFillType();
    error ErrInvalidIBCVersion();
    error ErrInvalidHops();
    error ErrInvalidAssetOrigin();
    error ErrInvalidAssetSymbol();
    error ErrInvalidAssetName();

    function encodeAssetTransferAck(
        AssetTransferAcknowledgement memory ack
    ) internal pure returns (bytes memory) {
        return abi.encode(ack.fillType, ack.marketMaker);
    }

    function decodeAssetTransferAck(
        bytes calldata stream
    ) internal pure returns (AssetTransferAcknowledgement calldata) {
        AssetTransferAcknowledgement calldata ack;
        assembly {
            ack := stream.offset
        }
        return ack;
    }

    function encodeBatchAck(
        BatchAcknowledgement memory ack
    ) internal pure returns (bytes memory) {
        return abi.encode(ack.acknowledgements);
    }

    function decodeBatchAck(
        bytes calldata stream
    ) internal pure returns (BatchAcknowledgement calldata) {
        BatchAcknowledgement calldata acks;
        assembly {
            acks := stream.offset
        }
        return acks;
    }

    function encodeAck(
        Acknowledgement memory ack
    ) internal pure returns (bytes memory) {
        return abi.encode(ack.tag, ack.innerAck);
    }

    function decodeAck(
        bytes calldata stream
    ) internal pure returns (Acknowledgement calldata) {
        Acknowledgement calldata packet;
        assembly {
            packet := stream.offset
        }
        return packet;
    }

    function encode(
        ZkgmPacket memory packet
    ) internal pure returns (bytes memory) {
        return abi.encode(packet.salt, packet.path, packet.syscall);
    }

    function decode(
        bytes calldata stream
    ) internal pure returns (ZkgmPacket calldata) {
        ZkgmPacket calldata packet;
        assembly {
            packet := stream.offset
        }
        return packet;
    }

    function decodeBatch(
        bytes calldata stream
    ) internal pure returns (BatchPacket calldata) {
        BatchPacket calldata packet;
        assembly {
            packet := stream.offset
        }
        return packet;
    }

    function decodeForward(
        bytes calldata stream
    ) internal pure returns (ForwardPacket calldata) {
        ForwardPacket calldata packet;
        assembly {
            packet := stream.offset
        }
        return packet;
    }

    function decodeMultiplex(
        bytes calldata stream
    ) internal pure returns (MultiplexPacket calldata) {
        MultiplexPacket calldata packet;
        assembly {
            packet := stream.offset
        }
        return packet;
    }

    function encodeFungibleAssetTransfer(
        FungibleAssetTransferPacket memory transfer
    ) internal pure returns (bytes memory) {
        return abi.encode(
            transfer.sender,
            transfer.receiver,
            transfer.sentToken,
            transfer.sentTokenPrefix,
            transfer.sentSymbol,
            transfer.sentName,
            transfer.sentAmount,
            transfer.askToken,
            transfer.askAmount,
            transfer.onlyMaker
        );
    }

    function decodeFungibleAssetTransfer(
        bytes calldata stream
    ) internal pure returns (FungibleAssetTransferPacket calldata) {
        FungibleAssetTransferPacket calldata packet;
        assembly {
            packet := stream.offset
        }
        return packet;
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

    IIBCPacket public ibcHandler;
    mapping(bytes32 => IBCPacket) public inFlightPacket;
    mapping(uint32 => mapping(address => uint256)) public channelBalance;
    mapping(address => uint256) public tokenOrigin;

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

    function transfer(
        uint32 channelId,
        uint64 timeoutHeight,
        uint64 timeoutTimestamp,
        bytes32 salt,
        bytes calldata receiver,
        address sentToken,
        uint256 sentAmount,
        bytes calldata askToken,
        uint256 askAmount,
        bool onlyMaker
    ) public {
        if(sentAmount == 0) {
            revert ZkgmLib.ErrInvalidAmount();
        }
        // TODO: make this non-failable as it's not guaranteed to exist
        IERC20Metadata sentTokenMeta = IERC20Metadata(sentToken);
        string memory tokenName = sentTokenMeta.name();
        string memory tokenSymbol = sentTokenMeta.symbol();
        uint256 origin = tokenOrigin[sentToken];
        if (ZkgmLib.lastChannelFromPath(origin) == channelId) {
            IZkgmERC20(sentToken).burn(msg.sender, sentAmount);
        } else {
            // TODO: extract this as a step before verifying to allow for ERC777
            // send hook
            SafeERC20.safeTransferFrom(
                sentTokenMeta, msg.sender, address(this), sentAmount
            );
            channelBalance[channelId][address(sentToken)] += sentAmount;
        }
        ibcHandler.sendPacket(
            channelId,
            timeoutHeight,
            timeoutTimestamp,
            ZkgmLib.encode(
                ZkgmPacket({
                    salt: salt,
                    path: 0,
                    syscall: SyscallPacket({
                        version: ZkgmLib.ZKGM_VERSION_0,
                        index: ZkgmLib.SYSCALL_FUNGIBLE_ASSET_TRANSFER,
                        packet: ZkgmLib.encodeFungibleAssetTransfer(
                            FungibleAssetTransferPacket({
                                sender: abi.encodePacked(msg.sender),
                                receiver: receiver,
                                sentToken: abi.encodePacked(sentToken),
                                sentTokenPrefix: origin,
                                sentSymbol: tokenSymbol,
                                sentName: tokenName,
                                sentAmount: sentAmount,
                                askToken: askToken,
                                askAmount: askAmount,
                                onlyMaker: onlyMaker
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
        SyscallPacket calldata syscallPacket
    ) public {
        verifyInternal(channelId, 0, syscallPacket);
        ibcHandler.sendPacket(
            channelId,
            timeoutHeight,
            timeoutTimestamp,
            ZkgmLib.encode(
                // TODO: change salt to string and then assert its prefixed with user address and keccak256 it
                ZkgmPacket({salt: salt, path: 0, syscall: syscallPacket})
            )
        );
    }

    function verifyInternal(
        uint32 channelId,
        uint256 path,
        SyscallPacket calldata syscallPacket
    ) internal {
        if (syscallPacket.version != ZkgmLib.ZKGM_VERSION_0) {
            revert ZkgmLib.ErrUnsupportedVersion();
        }
        if (syscallPacket.index == ZkgmLib.SYSCALL_FUNGIBLE_ASSET_TRANSFER) {
            verifyFungibleAssetTransfer(
                channelId,
                path,
                ZkgmLib.decodeFungibleAssetTransfer(syscallPacket.packet)
            );
        } else if (syscallPacket.index == ZkgmLib.SYSCALL_BATCH) {
            verifyBatch(
                channelId, path, ZkgmLib.decodeBatch(syscallPacket.packet)
            );
        } else if (syscallPacket.index == ZkgmLib.SYSCALL_FORWARD) {
            verifyForward(
                channelId, path, ZkgmLib.decodeForward(syscallPacket.packet)
            );
        } else if (syscallPacket.index == ZkgmLib.SYSCALL_MULTIPLEX) {
            verifyMultiplex(
                channelId, path, ZkgmLib.decodeMultiplex(syscallPacket.packet)
            );
        } else {
            revert ZkgmLib.ErrUnknownSyscall();
        }
    }

    function verifyFungibleAssetTransfer(
        uint32 channelId,
        uint256 path,
        FungibleAssetTransferPacket calldata assetTransferPacket
    ) internal {
        if(assetTransfer.sentToken == 0) {
            revert ZkgmLib.ErrInvalidAmount();
        }
        IERC20Metadata sentToken =
            IERC20Metadata(address(bytes20(assetTransferPacket.sentToken)));
        if (!assetTransferPacket.sentName.eq(sentToken.name())) {
            revert ZkgmLib.ErrInvalidAssetName();
        }
        if (!assetTransferPacket.sentSymbol.eq(sentToken.symbol())) {
            revert ZkgmLib.ErrInvalidAssetSymbol();
        }
        uint256 origin = tokenOrigin[address(sentToken)];
        if (ZkgmLib.lastChannelFromPath(origin) == channelId) {
            IZkgmERC20(address(sentToken)).burn(
                msg.sender, assetTransferPacket.sentAmount
            );
        } else {
            // TODO: extract this as a step before verifying to allow for ERC777
            // send hook
            SafeERC20.safeTransferFrom(
                sentToken,
                msg.sender,
                address(this),
                assetTransferPacket.sentAmount
            );
            channelBalance[channelId][address(sentToken)] +=
                assetTransferPacket.sentAmount;
        }
        if (!assetTransferPacket.onlyMaker) {
            if (assetTransferPacket.sentTokenPrefix != origin) {
                revert ZkgmLib.ErrInvalidAssetOrigin();
            }
        }
    }

    function verifyBatch(
        uint32 channelId,
        uint256 path,
        BatchPacket calldata batchPacket
    ) internal {
        uint256 l = batchPacket.syscallPackets.length;
        for (uint256 i = 0; i < l; i++) {
            verifyInternal(channelId, path, batchPacket.syscallPackets[i]);
        }
    }

    function verifyForward(
        uint32 channelId,
        uint256 path,
        ForwardPacket calldata forwardPacket
    ) internal {
        verifyInternal(
            channelId,
            ZkgmLib.updateChannelPath(path, forwardPacket.channelId),
            forwardPacket.syscallPacket
        );
    }

    function verifyMultiplex(
        uint32 channelId,
        uint256 path,
        MultiplexPacket calldata multiplexPacket
    ) internal {}

    function onRecvPacket(
        IBCPacket calldata packet,
        address relayer,
        bytes calldata relayerMsg
    ) external virtual override onlyIBC returns (bytes memory) {
        (bool success, bytes memory returnData) = address(this).call(
            abi.encodeCall(this.execute, (packet, relayer, relayerMsg))
        );
        bytes memory acknowledgement = abi.decode(returnData, (bytes));
        if (success) {
            // The acknowledgement may be asynchronous (forward/multiplex)
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
                Acknowledgement({
                    tag: ZkgmLib.ACK_SUCCESS,
                    innerAck: acknowledgement
                })
            );
        } else {
            return ZkgmLib.encodeAck(
                Acknowledgement({
                    tag: ZkgmLib.ACK_FAILURE,
                    innerAck: ZkgmLib.ACK_EMPTY
                })
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
            zkgmPacket.syscall
        );
    }

    function executeInternal(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        uint256 path,
        SyscallPacket calldata syscallPacket
    ) internal returns (bytes memory) {
        if (syscallPacket.version != ZkgmLib.ZKGM_VERSION_0) {
            revert ZkgmLib.ErrUnsupportedVersion();
        }
        if (syscallPacket.index == ZkgmLib.SYSCALL_FUNGIBLE_ASSET_TRANSFER) {
            return executeFungibleAssetTransfer(
                ibcPacket,
                relayer,
                relayerMsg,
                salt,
                path,
                ZkgmLib.decodeFungibleAssetTransfer(syscallPacket.packet)
            );
        } else if (syscallPacket.index == ZkgmLib.SYSCALL_BATCH) {
            return executeBatch(
                ibcPacket,
                relayer,
                relayerMsg,
                salt,
                path,
                ZkgmLib.decodeBatch(syscallPacket.packet)
            );
        } else if (syscallPacket.index == ZkgmLib.SYSCALL_FORWARD) {
            return executeForward(
                ibcPacket,
                relayer,
                relayerMsg,
                salt,
                path,
                ZkgmLib.decodeForward(syscallPacket.packet)
            );
        } else if (syscallPacket.index == ZkgmLib.SYSCALL_MULTIPLEX) {
            return executeMultiplex(
                ibcPacket,
                relayer,
                relayerMsg,
                salt,
                ZkgmLib.decodeMultiplex(syscallPacket.packet)
            );
        } else {
            revert ZkgmLib.ErrUnknownSyscall();
        }
    }

    function executeBatch(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        uint256 path,
        BatchPacket calldata batchPacket
    ) internal returns (bytes memory) {
        uint256 l = batchPacket.syscallPackets.length;
        bytes[] memory acks = new bytes[](l);
        for (uint256 i = 0; i < l; i++) {
            SyscallPacket calldata syscallPacket = batchPacket.syscallPackets[i];
            acks[i] = executeInternal(
                ibcPacket,
                relayer,
                relayerMsg,
                keccak256(abi.encode(salt)),
                path,
                syscallPacket
            );
            if (acks[i].length == 0) {
                revert ZkgmLib.ErrBatchMustBeSync();
            }
        }
        return ZkgmLib.encodeBatchAck(
            BatchAcknowledgement({acknowledgements: acks})
        );
    }

    function executeForward(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        uint256 path,
        ForwardPacket calldata forwardPacket
    ) internal returns (bytes memory) {
        // TODO: consider using a magic value for few bytes of the salt in order
        // to know that it's a forwarded packet in the acknowledgement, without
        // having to index in `inFlightPacket`, saving gas in the process.
        IBCPacket memory sentPacket = ibcHandler.sendPacket(
            forwardPacket.channelId,
            forwardPacket.timeoutHeight,
            forwardPacket.timeoutTimestamp,
            ZkgmLib.encode(
                ZkgmPacket({
                    salt: keccak256(abi.encode(salt)),
                    path: ZkgmLib.updateChannelPath(
                        path, ibcPacket.destinationChannelId
                    ),
                    syscall: forwardPacket.syscallPacket
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
        MultiplexPacket calldata multiplexPacket
    ) internal returns (bytes memory) {
        address contractAddress =
            address(bytes20(multiplexPacket.contractAddress));
        if (multiplexPacket.eureka) {
            IEurekaModule(contractAddress).onZkgm(
                multiplexPacket.sender, multiplexPacket.contractCalldata
            );
            return abi.encode(ZkgmLib.ACK_SUCCESS);
        } else {
            IBCPacket memory multiplexIbcPacket = IBCPacket({
                sourceChannelId: ibcPacket.sourceChannelId,
                destinationChannelId: ibcPacket.destinationChannelId,
                data: abi.encode(
                    multiplexPacket.sender, multiplexPacket.contractCalldata
                ),
                timeoutHeight: ibcPacket.timeoutHeight,
                timeoutTimestamp: ibcPacket.timeoutTimestamp
            });
            bytes memory acknowledgement = IIBCModule(contractAddress)
                .onRecvPacket(multiplexIbcPacket, relayer, relayerMsg);
            if (acknowledgement.length == 0) {
                /* TODO: store the packet for async ack To handle async acks on
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

    function predictWrappedToken(
        uint256 path,
        uint32 channel,
        bytes calldata token
    ) public view returns (address, bytes32) {
        return internalPredictWrappedToken(path, channel, token);
    }

    function executeFungibleAssetTransfer(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        uint256 path,
        FungibleAssetTransferPacket calldata assetTransferPacket
    ) internal returns (bytes memory) {
        if (assetTransferPacket.onlyMaker) {
            return ZkgmLib.ACK_ERR_ONLYMAKER;
        }
        // The protocol can only wrap or unwrap an asset, hence 1:1 baked.
        // The fee is the difference, which can only be positive.
        if (assetTransferPacket.askAmount > assetTransferPacket.sentAmount) {
            revert ZkgmLib.ErrInvalidAmount();
        }
        (address wrappedToken, bytes32 wrappedTokenSalt) =
        internalPredictWrappedToken(
            path, ibcPacket.destinationChannelId, assetTransferPacket.sentToken
        );
        address askToken = address(bytes20(assetTransferPacket.askToken));
        address receiver = address(bytes20(assetTransferPacket.receiver));
        // Previously asserted to be <=.
        uint256 fee =
            assetTransferPacket.sentAmount - assetTransferPacket.askAmount;
        if (askToken == wrappedToken) {
            if (!ZkgmLib.isDeployed(wrappedToken)) {
                CREATE3.deployDeterministic(
                    abi.encodePacked(
                        type(ZkgmERC20).creationCode,
                        abi.encode(
                            assetTransferPacket.sentName,
                            assetTransferPacket.sentSymbol,
                            address(this)
                        )
                    ),
                    wrappedTokenSalt
                );
                tokenOrigin[wrappedToken] = ZkgmLib.updateChannelPath(
                    path, ibcPacket.destinationChannelId
                );
            }
            IZkgmERC20(wrappedToken).mint(
                receiver, assetTransferPacket.askAmount
            );
            if (fee > 0) {
                IZkgmERC20(wrappedToken).mint(relayer, fee);
            }
        } else {
            if (
                assetTransferPacket.sentTokenPrefix == ibcPacket.sourceChannelId
            ) {
                channelBalance[ibcPacket.destinationChannelId][askToken] -=
                    assetTransferPacket.sentAmount;
                SafeERC20.safeTransfer(
                    IERC20(askToken), receiver, assetTransferPacket.askAmount
                );
                if (fee > 0) {
                    SafeERC20.safeTransfer(IERC20(askToken), relayer, fee);
                }
            } else {
                return ZkgmLib.ACK_ERR_ONLYMAKER;
            }
        }
        return ZkgmLib.encodeAssetTransferAck(
            AssetTransferAcknowledgement({
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
        bytes32 packetHash = IBCPacketLib.commitPacketMemory(ibcPacket);
        IBCPacket memory parent = inFlightPacket[packetHash];
        // Specific case of forwarding where the ack is threaded back directly.
        if (parent.timeoutTimestamp != 0 || parent.timeoutHeight != 0) {
            ibcHandler.writeAcknowledgement(parent, ack);
            delete inFlightPacket[packetHash];
        } else {
            ZkgmPacket calldata zkgmPacket = ZkgmLib.decode(ibcPacket.data);
            Acknowledgement calldata zkgmAck = ZkgmLib.decodeAck(ack);
            acknowledgeInternal(
                ibcPacket,
                relayer,
                zkgmPacket.salt,
                zkgmPacket.syscall,
                zkgmAck.tag == ZkgmLib.ACK_SUCCESS,
                zkgmAck.innerAck
            );
        }
    }

    function acknowledgeInternal(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        SyscallPacket calldata syscallPacket,
        bool successful,
        bytes calldata ack
    ) internal {
        if (syscallPacket.version != ZkgmLib.ZKGM_VERSION_0) {
            revert ZkgmLib.ErrUnsupportedVersion();
        }
        if (syscallPacket.index == ZkgmLib.SYSCALL_FUNGIBLE_ASSET_TRANSFER) {
            acknowledgeFungibleAssetTransfer(
                ibcPacket,
                relayer,
                salt,
                ZkgmLib.decodeFungibleAssetTransfer(syscallPacket.packet),
                successful,
                ack
            );
        } else if (syscallPacket.index == ZkgmLib.SYSCALL_BATCH) {
            acknowledgeBatch(
                ibcPacket,
                relayer,
                salt,
                ZkgmLib.decodeBatch(syscallPacket.packet),
                successful,
                ack
            );
        } else if (syscallPacket.index == ZkgmLib.SYSCALL_FORWARD) {
            acknowledgeForward(
                ibcPacket,
                relayer,
                salt,
                ZkgmLib.decodeForward(syscallPacket.packet),
                successful,
                ack
            );
        } else if (syscallPacket.index == ZkgmLib.SYSCALL_MULTIPLEX) {
            acknowledgeMultiplex(
                ibcPacket,
                relayer,
                salt,
                ZkgmLib.decodeMultiplex(syscallPacket.packet),
                successful,
                ack
            );
        } else {
            revert ZkgmLib.ErrUnknownSyscall();
        }
    }

    function acknowledgeBatch(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        BatchPacket calldata batchPacket,
        bool successful,
        bytes calldata ack
    ) internal {
        uint256 l = batchPacket.syscallPackets.length;
        BatchAcknowledgement calldata batchAck = ZkgmLib.decodeBatchAck(ack);
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
                keccak256(abi.encode(salt)),
                batchPacket.syscallPackets[i],
                successful,
                syscallAck
            );
        }
    }

    function acknowledgeForward(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        ForwardPacket calldata forwardPacket,
        bool successful,
        bytes calldata ack
    ) internal {}

    function acknowledgeMultiplex(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        MultiplexPacket calldata multiplexPacket,
        bool successful,
        bytes calldata ack
    ) internal {
        if (successful && !multiplexPacket.eureka) {
            IBCPacket memory multiplexIbcPacket = IBCPacket({
                sourceChannelId: ibcPacket.sourceChannelId,
                destinationChannelId: ibcPacket.destinationChannelId,
                data: abi.encode(
                    multiplexPacket.contractAddress,
                    multiplexPacket.contractCalldata
                ),
                timeoutHeight: ibcPacket.timeoutHeight,
                timeoutTimestamp: ibcPacket.timeoutTimestamp
            });
            IIBCModule(address(bytes20(multiplexPacket.sender)))
                .onAcknowledgementPacket(multiplexIbcPacket, ack, relayer);
        }
    }

    function acknowledgeFungibleAssetTransfer(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        FungibleAssetTransferPacket calldata assetTransferPacket,
        bool successful,
        bytes calldata ack
    ) internal {
        if (successful) {
            AssetTransferAcknowledgement calldata assetTransferAck =
                ZkgmLib.decodeAssetTransferAck(ack);
            if (assetTransferAck.fillType == ZkgmLib.FILL_TYPE_PROTOCOL) {
                // The protocol filled, fee was paid to relayer.
            } else if (
                assetTransferAck.fillType == ZkgmLib.FILL_TYPE_MARKETMAKER
            ) {
                // A market maker filled, we pay with the sent asset.
                address marketMaker =
                    address(bytes20(assetTransferAck.marketMaker));
                address sentToken =
                    address(bytes20(assetTransferPacket.sentToken));
                if (
                    ZkgmLib.lastChannelFromPath(
                        assetTransferPacket.sentTokenPrefix
                    ) == ibcPacket.sourceChannelId
                ) {
                    IZkgmERC20(address(sentToken)).mint(
                        marketMaker, assetTransferPacket.sentAmount
                    );
                } else {
                    SafeERC20.safeTransfer(
                        IERC20(sentToken),
                        marketMaker,
                        assetTransferPacket.sentAmount
                    );
                }
            } else {
                revert ZkgmLib.ErrInvalidFillType();
            }
        } else {
            refund(ibcPacket.sourceChannelId, assetTransferPacket);
        }
    }

    function onTimeoutPacket(
        IBCPacket calldata ibcPacket,
        address relayer
    ) external virtual override onlyIBC {
        bytes32 packetHash = IBCPacketLib.commitPacketMemory(ibcPacket);
        IBCPacket memory parent = inFlightPacket[packetHash];
        // Specific case of forwarding where the failure is threaded back directly.
        if (parent.timeoutTimestamp != 0 || parent.timeoutHeight != 0) {
            ibcHandler.writeAcknowledgement(
                parent,
                ZkgmLib.encodeAck(
                    Acknowledgement({
                        tag: ZkgmLib.ACK_FAILURE,
                        innerAck: ZkgmLib.ACK_EMPTY
                    })
                )
            );
            delete inFlightPacket[packetHash];
        } else {
            ZkgmPacket calldata zkgmPacket = ZkgmLib.decode(ibcPacket.data);
            timeoutInternal(
                ibcPacket, relayer, zkgmPacket.salt, zkgmPacket.syscall
            );
        }
    }

    function timeoutInternal(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        SyscallPacket calldata syscallPacket
    ) internal {
        if (syscallPacket.version != ZkgmLib.ZKGM_VERSION_0) {
            revert ZkgmLib.ErrUnsupportedVersion();
        }
        if (syscallPacket.index == ZkgmLib.SYSCALL_FUNGIBLE_ASSET_TRANSFER) {
            timeoutFungibleAssetTransfer(
                ibcPacket,
                relayer,
                salt,
                ZkgmLib.decodeFungibleAssetTransfer(syscallPacket.packet)
            );
        } else if (syscallPacket.index == ZkgmLib.SYSCALL_BATCH) {
            timeoutBatch(
                ibcPacket,
                relayer,
                salt,
                ZkgmLib.decodeBatch(syscallPacket.packet)
            );
        } else if (syscallPacket.index == ZkgmLib.SYSCALL_FORWARD) {
            timeoutForward(
                ibcPacket,
                relayer,
                salt,
                ZkgmLib.decodeForward(syscallPacket.packet)
            );
        } else if (syscallPacket.index == ZkgmLib.SYSCALL_MULTIPLEX) {
            timeoutMultiplex(
                ibcPacket,
                relayer,
                salt,
                ZkgmLib.decodeMultiplex(syscallPacket.packet)
            );
        } else {
            revert ZkgmLib.ErrUnknownSyscall();
        }
    }

    function timeoutBatch(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        BatchPacket calldata batchPacket
    ) internal {
        uint256 l = batchPacket.syscallPackets.length;
        for (uint256 i = 0; i < l; i++) {
            timeoutInternal(
                ibcPacket,
                relayer,
                keccak256(abi.encode(salt)),
                batchPacket.syscallPackets[i]
            );
        }
    }

    function timeoutForward(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        ForwardPacket calldata forwardPacket
    ) internal {}

    function timeoutMultiplex(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        MultiplexPacket calldata multiplexPacket
    ) internal {
        if (!multiplexPacket.eureka) {
            IBCPacket memory multiplexIbcPacket = IBCPacket({
                sourceChannelId: ibcPacket.sourceChannelId,
                destinationChannelId: ibcPacket.destinationChannelId,
                data: abi.encode(
                    multiplexPacket.contractAddress,
                    multiplexPacket.contractCalldata
                ),
                timeoutHeight: ibcPacket.timeoutHeight,
                timeoutTimestamp: ibcPacket.timeoutTimestamp
            });
            IIBCModule(address(bytes20(multiplexPacket.sender))).onTimeoutPacket(
                multiplexIbcPacket, relayer
            );
        }
    }

    function timeoutFungibleAssetTransfer(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes32 salt,
        FungibleAssetTransferPacket calldata assetTransferPacket
    ) internal {
        refund(ibcPacket.sourceChannelId, assetTransferPacket);
    }

    function refund(
        uint32 sourceChannelId,
        FungibleAssetTransferPacket calldata assetTransferPacket
    ) internal {
        address sender = address(bytes20(assetTransferPacket.sender));
        address sentToken = address(bytes20(assetTransferPacket.sentToken));
        if (
            ZkgmLib.lastChannelFromPath(assetTransferPacket.sentTokenPrefix)
                == sourceChannelId
        ) {
            IZkgmERC20(address(sentToken)).mint(
                sender, assetTransferPacket.sentAmount
            );
        } else {
            SafeERC20.safeTransfer(
                IERC20(sentToken), sender, assetTransferPacket.sentAmount
            );
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
