use std::collections::VecDeque;

use jsonrpsee::core::{async_trait, RpcResult};
use queue_msg::{optimize::OptimizationResult, Op};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tracing::{instrument, warn};
use voyager_message::{
    data::Data,
    default_subcommand_handler,
    plugin::{OptimizationPassPluginServer, PluginInfo, PluginModuleServer},
    run_module_server, VoyagerMessage,
};

use crate::{call::ModuleCall, callback::ModuleCallback, data::ModuleData};

pub mod call;
pub mod callback;
pub mod data;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    run_module_server(
        Module::new,
        OptimizationPassPluginServer::into_rpc,
        default_subcommand_handler,
    )
    .await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub connection_event_filters: Vec<ConnectionEventFilter>,
    pub channel_event_filters: Vec<ChannelEventFilter>,
    pub packet_event_filters: Vec<PacketEventFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub connection_event_filters: Vec<ConnectionEventFilter>,
    pub channel_event_filters: Vec<ChannelEventFilter>,
    pub packet_event_filters: Vec<PacketEventFilter>,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEventFilter {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub chain_id: Regex,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub client_id: Regex,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub counterparty_client_id: Regex,
}

impl ConnectionEventFilter {
    fn to_jaq(&self) -> String {
        let Self {
            chain_id,
            client_id,
            counterparty_client_id,
        } = self;

        format!(
            r#"(
                ($chain_id | test("{chain_id}"))
                and ($event.client_id | test("{client_id}"))
                and ($event.counterparty_client_id | test("{counterparty_client_id}"))
            )"#
        )
    }
}

fn match_any() -> Regex {
    Regex::new(".*").unwrap()
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelEventFilter {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub chain_id: Regex,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub connection_id: Regex,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub port_id: Regex,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub counterparty_port_id: Regex,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub channel_version: Regex,
}

impl ChannelEventFilter {
    fn to_jaq(&self) -> String {
        let Self {
            chain_id,
            connection_id,
            port_id,
            counterparty_port_id,
            channel_version,
        } = self;

        format!(
            r#"(
                ($chain_id | test("{chain_id}"))
                and ($event.port_id | test("{port_id}"))
                and ($event.counterparty_port_id | test("{counterparty_port_id }"))
                and ($event.connection_id | test("{connection_id}"))
                and ($event.version | test("{channel_version}"))
            )"#
        )
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketEventFilter {
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub chain_id: Regex,

    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub source_connection_id: Regex,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub source_port_id: Regex,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub source_channel_id: Regex,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub source_channel_version: Regex,

    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub destination_port_id: Regex,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub destination_channel_id: Regex,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub destination_connection_id: Regex,
    #[serde_as(as = "DisplayFromStr")]
    #[serde(default = "match_any")]
    pub destination_channel_version: Regex,
}

impl PacketEventFilter {
    fn to_jaq(&self) -> String {
        let Self {
            chain_id,
            source_connection_id,
            source_port_id,
            source_channel_id,
            source_channel_version,
            destination_port_id,
            destination_channel_id,
            destination_connection_id,
            destination_channel_version,
        } = self;

        format!(
            r#"(
                ($chain_id | test("{chain_id}"))
                and ($event.packet.source_channel.port_id | test("{source_port_id}"))
                and ($event.packet.source_channel.channel_id | test("{source_channel_id}"))
                and ($event.packet.source_channel.version | test("{source_channel_version}"))
                and ($event.packet.source_channel.connection.connection_id | test("{source_connection_id}"))

                and ($event.packet.destination_channel.port_id | test("{destination_port_id}"))
                and ($event.packet.destination_channel.channel_id | test("{destination_channel_id}"))
                and ($event.packet.destination_channel.version | test("{destination_channel_version}"))
                and ($event.packet.destination_channel.connection.connection_id | test("{destination_connection_id}"))
            )"#
        )
    }
}

impl Module {
    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        PLUGIN_NAME.to_owned()
    }

    pub async fn new(config: Config) -> Result<Self, ModuleInitError> {
        Ok(Self {
            connection_event_filters: config.connection_event_filters,
            channel_event_filters: config.channel_event_filters,
            packet_event_filters: config.packet_event_filters,
        })
    }

    /// Construct the filter that will run on every event. If this returns true, then this plugin will receive the event in it's optimization queue and drop it.
    pub fn make_filter(&self) -> String {
        // let filter = Term::<&str>::IfThenElse(vec![], Some(Box::new(Term::Call("false", vec![]))));

        // Filter::from(&filter)

        let packet_filter = self
            .packet_event_filters
            .iter()
            .map(|x| x.to_jaq())
            .collect::<Vec<_>>()
            .join(" or ");
        let channel_filter = self
            .channel_event_filters
            .iter()
            .map(|x| x.to_jaq())
            .collect::<Vec<_>>()
            .join(" or ");
        let connection_filter = self
            .connection_event_filters
            .iter()
            .map(|x| x.to_jaq())
            .collect::<Vec<_>>()
            .join(" or ");

        format!(
            r#"
    if ."@type" == "data" then
        ."@value" as $data |

        if $data."@type" == "ibc_event" then
            $data."@value".chain_id as $chain_id |
            $data."@value".event."@type" as $event_type |
            $data."@value".event."@value" as $event |

            (if $event_type == "send_packet" then
                ({packet_filter})
            elif $event_type == "recv_packet" then
                ({packet_filter})
            elif $event_type == "write_acknowledgement" then
                ({packet_filter})

            elif $event_type == "channel_open_init" then
                ({channel_filter})
            elif $event_type == "channel_open_try" then
                ({channel_filter})
            elif $event_type == "channel_open_ack" then
                ({channel_filter})

            elif $event_type == "connection_open_init" then
                ({connection_filter})
            elif $event_type == "connection_open_try" then
                ({connection_filter})
            elif $event_type == "connection_open_ack" then
                ({connection_filter})
            else
                true
            end) | not
        else
            false
        end
    else
        false
    end
    "#
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleInitError {}

#[async_trait]
impl PluginModuleServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument]
    async fn info(&self) -> RpcResult<PluginInfo> {
        Ok(PluginInfo {
            name: self.plugin_name(),
            kind: None,
            interest_filter: Some(self.make_filter()),
        })
    }

    #[instrument]
    async fn call(
        &self,
        msg: ModuleCall,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match msg {}
    }

    #[instrument]
    fn callback(
        &self,
        cb: ModuleCallback,
        data: VecDeque<Data<ModuleData>>,
    ) -> RpcResult<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        match cb {}
    }
}

#[async_trait]
impl OptimizationPassPluginServer<ModuleData, ModuleCall, ModuleCallback> for Module {
    #[instrument]
    fn run_pass(
        &self,
        msgs: Vec<Op<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>>,
    ) -> RpcResult<OptimizationResult<VoyagerMessage<ModuleData, ModuleCall, ModuleCallback>>> {
        warn!("dropping messages");

        Ok(OptimizationResult::default())
    }
}
