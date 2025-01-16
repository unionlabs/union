pub mod abci;
pub mod crypto;
pub mod p2p;
pub mod types;
pub mod version;

pub mod utils {
    use unionlabs::{
        errors::{ExpectedLength, InvalidLength},
        primitives::{encoding::HexUnprefixed, FixedBytesError, H256},
    };

    pub fn maybe_empty_h256(value: &[u8]) -> Result<Option<H256<HexUnprefixed>>, InvalidLength> {
        Ok(if value.is_empty() {
            None
        } else {
            Some(
                value
                    .try_into()
                    .map_err(|err: FixedBytesError| InvalidLength {
                        expected: ExpectedLength::Either(0, 32),
                        found: err.found_len,
                    })?,
            )
        })
    }
}

pub mod serde {
    pub mod maybe_empty_h256 {
        use serde::{de, Deserialize, Deserializer, Serializer};
        use unionlabs::primitives::{encoding::HexUnprefixed, H256};

        pub fn serialize<S>(
            data: &Option<H256<HexUnprefixed>>,
            serializer: S,
        ) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            match data {
                Some(data) => serializer.collect_str(&data),
                None => serializer.collect_str(""),
            }
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<H256<HexUnprefixed>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;

            if s.is_empty() {
                Ok(None)
            } else {
                s.parse().map_err(de::Error::custom).map(Some)
            }
        }
    }
}
