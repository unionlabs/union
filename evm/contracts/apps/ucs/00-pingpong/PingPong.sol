pragma solidity ^0.8.23;

import "../../Base.sol";
import "../../../core/25-handler/IBCHandler.sol";

// Protocol specific packet
struct PingPongPacket {
    bool ping;
    uint64 counterpartyTimeout;
}

library PingPongLib {
    bytes1 public constant ACK_SUCCESS = 0x01;

    error ErrOnlyOneChannel();
    error ErrInvalidAck();
    error ErrNoChannel();
    error ErrInfiniteGame();

    event Ring(bool ping);
    event TimedOut();
    event Acknowledged();

    function encode(PingPongPacket memory packet)
        internal
        pure
        returns (bytes memory)
    {
        return abi.encode(packet.ping, packet.counterpartyTimeout);
    }

    function decode(bytes memory packet)
        internal
        pure
        returns (PingPongPacket memory)
    {
        (bool ping, uint64 counterpartyTimeout) =
            abi.decode(packet, (bool, uint64));
        return PingPongPacket({
            ping: ping,
            counterpartyTimeout: counterpartyTimeout
        });
    }
}

contract PingPong is IBCAppBase {
    using PingPongLib for *;

    IBCHandler private ibcHandler;
    string private channelId;
    uint64 private revisionNumber;
    uint64 private timeout;

    constructor(
        IBCHandler _ibcHandler,
        uint64 _revisionNumber,
        uint64 _timeout
    ) {
        ibcHandler = _ibcHandler;
        revisionNumber = _revisionNumber;
        timeout = _timeout;
    }

    function ibcAddress() public view virtual override returns (address) {
        return address(ibcHandler);
    }

    function initiate(
        PingPongPacket memory packet,
        uint64 localTimeout
    ) public {
        if (bytes(channelId).length == 0) {
            revert PingPongLib.ErrNoChannel();
        }
        ibcHandler.sendPacket(
            channelId,
            // No height timeout
            IbcCoreClientV1Height.Data({revision_number: 0, revision_height: 0}),
            // Timestamp timeout
            localTimeout,
            // Raw protocol packet
            packet.encode()
        );
    }

    function onRecvPacket(
        IbcCoreChannelV1Packet.Data calldata packet,
        address
    )
        external
        virtual
        override
        onlyIBC
        returns (bytes memory acknowledgement)
    {
        PingPongPacket memory pp = PingPongLib.decode(packet.data);

        emit PingPongLib.Ring(pp.ping);

        uint64 localTimeout = pp.counterpartyTimeout;

        pp.ping = !pp.ping;
        pp.counterpartyTimeout = uint64(block.timestamp) + timeout;

        // Send back the packet after having reversed the bool and set the counterparty timeout
        initiate(pp, localTimeout);

        // Return protocol specific successful acknowledgement
        return abi.encodePacked(PingPongLib.ACK_SUCCESS);
    }

    function onAcknowledgementPacket(
        IbcCoreChannelV1Packet.Data calldata,
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
        IbcCoreChannelV1Packet.Data calldata,
        address
    ) external virtual override onlyIBC {
        /*
            Similarly to the onAcknowledgementPacket function, this indicates a failure to deliver the packet in expected time.
            A sophisticated protocol would revert the action done before sending this packet.
        */
        emit PingPongLib.TimedOut();
    }

    function onChanOpenInit(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata,
        string calldata,
        string calldata,
        IbcCoreChannelV1Counterparty.Data calldata,
        string calldata,
        address
    ) external virtual override onlyIBC {
        // This protocol is only accepting a single counterparty.
        if (bytes(channelId).length != 0) {
            revert PingPongLib.ErrOnlyOneChannel();
        }
    }

    function onChanOpenTry(
        IbcCoreChannelV1GlobalEnums.Order,
        string[] calldata,
        string calldata,
        string calldata,
        IbcCoreChannelV1Counterparty.Data calldata,
        string calldata,
        string calldata,
        address
    ) external virtual override onlyIBC {
        // Symmetric to onChanOpenInit
        if (bytes(channelId).length != 0) {
            revert PingPongLib.ErrOnlyOneChannel();
        }
    }

    function onChanOpenAck(
        string calldata,
        string calldata _channelId,
        string calldata,
        string calldata,
        address
    ) external virtual override onlyIBC {
        // Store the port/channel needed to send packets.
        channelId = _channelId;
    }

    function onChanOpenConfirm(
        string calldata,
        string calldata _channelId,
        address
    ) external virtual override onlyIBC {
        // Symmetric to onChanOpenAck
        channelId = _channelId;
    }

    function onChanCloseInit(
        string calldata,
        string calldata,
        address
    ) external virtual override onlyIBC {
        // The ping-pong is infinite, closing the channel is disallowed.
        revert PingPongLib.ErrInfiniteGame();
    }

    function onChanCloseConfirm(
        string calldata,
        string calldata,
        address
    ) external virtual override onlyIBC {
        // Symmetric to onChanCloseInit
        revert PingPongLib.ErrInfiniteGame();
    }
}
