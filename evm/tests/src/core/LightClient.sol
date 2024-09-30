pragma solidity ^0.8.23;

import "../../../contracts/core/02-client/ILightClient.sol";

contract TestLightClient is ILightClient {
    uint64 height;
    bool revertCreate;
    bool revertUpdate;
    uint64 validMembership;
    uint64 validNonMembership;
    bytes clientState;

    constructor() {
        revertCreate = false;
        revertUpdate = false;
        height = 0;
    }

    function setHeight(
        uint64 height_
    ) public {
        height = height_;
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
            height: height
        });
    }

    function getTimestampAtHeight(
        uint32,
        uint64
    ) external pure returns (uint64) {
        return 0;
    }

    function getLatestHeight(
        uint32
    ) external view returns (uint64) {
        return height;
    }

    function updateClient(
        uint32,
        bytes calldata clientMessageBytes
    ) external returns (ConsensusStateUpdate memory) {
        if (revertUpdate) {
            revert();
        }
        height = height + 1;
        return ConsensusStateUpdate({
            clientStateCommitment: keccak256(clientState),
            consensusStateCommitment: keccak256(clientMessageBytes),
            height: height
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
