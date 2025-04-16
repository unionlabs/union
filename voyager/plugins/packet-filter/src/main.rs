use std::collections::VecDeque;

use ibc_union_spec::IbcUnion;
use jsonrpsee::{
    core::{async_trait, RpcResult},
    Extensions,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tracing::{instrument, trace};
use unionlabs::never::Never;
use voyager_message::{
    data::Data,
    filter::simple_take_filter,
    module::{PluginInfo, PluginServer},
    primitives::IbcSpec,
    DefaultCmd, Plugin, VoyagerMessage,
};
use voyager_vm::{pass::PassResult, BoxDynError, Op};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(Debug, Clone)]
pub struct Module {
    pub connection_event_filters: Vec<ConnectionEventFilter>,
    pub channel_event_filters: Vec<ChannelEventFilter>,
    pub packet_event_filters: Vec<PacketEventFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub connection_event_filters: Vec<ConnectionEventFilter>,
    pub channel_event_filters: Vec<ChannelEventFilter>,
    pub packet_event_filters: Vec<PacketEventFilter>,
}

impl Plugin for Module {
    type Call = Never;
    type Callback = Never;

    type Config = Config;
    type Cmd = DefaultCmd;

    async fn new(config: Self::Config) -> Result<Self, BoxDynError> {
        Ok(Module::new(config))
    }

    fn info(config: Self::Config) -> PluginInfo {
        let module = Module::new(config);

        PluginInfo {
            name: module.plugin_name(),
            interest_filter: module.make_filter(),
        }
    }

