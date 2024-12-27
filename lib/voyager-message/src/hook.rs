use tracing::info;
use voyager_core::{ChainId, ClientType};
use voyager_vm::Visit;

use crate::{
    call::{Call, FetchUpdateHeaders},
    VoyagerMessage,
};

pub struct UpdateHook<'a, F: for<'b> Fn(&'b FetchUpdateHeaders) -> Call> {
    chain_id: &'a ChainId,
    client_type: &'a ClientType,
    mk_msg: F,
}

impl<'a, F: for<'b> Fn(&'b FetchUpdateHeaders) -> Call> UpdateHook<'a, F> {
    pub fn new(chain_id: &'a ChainId, client_type: &'a ClientType, mk_msg: F) -> Self {
        Self {
            chain_id,
            client_type,
            mk_msg,
        }
    }
}

impl UpdateHook<'_, for<'b> fn(&'b FetchUpdateHeaders) -> Call> {
    pub fn filter(chain_id: &ChainId, client_type: &ClientType) -> String {
        format!(
            r#"[.. | ."@type"? == "fetch_update_headers" and ."@value".chain_id == "{}" and ."@value".client_type == "{}"] | any"#,
            chain_id, client_type
        )
    }
}

impl<F: for<'b> Fn(&'b FetchUpdateHeaders) -> Call> Visit<VoyagerMessage> for UpdateHook<'_, F> {
    fn visit_call(&mut self, c: &mut Call) {
        match c {
            Call::FetchUpdateHeaders(fetch) if fetch.chain_id == self.chain_id => {
                info!(
                    "hooking for update (`{}` on `{}`, {} to {})",
                    fetch.client_type,
                    fetch.counterparty_chain_id,
                    fetch.update_from,
                    fetch.update_to
                );

                *c = (self.mk_msg)(fetch)
            }
            _ => {}
        }
    }
}
