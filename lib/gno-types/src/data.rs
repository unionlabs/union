use serde::{Deserialize, Serialize};
use unionlabs::primitives::{Bytes, encoding::Base64};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Data {
    pub txs: Option<Vec<Bytes<Base64>>>,
}
