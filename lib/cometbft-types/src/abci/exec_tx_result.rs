use serde::{Deserialize, Serialize};
use unionlabs::{
    bounded::BoundedI64,
    primitives::{encoding::Base64, Bytes},
};

use crate::{abci::event::Event, code::Code};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecTxResult {
    pub code: Code,
    pub data: Option<Bytes<Base64>>,
    /// nondeterministic
    pub log: String,
    /// nondeterministic
    pub info: String,
    #[serde(with = "::serde_utils::string")]
    pub gas_wanted: BoundedI64<0, { i64::MAX }>,
    #[serde(with = "::serde_utils::string")]
    pub gas_used: BoundedI64<0, { i64::MAX }>,
    /// nondeterministic
    pub events: Vec<Event>,
    pub codespace: String,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::bounded::BoundedIntError;

    use crate::abci::exec_tx_result::ExecTxResult;

    impl TryFrom<protos::cometbft::abci::v1::ExecTxResult> for ExecTxResult {
        type Error = Error;

        fn try_from(value: protos::cometbft::abci::v1::ExecTxResult) -> Result<Self, Self::Error> {
            Ok(Self {
                code: value.code.into(),
                data: Some(value.data.into()),
                log: value.log,
                info: value.info,
                gas_wanted: value.gas_wanted.try_into().map_err(Error::GasWanted)?,
                gas_used: value.gas_used.try_into().map_err(Error::GasUsed)?,
                events: value.events.into_iter().map(Into::into).collect(),
                codespace: value.codespace,
            })
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error("invalid gas_wanted")]
        GasWanted(#[source] BoundedIntError<i64>),
        #[error("invalid gas_used")]
        GasUsed(#[source] BoundedIntError<i64>),
    }

    impl From<ExecTxResult> for protos::cometbft::abci::v1::ExecTxResult {
        fn from(value: ExecTxResult) -> Self {
            Self {
                code: value.code.into(),
                data: value.data.unwrap_or_default().into(),
                log: value.log,
                info: value.info,
                gas_wanted: value.gas_wanted.into(),
                gas_used: value.gas_used.into(),
                events: value.events.into_iter().map(Into::into).collect(),
                codespace: value.codespace,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn json() {
        let json = serde_json::from_str::<protos::cometbft::abci::v1::ExecTxResult>(r#"{
            "code": 5,
            "data": null,
            "log": "failed to execute message; message index: 0: spendable balance 2173muno is smaller than 2400muno: insufficient funds",
            "info": "",
            "gas_wanted": "249046",
            "gas_used": "133330",
            "events": [],
            "codespace": "sdk"
        }"#).unwrap();

        dbg!(json);
    }
}
