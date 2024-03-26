use macros::model;

use crate::errors::UnknownEnumVariant;

#[model(proto(raw(protos::union::ics23::v1::InnerOp), into, from))]
pub struct InnerOp {
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub prefix: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub suffix: Vec<u8>,
}

impl From<protos::union::ics23::v1::InnerOp> for InnerOp {
    fn from(value: protos::union::ics23::v1::InnerOp) -> Self {
        Ok(Self {
            prefix: value.prefix,
            suffix: value.suffix,
        })
    }
}

impl From<InnerOp> for protos::union::ics23::v1::InnerOp {
    fn from(value: InnerOp) -> Self {
        Self {
            prefix: value.prefix,
            suffix: value.suffix,
        }
    }
}

// #[cfg(feature = "ethabi")]
// impl From<InnerOp> for contracts::glue::unionIcs23V1InnerOpData {
//     fn from(value: InnerOp) -> Self {
//         Self {
//             hash: value.hash.into(),
//             prefix: value.prefix.into(),
//             suffix: value.suffix.into(),
//         }
//     }
// }
