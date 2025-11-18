use enumorph::Enumorph;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Enumorph)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum ExecuteMsg {
    #[serde(untagged)]
    AccessManager(access_manager_types::manager::msg::ExecuteMsg),
    #[serde(untagged)]
    Upgradable(upgradable::msg::Upgradable),
}
