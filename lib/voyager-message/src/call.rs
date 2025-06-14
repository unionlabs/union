use enumorph::Enumorph;
use macros::model;
use serde::de::DeserializeOwned;
use unionlabs::ibc::core::client::height::Height;
use voyager_primitives::{ChainId, ClientType, IbcSpecId, Timestamp};
use voyager_types::RawClientId;

use crate::{data::IbcDatagram, PluginMessage};

#[model]
#[derive(Enumorph)]
pub enum Call {
    // hooks
    FetchBlocks(FetchBlocks),
    FetchUpdateHeaders(FetchUpdateHeaders),
    SubmitTx(SubmitTx),

    // generic waiting logic
    WaitForHeight(WaitForHeight),
    WaitForTimestamp(WaitForTimestamp),

    // wait for a window relative to when this message is first handled
    WaitForHeightRelative(WaitForHeightRelative),
    // NOTE: impl if needed
    // WaitForTimestampRelative(WaitForTimestampRelative),

    // wait for counterparty trusted state
    WaitForTrustedHeight(WaitForTrustedHeight),
    WaitForTrustedTimestamp(WaitForTrustedTimestamp),

    WaitForClientUpdate(WaitForClientUpdate),

    Plugin(PluginMessage),
}

impl Call {
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

#[model]
pub struct FetchBlockRange {
    pub chain_id: ChainId,
    pub from_height: Height,
    pub to_height: Height,
}

/// Fetch blocks on a chain, starting at height `start_height`.
///
/// This represents a request for IBC events on a chain and must be
/// picked up by a plugin. If it is not handled by a plugin, this will
/// return with a fatal error.
///
/// # Implementation Note
///
/// This message is intended to act as a "seed" to an infinite stream of
/// unfolding messages. For example, if this is queued with height 10,
/// the plugin message this is replaced with should fetch all events in
/// block 10 and then wait for block 11 (which would then wait for block
/// 12, etc). Due to differing behaviours between chains, this may not
/// be the exact implementation, but the semantics of the unfold should
/// still hold.
#[model]
pub struct FetchBlocks {
    pub chain_id: ChainId,
    pub start_height: Height,
}

/// Generate a client update for a chain, tracked by a client type.
///
/// This represents a request for a client update and must be picked up
/// by a plugin. If it is not handled by a plugin, this will return with
/// a fatal error.
///
/// # Implementation Note
///
/// The returned [`Op`] ***MUST*** resolve to an [`crate::data::OrderedHeaders`] data.
/// This is the entrypoint called when a client update is requested, and
/// is intended to be called in the queue of an
/// [`crate::Callback::AggregateSubmitTxFromOrderedHeaders`] message, which will
/// be used to build the actual [`unionlabs::ibc::core::client::msg_update_client::MsgUpdateClient`]s.
#[model]
pub struct FetchUpdateHeaders {
    /// The type of client that is tracking the consensus on `self.chain_id`.
    pub client_type: ClientType,
    /// The ID of the chain that is being tracked by the `self.client_id` client on
    /// `self.counterparty_chain_id`.
    pub chain_id: ChainId,
    /// The chain that the light client tracking `self.chain_id` is on.
    pub counterparty_chain_id: ChainId,
    /// The ID of the client that is being updated.
    pub client_id: RawClientId,
    /// The currently trusted height of the client on `self.chain_id`.
    pub update_from: Height,
    /// The *minimum* height to update the client to. This is assumed to be finalized. Note that
    /// the generated update may not be to this exact height, but it *must* be >= it.
    pub update_to: Height,
}

/// Submit a batch of transactions on the specified chain.
///
/// This represents a request for transaction submission and must be picked up by a plugin. If it is
/// not handled by a plugin, this will return with a fatal error.
///
/// # Implementation Note
///
/// The returned [`Op`] ***MUST*** resolve to a [`Op::Noop`].
#[model]
pub struct SubmitTx {
    /// The chain to submit the messages on.
    pub chain_id: ChainId,
    // TODO: Ensure this is non-empty
    pub datagrams: Vec<IbcDatagram>,
}

#[model]
pub struct WaitForHeight {
    pub chain_id: ChainId,
    pub height: Height,
    pub finalized: bool,
}

#[model]
pub struct WaitForTimestamp {
    pub chain_id: ChainId,
    pub timestamp: Timestamp,
    pub finalized: bool,
}

#[model]
pub struct WaitForHeightRelative {
    pub chain_id: ChainId,
    pub height_diff: u64,
    pub finalized: bool,
}

/// Wait for the client `.client_id` on `.chain_id` to trust a height >=
/// `.height`.
#[model]
pub struct WaitForTrustedHeight {
    pub chain_id: ChainId,
    pub ibc_spec_id: IbcSpecId,
    pub client_id: RawClientId,
    pub height: Height,
    pub finalized: bool,
}

/// Wait for the client `.client_id` on `.chain_id` to trust a timestamp >=
/// `.timestamp`.
#[model]
pub struct WaitForTrustedTimestamp {
    pub chain_id: ChainId,
    pub ibc_spec_id: IbcSpecId,
    pub client_id: RawClientId,
    pub timestamp: Timestamp,
    pub finalized: bool,
}

/// Wait for the client `.client_id` on `.chain_id` to trust a height >=
/// `.height`.
#[model]
pub struct WaitForClientUpdate {
    pub chain_id: ChainId,
    pub ibc_spec_id: IbcSpecId,
    pub client_id: RawClientId,
    pub height: Height,
    // pub finalized: bool,
}
