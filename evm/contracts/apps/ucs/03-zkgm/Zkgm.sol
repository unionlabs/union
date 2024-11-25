pragma solidity ^0.8.27;

import "../../Base.sol";
import "../../../core/25-handler/IBCHandler.sol";
import "../../../core/05-port/IIBCModule.sol";

import "./IEurekaModule.sol";

struct ZkgmPacket {
    uint8 version;
    bytes32 salt;
    uint8 syscallIndex;
    bytes packet;
}

struct ForwardPacket {
    uint32 channelId;
    bytes zkgmPacket;
}

struct MultiplexPacket {
    bytes sender;
    bool eureka;
    bytes contractAddress;
    bytes contractCalldata;
}

struct BatchPacket {
    bytes[] zkgmPackets;
}

struct FungibleAssetTransferPacket {
    bytes sender;
    bytes receiver;
    bytes sentToken;
    uint256 sentAmount;
    bytes askToken;
    bytes askAmount;
}

library ZkgmLib {
    bytes public constant ACK_EMPTY = hex"";
    bytes public constant ACK_FAILURE = abi.encode(0x00);
    bytes public constant ACK_SUCCESS = abi.encode(0x01);

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
}

contract Zkgm is IBCAppBase {
    using ZkgmLib for *;

    IBCHandler private ibcHandler;

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
            } else {
                return abi.encode(ZkgmLib.ACK_SUCCESS, acknowledgement);
            }
        } else {
            return ZkgmLib.ACK_FAILURE;
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
            zkgmPacket.version,
            zkgmPacket.salt,
            zkgmPacket.syscallIndex,
            zkgmPacket.packet
        );
    }

    function executeInternal(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        uint8 version,
        bytes32 salt,
        uint8 syscallIndex,
        bytes calldata packet
    ) public returns (bytes memory) {
        if (version != ZkgmLib.ZKGM_VERSION_0) {
            revert ZkgmLib.ErrUnsupportedVersion();
        }
        if (syscallIndex == ZkgmLib.SYSCALL_FUNGIBLE_ASSET_TRANSFER) {
            return executeFungibleAssetTransfer(
                ibcPacket,
                relayer,
                relayerMsg,
                salt,
                ZkgmLib.decodeFungibleAssetTransfer(packet)
            );
        } else if (syscallIndex == ZkgmLib.SYSCALL_BATCH) {
            return executeBatch(
                ibcPacket,
                relayer,
                relayerMsg,
                salt,
                ZkgmLib.decodeBatch(packet)
            );
        } else if (syscallIndex == ZkgmLib.SYSCALL_FORWARD) {
            return executeForward(
                ibcPacket,
                relayer,
                relayerMsg,
                salt,
                ZkgmLib.decodeForward(packet)
            );
        } else if (syscallIndex == ZkgmLib.SYSCALL_MULTIPLEX) {
            return executeMultiplex(
                ibcPacket,
                relayer,
                relayerMsg,
                salt,
                ZkgmLib.decodeMultiplex(packet)
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
        uint256 l = batchPacket.zkgmPackets.length;
        bytes[] memory acknowledgements = new bytes[](l);
        for (uint256 i = 0; i < l; i++) {
            ZkgmPacket calldata zkgmPacket =
                ZkgmLib.decode(batchPacket.zkgmPackets[i]);
            acknowledgements[i] = executeInternal(
                ibcPacket,
                relayer,
                relayerMsg,
                zkgmPacket.version,
                keccak256(abi.encode(salt, zkgmPacket.salt)),
                zkgmPacket.syscallIndex,
                zkgmPacket.packet
            );
            if (acknowledgements[i].length == 0) {
                revert ZkgmLib.ErrBatchMustBeSync();
            }
        }
        return abi.encode(acknowledgements);
    }

    function executeForward(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        ForwardPacket calldata forwardPacket
    ) internal returns (bytes memory) {
        revert ZkgmLib.ErrUnimplemented();
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
            return ZkgmLib.ACK_SUCCESS;
        } else {
            IBCPacket memory multiplexIbcPacket = IBCPacket({
                sourceChannel: ibcPacket.sourceChannel,
                destinationChannel: ibcPacket.destinationChannel,
                data: multiplexPacket.contractCalldata,
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

    function executeFungibleAssetTransfer(
        IBCPacket calldata ibcPacket,
        address relayer,
        bytes calldata relayerMsg,
        bytes32 salt,
        FungibleAssetTransferPacket calldata assetTransferPacket
    ) internal returns (bytes memory) {
        revert ZkgmLib.ErrUnimplemented();
    }

    function onAcknowledgementPacket(
        IBCPacket calldata,
        bytes calldata acknowledgement,
        address
    ) external virtual override onlyIBC {}

    function onTimeoutPacket(
        IBCPacket calldata,
        address
    ) external virtual override onlyIBC {}

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
