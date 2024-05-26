// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.23;

import "../../../../ProtoBufRuntime.sol";
import "../../../../GoogleProtobufAny.sol";
import "../../../../tendermint/types/validator.sol";
import "../../../../tendermint/types/types.sol";
import "../../../../cosmos/ics23/v1/proofs.sol";
import "../../../core/client/v1/client.sol";
import "../../../core/commitment/v1/commitment.sol";

library IbcLightclientsTendermintV1ClientState {
    //struct definition
    struct Data {
        string chain_id;
        IbcLightclientsTendermintV1Fraction.Data trust_level;
        GoogleProtobufDuration.Data trusting_period;
        GoogleProtobufDuration.Data unbonding_period;
        GoogleProtobufDuration.Data max_clock_drift;
        IbcCoreClientV1Height.Data frozen_height;
        IbcCoreClientV1Height.Data latest_height;
        CosmosIcs23V1ProofSpec.Data[] proof_specs;
        string[] upgrade_path;
        bool allow_update_after_expiry;
        bool allow_update_after_misbehaviour;
    }

    // Decoder section

    /**
     * @dev The main decoder for memory
     * @param bs The bytes array to be decoded
     * @return The decoded struct
     */
    function decode(bytes memory bs) internal pure returns (Data memory) {
        (Data memory x,) = _decode(32, bs, bs.length);
        return x;
    }

    /**
     * @dev The main decoder for storage
     * @param self The in-storage struct
     * @param bs The bytes array to be decoded
     */
    function decode(Data storage self, bytes memory bs) internal {
        (Data memory x,) = _decode(32, bs, bs.length);
        store(x, self);
    }

    // inner decoder

    /**
     * @dev The decoder for internal usage
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param sz The number of bytes expected
     * @return The decoded struct
     * @return The number of bytes decoded
     */
    function _decode(
        uint256 p,
        bytes memory bs,
        uint256 sz
    ) internal pure returns (Data memory, uint256) {
        Data memory r;
        uint256[12] memory counters;
        uint256 fieldId;
        ProtoBufRuntime.WireType wireType;
        uint256 bytesRead;
        uint256 offset = p;
        uint256 pointer = p;
        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer += _read_chain_id(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_trust_level(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_trusting_period(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_unbonding_period(pointer, bs, r);
            } else if (fieldId == 5) {
                pointer += _read_max_clock_drift(pointer, bs, r);
            } else if (fieldId == 6) {
                pointer += _read_frozen_height(pointer, bs, r);
            } else if (fieldId == 7) {
                pointer += _read_latest_height(pointer, bs, r);
            } else if (fieldId == 8) {
                pointer += _read_unpacked_repeated_proof_specs(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 9) {
                pointer += _read_unpacked_repeated_upgrade_path(
                    pointer, bs, nil(), counters
                );
            } else if (fieldId == 10) {
                pointer += _read_allow_update_after_expiry(pointer, bs, r);
            } else if (fieldId == 11) {
                pointer += _read_allow_update_after_misbehaviour(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        pointer = offset;
        if (counters[8] > 0) {
            require(r.proof_specs.length == 0);
            r.proof_specs = new CosmosIcs23V1ProofSpec.Data[](counters[8]);
        }
        if (counters[9] > 0) {
            require(r.upgrade_path.length == 0);
            r.upgrade_path = new string[](counters[9]);
        }

        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 8) {
                pointer += _read_unpacked_repeated_proof_specs(
                    pointer, bs, r, counters
                );
            } else if (fieldId == 9) {
                pointer += _read_unpacked_repeated_upgrade_path(
                    pointer, bs, r, counters
                );
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        return (r, sz);
    }

    // field readers

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_chain_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.chain_id = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_trust_level(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcLightclientsTendermintV1Fraction.Data memory x, uint256 sz) =
            _decode_IbcLightclientsTendermintV1Fraction(p, bs);
        r.trust_level = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_trusting_period(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (GoogleProtobufDuration.Data memory x, uint256 sz) =
            _decode_GoogleProtobufDuration(p, bs);
        r.trusting_period = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_unbonding_period(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (GoogleProtobufDuration.Data memory x, uint256 sz) =
            _decode_GoogleProtobufDuration(p, bs);
        r.unbonding_period = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_max_clock_drift(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (GoogleProtobufDuration.Data memory x, uint256 sz) =
            _decode_GoogleProtobufDuration(p, bs);
        r.max_clock_drift = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_frozen_height(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcCoreClientV1Height.Data memory x, uint256 sz) =
            _decode_IbcCoreClientV1Height(p, bs);
        r.frozen_height = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_latest_height(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcCoreClientV1Height.Data memory x, uint256 sz) =
            _decode_IbcCoreClientV1Height(p, bs);
        r.latest_height = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @param counters The counters for repeated fields
     * @return The number of bytes decoded
     */
    function _read_unpacked_repeated_proof_specs(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[12] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (CosmosIcs23V1ProofSpec.Data memory x, uint256 sz) =
            _decode_CosmosIcs23V1ProofSpec(p, bs);
        if (isNil(r)) {
            counters[8] += 1;
        } else {
            r.proof_specs[r.proof_specs.length - counters[8]] = x;
            counters[8] -= 1;
        }
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @param counters The counters for repeated fields
     * @return The number of bytes decoded
     */
    function _read_unpacked_repeated_upgrade_path(
        uint256 p,
        bytes memory bs,
        Data memory r,
        uint256[12] memory counters
    ) internal pure returns (uint256) {
        /**
         * if `r` is NULL, then only counting the number of fields.
         */
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        if (isNil(r)) {
            counters[9] += 1;
        } else {
            r.upgrade_path[r.upgrade_path.length - counters[9]] = x;
            counters[9] -= 1;
        }
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_allow_update_after_expiry(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bool x, uint256 sz) = ProtoBufRuntime._decode_bool(p, bs);
        r.allow_update_after_expiry = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_allow_update_after_misbehaviour(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bool x, uint256 sz) = ProtoBufRuntime._decode_bool(p, bs);
        r.allow_update_after_misbehaviour = x;
        return sz;
    }

    // struct decoder
    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcLightclientsTendermintV1Fraction(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcLightclientsTendermintV1Fraction.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcLightclientsTendermintV1Fraction.Data memory r,) =
            IbcLightclientsTendermintV1Fraction._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_GoogleProtobufDuration(
        uint256 p,
        bytes memory bs
    ) internal pure returns (GoogleProtobufDuration.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (GoogleProtobufDuration.Data memory r,) =
            GoogleProtobufDuration._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcCoreClientV1Height(
        uint256 p,
        bytes memory bs
    ) internal pure returns (IbcCoreClientV1Height.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreClientV1Height.Data memory r,) =
            IbcCoreClientV1Height._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_CosmosIcs23V1ProofSpec(
        uint256 p,
        bytes memory bs
    ) internal pure returns (CosmosIcs23V1ProofSpec.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (CosmosIcs23V1ProofSpec.Data memory r,) =
            CosmosIcs23V1ProofSpec._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    // Encoder section

    /**
     * @dev The main encoder for memory
     * @param r The struct to be encoded
     * @return The encoded byte array
     */
    function encode(Data memory r) internal pure returns (bytes memory) {
        bytes memory bs = new bytes(_estimate(r));
        uint256 sz = _encode(r, 32, bs);
        assembly {
            mstore(bs, sz)
        }
        return bs;
    }

    // inner encoder

    /**
     * @dev The encoder for internal usage
     * @param r The struct to be encoded
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The number of bytes encoded
     */
    function _encode(
        Data memory r,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint256 offset = p;
        uint256 pointer = p;
        uint256 i;
        if (bytes(r.chain_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.chain_id, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += IbcLightclientsTendermintV1Fraction._encode_nested(
            r.trust_level, pointer, bs
        );

        pointer += ProtoBufRuntime._encode_key(
            3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += GoogleProtobufDuration._encode_nested(
            r.trusting_period, pointer, bs
        );

        pointer += ProtoBufRuntime._encode_key(
            4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += GoogleProtobufDuration._encode_nested(
            r.unbonding_period, pointer, bs
        );

        pointer += ProtoBufRuntime._encode_key(
            5, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += GoogleProtobufDuration._encode_nested(
            r.max_clock_drift, pointer, bs
        );

        pointer += ProtoBufRuntime._encode_key(
            6, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            IbcCoreClientV1Height._encode_nested(r.frozen_height, pointer, bs);

        pointer += ProtoBufRuntime._encode_key(
            7, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            IbcCoreClientV1Height._encode_nested(r.latest_height, pointer, bs);

        if (r.proof_specs.length != 0) {
            for (i = 0; i < r.proof_specs.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    8, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += CosmosIcs23V1ProofSpec._encode_nested(
                    r.proof_specs[i], pointer, bs
                );
            }
        }
        if (r.upgrade_path.length != 0) {
            for (i = 0; i < r.upgrade_path.length; i++) {
                pointer += ProtoBufRuntime._encode_key(
                    9, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
                );
                pointer += ProtoBufRuntime._encode_string(
                    r.upgrade_path[i], pointer, bs
                );
            }
        }
        if (r.allow_update_after_expiry != false) {
            pointer += ProtoBufRuntime._encode_key(
                10, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bool(
                r.allow_update_after_expiry, pointer, bs
            );
        }
        if (r.allow_update_after_misbehaviour != false) {
            pointer += ProtoBufRuntime._encode_key(
                11, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bool(
                r.allow_update_after_misbehaviour, pointer, bs
            );
        }
        return pointer - offset;
    }

    // nested encoder

    /**
     * @dev The encoder for inner struct
     * @param r The struct to be encoded
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The number of bytes encoded
     */
    function _encode_nested(
        Data memory r,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        /**
         * First encoded `r` into a temporary array, and encode the actual size used.
         * Then copy the temporary array into `bs`.
         */
        uint256 offset = p;
        uint256 pointer = p;
        bytes memory tmp = new bytes(_estimate(r));
        uint256 tmpAddr = ProtoBufRuntime.getMemoryAddress(tmp);
        uint256 bsAddr = ProtoBufRuntime.getMemoryAddress(bs);
        uint256 size = _encode(r, 32, tmp);
        pointer += ProtoBufRuntime._encode_varint(size, pointer, bs);
        ProtoBufRuntime.copyBytes(tmpAddr + 32, bsAddr + pointer, size);
        pointer += size;
        delete tmp;
        return pointer - offset;
    }

    // estimator

    /**
     * @dev The estimator for a struct
     * @param r The struct to be encoded
     * @return The number of bytes encoded in estimation
     */
    function _estimate(Data memory r) internal pure returns (uint256) {
        uint256 e;
        uint256 i;
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.chain_id).length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcLightclientsTendermintV1Fraction._estimate(r.trust_level)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                GoogleProtobufDuration._estimate(r.trusting_period)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                GoogleProtobufDuration._estimate(r.unbonding_period)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                GoogleProtobufDuration._estimate(r.max_clock_drift)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreClientV1Height._estimate(r.frozen_height)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreClientV1Height._estimate(r.latest_height)
            );
        for (i = 0; i < r.proof_specs.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(
                    CosmosIcs23V1ProofSpec._estimate(r.proof_specs[i])
                );
        }
        for (i = 0; i < r.upgrade_path.length; i++) {
            e += 1
                + ProtoBufRuntime._sz_lendelim(bytes(r.upgrade_path[i]).length);
        }
        e += 1 + 1;
        e += 1 + 1;
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.chain_id).length != 0) {
            return false;
        }

        if (r.proof_specs.length != 0) {
            return false;
        }

        if (r.upgrade_path.length != 0) {
            return false;
        }

        if (r.allow_update_after_expiry != false) {
            return false;
        }

        if (r.allow_update_after_misbehaviour != false) {
            return false;
        }

        return true;
    }

    //store function
    /**
     * @dev Store in-memory struct to storage
     * @param input The in-memory struct
     * @param output The in-storage struct
     */
    function store(Data memory input, Data storage output) internal {
        output.chain_id = input.chain_id;
        IbcLightclientsTendermintV1Fraction.store(
            input.trust_level, output.trust_level
        );
        GoogleProtobufDuration.store(
            input.trusting_period, output.trusting_period
        );
        GoogleProtobufDuration.store(
            input.unbonding_period, output.unbonding_period
        );
        GoogleProtobufDuration.store(
            input.max_clock_drift, output.max_clock_drift
        );
        IbcCoreClientV1Height.store(input.frozen_height, output.frozen_height);
        IbcCoreClientV1Height.store(input.latest_height, output.latest_height);

        for (uint256 i8 = 0; i8 < input.proof_specs.length; i8++) {
            output.proof_specs.push(input.proof_specs[i8]);
        }

        output.upgrade_path = input.upgrade_path;
        output.allow_update_after_expiry = input.allow_update_after_expiry;
        output.allow_update_after_misbehaviour =
            input.allow_update_after_misbehaviour;
    }

    //array helpers for ProofSpecs
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addProofSpecs(
        Data memory self,
        CosmosIcs23V1ProofSpec.Data memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        CosmosIcs23V1ProofSpec.Data[] memory tmp =
            new CosmosIcs23V1ProofSpec.Data[](self.proof_specs.length + 1);
        for (uint256 i; i < self.proof_specs.length; i++) {
            tmp[i] = self.proof_specs[i];
        }
        tmp[self.proof_specs.length] = value;
        self.proof_specs = tmp;
    }

    //array helpers for UpgradePath
    /**
     * @dev Add value to an array
     * @param self The in-memory struct
     * @param value The value to add
     */
    function addUpgradePath(
        Data memory self,
        string memory value
    ) internal pure {
        /**
         * First resize the array. Then add the new element to the end.
         */
        string[] memory tmp = new string[](self.upgrade_path.length + 1);
        for (uint256 i; i < self.upgrade_path.length; i++) {
            tmp[i] = self.upgrade_path[i];
        }
        tmp[self.upgrade_path.length] = value;
        self.upgrade_path = tmp;
    }

    //utility functions
    /**
     * @dev Return an empty struct
     * @return r The empty struct
     */
    function nil() internal pure returns (Data memory r) {
        assembly {
            r := 0
        }
    }

    /**
     * @dev Test whether a struct is empty
     * @param x The struct to be tested
     * @return r True if it is empty
     */
    function isNil(Data memory x) internal pure returns (bool r) {
        assembly {
            r := iszero(x)
        }
    }
}

//library IbcLightclientsTendermintV1ClientState

library IbcLightclientsTendermintV1ConsensusState {
    //struct definition
    struct Data {
        GoogleProtobufTimestamp.Data timestamp;
        IbcCoreCommitmentV1MerkleRoot.Data root;
        bytes next_validators_hash;
    }

    // Decoder section

    /**
     * @dev The main decoder for memory
     * @param bs The bytes array to be decoded
     * @return The decoded struct
     */
    function decode(bytes memory bs) internal pure returns (Data memory) {
        (Data memory x,) = _decode(32, bs, bs.length);
        return x;
    }

    /**
     * @dev The main decoder for storage
     * @param self The in-storage struct
     * @param bs The bytes array to be decoded
     */
    function decode(Data storage self, bytes memory bs) internal {
        (Data memory x,) = _decode(32, bs, bs.length);
        store(x, self);
    }

    // inner decoder

    /**
     * @dev The decoder for internal usage
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param sz The number of bytes expected
     * @return The decoded struct
     * @return The number of bytes decoded
     */
    function _decode(
        uint256 p,
        bytes memory bs,
        uint256 sz
    ) internal pure returns (Data memory, uint256) {
        Data memory r;
        uint256 fieldId;
        ProtoBufRuntime.WireType wireType;
        uint256 bytesRead;
        uint256 offset = p;
        uint256 pointer = p;
        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer += _read_timestamp(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_root(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_next_validators_hash(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        return (r, sz);
    }

    // field readers

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_timestamp(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (GoogleProtobufTimestamp.Data memory x, uint256 sz) =
            _decode_GoogleProtobufTimestamp(p, bs);
        r.timestamp = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_root(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcCoreCommitmentV1MerkleRoot.Data memory x, uint256 sz) =
            _decode_IbcCoreCommitmentV1MerkleRoot(p, bs);
        r.root = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_next_validators_hash(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (bytes memory x, uint256 sz) = ProtoBufRuntime._decode_bytes(p, bs);
        r.next_validators_hash = x;
        return sz;
    }

    // struct decoder
    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_GoogleProtobufTimestamp(
        uint256 p,
        bytes memory bs
    ) internal pure returns (GoogleProtobufTimestamp.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (GoogleProtobufTimestamp.Data memory r,) =
            GoogleProtobufTimestamp._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcCoreCommitmentV1MerkleRoot(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcCoreCommitmentV1MerkleRoot.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreCommitmentV1MerkleRoot.Data memory r,) =
            IbcCoreCommitmentV1MerkleRoot._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    // Encoder section

    /**
     * @dev The main encoder for memory
     * @param r The struct to be encoded
     * @return The encoded byte array
     */
    function encode(Data memory r) internal pure returns (bytes memory) {
        bytes memory bs = new bytes(_estimate(r));
        uint256 sz = _encode(r, 32, bs);
        assembly {
            mstore(bs, sz)
        }
        return bs;
    }

    // inner encoder

    /**
     * @dev The encoder for internal usage
     * @param r The struct to be encoded
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The number of bytes encoded
     */
    function _encode(
        Data memory r,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint256 offset = p;
        uint256 pointer = p;

        pointer += ProtoBufRuntime._encode_key(
            1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            GoogleProtobufTimestamp._encode_nested(r.timestamp, pointer, bs);

        pointer += ProtoBufRuntime._encode_key(
            2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            IbcCoreCommitmentV1MerkleRoot._encode_nested(r.root, pointer, bs);

        if (r.next_validators_hash.length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_bytes(
                r.next_validators_hash, pointer, bs
            );
        }
        return pointer - offset;
    }

    // nested encoder

    /**
     * @dev The encoder for inner struct
     * @param r The struct to be encoded
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The number of bytes encoded
     */
    function _encode_nested(
        Data memory r,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        /**
         * First encoded `r` into a temporary array, and encode the actual size used.
         * Then copy the temporary array into `bs`.
         */
        uint256 offset = p;
        uint256 pointer = p;
        bytes memory tmp = new bytes(_estimate(r));
        uint256 tmpAddr = ProtoBufRuntime.getMemoryAddress(tmp);
        uint256 bsAddr = ProtoBufRuntime.getMemoryAddress(bs);
        uint256 size = _encode(r, 32, tmp);
        pointer += ProtoBufRuntime._encode_varint(size, pointer, bs);
        ProtoBufRuntime.copyBytes(tmpAddr + 32, bsAddr + pointer, size);
        pointer += size;
        delete tmp;
        return pointer - offset;
    }

    // estimator

    /**
     * @dev The estimator for a struct
     * @param r The struct to be encoded
     * @return The number of bytes encoded in estimation
     */
    function _estimate(Data memory r) internal pure returns (uint256) {
        uint256 e;
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                GoogleProtobufTimestamp._estimate(r.timestamp)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreCommitmentV1MerkleRoot._estimate(r.root)
            );
        e += 1 + ProtoBufRuntime._sz_lendelim(r.next_validators_hash.length);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.next_validators_hash.length != 0) {
            return false;
        }

        return true;
    }

    //store function
    /**
     * @dev Store in-memory struct to storage
     * @param input The in-memory struct
     * @param output The in-storage struct
     */
    function store(Data memory input, Data storage output) internal {
        GoogleProtobufTimestamp.store(input.timestamp, output.timestamp);
        IbcCoreCommitmentV1MerkleRoot.store(input.root, output.root);
        output.next_validators_hash = input.next_validators_hash;
    }

    //utility functions
    /**
     * @dev Return an empty struct
     * @return r The empty struct
     */
    function nil() internal pure returns (Data memory r) {
        assembly {
            r := 0
        }
    }

    /**
     * @dev Test whether a struct is empty
     * @param x The struct to be tested
     * @return r True if it is empty
     */
    function isNil(Data memory x) internal pure returns (bool r) {
        assembly {
            r := iszero(x)
        }
    }
}

//library IbcLightclientsTendermintV1ConsensusState

library IbcLightclientsTendermintV1Misbehaviour {
    //struct definition
    struct Data {
        string client_id;
        IbcLightclientsTendermintV1Header.Data header_1;
        IbcLightclientsTendermintV1Header.Data header_2;
    }

    // Decoder section

    /**
     * @dev The main decoder for memory
     * @param bs The bytes array to be decoded
     * @return The decoded struct
     */
    function decode(bytes memory bs) internal pure returns (Data memory) {
        (Data memory x,) = _decode(32, bs, bs.length);
        return x;
    }

    /**
     * @dev The main decoder for storage
     * @param self The in-storage struct
     * @param bs The bytes array to be decoded
     */
    function decode(Data storage self, bytes memory bs) internal {
        (Data memory x,) = _decode(32, bs, bs.length);
        store(x, self);
    }

    // inner decoder

    /**
     * @dev The decoder for internal usage
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param sz The number of bytes expected
     * @return The decoded struct
     * @return The number of bytes decoded
     */
    function _decode(
        uint256 p,
        bytes memory bs,
        uint256 sz
    ) internal pure returns (Data memory, uint256) {
        Data memory r;
        uint256 fieldId;
        ProtoBufRuntime.WireType wireType;
        uint256 bytesRead;
        uint256 offset = p;
        uint256 pointer = p;
        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer += _read_client_id(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_header_1(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_header_2(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        return (r, sz);
    }

    // field readers

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_client_id(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (string memory x, uint256 sz) = ProtoBufRuntime._decode_string(p, bs);
        r.client_id = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_header_1(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcLightclientsTendermintV1Header.Data memory x, uint256 sz) =
            _decode_IbcLightclientsTendermintV1Header(p, bs);
        r.header_1 = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_header_2(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcLightclientsTendermintV1Header.Data memory x, uint256 sz) =
            _decode_IbcLightclientsTendermintV1Header(p, bs);
        r.header_2 = x;
        return sz;
    }

    // struct decoder
    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcLightclientsTendermintV1Header(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (IbcLightclientsTendermintV1Header.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcLightclientsTendermintV1Header.Data memory r,) =
            IbcLightclientsTendermintV1Header._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    // Encoder section

    /**
     * @dev The main encoder for memory
     * @param r The struct to be encoded
     * @return The encoded byte array
     */
    function encode(Data memory r) internal pure returns (bytes memory) {
        bytes memory bs = new bytes(_estimate(r));
        uint256 sz = _encode(r, 32, bs);
        assembly {
            mstore(bs, sz)
        }
        return bs;
    }

    // inner encoder

    /**
     * @dev The encoder for internal usage
     * @param r The struct to be encoded
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The number of bytes encoded
     */
    function _encode(
        Data memory r,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint256 offset = p;
        uint256 pointer = p;

        if (bytes(r.client_id).length != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_string(r.client_id, pointer, bs);
        }

        pointer += ProtoBufRuntime._encode_key(
            2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += IbcLightclientsTendermintV1Header._encode_nested(
            r.header_1, pointer, bs
        );

        pointer += ProtoBufRuntime._encode_key(
            3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += IbcLightclientsTendermintV1Header._encode_nested(
            r.header_2, pointer, bs
        );

        return pointer - offset;
    }

    // nested encoder

    /**
     * @dev The encoder for inner struct
     * @param r The struct to be encoded
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The number of bytes encoded
     */
    function _encode_nested(
        Data memory r,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        /**
         * First encoded `r` into a temporary array, and encode the actual size used.
         * Then copy the temporary array into `bs`.
         */
        uint256 offset = p;
        uint256 pointer = p;
        bytes memory tmp = new bytes(_estimate(r));
        uint256 tmpAddr = ProtoBufRuntime.getMemoryAddress(tmp);
        uint256 bsAddr = ProtoBufRuntime.getMemoryAddress(bs);
        uint256 size = _encode(r, 32, tmp);
        pointer += ProtoBufRuntime._encode_varint(size, pointer, bs);
        ProtoBufRuntime.copyBytes(tmpAddr + 32, bsAddr + pointer, size);
        pointer += size;
        delete tmp;
        return pointer - offset;
    }

    // estimator

    /**
     * @dev The estimator for a struct
     * @param r The struct to be encoded
     * @return The number of bytes encoded in estimation
     */
    function _estimate(Data memory r) internal pure returns (uint256) {
        uint256 e;
        e += 1 + ProtoBufRuntime._sz_lendelim(bytes(r.client_id).length);
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcLightclientsTendermintV1Header._estimate(r.header_1)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcLightclientsTendermintV1Header._estimate(r.header_2)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (bytes(r.client_id).length != 0) {
            return false;
        }

        return true;
    }

    //store function
    /**
     * @dev Store in-memory struct to storage
     * @param input The in-memory struct
     * @param output The in-storage struct
     */
    function store(Data memory input, Data storage output) internal {
        output.client_id = input.client_id;
        IbcLightclientsTendermintV1Header.store(input.header_1, output.header_1);
        IbcLightclientsTendermintV1Header.store(input.header_2, output.header_2);
    }

    //utility functions
    /**
     * @dev Return an empty struct
     * @return r The empty struct
     */
    function nil() internal pure returns (Data memory r) {
        assembly {
            r := 0
        }
    }

    /**
     * @dev Test whether a struct is empty
     * @param x The struct to be tested
     * @return r True if it is empty
     */
    function isNil(Data memory x) internal pure returns (bool r) {
        assembly {
            r := iszero(x)
        }
    }
}

//library IbcLightclientsTendermintV1Misbehaviour

library IbcLightclientsTendermintV1Header {
    //struct definition
    struct Data {
        TendermintTypesSignedHeader.Data signed_header;
        TendermintTypesValidatorSet.Data validator_set;
        IbcCoreClientV1Height.Data trusted_height;
        TendermintTypesValidatorSet.Data trusted_validators;
    }

    // Decoder section

    /**
     * @dev The main decoder for memory
     * @param bs The bytes array to be decoded
     * @return The decoded struct
     */
    function decode(bytes memory bs) internal pure returns (Data memory) {
        (Data memory x,) = _decode(32, bs, bs.length);
        return x;
    }

    /**
     * @dev The main decoder for storage
     * @param self The in-storage struct
     * @param bs The bytes array to be decoded
     */
    function decode(Data storage self, bytes memory bs) internal {
        (Data memory x,) = _decode(32, bs, bs.length);
        store(x, self);
    }

    // inner decoder

    /**
     * @dev The decoder for internal usage
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param sz The number of bytes expected
     * @return The decoded struct
     * @return The number of bytes decoded
     */
    function _decode(
        uint256 p,
        bytes memory bs,
        uint256 sz
    ) internal pure returns (Data memory, uint256) {
        Data memory r;
        uint256 fieldId;
        ProtoBufRuntime.WireType wireType;
        uint256 bytesRead;
        uint256 offset = p;
        uint256 pointer = p;
        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer += _read_signed_header(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_validator_set(pointer, bs, r);
            } else if (fieldId == 3) {
                pointer += _read_trusted_height(pointer, bs, r);
            } else if (fieldId == 4) {
                pointer += _read_trusted_validators(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        return (r, sz);
    }

    // field readers

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_signed_header(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (TendermintTypesSignedHeader.Data memory x, uint256 sz) =
            _decode_TendermintTypesSignedHeader(p, bs);
        r.signed_header = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_validator_set(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (TendermintTypesValidatorSet.Data memory x, uint256 sz) =
            _decode_TendermintTypesValidatorSet(p, bs);
        r.validator_set = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_trusted_height(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (IbcCoreClientV1Height.Data memory x, uint256 sz) =
            _decode_IbcCoreClientV1Height(p, bs);
        r.trusted_height = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_trusted_validators(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (TendermintTypesValidatorSet.Data memory x, uint256 sz) =
            _decode_TendermintTypesValidatorSet(p, bs);
        r.trusted_validators = x;
        return sz;
    }

    // struct decoder
    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_TendermintTypesSignedHeader(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (TendermintTypesSignedHeader.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (TendermintTypesSignedHeader.Data memory r,) =
            TendermintTypesSignedHeader._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_TendermintTypesValidatorSet(
        uint256 p,
        bytes memory bs
    )
        internal
        pure
        returns (TendermintTypesValidatorSet.Data memory, uint256)
    {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (TendermintTypesValidatorSet.Data memory r,) =
            TendermintTypesValidatorSet._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    /**
     * @dev The decoder for reading a inner struct field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The decoded inner-struct
     * @return The number of bytes used to decode
     */
    function _decode_IbcCoreClientV1Height(
        uint256 p,
        bytes memory bs
    ) internal pure returns (IbcCoreClientV1Height.Data memory, uint256) {
        uint256 pointer = p;
        (uint256 sz, uint256 bytesRead) =
            ProtoBufRuntime._decode_varint(pointer, bs);
        pointer += bytesRead;
        (IbcCoreClientV1Height.Data memory r,) =
            IbcCoreClientV1Height._decode(pointer, bs, sz);
        return (r, sz + bytesRead);
    }

    // Encoder section

    /**
     * @dev The main encoder for memory
     * @param r The struct to be encoded
     * @return The encoded byte array
     */
    function encode(Data memory r) internal pure returns (bytes memory) {
        bytes memory bs = new bytes(_estimate(r));
        uint256 sz = _encode(r, 32, bs);
        assembly {
            mstore(bs, sz)
        }
        return bs;
    }

    // inner encoder

    /**
     * @dev The encoder for internal usage
     * @param r The struct to be encoded
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The number of bytes encoded
     */
    function _encode(
        Data memory r,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint256 offset = p;
        uint256 pointer = p;

        pointer += ProtoBufRuntime._encode_key(
            1, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += TendermintTypesSignedHeader._encode_nested(
            r.signed_header, pointer, bs
        );

        pointer += ProtoBufRuntime._encode_key(
            2, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += TendermintTypesValidatorSet._encode_nested(
            r.validator_set, pointer, bs
        );

        pointer += ProtoBufRuntime._encode_key(
            3, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer +=
            IbcCoreClientV1Height._encode_nested(r.trusted_height, pointer, bs);

        pointer += ProtoBufRuntime._encode_key(
            4, ProtoBufRuntime.WireType.LengthDelim, pointer, bs
        );
        pointer += TendermintTypesValidatorSet._encode_nested(
            r.trusted_validators, pointer, bs
        );

        return pointer - offset;
    }

    // nested encoder

    /**
     * @dev The encoder for inner struct
     * @param r The struct to be encoded
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The number of bytes encoded
     */
    function _encode_nested(
        Data memory r,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        /**
         * First encoded `r` into a temporary array, and encode the actual size used.
         * Then copy the temporary array into `bs`.
         */
        uint256 offset = p;
        uint256 pointer = p;
        bytes memory tmp = new bytes(_estimate(r));
        uint256 tmpAddr = ProtoBufRuntime.getMemoryAddress(tmp);
        uint256 bsAddr = ProtoBufRuntime.getMemoryAddress(bs);
        uint256 size = _encode(r, 32, tmp);
        pointer += ProtoBufRuntime._encode_varint(size, pointer, bs);
        ProtoBufRuntime.copyBytes(tmpAddr + 32, bsAddr + pointer, size);
        pointer += size;
        delete tmp;
        return pointer - offset;
    }

    // estimator

    /**
     * @dev The estimator for a struct
     * @param r The struct to be encoded
     * @return The number of bytes encoded in estimation
     */
    function _estimate(Data memory r) internal pure returns (uint256) {
        uint256 e;
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                TendermintTypesSignedHeader._estimate(r.signed_header)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                TendermintTypesValidatorSet._estimate(r.validator_set)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                IbcCoreClientV1Height._estimate(r.trusted_height)
            );
        e += 1
            + ProtoBufRuntime._sz_lendelim(
                TendermintTypesValidatorSet._estimate(r.trusted_validators)
            );
        return e;
    }

    // empty checker

    function _empty(Data memory) internal pure returns (bool) {
        return true;
    }

    //store function
    /**
     * @dev Store in-memory struct to storage
     * @param input The in-memory struct
     * @param output The in-storage struct
     */
    function store(Data memory input, Data storage output) internal {
        TendermintTypesSignedHeader.store(
            input.signed_header, output.signed_header
        );
        TendermintTypesValidatorSet.store(
            input.validator_set, output.validator_set
        );
        IbcCoreClientV1Height.store(input.trusted_height, output.trusted_height);
        TendermintTypesValidatorSet.store(
            input.trusted_validators, output.trusted_validators
        );
    }

    //utility functions
    /**
     * @dev Return an empty struct
     * @return r The empty struct
     */
    function nil() internal pure returns (Data memory r) {
        assembly {
            r := 0
        }
    }

    /**
     * @dev Test whether a struct is empty
     * @param x The struct to be tested
     * @return r True if it is empty
     */
    function isNil(Data memory x) internal pure returns (bool r) {
        assembly {
            r := iszero(x)
        }
    }
}

//library IbcLightclientsTendermintV1Header

library IbcLightclientsTendermintV1Fraction {
    //struct definition
    struct Data {
        uint64 numerator;
        uint64 denominator;
    }

    // Decoder section

    /**
     * @dev The main decoder for memory
     * @param bs The bytes array to be decoded
     * @return The decoded struct
     */
    function decode(bytes memory bs) internal pure returns (Data memory) {
        (Data memory x,) = _decode(32, bs, bs.length);
        return x;
    }

    /**
     * @dev The main decoder for storage
     * @param self The in-storage struct
     * @param bs The bytes array to be decoded
     */
    function decode(Data storage self, bytes memory bs) internal {
        (Data memory x,) = _decode(32, bs, bs.length);
        store(x, self);
    }

    // inner decoder

    /**
     * @dev The decoder for internal usage
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param sz The number of bytes expected
     * @return The decoded struct
     * @return The number of bytes decoded
     */
    function _decode(
        uint256 p,
        bytes memory bs,
        uint256 sz
    ) internal pure returns (Data memory, uint256) {
        Data memory r;
        uint256 fieldId;
        ProtoBufRuntime.WireType wireType;
        uint256 bytesRead;
        uint256 offset = p;
        uint256 pointer = p;
        while (pointer < offset + sz) {
            (fieldId, wireType, bytesRead) =
                ProtoBufRuntime._decode_key(pointer, bs);
            pointer += bytesRead;
            if (fieldId == 1) {
                pointer += _read_numerator(pointer, bs, r);
            } else if (fieldId == 2) {
                pointer += _read_denominator(pointer, bs, r);
            } else {
                pointer +=
                    ProtoBufRuntime._skip_field_decode(wireType, pointer, bs);
            }
        }
        return (r, sz);
    }

    // field readers

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_numerator(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (uint64 x, uint256 sz) = ProtoBufRuntime._decode_uint64(p, bs);
        r.numerator = x;
        return sz;
    }

    /**
     * @dev The decoder for reading a field
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @param r The in-memory struct
     * @return The number of bytes decoded
     */
    function _read_denominator(
        uint256 p,
        bytes memory bs,
        Data memory r
    ) internal pure returns (uint256) {
        (uint64 x, uint256 sz) = ProtoBufRuntime._decode_uint64(p, bs);
        r.denominator = x;
        return sz;
    }

    // Encoder section

    /**
     * @dev The main encoder for memory
     * @param r The struct to be encoded
     * @return The encoded byte array
     */
    function encode(Data memory r) internal pure returns (bytes memory) {
        bytes memory bs = new bytes(_estimate(r));
        uint256 sz = _encode(r, 32, bs);
        assembly {
            mstore(bs, sz)
        }
        return bs;
    }

    // inner encoder

    /**
     * @dev The encoder for internal usage
     * @param r The struct to be encoded
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The number of bytes encoded
     */
    function _encode(
        Data memory r,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        uint256 offset = p;
        uint256 pointer = p;

        if (r.numerator != 0) {
            pointer += ProtoBufRuntime._encode_key(
                1, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer += ProtoBufRuntime._encode_uint64(r.numerator, pointer, bs);
        }
        if (r.denominator != 0) {
            pointer += ProtoBufRuntime._encode_key(
                2, ProtoBufRuntime.WireType.Varint, pointer, bs
            );
            pointer +=
                ProtoBufRuntime._encode_uint64(r.denominator, pointer, bs);
        }
        return pointer - offset;
    }

    // nested encoder

    /**
     * @dev The encoder for inner struct
     * @param r The struct to be encoded
     * @param p The offset of bytes array to start decode
     * @param bs The bytes array to be decoded
     * @return The number of bytes encoded
     */
    function _encode_nested(
        Data memory r,
        uint256 p,
        bytes memory bs
    ) internal pure returns (uint256) {
        /**
         * First encoded `r` into a temporary array, and encode the actual size used.
         * Then copy the temporary array into `bs`.
         */
        uint256 offset = p;
        uint256 pointer = p;
        bytes memory tmp = new bytes(_estimate(r));
        uint256 tmpAddr = ProtoBufRuntime.getMemoryAddress(tmp);
        uint256 bsAddr = ProtoBufRuntime.getMemoryAddress(bs);
        uint256 size = _encode(r, 32, tmp);
        pointer += ProtoBufRuntime._encode_varint(size, pointer, bs);
        ProtoBufRuntime.copyBytes(tmpAddr + 32, bsAddr + pointer, size);
        pointer += size;
        delete tmp;
        return pointer - offset;
    }

    // estimator

    /**
     * @dev The estimator for a struct
     * @param r The struct to be encoded
     * @return The number of bytes encoded in estimation
     */
    function _estimate(Data memory r) internal pure returns (uint256) {
        uint256 e;
        e += 1 + ProtoBufRuntime._sz_uint64(r.numerator);
        e += 1 + ProtoBufRuntime._sz_uint64(r.denominator);
        return e;
    }

    // empty checker

    function _empty(Data memory r) internal pure returns (bool) {
        if (r.numerator != 0) {
            return false;
        }

        if (r.denominator != 0) {
            return false;
        }

        return true;
    }

    //store function
    /**
     * @dev Store in-memory struct to storage
     * @param input The in-memory struct
     * @param output The in-storage struct
     */
    function store(Data memory input, Data storage output) internal {
        output.numerator = input.numerator;
        output.denominator = input.denominator;
    }

    //utility functions
    /**
     * @dev Return an empty struct
     * @return r The empty struct
     */
    function nil() internal pure returns (Data memory r) {
        assembly {
            r := 0
        }
    }

    /**
     * @dev Test whether a struct is empty
     * @param x The struct to be tested
     * @return r True if it is empty
     */
    function isNil(Data memory x) internal pure returns (bool r) {
        assembly {
            r := iszero(x)
        }
    }
}
//library IbcLightclientsTendermintV1Fraction
