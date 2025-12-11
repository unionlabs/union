// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.
//
// Parameters
//
// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's cairo subdirectory
//                       The Licensed Work is (c) 2025 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//
//
// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.
//
// Notice
//
// Business Source License 1.1
//
// Terms
//
// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.
//
// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.
//
// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.
//
// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.
//
// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.
//
// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.
//
// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).
//
// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

use alexandria_math::opt_math::OptBitShift;
use core::sha256::compute_sha256_byte_array;

pub const IAVL_PROOF_SPEC: ProofSpec = ProofSpec {
    child_size: 33, min_prefix_length: 4, max_prefix_length: 12,
};

pub const TM_PROOF_SPEC: ProofSpec = ProofSpec {
    child_size: 32, min_prefix_length: 1, max_prefix_length: 1,
};

#[derive(Copy, Debug, Drop, PartialEq)]
pub enum Error {
    EmptyInnerKey,
    EmptyInnerValue,
    EmptyChild,
    EmptyLeafPrefix,
    ProofKeyMismatch,
    ProofValueMismatch,
    CommitmentRootMismatch,
    InvalidInnerPrefix,
    InvalidLeafPrefix,
}

#[derive(Drop, Serde)]
pub struct MembershipProof {
    sub_proof: ExistenceProof,
    top_level_proof: ExistenceProof,
}

#[derive(Drop, Serde)]
pub struct ExistenceProof {
    key: ByteArray,
    value: ByteArray,
    leaf_prefix: ByteArray,
    path: Array<InnerOp>,
}

#[derive(Drop, Serde)]
pub struct InnerOp {
    prefix: ByteArray,
    suffix: ByteArray,
}

#[derive(Drop)]
pub struct NonExistenceProof {
    key: ByteArray,
    left: Option<ExistenceProof>,
    right: Option<ExistenceProof>,
}

#[derive(Drop)]
pub struct ProofSpec {
    child_size: u64,
    min_prefix_length: u64,
    max_prefix_length: u64,
}

#[generate_trait]
pub impl MembershipProofImpl of MembershipProofTrait {
    fn verify(
        self: @MembershipProof, root: [u32; 8], prefix: ByteArray, key: ByteArray, value: ByteArray,
    ) -> Result<(), Error> {
        let subroot = self.sub_proof.calculate_root()?;

        self.sub_proof.verify_no_root_check(@IAVL_PROOF_SPEC, @key, @value)?;

        let mut bytes = Default::default();
        for i in subroot.span() {
            bytes.append_word((*i).into(), 4);
        }

        self.top_level_proof.verify(@TM_PROOF_SPEC, root, @prefix, @bytes)?;

        Ok(())
    }
}

#[generate_trait]
impl ExistenceProofImpl of ExistenceProofTrait {
    #[inline]
    fn verify_no_root_check(
        self: @ExistenceProof, proof_spec: @ProofSpec, key: @ByteArray, value: @ByteArray,
    ) -> Result<(), Error> {
        if key != self.key {
            return Err(Error::ProofKeyMismatch);
        }

        if value != self.value {
            return Err(Error::ProofValueMismatch);
        }

        self.check_against_spec(proof_spec)?;

        Ok(())
    }

    fn verify(
        self: @ExistenceProof,
        proof_spec: @ProofSpec,
        commitment_root: [u32; 8],
        key: @ByteArray,
        value: @ByteArray,
    ) -> Result<(), Error> {
        self.verify_no_root_check(proof_spec, key, value)?;

        let root = self.calculate_root()?;

        if root != commitment_root {
            return Err(Error::CommitmentRootMismatch);
        }

        Ok(())
    }

    fn check_against_spec(self: @ExistenceProof, proof_spec: @ProofSpec) -> Result<(), Error> {
        if self.leaf_prefix.len() == 0 {
            return Err(Error::EmptyLeafPrefix);
        }

        if self.leaf_prefix[0] != 0 {
            return Err(Error::InvalidLeafPrefix);
        }

        let max = *proof_spec.max_prefix_length + *proof_spec.child_size;

        for inner_op in self.path {
            if !(inner_op.prefix.len().into() >= *proof_spec.min_prefix_length
                && inner_op.prefix[0] != 0 || inner_op.prefix.len().into() <= max) {
                return Err(Error::InvalidInnerPrefix);
            }
        }

        Ok(())
    }

    fn calculate_root(self: @ExistenceProof) -> Result<[u32; 8], Error> {
        if self.leaf_prefix.len() == 0 {
            return Err(Error::EmptyLeafPrefix);
        }

        let mut root = apply_leaf_op(self.leaf_prefix, self.key, self.value)?;

        for p in self.path {
            root = p.apply(root);
        }

        Ok(root)
    }
}

#[generate_trait]
impl InnerOpImpl of InnerOpTrait {
    fn apply(self: @InnerOp, child: [u32; 8]) -> [u32; 8] {
        let mut pre_image = self.prefix.clone();

        let child = child.span();
        for i in child {
            pre_image.append_word((*i).into(), 4);
        }
        pre_image.append(self.suffix);

        compute_sha256_byte_array(@pre_image)
    }
}

