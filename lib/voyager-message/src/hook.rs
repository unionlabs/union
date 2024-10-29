use tracing::info;
use voyager_core::ChainId;
use voyager_vm::Visit;

use crate::{
    call::{Call, FetchUpdateHeaders},
    VoyagerMessage,
};

pub struct UpdateHook<'a, F: for<'b> Fn(&'b FetchUpdateHeaders) -> Call> {
    chain_id: &'a ChainId<'static>,
    mk_msg: F,
}

impl<'a, F: for<'b> Fn(&'b FetchUpdateHeaders) -> Call> UpdateHook<'a, F> {
    pub fn new(chain_id: &'a ChainId<'static>, mk_msg: F) -> Self {
        Self { chain_id, mk_msg }
    }
}

impl UpdateHook<'_, for<'b> fn(&'b FetchUpdateHeaders) -> Call> {
    pub fn filter(chain_id: &ChainId<'_>) -> String {
        format!(
            r#"[.. | ."@type"? == "fetch_update_headers" and ."@value".chain_id == "{}"] | any"#,
            chain_id
        )
    }
}

impl<F: for<'b> Fn(&'b FetchUpdateHeaders) -> Call> Visit<VoyagerMessage> for UpdateHook<'_, F> {
    fn visit_call(&mut self, c: &mut Call) {
        match c {
            Call::FetchUpdateHeaders(fetch) if &fetch.chain_id == self.chain_id => {
                info!(
                    "hooking for update (`{}`, {} to {})",
                    fetch.counterparty_chain_id, fetch.update_from, fetch.update_to
                );

                *c = (self.mk_msg)(fetch)
            }
            _ => {}
        }
    }
}
