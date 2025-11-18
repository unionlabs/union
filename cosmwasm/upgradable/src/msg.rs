use std::num::NonZero;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Upgradable {
    Upgrade {
        /// The new code id to upgrade to.
        ///
        /// Note that the contract may have already been migrated to this code id. It is up to the contract to do necessary checks to guard against this case if it is not desired.
        #[cfg_attr(feature = "schemars", schemars(with = "String"))]
        #[serde(with = "::serde_utils::string")]
        new_code_id: NonZero<u64>,
        /// The `MigrateMsg` that will be forward to the contract, within [`frissitheto::UpgradeMsg::Migrate`].
        msg: Value,
    },
}
