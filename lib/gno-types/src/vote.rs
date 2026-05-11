use core::fmt;

use serde::{Deserialize, Serialize};
use unionlabs::{
    bounded::{BoundedI32, BoundedI64},
    google::protobuf::timestamp::Timestamp,
    primitives::{Bech32, Bytes, H160, encoding::Base64},
};

use crate::{
    Amino, BlockId, SignedMsgType,
    block_id::{CanonicalBlockId, canonicalize_block_id},
    fingerprint,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Vote {
    #[serde(rename = "type")]
    pub ty: SignedMsgType,
    #[serde(with = "::serde_utils::string")]
    pub height: BoundedI64<0, { i64::MAX }>,
    #[serde(with = "::serde_utils::string")]
    pub round: BoundedI32<-1, { i32::MAX }>,
    pub block_id: BlockId,
    pub timestamp: Timestamp,
    pub validator_address: Bech32<H160>,
    #[serde(with = "::serde_utils::string")]
    pub validator_index: i32,
    pub signature: Bytes<Base64>,
}

pub const NIL_VOTE_STR: &str = "nil-Vote";

impl fmt::Display for Vote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // NOTE: This needs to be handled at the call site, since rust does not have null pointers but instead uses Option<T>
        // if vote == nil {
        //     return NIL_VOTE_STR;
        // }

        let type_string = match self.ty {
            SignedMsgType::Prevote => "Prevote",
            SignedMsgType::Precommit => "Precommit",
            // NOTE: The original go implementation panics on an unknown vote.type, let's not do that here
            SignedMsgType::Proposal => "Proposal",
        };

        write!(
            f,
            "Vote{{{}:{} {}/{}/{}({}) {} {} @ {}}}",
            self.validator_index,
            fingerprint(self.validator_address.data()),
            self.height,
            self.round,
            i32::from(self.ty),
            type_string,
            fingerprint(
                self.block_id
                    .hash
                    .as_ref()
                    .map_or::<&[_], _>(&[], |hash| &hash[..])
            ),
            fingerprint(&self.signature),
            self.timestamp,
        )
    }
}

#[derive(Clone, PartialEq, prost::Message)]
pub struct CanonicalVote {
    #[prost(int32, tag = "1")]
    pub ty: i32,
    /// canonicalization requires fixed size encoding here
    #[prost(sfixed64, tag = "2")]
    pub height: i64,
    /// canonicalization requires fixed size encoding here
    #[prost(sfixed64, tag = "3")]
    pub round: i64,
    #[prost(message, optional, tag = "4")]
    pub block_id: Option<CanonicalBlockId>,
    #[prost(message, optional, tag = "5")]
    pub timestamp: Option<protos::google::protobuf::Timestamp>,
    #[prost(string, tag = "6")]
    pub chain_id: String,
}

impl Amino for CanonicalVote {
    fn marshal_sized(&self) -> Bytes {
        let mut out = vec![];
        prost::Message::encode_length_delimited(self, &mut out).expect("infallible");
        out.into()
    }
}

impl Vote {
    pub fn sign_bytes(&self, chain_id: String) -> Bytes {
        canonicalize_vote(chain_id, self).marshal_sized()
    }
}

