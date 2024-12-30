use tracing::info;
use voyager_core::{ChainId, ClientType};
use voyager_vm::Visit;

use crate::{
    call::{Call, FetchUpdateHeaders, SubmitTx},
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
        format!(
            r#"[.. | ."@type"? == "fetch_update_headers" and ."@value".chain_id == "{}" and ."@value".client_type == "{}"] | any"#,
            chain_id, client_type
        )
    }
}

impl<F: for<'b> Fn(&'b FetchUpdateHeaders) -> Call> Visit<VoyagerMessage> for UpdateHook<'_, F> {
    fn visit_call(&mut self, c: &mut Call) {
        match c {
            Call::FetchUpdateHeaders(fetch)
                if fetch.chain_id == self.chain_id && fetch.client_type == self.client_type =>
            {
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
        // if ."@type" == "data" then
        //     ."@value" as $data |

        //     # pull all transaction data messages
        //     ($data."@type" == "identified_ibc_datagram"
        //         and $data."@value".chain_id == "{chain_id}"
        //         and $data."@value".message.ibc_spec_id == "{ibc_spec_id}")
        //     or ($data."@type" == "identified_ibc_datagram_batch"
        //         and $data."@value".chain_id == "{chain_id}"
        //         and all($data."@value".message[] | select(.ibc_spec_id == "{ibc_spec_id}")))
        // else
        //     false
        // end

        format!(
            r#"[.. | ."@type"? == "submit_tx" and ."@value".chain_id == "{}"] | any"#,
            chain_id
        )
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
