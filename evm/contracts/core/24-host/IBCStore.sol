// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import "../../proto/Connection.sol";
import "../../proto/Channel.sol";
import "../02-client/ILightClient.sol";

abstract contract IBCStore {
    // Commitments
    // keccak256(IBC-compatible-store-path) => keccak256(IBC-compatible-commitment)
    mapping(bytes32 => bytes32) internal commitments;

    // Store
    mapping(string => address) internal clientRegistry; // clientType => clientImpl
    mapping(string => string) internal clientTypes; // clientID => clientType
    mapping(string => address) internal clientImpls; // clientID => clientImpl
    mapping(string => ConnectionEnd.Data) internal connections;
    mapping(string => mapping(string => Channel.Data)) internal channels;
    mapping(string => mapping(string => uint64)) internal nextSequenceSends;
    mapping(string => mapping(string => uint64)) internal nextSequenceRecvs;
    mapping(string => mapping(string => uint64)) internal nextSequenceAcks;
    mapping(string => mapping(string => mapping(uint64 => uint8))) internal packetReceipts;
    mapping(bytes => address[]) internal capabilities;

    // Host parameters
    uint64 internal expectedTimePerBlock;

    // Sequences for identifier
    uint64 internal nextClientSequence;
    uint64 internal nextConnectionSequence;
    uint64 internal nextChannelSequence;

    // Storage accessors

    function getClient(string memory clientId) internal view returns (ILightClient) {
        address clientImpl = clientImpls[clientId];
        require(clientImpl != address(0));
        return ILightClient(clientImpl);
    }
}
