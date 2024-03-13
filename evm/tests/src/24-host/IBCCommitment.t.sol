pragma solidity ^0.8.23;

import {IBCCommitment} from "../../../contracts/core/24-host/IBCCommitment.sol";

import "../TestPlus.sol";

contract IBCCommitmentTest is TestPlus {
    function test_clientStatePath() public {
        assertStrEq(
            IBCCommitment.clientStatePath("client-id"),
            "clients/client-id/clientState"
        );
    }

    function test_consensusStatePath() public {
        assertStrEq(
            IBCCommitment.consensusStatePath("client-id", 1, 2),
            "clients/client-id/consensusStates/1-2"
        );
    }

    function test_connectionPath() public {
        assertStrEq(
            IBCCommitment.connectionPath("conn-id"), "connections/conn-id"
        );
    }

    function test_channelPath() public {
        assertStrEq(
            IBCCommitment.channelPath("port-id", "channel-id"),
            "channelEnds/ports/port-id/channels/channel-id"
        );
    }

    function test_packetCommitmentPath() public {
        assertStrEq(
            IBCCommitment.packetCommitmentPath("port-id", "channel-id", 1337),
            "commitments/ports/port-id/channels/channel-id/sequences/1337"
        );
    }

    function test_packetAcknowledgmentCommitmentPath() public {
        assertStrEq(
            IBCCommitment.packetAcknowledgementCommitmentPath(
                "port-id", "channel-id", 1337
            ),
            "acks/ports/port-id/channels/channel-id/sequences/1337"
        );
    }

    function test_packetReceiptCommitmentPath() public {
        assertStrEq(
            IBCCommitment.packetReceiptCommitmentPath(
                "port-id", "channel-id", 1337
            ),
            "receipts/ports/port-id/channels/channel-id/sequences/1337"
        );
    }

    function test_nextSequenceRecvCommitmentPath() public {
        assertStrEq(
            IBCCommitment.nextSequenceRecvCommitmentPath(
                "port-id", "channel-id"
            ),
            "nextSequenceRecv/ports/port-id/channels/channel-id"
        );
    }

    function test_clientStateCommitmentKey() public {
        assertEq(
            IBCCommitment.clientStateCommitmentKey("client-id"),
            keccak256("clients/client-id/clientState")
        );
    }

    function test_consensusStateCommitmentKey() public {
        assertEq(
            IBCCommitment.consensusStateCommitmentKey("client-id", 1, 2),
            keccak256("clients/client-id/consensusStates/1-2")
        );
    }

    function test_connectionCommitmentKey() public {
        assertEq(
            IBCCommitment.connectionCommitmentKey("conn-id"),
            keccak256("connections/conn-id")
        );
    }

    function test_channelCommitmentKey() public {
        assertEq(
            IBCCommitment.channelCommitmentKey("port-id", "channel-id"),
            keccak256("channelEnds/ports/port-id/channels/channel-id")
        );
    }

    function test_packetCommitmentKey() public {
        assertEq(
            IBCCommitment.packetCommitmentKey("port-id", "channel-id", 1337),
            keccak256(
                "commitments/ports/port-id/channels/channel-id/sequences/1337"
            )
        );
    }

    function test_packetAcknowledgmentCommitmentKey() public {
        assertEq(
            IBCCommitment.packetAcknowledgementCommitmentKey(
                "port-id", "channel-id", 1337
            ),
            keccak256("acks/ports/port-id/channels/channel-id/sequences/1337")
        );
    }

    function test_nextSequenceRecvCommitmentKey() public {
        assertEq(
            IBCCommitment.nextSequenceRecvCommitmentKey("port-id", "channel-id"),
            keccak256("nextSequenceRecv/ports/port-id/channels/channel-id")
        );
    }
}
