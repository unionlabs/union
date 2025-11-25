use std::collections::HashMap;

use serde_json::Value;
use unionlabs::primitives::Bytes;
use voyager_primitives::{IbcSpec, IbcSpecId};
use voyager_rpc::{RpcError, RpcResult};
use voyager_types::RawClientId;

pub struct IbcSpecHandlers {
    pub(crate) handlers: HashMap<IbcSpecId, IbcSpecHandler>,
}

impl IbcSpecHandlers {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            handlers: HashMap::default(),
        }
    }

    pub fn register<S: IbcSpec>(&mut self) {
        self.handlers.insert(S::ID, IbcSpecHandler::new::<S>());
    }

    pub fn get(&self, ibc_spec_id: &IbcSpecId) -> RpcResult<&IbcSpecHandler> {
        self.handlers.get(ibc_spec_id).ok_or_else(|| {
            RpcError::fatal_from_message(format!("unknown IBC spec `{ibc_spec_id}`"))
        })
    }
}

/// A type-erased version of the methods on [`IbcSpec`] (essentially a vtable).
pub struct IbcSpecHandler {
    pub client_state_path: fn(RawClientId) -> anyhow::Result<Value>,
    pub consensus_state_path: fn(RawClientId, String) -> anyhow::Result<Value>,
    pub msg_update_client: fn(RawClientId, Bytes) -> anyhow::Result<Value>,
}

impl IbcSpecHandler {
    pub const fn new<T: IbcSpec>() -> Self {
        Self {
            client_state_path: |client_id| {
                Ok(
                    serde_json::to_value(T::client_state_path(serde_json::from_value(
                        client_id.into_raw(),
                    )?))
                    .unwrap(),
                )
            },
            consensus_state_path: |client_id, height| {
                Ok(serde_json::to_value(T::consensus_state_path(
                    serde_json::from_value(client_id.into_raw())?,
                    height.parse()?,
                ))
                .unwrap())
            },
            msg_update_client: |client_id, client_message| {
                Ok(serde_json::to_value(T::update_client_datagram(
                    serde_json::from_value(client_id.into_raw())?,
                    client_message,
                ))
                .unwrap())
            },
        }
    }
}
