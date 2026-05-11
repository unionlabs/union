use serde::{Deserialize, Serialize};
use unionlabs::{
    bounded::BoundedI64,
    primitives::{Bech32, Bytes, H160},
};

use crate::PublicKey;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Validator {
    pub address: Bech32<H160>,
    pub pub_key: PublicKey,
    #[serde(with = "::serde_utils::string")]
    pub voting_power: BoundedI64<0, { i64::MAX }>,
    #[serde(with = "::serde_utils::string")]
    pub proposer_priority: i64,
}

impl Validator {
    /// Bytes computes the unique encoding of a validator with a given voting power.
    /// These are the bytes that gets hashed in consensus. It excludes address
    /// as its redundant with the pubkey. This also excludes ProposerPriority
    /// which changes every round.
    ///
    /// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/types/validator.go#L88>
    pub fn bytes(&self) -> Bytes {
        #[derive(prost::Message)]
        struct AnyPubKey {
            #[prost(string, tag = "1")]
            type_url: String,
            #[prost(message, optional, tag = "2")]
            value: Option<RawPubKey>,
        }

        #[derive(prost::Message)]
        struct RawPubKey {
            #[prost(bytes, tag = "1")]
            bytes: Vec<u8>,
        }

        #[derive(prost::Message)]
        struct Raw {
            #[prost(message, optional, tag = "1")]
            pub_key: Option<AnyPubKey>,
            #[prost(sint64, tag = "2")]
            voting_power: i64,
        }

        prost::Message::encode_to_vec(&Raw {
            pub_key: Some(match &self.pub_key {
                PublicKey::Ed25519(bytes) => AnyPubKey {
                    type_url: "/tm.PubKeyEd25519".to_owned(),
                    value: Some(RawPubKey {
                        bytes: bytes.to_vec(),
                    }),
                },
                PublicKey::Secp256k1(bytes) => AnyPubKey {
                    type_url: "/tm.PubKeySecp256k1".to_owned(),
                    value: Some(RawPubKey {
                        bytes: bytes.to_vec(),
                    }),
                },
            }),
            voting_power: self.voting_power.inner(),
        })
        .into()
    }
}

#[cfg(test)]
mod tests {
    use hex_literal::hex;

    use super::*;

    #[test]
    fn validator_bytes() {
        let validator = Validator {
            address: Bech32::new("unused".to_owned(), H160::default()), // unused for bytes calculation
            pub_key: PublicKey::Ed25519(
                hex!("0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20").into(),
            ),
            voting_power: BoundedI64::new(10).unwrap(),
            proposer_priority: -1, // unused for bytes calculation
        };

        let bz = validator.bytes();

        assert_eq!(
            bz,
            <Bytes>::new(
                &hex!(
                    "0a370a112f746d2e5075624b65794564323535313912220a200102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f201014"
                )[..]
            )
        );
    }
}

// 3b0a370a112f746d2e5075624b65794564323535313912220a200102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20100a
//   0a370a112f746d2e5075624b65794564323535313912220a200102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f201014
