use tracing::info;
use voyager_primitives::{ChainId, ClientType};
use voyager_vm::Visit;

use crate::{
    call::{Call, FetchUpdateHeaders, SubmitTx},
    filter::simple_take_filter,
    VoyagerMessage,
};

/// A hook for a plugin that handles [`FetchUpdateHeaders`] messages.
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
        simple_take_filter(format!(
            r#"[.. | ."@type"? == "fetch_update_headers" and ."@value".chain_id == "{}" and ."@value".client_type == "{}"] | any"#,
            chain_id, client_type
        ))
    }
}

impl<F: for<'b> Fn(&'b FetchUpdateHeaders) -> Call> Visit<VoyagerMessage> for UpdateHook<'_, F> {
    fn visit_call(&mut self, c: &mut Call) {
        match c {
            Call::FetchUpdateHeaders(fetch)
                if fetch.chain_id == self.chain_id && fetch.client_type == self.client_type =>
            {
                info!(
                    %fetch.client_type,
                    %fetch.chain_id,
                    %fetch.counterparty_chain_id,
                    %fetch.client_id,
                    %fetch.update_from,
                    %fetch.update_to,
                    "hooking for update (`{}` on `{}` tracking `{}`, id {}, {} to {})",
                    fetch.client_type,
                    fetch.chain_id,
                    fetch.counterparty_chain_id,
                    fetch.client_id,
                    fetch.update_from,
                    fetch.update_to
                );

                *c = (self.mk_msg)(fetch)
            }
            _ => {}
        }
    }
}

/// A hook for a plugin that handles [`SubmitTx`] messages.
pub struct SubmitTxHook<'a, F: for<'b> Fn(&'b SubmitTx) -> Call> {
    chain_id: &'a ChainId,
    mk_msg: F,
}

impl<'a, F: for<'b> Fn(&'b SubmitTx) -> Call> SubmitTxHook<'a, F> {
    pub fn new(chain_id: &'a ChainId, mk_msg: F) -> Self {
        Self { chain_id, mk_msg }
    }
}

impl SubmitTxHook<'_, for<'b> fn(&'b SubmitTx) -> Call> {
    pub fn filter(chain_id: &ChainId) -> String {
        simple_take_filter(format!(
            r#"[.. | ."@type"? == "submit_tx" and ."@value".chain_id == "{}"] | any"#,
            chain_id
        ))
    }

    pub fn filter_many<'a>(chain_ids: impl IntoIterator<Item = &'a ChainId>) -> String {
        let chain_ids = chain_ids
            .into_iter()
            .map(|c| format!(r#""{c}""#))
            .collect::<Vec<_>>()
            .join(",");

        simple_take_filter(format!(
            r#"[.. | . as $o | $o."@type"? == "submit_tx" and ([{chain_ids}] | any(. == $o."@value".chain_id))] | any"#,
        ))
    }
}

impl<F: for<'b> Fn(&'b SubmitTx) -> Call> Visit<VoyagerMessage> for SubmitTxHook<'_, F> {
    fn visit_call(&mut self, c: &mut Call) {
        match c {
            Call::SubmitTx(submit_tx) if submit_tx.chain_id == self.chain_id => {
                info!(
                    "hooking for transaction submission on `{}`",
                    submit_tx.chain_id
                );

                *c = (self.mk_msg)(submit_tx)
            }
            _ => {}
        }
    }
}
