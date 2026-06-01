use serde::{Deserialize, Serialize};
use unionlabs::primitives::{Bytes, encoding::Base64};

use crate::Event;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseBase {
    /// This is an interface, with a dynamic amino object json representation:
    ///
    /// ```json
    /// {
    ///   "@type": "/abci.StringError",
    ///   "value": "unknownrequest error:no such store: oogabooga"
    /// }
    /// ```
    ///
    /// ```json
    /// {
    ///   "@type": "/std.UnknownRequestError"
    /// }
    /// ```
    ///
    /// Source: <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/bft/abci/types/types.go#L207-L210>
    ///
    /// This is quite difficult to represent in rust, so this is ignored for now.
    ///
    /// Can potentially use this as a mostly-exhaustive list? <https://github.com/gnolang/gno/blob/db1e3ec26c613fd5d119c4466b32c2c0806b2e5c/tm2/pkg/std/errors.go>
    #[serde(rename = "Error")]
    pub error: Option<Error>,
    #[serde(rename = "Data")]
    pub data: Option<Bytes<Base64>>,
    #[serde(rename = "Events")]
    pub events: Option<Vec<Event>>,
    /// nondeterministic
    #[serde(rename = "Log")]
    pub log: String,
    /// nondeterministic
    #[serde(rename = "Info")]
    pub info: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, thiserror::Error)]
#[serde(deny_unknown_fields, tag = "@type")]
pub enum Error {
    // TODO: Probably pull this into a separate enum?
    #[serde(rename = "/abci.StringError")]
    #[error("/abci.StringError: {value}")]
    AbciString { value: String },

    #[serde(untagged)]
    #[error(transparent)]
    Std(#[from] StdError),

    #[serde(untagged)]
    #[error(transparent)]
    Vm(#[from] VmError),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, thiserror::Error)]
#[serde(deny_unknown_fields, tag = "@type")]
pub enum StdError {
    #[serde(rename = "/std.UnknownRequestError")]
    #[error("/std.UnknownRequestError")]
    StdUnknownRequest {},
    #[serde(rename = "/std.NoSignaturesError")]
    #[error("/std.NoSignaturesError")]
    StdNoSignatures,
    #[serde(rename = "/std.OutOfGasError")]
    #[error("/std.OutOfGasError")]
    StdOutOfGas,
    #[serde(rename = "/std.InternalError")]
    #[error("/std.InternalError")]
    StdInternal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, thiserror::Error)]
#[serde(deny_unknown_fields, tag = "@type")]
pub enum VmError {
    #[serde(rename = "/vm.TypeCheckError")]
    #[error("/vm.TypeCheckError: [{errors:?}]")]
    VmTypeCheck { errors: Vec<String> },
    #[serde(rename = "/vm.InvalidPkgPathError")]
    #[error("/vm.InvalidPkgPathError")]
    VmInvalidPkgPath,
}
