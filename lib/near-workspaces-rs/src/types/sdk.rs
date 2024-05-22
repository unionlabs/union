use std::convert::TryFrom;

use crate::error::{Error, ErrorKind};

use super::PublicKey;

impl TryFrom<near_sdk::PublicKey> for PublicKey {
    type Error = Error;

    fn try_from(pk: near_sdk::PublicKey) -> Result<Self, Self::Error> {
        Self::try_from_bytes(pk.as_bytes()).map_err(|e| {
            ErrorKind::DataConversion.full(
                "Could not convert sdk::PublicKey into workspaces::PublicKey",
                e,
            )
        })
    }
}
