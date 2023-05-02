// SPDX-License-Identifier: TBD

pragma solidity ^0.8.18;

import "../proto/ibc/core/commitment/v1/commitment.sol";
import "../proto/ibc/lightclients/tendermint/v1/tendermint.sol";
import "../proto/tendermint/types/types.sol";
import "../proto/tendermint/types/validator.sol";
import "./Encoder.sol";
import "./MerkleTree.sol";
import "@openzeppelin/contracts/utils/math/SafeCast.sol";

library TendermintHelper {
    /* function toCanonicalBlockID(BlockID.Data memory blockID) internal pure returns (CanonicalBlockID.Data memory) { */
    /*     return */
    /*         CanonicalBlockID.Data({ */
    /*             hash: blockID.hash, */
    /*             part_set_header: CanonicalPartSetHeader.Data({ */
    /*                 total: blockID.part_set_header.total, */
    /*                 hash: blockID.part_set_header.hash */
    /*             }) */
    /*         }); */
    /* } */

    /* function toCanonicalVote(Vote.Data memory vote, string memory chainID) */
    /*     internal */
    /*     pure */
    /*     returns (CanonicalVote.Data memory) */
    /* { */
    /*     return */
    /*         CanonicalVote.Data({ */
    /*             Type: vote.Type, */
    /*             height: vote.height, */
    /*             round: int64(vote.round), */
    /*             block_id: toCanonicalBlockID(vote.block_id), */
    /*             timestamp: vote.timestamp, */
    /*             chain_id: chainID */
    /*         }); */
    /* } */

    function toConsensusState(IbcLightclientsTendermintV1Header.Data memory header) internal pure returns (IbcLightclientsTendermintV1ConsensusState.Data memory) {
        return
            IbcLightclientsTendermintV1ConsensusState.Data({
                timestamp: header.signed_header.header.time,
                root: IbcCoreCommitmentV1MerkleRoot.Data({hash: header.signed_header.header.app_hash}),
                next_validators_hash: header.signed_header.header.next_validators_hash
            });
    }

    /* function toVote(Commit.Data memory commit, uint256 valIdx) internal pure returns (Vote.Data memory) { */
    /*     CommitSig.Data memory commitSig = commit.signatures[valIdx]; */

    /*     return */
    /*         Vote.Data({ */
    /*             Type: TENDERMINTLIGHT_PROTO_GLOBAL_ENUMS.SignedMsgType.SIGNED_MSG_TYPE_PRECOMMIT, */
    /*             height: commit.height, */
    /*             round: commit.round, */
    /*             block_id: commit.block_id, */
    /*             timestamp: commitSig.timestamp, */
    /*             validator_address: commitSig.validator_address, */
    /*             validator_index: SafeCast.toInt32(int256(valIdx)), */
    /*             signature: commitSig.signature */
    /*         }); */
    /* } */

    /* function isEqual(BlockID.Data memory b1, BlockID.Data memory b2) internal pure returns (bool) { */
    /*     if (keccak256(abi.encodePacked(b1.hash)) != keccak256(abi.encodePacked(b2.hash))) { */
    /*         return false; */
    /*     } */

    /*     if (b1.part_set_header.total != b2.part_set_header.total) { */
    /*         return false; */
    /*     } */

    /*     if ( */
    /*         keccak256(abi.encodePacked(b1.part_set_header.hash)) != keccak256(abi.encodePacked(b2.part_set_header.hash)) */
    /*     ) { */
    /*         return false; */
    /*     } */

    /*     return true; */
    /* } */

    /* function isEqual(ConsensusState.Data memory cs1, ConsensusState.Data memory cs2) internal pure returns (bool) { */
    /*     return */
    /*         keccak256(abi.encodePacked(ConsensusState.encode(cs1))) == */
    /*         keccak256(abi.encodePacked(ConsensusState.encode(cs2))); */
    /* } */

    function isExpired(
        GoogleProtobufTimestamp.Data memory headerTime,
        GoogleProtobufDuration.Data memory trustingPeriod,
        GoogleProtobufDuration.Data memory currentTime
    ) internal pure returns (bool) {
        GoogleProtobufTimestamp.Data memory expirationTime = GoogleProtobufTimestamp.Data({
            secs: headerTime.secs + int64(trustingPeriod.Seconds),
            nanos: headerTime.nanos
        });
        return gt(GoogleProtobufTimestamp.Data({secs: int64(currentTime.Seconds), nanos: 0}), expirationTime);
    }

    function gt(GoogleProtobufTimestamp.Data memory t1, GoogleProtobufTimestamp.Data memory t2) internal pure returns (bool) {
        if (t1.secs > t2.secs) {
            return true;
        } else if (t1.secs == t2.secs && t1.nanos > t2.nanos) {
            return true;
        } else {
            return false;
        }
    }

    /* function hash(SignedHeader.Data memory h) internal pure returns (bytes32) { */
    /*     require(h.header.validators_hash.length > 0, "Tendermint: hash can't be empty"); */

    /*     bytes memory hbz = Consensus.encode(h.header.version); */
    /*     bytes memory pbt = Timestamp.encode(h.header.time); */
    /*     bytes memory bzbi = BlockID.encode(h.header.last_block_id); */

    /*     bytes[14] memory all = [ */
    /*         hbz, */
    /*         Encoder.cdcEncode(h.header.chain_id), */
    /*         Encoder.cdcEncode(h.header.height), */
    /*         pbt, */
    /*         bzbi, */
    /*         Encoder.cdcEncode(h.header.last_commit_hash), */
    /*         Encoder.cdcEncode(h.header.data_hash), */
    /*         Encoder.cdcEncode(h.header.validators_hash), */
    /*         Encoder.cdcEncode(h.header.next_validators_hash), */
    /*         Encoder.cdcEncode(h.header.consensus_hash), */
    /*         Encoder.cdcEncode(h.header.app_hash), */
    /*         Encoder.cdcEncode(h.header.last_results_hash), */
    /*         Encoder.cdcEncode(h.header.evidence_hash), */
    /*         Encoder.cdcEncode(h.header.proposer_address) */
    /*     ]; */

    /*     return MerkleTree.merkleRootHash(all, 0, all.length); */
    /* } */

    /* function hash(ValidatorSet.Data memory vs) internal pure returns (bytes32) { */
    /*     return MerkleTree.merkleRootHash(vs.validators, 0, vs.validators.length); */
    /* } */

    /* function getByAddress(ValidatorSet.Data memory vals, bytes memory addr) */
    /*     internal */
    /*     pure */
    /*     returns (uint256 index, bool found) */
    /* { */
    /*     bytes32 addrHash = keccak256(abi.encodePacked(addr)); */
    /*     for (uint256 idx; idx < vals.validators.length; idx++) { */
    /*         if (keccak256(abi.encodePacked(vals.validators[idx].Address)) == addrHash) { */
    /*             return (idx, true); */
    /*         } */
    /*     } */

    /*     return (0, false); */
    /* } */

    /* function getTotalVotingPower(ValidatorSet.Data memory vals) internal pure returns (int64) { */
    /*     if (vals.total_voting_power == 0) { */
    /*         uint256 sum = 0; */
    /*         uint256 maxInt64 = 1 << (63 - 1); */
    /*         uint256 maxTotalVotingPower = maxInt64 / 8; */

    /*         for (uint256 i = 0; i < vals.validators.length; i++) { */
    /*             sum += (SafeCast.toUint256(int256(vals.validators[i].voting_power))); */
    /*             require(sum <= maxTotalVotingPower, "total voting power should be guarded to not exceed"); */
    /*         } */

    /*         vals.total_voting_power = SafeCast.toInt64(int256(sum)); */
    /*     } */

    /*     return vals.total_voting_power; */
    /* } */
}
