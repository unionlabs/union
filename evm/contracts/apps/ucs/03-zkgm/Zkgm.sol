pragma solidity ^0.8.27;

import "@openzeppelin/token/ERC20/IERC20.sol";
import "solady/utils/CREATE3.sol";

import "../../Base.sol";
import "../../../core/25-handler/IBCHandler.sol";
import "../../../core/04-channel/IBCPacket.sol";
import "../../../core/05-port/IIBCModule.sol";

import "./IEurekaModule.sol";
import "./IZkgmERC20.sol";
import "./ZkgmERC20.sol";

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

struct ZkgmPacket {
    bytes32 salt;
    bytes syscall;
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
    bytes syscallPacket;
}

struct MultiplexPacket {
    bytes sender;
    bool eureka;
    bytes contractAddress;
    bytes contractCalldata;
}

struct BatchPacket {
    bytes[] syscallPackets;
}

struct FungibleAssetTransferPacket {
    bytes sender;
    bytes receiver;
    bytes sentToken;
    uint256 sentAmount;
    bytes askToken;
    uint256 askAmount;
    bool onlyMaker;
}

library ZkgmLib {
    bytes public constant ACK_EMPTY = hex"";

    uint256 public constant ACK_FAILURE = 0x00;
    uint256 public constant ACK_SUCCESS = 0x01;

    bytes public constant ACK_ERR_ONLYMAKER = abi.encode(0xDEADC0DE);

    uint256 public constant FILL_TYPE_PROTOCOL = 0xB0CAD0;
    uint256 public constant FILL_TYPE_MARKETMAKER = 0xD1CEC45E;

    uint8 public constant SYSCALL_FORWARD = 0x00;
    uint8 public constant SYSCALL_MULTIPLEX = 0x01;
    uint8 public constant SYSCALL_BATCH = 0x02;
    uint8 public constant SYSCALL_FUNGIBLE_ASSET_TRANSFER = 0x03;

    uint8 public constant ZKGM_VERSION_0 = 0x00;

    error ErrUnsupportedVersion();
    error ErrUnimplemented();
    error ErrBatchMustBeSync();
    error ErrUnknownSyscall();
    error ErrInfiniteGame();
    error ErrUnauthorized();
    error ErrInvalidAmount();
    error ErrOnlyMaker();
    error ErrInvalidFillType();

    function encodeAssetTransferAck(
        AssetTransferAcknowledgement memory ack
    ) internal pure returns (bytes memory) {
        return abi.encode(ack);
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
        return abi.encode(ack);
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
        Acknowledgement memory packet
    ) internal pure returns (bytes memory) {
        return abi.encode(packet);
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
        return abi.encode(packet);
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

    function decodeSyscall(
        bytes calldata stream
    ) internal pure returns (SyscallPacket calldata) {
        SyscallPacket calldata packet;
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
        return (size > 0);
    }
}

contract Zkgm is IBCAppBase {
    using ZkgmLib for *;

    IBCHandler private ibcHandler;
    mapping(bytes32 => IBCPacket) private inFlightPacket;
    mapping(uint32 => mapping(address => uint256)) private channelBalance;

    constructor(
        IBCHandler _ibcHandler
    ) {
        ibcHandler = _ibcHandler;
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }

    function onRecvPacket(
        IBCPacket calldata packet,
        address relayer,
        bytes calldata relayerMsg
    ) external virtual override onlyIBC returns (bytes memory) {
        (bool success, bytes memory acknowledgement) = address(this).call(
            abi.encodeWithSelector(
                this.execute.selector, packet, packet.data, relayer, relayerMsg
            )
        );
        if (success) {
            // The acknowledgement may be asynchronous (forward/multiplex)
            if (acknowledgement.length == 0) {
                return ZkgmLib.ACK_EMPTY;
            } else if (
                keccak256(acknowledgement) == keccak256(ZkgmLib.ACK_ERR_ONLYMAKER)
            ) {
                // Special case where we should avoid the packet from being
                // received entirely as it is only fillable by a market maker.
                revert ZkgmLib.ErrOnlyMaker();
            } else {
                return ZkgmLib.encodeAck(
                    Acknowledgement({
                        tag: ZkgmLib.ACK_SUCCESS,
                        innerAck: acknowledgement
                    })
                );
            }
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
        bytes calldata relayerMsg,
        bytes calldata rawZkgmPacket
    ) public returns (bytes memory) {
        // Only callable through the onRecvPacket endpoint.
        if (msg.sender != address(this)) {
            revert ZkgmLib.ErrUnauthorized();
        }
        ZkgmPacket calldata zkgmPacket = ZkgmLib.decode(rawZkgmPacket);
        return executeInternal(
            ibcPacket,
            relayer,
            relayerMsg,
            zkgmPacket.salt,
            ZkgmLib.decodeSyscall(zkgmPacket.syscall)
        );
    }

    function executeInternal(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
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
                ZkgmLib.decodeFungibleAssetTransfer(syscallPacket.packet)
            );
        } else if (syscallPacket.index == ZkgmLib.SYSCALL_BATCH) {
            return executeBatch(
                ibcPacket,
                relayer,
                relayerMsg,
                salt,
                ZkgmLib.decodeBatch(syscallPacket.packet)
            );
        } else if (syscallPacket.index == ZkgmLib.SYSCALL_FORWARD) {
            return executeForward(
                ibcPacket,
                relayer,
                relayerMsg,
                salt,
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
        BatchPacket calldata batchPacket
    ) internal returns (bytes memory) {
        uint256 l = batchPacket.syscallPackets.length;
        bytes[] memory acks = new bytes[](l);
        for (uint256 i = 0; i < l; i++) {
            SyscallPacket calldata syscallPacket =
                ZkgmLib.decodeSyscall(batchPacket.syscallPackets[i]);
            acks[i] = executeInternal(
                ibcPacket,
                relayer,
                relayerMsg,
                keccak256(abi.encode(salt)),
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
        ForwardPacket calldata forwardPacket
    ) internal returns (bytes memory) {
        IBCPacket memory sentPacket = ibcHandler.sendPacket(
            forwardPacket.channelId,
            forwardPacket.timeoutHeight,
            forwardPacket.timeoutTimestamp,
            ZkgmLib.encode(
                ZkgmPacket({
                    salt: keccak256(abi.encode(salt)),
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
                sourceChannel: ibcPacket.sourceChannel,
                destinationChannel: ibcPacket.destinationChannel,
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

    function predictWrappedToken(
        uint32 channel,
        bytes calldata token
    ) internal returns (address, bytes32) {
        bytes32 wrappedTokenSalt = keccak256(abi.encode(channel, token));
        address wrappedToken =
            CREATE3.predictDeterministicAddress(wrappedTokenSalt);
        return (wrappedToken, wrappedTokenSalt);
    }

    function executeFungibleAssetTransfer(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        FungibleAssetTransferPacket calldata assetTransferPacket
    ) internal returns (bytes memory) {
        if (assetTransferPacket.onlyMaker) {
            return ZkgmLib.ACK_ERR_ONLYMAKER;
        }
        if (assetTransferPacket.askAmount > assetTransferPacket.sentAmount) {
            revert ZkgmLib.ErrInvalidAmount();
        }
        (address wrappedToken, bytes32 wrappedTokenSalt) = predictWrappedToken(
            ibcPacket.destinationChannel, assetTransferPacket.sentToken
        );
        address askToken = address(bytes20(assetTransferPacket.askToken));
        address receiver = address(bytes20(assetTransferPacket.receiver));
        uint256 fee =
            assetTransferPacket.sentAmount - assetTransferPacket.askAmount;
        if (askToken == wrappedToken) {
            if (!ZkgmLib.isDeployed(wrappedToken)) {
                CREATE3.deployDeterministic(
                    abi.encodePacked(
                        type(ZkgmERC20).creationCode, "test", "test"
                    ),
                    wrappedTokenSalt
                );
            }
            IZkgmERC20(wrappedToken).mint(
                receiver, assetTransferPacket.askAmount
            );
            if (fee > 0) {
                IZkgmERC20(wrappedToken).mint(relayer, fee);
            }
        } else {
            channelBalance[ibcPacket.destinationChannel][askToken] -=
                assetTransferPacket.askAmount;
            IERC20(askToken).transfer(receiver, assetTransferPacket.askAmount);
            if (fee > 0) {
                IERC20(askToken).transfer(relayer, fee);
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
                ZkgmLib.decodeSyscall(zkgmPacket.syscall),
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
                ZkgmLib.decodeSyscall(batchPacket.syscallPackets[i]),
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
                sourceChannel: ibcPacket.sourceChannel,
                destinationChannel: ibcPacket.destinationChannel,
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
        AssetTransferAcknowledgement calldata assetTransferAck =
            ZkgmLib.decodeAssetTransferAck(ack);
        if (assetTransferAck.fillType == ZkgmLib.FILL_TYPE_PROTOCOL) {
            // The protocol filled, fee was paid to relayer.
        } else if (assetTransferAck.fillType == ZkgmLib.FILL_TYPE_MARKETMAKER) {
            IERC20(address(bytes20(assetTransferPacket.sentToken))).transfer(
                address(bytes20(assetTransferAck.marketMaker)),
                assetTransferPacket.sentAmount
            );
        } else {
            revert ZkgmLib.ErrInvalidFillType();
        }
    }

    function onTimeoutPacket(
        IBCPacket calldata ibcPacket,
        address relayer
    ) external virtual override onlyIBC {
        ZkgmPacket calldata zkgmPacket = ZkgmLib.decode(ibcPacket.data);
        acknowledgeInternal(
            ibcPacket,
            relayer,
            zkgmPacket.salt,
            ZkgmLib.decodeSyscall(zkgmPacket.syscall),
            false,
            ibcPacket.data
        );
    }

    function onChanOpenInit(
        uint32,
        uint32,
        string calldata,
        address
    ) external virtual override onlyIBC {}

    function onChanOpenTry(
        uint32,
        uint32,
        uint32,
        string calldata,
        string calldata,
        address
    ) external virtual override onlyIBC {}

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
}
