pragma solidity ^0.8.27;

import "@openzeppelin-upgradeable/proxy/utils/Initializable.sol";
import "@openzeppelin-upgradeable/proxy/utils/UUPSUpgradeable.sol";
import "@openzeppelin-upgradeable/access/OwnableUpgradeable.sol";
import "@openzeppelin-upgradeable/utils/PausableUpgradeable.sol";

import "../../Base.sol";
import "../../../core/25-handler/IBCHandler.sol";

// Protocol specific packet
struct PingPongPacket {
    bool ping;
}
// uint64 counterpartyTimeout;

library PingPongLib {
    bytes1 public constant ACK_SUCCESS = 0x01;

    error ErrOnlyOneChannel();
    error ErrInvalidAck();
    error ErrNoChannel();
    error ErrInfiniteGame();

    event Ring(bool ping);
    event TimedOut();
    event Acknowledged();

    function encode(
        PingPongPacket memory packet
    ) internal pure returns (bytes memory) {
        return abi.encode(packet.ping)
        // packet.counterpartyTimeout
        ;
    }

    function decode(
        bytes memory packet
    ) internal pure returns (PingPongPacket memory) {
        bool ping = abi.decode(packet, (bool));
        return PingPongPacket({ping: ping})
        // counterpartyTimeout: counterpartyTimeout
        ;
    }
}

contract PingPong is
    IBCAppBase,
    Initializable,
    UUPSUpgradeable,
    OwnableUpgradeable,
    PausableUpgradeable
{
    using PingPongLib for *;

    IIBCPacket private ibcHandler;
    uint32 private srcChannelId;
    uint64 private timeout;

    constructor() {
        _disableInitializers();
    }

    function initialize(
        IIBCPacket _ibcHandler,
        address admin,
        uint64 _timeout
    ) public initializer {
        __Ownable_init(admin);
        ibcHandler = _ibcHandler;
        timeout = _timeout;
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }

    function initiate(
        PingPongPacket memory packet,
        uint64 localTimeout
    ) public {
        if (srcChannelId == 0) {
            revert PingPongLib.ErrNoChannel();
        }
        ibcHandler.sendPacket(
            srcChannelId,
            // No height timeout
            0,
            // Timestamp timeout
            localTimeout,
            // Raw protocol packet
            packet.encode()
        );
    }

    function onRecvPacket(
        IBCPacket calldata packet,
        address,
        bytes calldata
    )
        external
        virtual
        override
        onlyIBC
        returns (bytes memory acknowledgement)
    {
        PingPongPacket memory pp = PingPongLib.decode(packet.data);

        emit PingPongLib.Ring(pp.ping);

        // uint64 localTimeout = pp.counterpartyTimeout;

        pp.ping = !pp.ping;
        uint64 localTimeout = uint64(block.timestamp * 1e9) + timeout;

        // Send back the packet after having reversed the bool and set the counterparty timeout
        initiate(pp, localTimeout);

        // Return protocol specific successful acknowledgement
        return abi.encodePacked(PingPongLib.ACK_SUCCESS);
    }

    function onAcknowledgementPacket(
        IBCPacket calldata,
        bytes calldata acknowledgement,
        address
    ) external virtual override onlyIBC {
        /*
            In practice, a more sophisticated protocol would check
            and execute code depending on the counterparty outcome (refund etc...).
            In our case, the acknowledgement will always be ACK_SUCCESS
        */
        if (
            keccak256(acknowledgement)
                != keccak256(abi.encodePacked(PingPongLib.ACK_SUCCESS))
        ) {
            revert PingPongLib.ErrInvalidAck();
        }
        emit PingPongLib.Acknowledged();
    }

    function onTimeoutPacket(
        IBCPacket calldata,
        address
    ) external virtual override onlyIBC {
        /*
            Similarly to the onAcknowledgementPacket function, this indicates a failure to deliver the packet in expected time.
            A sophisticated protocol would revert the action done before sending this packet.
        */
        emit PingPongLib.TimedOut();
    }

    function onChanOpenInit(
        uint32,
        uint32,
        string calldata,
        address
    ) external virtual override onlyIBC {
        // This protocol is only accepting a single counterparty.
        if (srcChannelId != 0) {
            revert PingPongLib.ErrOnlyOneChannel();
        }
    }

    function onChanOpenTry(
        uint32,
        uint32,
        uint32,
        string calldata,
        string calldata,
        address
    ) external virtual override onlyIBC {
        // Symmetric to onChanOpenInit
        if (srcChannelId != 0) {
            revert PingPongLib.ErrOnlyOneChannel();
        }
    }

    function onChanOpenAck(
        uint32 channelId,
        uint32,
        string calldata,
        address
    ) external virtual override onlyIBC {
        // Store the port/channel needed to send packets.
        srcChannelId = channelId;
    }

    function onChanOpenConfirm(
        uint32 channelId,
        address
    ) external virtual override onlyIBC {
        // Symmetric to onChanOpenAck
        srcChannelId = channelId;
    }

    function onChanCloseInit(
        uint32,
        address
    ) external virtual override onlyIBC {
        // The ping-pong is infinite, closing the channel is disallowed.
        revert PingPongLib.ErrInfiniteGame();
    }

    function onChanCloseConfirm(
        uint32,
        address
    ) external virtual override onlyIBC {
        // Symmetric to onChanCloseInit
        revert PingPongLib.ErrInfiniteGame();
    }

    function _authorizeUpgrade(
        address newImplementation
    ) internal override onlyOwner {}
}
