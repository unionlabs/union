use std::fmt::{self, Display, Formatter};

pub(crate) mod channel_open_ack_event_handler;
pub(crate) mod channel_open_confirm_event_handler;
pub(crate) mod channel_open_init_event_handler;
pub(crate) mod channel_open_try_event_handler;
pub(crate) mod connection_open_ack_event_handler;
pub(crate) mod connection_open_confirm_event_handler;
pub(crate) mod connection_open_init_event_handler;
pub(crate) mod connection_open_try_event_handler;
pub(crate) mod create_client_handler;
pub(crate) mod create_lens_client_handler;
pub(crate) mod create_proxy_account_handler;
pub(crate) mod create_wrapped_token_handler;
pub(crate) mod packet_ack_event_handler;
pub(crate) mod packet_recv_event_handler;
pub(crate) mod packet_send_event_handler;
pub(crate) mod packet_timeout_event_handler;
pub(crate) mod token_bucket_update_handler;
pub(crate) mod types;
pub(crate) mod update_client_handler;
pub(crate) mod wallet_mutation_entry_event_handler;
pub(crate) mod write_ack_event_handler;

/// wrapper required until we've migrated to use universal-chain-ids
pub struct EventContext<'a, C, E> {
    pub context: &'a C,
    pub event: &'a E,
}

impl<'a, C, E> Display for EventContext<'a, C, E>
where
    C: Display,
    E: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.context, self.event)
    }
}

impl<'a, C, E> std::fmt::Debug for EventContext<'a, C, E>
where
    C: std::fmt::Debug,
    E: std::fmt::Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("InternalChainIdContext")
            .field("context", &self.context)
            .field("event", &self.event)
            .finish()
    }
}
