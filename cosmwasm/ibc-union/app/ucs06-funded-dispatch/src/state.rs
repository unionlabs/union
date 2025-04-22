use cw_storage_plus::Item;

use crate::{msg::Config, types::StoredFundedDispatchParameters};

/// Stores the configuration for the contract.
pub const CONFIG: Item<Config> = Item::new("config");

/// Stores the currently executing funded call.
pub const EXECUTING_PARAMS: Item<StoredFundedDispatchParameters> =
    Item::new("funded_dispatch_parameters");
