#![feature(trait_alias)]

use std::fmt::Debug;

use macros::model;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use voyager_vm::QueueMessage;

use crate::{call::Call, callback::Callback, data::Data};

pub mod call;
pub mod callback;
pub mod data;

#[derive(Debug, Clone, PartialEq)]
pub enum VoyagerMessage {}

impl QueueMessage for VoyagerMessage {
    type Call = Call;
    type Data = Data;
    type Callback = Callback;
}

/// A message specific to a plugin.
///
/// This is used in [`Call`], [`Callback`], and [`Data`] to route messages to plugins.
#[model]
pub struct PluginMessage {
    pub plugin: String,
    pub message: Value,
}

impl PluginMessage {
    pub fn new(plugin_name: impl Into<String>, message: impl Serialize) -> Self {
        Self {
            plugin: plugin_name.into(),
            message: serde_json::to_value(message).expect(
                "serialization must be infallible, this is a bug in the plugin implementation",
            ),
        }
    }

    pub fn downcast<T: DeserializeOwned>(self, plugin_name: impl AsRef<str>) -> Result<T, Self> {
        if self.plugin == plugin_name.as_ref() {
            if let Ok(t) = serde_json::from_value(self.message.clone()) {
                Ok(t)
            } else {
                Err(self)
            }
        } else {
            Err(self)
        }
    }
}
