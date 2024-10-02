pragma solidity ^0.8.27;

import "../../../contracts/core/02-client/ILightClient.sol";

contract TestLightClient is ILightClient {
    bool revertCreate;
    bool revertUpdate;
    uint64 validMembership;
    uint64 validNonMembership;
    bytes clientState;

    uint64 latestHeight;

    mapping(uint64 => uint64) timestamps;

    constructor() {
        revertCreate = false;
        revertUpdate = false;
        latestHeight = 0;
    }

    function setRevertCreate(
        bool v
    ) public {
        revertCreate = v;
    }

    function setRevertUpdate(
        bool v
    ) public {
        revertUpdate = v;
    }

    function pushValidMembership() public {
        validMembership += 1;
    }

    function pushValidNonMembership() public {
        validNonMembership += 1;
    }

    function setLatestTimestamp(
        uint64 timestamp
    ) public {
        timestamps[latestHeight] = timestamp;
    }

    function setLatestHeight(
        uint64 height
    ) public {
        latestHeight = height;
        timestamps[height] = 1;
    }

    function createClient(
        uint32,
        bytes calldata clientStateBytes,
        bytes calldata consensusStateBytes
    ) external returns (ConsensusStateUpdate memory) {
        if (revertCreate) {
            revert();
        }
        clientState = clientStateBytes;
        return ConsensusStateUpdate({
            clientStateCommitment: keccak256(clientStateBytes),
            consensusStateCommitment: keccak256(consensusStateBytes),
            height: latestHeight
        });
    }

    function getTimestampAtHeight(
        uint32,
        uint64 height
    ) external view returns (uint64) {
        return timestamps[height];
    }

    function getLatestHeight(
        uint32
    ) external view returns (uint64) {
        return latestHeight;
    }

    function updateClient(
        uint32,
        bytes calldata clientMessageBytes
    ) external returns (ConsensusStateUpdate memory) {
        if (revertUpdate) {
            revert();
        }
        latestHeight += 1;
        return ConsensusStateUpdate({
            clientStateCommitment: keccak256(clientState),
            consensusStateCommitment: keccak256(clientMessageBytes),
            height: latestHeight
        });
    }

    function verifyMembership(
        uint32,
        uint64,
        bytes calldata,
        bytes calldata,
        bytes calldata,
        bytes calldata
    ) external returns (bool) {
        bool valid = validMembership > 0;
        if (validMembership > 0) {
            validMembership -= 1;
        }
        return valid;
    }

    function verifyNonMembership(
        uint32,
        uint64,
        bytes calldata,
        bytes calldata,
        bytes calldata
    ) external returns (bool) {
        bool valid = validNonMembership > 0;
        if (validNonMembership > 0) {
            validNonMembership -= 1;
        }
        return valid;
    }

    function getClientState(
        uint32
    ) external pure returns (bytes memory) {
        return hex"";
    }

    function getConsensusState(
        uint32,
        uint64
    ) external pure returns (bytes memory) {
        return hex"";
    }

    function isFrozen(
        uint32
    ) external pure returns (bool) {
        return false;
    }
}