pub fn canonicalize_vote(chain_id: String, vote: &Vote) -> CanonicalVote {
    CanonicalVote {
        ty: vote.ty.into(),
        height: vote.height.into(),
        round: vote.round.inner().into(),
        block_id: if vote.block_id == BlockId::default() {
            None
        } else {
            Some(canonicalize_block_id(&vote.block_id))
        },
        timestamp: Some(vote.timestamp.into()),
        chain_id,
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;
    use crate::PartSetHeader;

    #[test]
    fn vectors() {
        let tests = [
            // (
            //     "",
            //     Vote { ..default_vote() },
            //     // NOTE: Height and Round are skipped here. This case needs to be considered while parsing.
            //     vec![
            //         0xd, 0x2a, 0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
            //     ],
            // ),
            // with proper (fixed size) height and round (PreCommit):
            (
                "",
                Vote {
                    height: BoundedI64::new(1).unwrap(),
                    round: BoundedI32::new(1).unwrap(),
                    ty: SignedMsgType::Precommit,
                    ..default_vote()
                },
                vec![
                    0x21, // length
                    0x8,  // (field_number << 3) | wire_type
                    0x2,  // PrecommitType
                    0x11, // (field_number << 3) | wire_type
                    0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                    0x19, // (field_number << 3) | wire_type
                    0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // round
                    0x2a, // (field_number << 3) | wire_type
                    // remaining fields (timestamp):
                    0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
                ],
            ),
            // with proper (fixed size) height and round (PreVote):
            (
                "",
                Vote {
                    height: BoundedI64::new(1).unwrap(),
                    round: BoundedI32::new(1).unwrap(),
                    ty: SignedMsgType::Prevote,
                    ..default_vote()
                },
                vec![
                    0x21, // length
                    0x8,  // (field_number << 3) | wire_type
                    0x1,  // PrevoteType
                    0x11, // (field_number << 3) | wire_type
                    0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                    0x19, // (field_number << 3) | wire_type
                    0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // round
                    0x2a, // (field_number << 3) | wire_type
                    // remaining fields (timestamp):
                    0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
                ],
            ),
            (
                "",
                Vote {
                    ty: SignedMsgType::Precommit,
                    height: BoundedI64::new(1).unwrap(),
                    round: BoundedI32::new(1).unwrap(),
                    ..default_vote()
                },
                vec![
                    0x21, // length
                    0x8,  // (field_number << 3) | wire_type
                    0x2,  // PrecommitType
                    0x11, // (field_number << 3) | wire_type
                    0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                    0x19, // (field_number << 3) | wire_type
                    0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // round
                    // remaining fields (timestamp):
                    0x2a, 0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x1,
                ],
            ),
            // containing non-empty chain_id:
            (
                "test_chain_id",
                Vote {
                    ty: SignedMsgType::Precommit,
                    height: BoundedI64::new(1).unwrap(),
                    round: BoundedI32::new(1).unwrap(),
                    ..default_vote()
                },
                vec![
                    0x30, // length
                    0x8,  // (field_number << 3) | wire_type
                    0x2,  // PrecommitType
                    0x11, // (field_number << 3) | wire_type
                    0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,  // height
                    0x19, // (field_number << 3) | wire_type
                    0x1, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, // round
                    // remaining fields:
                    0x2a, // (field_number << 3) | wire_type
                    0xb, 0x8, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff,
                    0x1,  // timestamp
                    0x32, // (field_number << 3) | wire_type
                    0xd, 0x74, 0x65, 0x73, 0x74, 0x5f, 0x63, 0x68, 0x61, 0x69, 0x6e, 0x5f, 0x69,
                    0x64,
                ], // chainID
            ),
            // NOTE: Not valid for us, since we bound the field types
            // Edge value: math.MinInt64 height and -1 round. Locks down fixed64
            // encoding for the most-negative int64 (0x80 high byte, seven zeros)
            // and for -1 (all 0xff bytes) — a silent endianness or sign-extension
            // regression in the fixed64 path would break every precommit signature.

            // NOTE: Not valid for us, since we bound the round field to be i32 and only unwrap it and cast to i64 immediately before encoding
            // Edge value: math.MaxInt64 for both height and round.

            // Height = 0, Round = amino omits zero-valued fields (no write_empty
            // override on fixed64). If a future change forced fixed-width fields
            // to always emit 8 bytes regardless of value, sign-bytes would diverge
            // from every historical precommit signature — this case locks that in.
            (
                "",
                Vote {
                    ty: SignedMsgType::Precommit,
                    height: BoundedI64::new(0).unwrap(),
                    round: BoundedI32::new(0).unwrap(),
                    ..default_vote()
                },
                vec![
                    0x0f, // length (much shorter — no Height/Round emitted)
                    0x08, 0x02, // Type = PrecommitType
                    0x2a, 0x0b, // timestamp
                    0x08, 0x80, 0x92, 0xb8, 0xc3, 0x98, 0xfe, 0xff, 0xff, 0xff, 0x01,
                ],
            ),
        ];

        for (i, (chain_id, vote, want)) in tests.into_iter().enumerate() {
            let got = vote.sign_bytes(chain_id.to_owned());
            assert_eq!(
                got,
                <Bytes>::new(want),
                "test case #{i}: got unexpected sign bytes for Vote."
            )
        }
    }

    fn default_vote() -> Vote {
        Vote {
            ty: SignedMsgType::Prevote, // NOTE: This type is stricter than the go type, we do not allow zero
            height: BoundedI64::new(0).unwrap(),
            round: BoundedI32::new(0).unwrap(),
            block_id: BlockId::default(),
            timestamp: Timestamp::default(),
            validator_address: Bech32::new(
                "unused".to_owned(),
                hex!("aabbccddeeffaabbccddeeffaabbccddeeffaabb").into(),
            ),
            validator_index: 0,
            signature: Bytes::default(),
        }
    }

    #[test]
    fn from_amino() {
        let bz = hex!("21080211000000000000008019ffffffffffffffff2a0b088092b8c398feffffff01");

        let cv: CanonicalVote = prost::Message::decode_length_delimited(&bz[..]).unwrap();

        assert_eq!(
            cv,
            CanonicalVote {
                ty: 2,
                height: i64::MIN,
                round: -1,
                block_id: None,
                timestamp: Some(protos::google::protobuf::Timestamp {
                    seconds: -62135596800,
                    nanos: 0,
                }),
                chain_id: "".to_owned(),
            }
        );
    }

    #[test]
    fn test_sign_bytes() {
        let vote = Vote {
            ty: SignedMsgType::Precommit,
            height: BoundedI64::new(1008285).unwrap(),
            round: BoundedI32::new(0).unwrap(),
            block_id: BlockId {
                hash: Some(
                    "JdVemkdH1o1q9HiVKcmdnWg92+Amf0l02Odz98ITvdY="
                        .parse()
                        .unwrap(),
                ),
                parts_header: PartSetHeader {
                    total: 1,
                    hash: Some(
                        "+2yjK9nGQPc7bSMi4dDomFaLm5tAdSFjdeJ/hWxYz78="
                            .parse()
                            .unwrap(),
                    ),
                },
            },
            timestamp: "2026-04-30T11:42:22.636135891Z".parse().unwrap(),
            validator_address: "g1pdg6ugp4r8r9km4kqxfz7e7qvn9ssq3fv2yn5x".parse().unwrap(),
            validator_index: 0,
            signature: Bytes::default(),
        };

        assert_eq!(
            vote.sign_bytes("gnoland".to_owned()),
            <Bytes>::from([
                0x6c, // length
                0x08, 0x02, // precommit
                0x11, 0x9d, 0x62, 0x0f, 0x00, 0x00, 0x00, 0x00, 0x00, // height
                // no round
                // block id:
                0x22, // message
                0x48, // length?
                0x0a, // field 1, bytes
                0x20, // length
                0x25, 0xd5, 0x5e, 0x9a, 0x47, 0x47, 0xd6, 0x8d, 0x6a, 0xf4, 0x78, 0x95, 0x29, 0xc9,
                0x9d, 0x9d, 0x68, 0x3d, 0xdb, 0xe0, 0x26, 0x7f, 0x49, 0x74, 0xd8, 0xe7, 0x73, 0xf7,
                0xc2, 0x13, 0xbd, 0xd6, // hash
                0x12, //
                0x24, // part set header length
                0x0a, // field 1, bytes
                0x20, // length
                0xfb, 0x6c, 0xa3, 0x2b, 0xd9, 0xc6, 0x40, 0xf7, 0x3b, 0x6d, 0x23, 0x22, 0xe1, 0xd0,
                0xe8, 0x98, 0x56, 0x8b, 0x9b, 0x9b, 0x40, 0x75, 0x21, 0x63, 0x75, 0xe2, 0x7f, 0x85,
                0x6c, 0x58, 0xcf, 0xbf, // hash
                0x10, // field?
                0x02, // ???
                0x2a, 0x0c, 0x08, 0x9e, 0x80, 0xcd, 0xcf, 0x06, 0x10, 0xd3, 0xd3, 0xaa, 0xaf, 0x02,
                0x32, 0x07, 0x67, 0x6e, 0x6f, 0x6c, 0x61, 0x6e, 0x64
            ])
        )
    }
}
