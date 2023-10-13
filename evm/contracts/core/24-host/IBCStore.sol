pragma solidity ^0.8.21;

import "../../proto/ibc/core/connection/v1/connection.sol";
import "../../proto/ibc/core/channel/v1/channel.sol";
import "../02-client/ILightClient.sol";

abstract contract IBCStore {
    // Commitments
    // keccak256(IBC-compatible-store-path) => keccak256(IBC-compatible-commitment)
    mapping(bytes32 => bytes32) public commitments;

    // Store
    mapping(string => address) public clientRegistry; // clientType => clientImpl
    mapping(string => string) public clientTypes; // clientID => clientType
    mapping(string => address) public clientImpls; // clientID => clientImpl
    mapping(string => IbcCoreConnectionV1ConnectionEnd.Data) public connections;
    mapping(string => mapping(string => IbcCoreChannelV1Channel.Data))
        public channels;
    mapping(string => mapping(string => uint64)) public nextSequenceSends;
    mapping(string => mapping(string => uint64)) public nextSequenceRecvs;
    mapping(string => mapping(string => uint64)) public nextSequenceAcks;
    mapping(string => mapping(string => mapping(uint64 => uint8)))
        public packetReceipts;
    mapping(bytes => address[]) public capabilities;

    // Host parameters
    uint64 public expectedTimePerBlock;

    // Sequences for identifier
    uint64 internal nextClientSequence;
    uint64 internal nextConnectionSequence;
    uint64 internal nextChannelSequence;

    // Storage accessors
    function getClient(
        string memory clientId
    ) internal view returns (ILightClient) {
        address clientImpl = clientImpls[clientId];
        require(clientImpl != address(0), "IBCStore: client not found");
        return ILightClient(clientImpl);
    }
}
