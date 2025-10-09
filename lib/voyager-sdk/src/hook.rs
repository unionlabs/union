use std::collections::BTreeSet;

use tracing::info;
use voyager_message::{
    VoyagerMessage,
    call::{Call, FetchUpdateHeaders, SubmitTx},
};
use voyager_primitives::{ChainId, ClientType};
use voyager_vm::Visit;

pub const NEVER_FILTER: &str = "null";

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
                let FetchUpdateHeaders {
                    client_type,
                    chain_id,
                    counterparty_chain_id,
                    client_id,
                    update_from,
                    update_to,
                } = &fetch;

                info!(
                    %client_type,
                    %chain_id,
                    %counterparty_chain_id,
                    %client_id,
                    %update_from,
                    %update_to,
                    "hooking for update (`{client_type}` on `{counterparty_chain_id}` \
                    tracking `{chain_id}`, id {client_id}, {update_from} to {update_to})",
                );

                *c = (self.mk_msg)(fetch)
            }
            _ => {}
        }
    }
}

/// A hook for a plugin that handles [`SubmitTx`] messages.
pub struct SubmitTxHook<'a, F: for<'b> Fn(&'b SubmitTx) -> Call> {
    chain_ids: BTreeSet<&'a ChainId>,
    mk_msg: F,
}

impl<'a, F: for<'b> Fn(&'b SubmitTx) -> Call> SubmitTxHook<'a, F> {
    pub fn new(chain_id: &'a ChainId, mk_msg: F) -> Self {
        Self {
            chain_ids: [chain_id].into_iter().collect(),
            mk_msg,
        }
    }

    pub fn new_many(chain_ids: impl Iterator<Item = &'a ChainId>, mk_msg: F) -> Self {
        Self {
            chain_ids: chain_ids.collect(),
            mk_msg,
        }
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
            Call::SubmitTx(submit_tx) if self.chain_ids.contains(&submit_tx.chain_id) => {
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

/// For simple filters that either take the item they're interested in or express no interest (i.e. they never just copy an item). This wraps the provided filter (which is expected to return a bool) in an expression maps that maps false to null.
pub fn simple_take_filter(inner_filter: String) -> String {
    format!(r#"if {inner_filter} then true else null end"#)
}
