pragma solidity ^0.8.23;

import "../../proto/ibc/core/connection/v1/connection.sol";
import "../../proto/ibc/core/channel/v1/channel.sol";
import "../02-client/ILightClient.sol";
import {IBCChannelTypes, ChannelId} from "../04-channel/IBCChannelTypes.sol";

library IBCStoreLib {
    error ErrClientNotFound();
}

abstract contract IBCStore {
    // Commitments
    // keccak256(IBC-compatible-store-path) => keccak256(IBC-compatible-commitment)
    mapping(bytes32 => bytes32) public commitments;

    // Store
    mapping(string => address) public clientRegistry;
    mapping(string => string) public clientTypes;
    mapping(string => address) public clientImpls;
    mapping(string => IbcCoreConnectionV1ConnectionEnd.Data) public connections;
    // portId -> channelId -> channel
    mapping(string => mapping(ChannelId => IBCChannelTypes.Channel)) public
        channels;
    // portId -> channelId -> sequence
    mapping(string => mapping(ChannelId => uint64)) public nextSequenceSends;
    // portId -> channelId -> sequence
    mapping(string => mapping(ChannelId => uint64)) public nextSequenceRecvs;
    // portId -> channelId -> sequence
    mapping(string => mapping(ChannelId => uint64)) public nextSequenceAcks;
    // portId -> channelId -> sequence -> ?
    mapping(string => mapping(ChannelId => mapping(uint64 => uint8))) public
        packetReceipts;
    mapping(ChannelId => address) public capabilities;

    // Sequences for identifier
    uint64 public nextClientSequence;
    uint64 public nextConnectionSequence;
    uint64 public nextChannelSequence;

    string public constant COMMITMENT_PREFIX = "ibc";

    // Storage accessors
    function getClient(string memory clientId)
        public
        view
        returns (ILightClient)
    {
        address clientImpl = clientImpls[clientId];
        if (clientImpl == address(0)) {
            revert IBCStoreLib.ErrClientNotFound();
        }
        return ILightClient(clientImpl);
    }
}