    async fn cmd(_config: Self::Config, cmd: Self::Cmd) {
        match cmd {}
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEventFilter {
    #[serde(default)]
    pub chain_id: FieldFilter,
    #[serde(default)]
    pub counterparty_chain_id: FieldFilter,
    #[serde(default)]
    pub client_id: FieldFilter,
    #[serde(default)]
    pub counterparty_client_id: FieldFilter,
}

impl ConnectionEventFilter {
    fn to_jaq(&self) -> String {
        let Self {
            chain_id,
            counterparty_chain_id,
            client_id,
            counterparty_client_id,
        } = self;

        format!(
            r#"(
                ($chain_id | tostring | {chain_id})
                and ($counterparty_chain_id | tostring | {counterparty_chain_id})
                and ($event.client_id | tostring | {client_id})
                and ($event.counterparty_client_id | tostring | {counterparty_client_id})
            )"#,
            chain_id = chain_id.to_jaq(),
            counterparty_chain_id = counterparty_chain_id.to_jaq(),
            client_id = client_id.to_jaq(),
            counterparty_client_id = counterparty_client_id.to_jaq(),
        )
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelEventFilter {
    #[serde(default)]
    pub chain_id: FieldFilter,
    #[serde(default)]
    pub counterparty_chain_id: FieldFilter,
    // pub connection_id: FieldFilter,
    #[serde(default)]
    pub port_id: FieldFilter,
    #[serde(default)]
    pub counterparty_port_id: FieldFilter,
    #[serde(default)]
    pub channel_version: FieldFilter,
}

impl ChannelEventFilter {
    fn to_jaq(&self) -> String {
        let Self {
            chain_id,
            counterparty_chain_id,
            // connection_id,
            port_id,
            counterparty_port_id,
            channel_version,
        } = self;

        // TODO: Fix this, currently the connection model doesn't contain the connection id
        // and ($event.connection_id | tostring | test("{connection_id}"))

        format!(
            r#"(
                ($chain_id | tostring | {chain_id})
                and ($counterparty_chain_id | tostring | {counterparty_chain_id})
                and ($event.port_id | tostring | {port_id})
                and ($event.counterparty_port_id | tostring | {counterparty_port_id })
                and ($event.version | tostring | {channel_version})
            )"#,
            chain_id = chain_id.to_jaq(),
            counterparty_chain_id = counterparty_chain_id.to_jaq(),
            // connection_id = connection_id.to_jaq(),
            port_id = port_id.to_jaq(),
            counterparty_port_id = counterparty_port_id.to_jaq(),
            channel_version = channel_version.to_jaq(),
        )
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketEventFilter {
    #[serde(default)]
    pub chain_id: FieldFilter,
    #[serde(default)]
    pub counterparty_chain_id: FieldFilter,

    #[serde(default)]
    pub source_connection_id: FieldFilter,
    // #[serde(default)]
    // pub source_port_id: FieldFilter,
    #[serde(default)]
    pub source_channel_id: FieldFilter,
    #[serde(default)]
    pub source_channel_version: FieldFilter,

    // #[serde(default)]
    // pub destination_port_id: FieldFilter,
    #[serde(default)]
    pub destination_channel_id: FieldFilter,
    #[serde(default)]
    pub destination_connection_id: FieldFilter,
    #[serde(default)]
    pub destination_channel_version: FieldFilter,
}

impl PacketEventFilter {
    fn to_jaq(&self) -> String {
        let Self {
            chain_id,
            counterparty_chain_id,
            source_connection_id,
            // source_port_id,
            source_channel_id,
            source_channel_version,
            // destination_port_id,
            destination_channel_id,
            destination_connection_id,
            destination_channel_version,
        } = self;

        // TODO: Add this back, currently the port id is not available in the event
        // and ($event.packet.source_channel.port_id | tostring | test("{source_port_id}"))
        // and ($event.packet.destination_channel.port_id | tostring | test("{destination_port_id}"))
        format!(
            r#"(
                ($chain_id | tostring | {chain_id})
                and ($counterparty_chain_id | tostring | {counterparty_chain_id})
                and ($event.packet.source_channel.channel_id | tostring | {source_channel_id})
                and ($event.packet.source_channel.version | tostring | {source_channel_version})
                and ($event.packet.source_channel.connection.connection_id | tostring | {source_connection_id})

                and ($event.packet.destination_channel.channel_id | tostring | {destination_channel_id})
                and ($event.packet.destination_channel.version | tostring | {destination_channel_version})
                and ($event.packet.destination_channel.connection.connection_id | tostring | {destination_connection_id})
            )"#,
            chain_id = chain_id.to_jaq(),
            counterparty_chain_id = counterparty_chain_id.to_jaq(),
            source_connection_id = source_connection_id.to_jaq(),
            // source_port_id = source_port_id.to_jaq(),
            source_channel_id = source_channel_id.to_jaq(),
            source_channel_version = source_channel_version.to_jaq(),
            // destination_port_id = destination_port_id.to_jaq(),
            destination_channel_id = destination_channel_id.to_jaq(),
            destination_connection_id = destination_connection_id.to_jaq(),
            destination_channel_version = destination_channel_version.to_jaq(),
        )
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldFilter {
    #[serde(rename = "not")]
    Not(
        #[serde_as(as = "DisplayFromStr")]
        // #[serde(default = "match_any")]
        Regex,
    ),
    #[serde(untagged)]
    Match(
        #[serde_as(as = "DisplayFromStr")]
        #[serde(default = "match_any")]
        Regex,
    ),
}

impl Default for FieldFilter {
    fn default() -> Self {
        Self::Match(match_any())
    }
}

fn match_any() -> Regex {
    Regex::new(".*").unwrap()
}

impl FieldFilter {
    fn to_jaq(&self) -> String {
        match self {
            FieldFilter::Not(regex) => {
                format!(r#"test("{regex}") | not"#)
            }
            FieldFilter::Match(regex) => {
                format!(r#"test("{regex}")"#)
            }
        }
    }
}

impl Module {
    fn plugin_name(&self) -> String {
        pub const PLUGIN_NAME: &str = env!("CARGO_PKG_NAME");

        PLUGIN_NAME.to_owned()
    }

    pub fn new(config: Config) -> Self {
        Self {
            connection_event_filters: config.connection_event_filters,
            channel_event_filters: config.channel_event_filters,
            packet_event_filters: config.packet_event_filters,
        }
    }

    /// Construct the filter that will run on every event. If this returns true, then this plugin will receive the event in it's optimization queue and drop it.
    /// To accomplish this, the filter expresses "inverted interest" - since the regex filters filter *in* what we want to keep, this filter must return false for all messages that match the regex filters, the regex filters and true for everything else.
    // TODO: Support IBC union as well
    pub fn make_filter(&self) -> String {
        // let filter = Term::<&str>::IfThenElse(vec![], Some(Box::new(Term::Call("false", vec![]))));

        // Filter::from(&filter)

        // if no filters are provided, then none all events for that specific IBC message type will be filtered out (i.e we express interest here). to do this, we return `false`, since in the context that this will be called in expresses whether or not the event matched one of the "filter in" regex filters.
        let packet_filter = ["false".to_owned()]
            .into_iter()
            .chain(self.packet_event_filters.iter().map(|x| x.to_jaq()))
            .collect::<Vec<_>>()
            .join(" or ");
        let channel_filter = ["false".to_owned()]
            .into_iter()
            .chain(self.channel_event_filters.iter().map(|x| x.to_jaq()))
            .collect::<Vec<_>>()
            .join(" or ");
        let connection_filter = ["false".to_owned()]
            .into_iter()
            .chain(self.connection_event_filters.iter().map(|x| x.to_jaq()))
            .collect::<Vec<_>>()
            .join(" or ");

        let filter = format!(
            r#"
if ."@type" == "data" then
    ."@value" as $data |

    if $data."@type" == "ibc_event" and $data."@value".ibc_spec_id == "{ibc_spec_id}" then
        $data."@value".chain_id as $chain_id |
        $data."@value".counterparty_chain_id as $counterparty_chain_id |
        $data."@value".event."@type" as $event_type |
        $data."@value".event."@value" as $event |

        (if $event_type == "packet_send" then
            ({packet_filter})
        elif $event_type == "packet_recv" then
            ({packet_filter})
        elif $event_type == "write_ack" then
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
        end)
        # the bool returned from the above expression denotes whether or not
        # an IBC event matched the inclusion filters - invert the result to
        # only express interest in messages that didn't match such that they
        # can be dropped in our optimization pass
        | not
    else
        # don't filter out data messages that aren't IBC events
        false
    end
else
    # don't filter out non-data messages
    false
end
    "#,
            ibc_spec_id = IbcUnion::ID
        );

        simple_take_filter(filter)
    }
}

#[async_trait]
impl PluginServer<Never, Never> for Module {
    #[instrument(skip_all)]
    async fn run_pass(
        &self,
        _: &Extensions,
        msgs: Vec<Op<VoyagerMessage>>,
    ) -> RpcResult<PassResult<VoyagerMessage>> {
        trace!("dropping {} messages", msgs.len());

        Ok(PassResult::default())
    }

    #[instrument]
    async fn call(&self, _: &Extensions, msg: Never) -> RpcResult<Op<VoyagerMessage>> {
        match msg {}
    }

    #[instrument]
    async fn callback(
        &self,
        _: &Extensions,
        cb: Never,
        _data: VecDeque<Data>,
    ) -> RpcResult<Op<VoyagerMessage>> {
        match cb {}
    }
}
