use std::{
    collections::HashMap,
    fmt::{self, Display},
};

use itertools::Itertools;

use crate::indexer::event::{
    supported::SupportedBlockEvent,
    types::{BlockEvent, BlockEvents, BlockHeight, Chunk, Range, UniversalChainId},
};

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HubbleEvent {
    /// Schema version. Consumers must reject messages with unsupported future versions.
    pub version: u8,
    pub universal_chain_id: UniversalChainId,
    pub range: Range,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chunk: Option<Chunk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub events: Option<BlockEvents>,
}

impl HubbleEvent {
    pub fn events_by_height<'a>(&'a self) -> HashMap<BlockHeight, Vec<&'a SupportedBlockEvent>> {
        let mut by_height: HashMap<BlockHeight, Vec<&'a SupportedBlockEvent>> = HashMap::new();

        if let Some(es) = &self.events {
            for event in &es.events {
                if let BlockEvent::Supported(event) = event {
                    by_height.entry(event.height()).or_default().push(event);
                }
            }
        }

        by_height
    }
}

impl Display for HubbleEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} (events: {}, height: {})",
            self.universal_chain_id,
            self.range,
            self.events
                .as_ref()
                .map(|e| e.events.len().to_string())
                .unwrap_or("-".to_string()),
            self.events
                .as_ref()
                .map(|es| {
                    es.events
                        .iter()
                        .filter_map(|e| match e {
                            BlockEvent::Supported(supported_block_event) => {
                                Some(supported_block_event.height())
                            }
                            BlockEvent::Unsupported(_) => None,
                        })
                        .sorted()
                        .unique()
                        .join(", ")
                })
                .unwrap_or_else(|| "-".to_string()),
        )
    }
}
