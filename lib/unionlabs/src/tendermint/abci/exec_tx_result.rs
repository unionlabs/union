use macros::model;

use crate::{
    bounded::{BoundedI64, BoundedIntError},
    tendermint::abci::event::Event,
};

#[model(proto(raw(protos::tendermint::abci::ExecTxResult), into, from))]
pub struct ExecTxResult {
    pub code: u32,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub data: Vec<u8>,
    /// nondeterministic
    pub log: String,
    /// nondeterministic
    pub info: String,
    pub gas_wanted: BoundedI64<0, { i64::MAX }>,
    pub gas_used: BoundedI64<0, { i64::MAX }>,
    /// nondeterministic
    pub events: Vec<Event>,
    pub codespace: String,
}

impl From<ExecTxResult> for protos::tendermint::abci::ExecTxResult {
    fn from(value: ExecTxResult) -> Self {
        Self {
            code: value.code,
            data: value.data,
            log: value.log,
            info: value.info,
            gas_wanted: value.gas_wanted.into(),
            gas_used: value.gas_used.into(),
            events: value.events.into_iter().map(Into::into).collect(),
            codespace: value.codespace,
        }
    }
}

impl TryFrom<protos::tendermint::abci::ExecTxResult> for ExecTxResult {
    type Error = TryFromExecTxResultError;

    fn try_from(value: protos::tendermint::abci::ExecTxResult) -> Result<Self, Self::Error> {
        Ok(Self {
            code: value.code,
            data: value.data,
            log: value.log,
            info: value.info,
            gas_wanted: value
                .gas_wanted
                .try_into()
                .map_err(TryFromExecTxResultError::GasWanted)?,
            gas_used: value
                .gas_used
                .try_into()
                .map_err(TryFromExecTxResultError::GasUsed)?,
            events: value.events.into_iter().map(Into::into).collect(),
            codespace: value.codespace,
        })
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromExecTxResultError {
    #[error("invalid gas_wanted")]
    GasWanted(#[source] BoundedIntError<i64>),
    #[error("invalid gas_used")]
    GasUsed(#[source] BoundedIntError<i64>),
}

#[cfg(test)]
mod tests {
    #[test]
    fn json() {
        let json = serde_json::from_str::<protos::tendermint::abci::ExecTxResult>(r#"{
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
