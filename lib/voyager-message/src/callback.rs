use enumorph::Enumorph;
use macros::model;
use serde::de::DeserializeOwned;
use voyager_primitives::{ChainId, IbcSpecId};
use voyager_types::RawClientId;

use crate::PluginMessage;

#[model]
#[derive(Enumorph)]
pub enum Callback {
    AggregateSubmitTxFromOrderedHeaders(AggregateSubmitTxFromOrderedHeaders),

    Plugin(PluginMessage),
}

impl Callback {
    #[allow(clippy::result_large_err)]
    pub fn as_plugin<T: DeserializeOwned>(self, plugin_name: impl AsRef<str>) -> Result<T, Self> {
        match self {
            Self::Plugin(plugin_message) => {
                plugin_message.downcast(plugin_name).map_err(Self::Plugin)
            }
            this => Err(this),
        }
    }
}

/// Required data: [`OrderedHeaders`]
///
/// Returns: [`SubmitTx`]
#[model]
pub struct AggregateSubmitTxFromOrderedHeaders {
    pub ibc_spec_id: IbcSpecId,
    pub chain_id: ChainId,
    pub client_id: RawClientId,
}
