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
    pub error: Option<()>,
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
