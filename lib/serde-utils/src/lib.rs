extern crate alloc;

pub mod base64 {
    use alloc::string::String;
    use alloc::vec::Vec;

    use base64::prelude::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error> {
        let encoded = BASE64_STANDARD.encode(bytes);
        String::serialize(&encoded, serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        let base64 = String::deserialize(deserializer)?;
        let bytes = BASE64_STANDARD
            .decode(base64.as_bytes())
            .map_err(serde::de::Error::custom)?;

        Ok(bytes)
    }
}

pub mod inner_base64 {
    use alloc::string::String;
    use alloc::vec::Vec;

    use base64::prelude::*;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        #[allow(clippy::ptr_arg)] // required by serde
        bytes: &Vec<Vec<u8>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let encoded: Vec<String> = bytes.iter().map(|b| BASE64_STANDARD.encode(b)).collect();
        Vec::serialize(&encoded, serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<Vec<u8>>, D::Error> {
        let vec: Vec<String> = Vec::deserialize(deserializer)?;
        vec.into_iter()
            .map(|item| BASE64_STANDARD.decode(item))
            .collect::<Result<Vec<_>, _>>()
            .map_err(serde::de::Error::custom)
    }
}