fn apply_leaf_op(
    prefix: @ByteArray, key: @ByteArray, value: @ByteArray,
) -> Result<[u32; 8], Error> {
    if key.len() == 0 {
        return Err(Error::EmptyInnerKey);
    }

    if value.len() == 0 {
        return Err(Error::EmptyInnerValue);
    }

    let encoded_key = encode_varint(key.len().into());

    let hashed_value = compute_sha256_byte_array(value).span();
    let encoded_value = encode_varint(32);

    let mut hash_data = prefix.clone();
    hash_data.append(@encoded_key);
    hash_data.append(key);
    hash_data.append(@encoded_value);

    for i in hashed_value {
        hash_data.append_word((*i).into(), 4);
    }

    Ok(compute_sha256_byte_array(@hash_data))
}

fn encode_varint(mut value: u64) -> ByteArray {
    let mut buf = Default::default();

    for _ in 0..10_u64 {
        if value < 0x80 {
            buf.append_byte(value.try_into().expect('it fits'));
            break;
        } else {
            buf.append_byte(((value & 0x7F) | 0x80).try_into().expect('it fits'));
            value = OptBitShift::shr(value, 7);
        }
    }

    buf
}

#[cfg(test)]
mod tests {
    use alexandria_bytes::byte_array_ext::ByteArrayTraitExt;
    use core::to_byte_array::AppendFormattedToByteArray;
    use super::{*, ExistenceProof};

    fn hex(data: u256) -> ByteArray {
        let mut byte_array = Default::default();
        data.append_formatted_to_byte_array(ref byte_array, 16);

        let mut data_bytes: ByteArray = Default::default();

        let len = byte_array.len() / 2 + (byte_array.len() % 2);

        if len == 32 {
            data_bytes.append_u256(data);
        } else {
            data_bytes.append_word(data.try_into().unwrap(), len);
        }

        data_bytes
    }


    #[test]
    fn test_verify_membership_left() {
        let proof = ExistenceProof {
            key: hex(0x303142424373615a55715146735259436c6a5767),
            value: hex(0x76616c75655f666f725f303142424373615a55715146735259436c6a5767),
            leaf_prefix: hex(0x00),
            path: array![
                InnerOp {
                    prefix: hex(0x01),
                    suffix: hex(0xcb3131cd98b069efcc0e8c7e68da47370adbff32266d7fcd1b0580fdf3961266),
                },
                InnerOp {
                    prefix: hex(0x01),
                    suffix: hex(0x21d1205c1f8537205e8fb4b176f960b459d9131669968d59c456442f7673b68b),
                },
                InnerOp {
                    prefix: hex(0x01),
                    suffix: hex(0xb82a0e7f4434b3cedb87ea83eb5a70c7dc664c77b2fe21c6245f315e58fdf745),
                },
                InnerOp {
                    prefix: hex(0x01),
                    suffix: hex(0xbf0657a0e6fbd8f2043eb2cf751561adcf50547d16201224133eeb8d38145229),
                },
                InnerOp {
                    prefix: hex(0x01),
                    suffix: hex(0x6d47c03df91a4a0252055d116439d34b5b73f3a24d5cb3cf0d4b08caa540cac4),
                },
                InnerOp {
                    prefix: hex(0x01),
                    suffix: hex(0xd5d2926993fa15c7410ac4ee1f1d81afddfb0ab5f6f4706b05f407bc01638149),
                },
                InnerOp {
                    prefix: hex(0x01),
                    suffix: hex(0x540719b26a7301ad012ac45ebe716679e5595e5570d78be9b6da8d8591afb374),
                },
                InnerOp {
                    prefix: hex(0x01),
                    suffix: hex(0xfccaaa9950730e80b9ccf75ad2cfeab26ae750b8bd6ac1ff1c7a7502f3c64be2),
                },
                InnerOp {
                    prefix: hex(0x01),
                    suffix: hex(0xecb61a6d70accb79c2325fb0b51677ed1561c91af5e10578c8294002fbb3c21e),
                },
                InnerOp {
                    prefix: hex(0x01),
                    suffix: hex(0x1b3bc1bd8d08af9f6199de84e95d646570cbd9b306a632a5acf617cbd7d1ab0a),
                },
            ],
        };

        assert_eq!(
            proof
                .verify(
                    @TM_PROOF_SPEC,
                    [
                        0xc569a38a, 0x5775bbda, 0x2051c34a, 0xe0089418, 0x6f837c39, 0xd11dca55,
                        0x495b9aed, 0x14f17ddf,
                    ],
                    @hex(0x303142424373615a55715146735259436c6a5767),
                    @hex(0x76616c75655f666f725f303142424373615a55715146735259436c6a5767),
                ),
            Ok(()),
        );
    }
}
