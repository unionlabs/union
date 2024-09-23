use std::str::FromStr;

use clap::{
    error::{ContextKind, ContextValue, ErrorKind},
    Args, FromArgMatches,
};
use unionlabs::{
    ibc::core::channel,
    id::{ClientId, ConnectionId, PortId},
};
use voyager_message::core::ChainId;

#[derive(Debug)]
pub struct HandshakeCmd {
    pub chain_a: ChainId<'static>,
    pub chain_b: ChainId<'static>,

    pub ty: HandshakeType,
}

impl Args for HandshakeCmd {
    fn augment_args(cmd: clap::Command) -> clap::Command {
        HandshakeRaw::augment_args(cmd)
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        HandshakeRaw::augment_args_for_update(cmd)
    }
}

impl HandshakeCmd {
    pub fn from_raw(mut raw: HandshakeRaw) -> Result<Self, clap::Error> {
        use HandshakeType::*;

        let msg = |arg: &str| {
            let mut err = clap::Error::new(ErrorKind::MissingRequiredArgument);
            err.insert(
                ContextKind::InvalidArg,
                ContextValue::Strings(vec![format!("--{}", arg.replace('_', "-"))]),
            );
            err
        };

        macro_rules! fields {
            ($Ty:ident { $($field:ident,)+ }) => {
                $Ty {
                    $(
                        $field: raw.$field.take().ok_or_else(|| msg(stringify!($field)))?,
                    )+
                }
            };
        }

        let ty = match (raw.create_clients, raw.open_connection, raw.open_channel) {
            (true, true, true) => fields!(ClientConnectionChannel {
                client_a_config,
                client_b_config,
                port_a,
                port_b,
                channel_version,
                connection_ordering,
                channel_ordering,
            }),
            (true, true, false) => fields!(ClientConnection {
                client_a_config,
                client_b_config,
                connection_ordering,
            }),
            (true, false, true) => {
                let mut err = clap::Error::new(ErrorKind::MissingRequiredArgument);
                err.insert(
                    ContextKind::Usage,
                    ContextValue::StyledStr(
                        "--open-connection is required when passing --create-clients and --open-channel".into()
                    ),
                );
                return Err(err);
            }
            (true, false, false) => fields!(Client {
                client_a_config,
                client_b_config,
            }),
            (false, true, true) => fields!(ConnectionChannel {
                client_a,
                client_b,
                port_a,
                port_b,
                channel_version,
                connection_ordering,
                channel_ordering,
            }),
            (false, true, false) => fields!(Connection {
                client_a,
                client_b,
                connection_ordering,
            }),
            (false, false, true) => fields!(Channel {
                connection_a,
                port_a,
                port_b,
                channel_version,
                channel_ordering,
            }),
            (false, false, false) => {
                let mut err = clap::Error::new(ErrorKind::MissingRequiredArgument);
                err.insert(
                    ContextKind::Usage,
                    ContextValue::StyledStr(
                        "One of --create-clients, --open-connection, or --open-channel is required"
                            .into(),
                    ),
                );
                return Err(err);
            }
        };

        raw.assert_empty()?;

        Ok(Self {
            chain_a: ChainId::new(raw.chain_a),
            chain_b: ChainId::new(raw.chain_b),
            ty,
        })
    }
}

#[derive(Debug)]
pub enum HandshakeType {
    Client {
        client_a_config: serde_json::Value,
        client_b_config: serde_json::Value,
    },
    ClientConnection {
        client_a_config: serde_json::Value,
        client_b_config: serde_json::Value,
        connection_ordering: Vec<channel::order::Order>,
    },
    ClientConnectionChannel {
        client_a_config: serde_json::Value,
        client_b_config: serde_json::Value,
        port_a: PortId,
        port_b: PortId,
        channel_version: String,
        connection_ordering: Vec<channel::order::Order>,
        channel_ordering: channel::order::Order,
    },
    ConnectionChannel {
        client_a: ClientId,
        client_b: ClientId,
        port_a: PortId,
        port_b: PortId,
        channel_version: String,
        connection_ordering: Vec<channel::order::Order>,
        channel_ordering: channel::order::Order,
    },
    Connection {
        client_a: ClientId,
        client_b: ClientId,
        connection_ordering: Vec<channel::order::Order>,
    },
    Channel {
        connection_a: ConnectionId,
        port_a: PortId,
        port_b: PortId,
        channel_version: String,
        channel_ordering: channel::order::Order,
    },
}

impl FromArgMatches for HandshakeCmd {
    fn from_arg_matches(matches: &clap::ArgMatches) -> Result<Self, clap::Error> {
        HandshakeRaw::from_arg_matches(matches).and_then(Self::from_raw)
    }

    fn update_from_arg_matches(&mut self, _matches: &clap::ArgMatches) -> Result<(), clap::Error> {
        todo!()
    }
}

#[derive(Debug, Args)]
pub struct HandshakeRaw {
    #[arg(required = true)]
    pub chain_a: String,
    #[arg(required = true)]
    pub chain_b: String,

    #[arg(long)]
    pub create_clients: bool,
    #[arg(
        long,
        value_parser(<serde_json::Value as FromStr>::from_str),
    )]
    pub client_a_config: Option<serde_json::Value>,
    #[arg(
        long,
        value_parser(<serde_json::Value as FromStr>::from_str),
    )]
    pub client_b_config: Option<serde_json::Value>,

    #[arg(long)]
    pub client_a: Option<ClientId>,
    #[arg(long)]
    pub client_b: Option<ClientId>,

    #[arg(long)]
    pub open_connection: bool,

    #[arg(long)]
    pub connection_a: Option<ConnectionId>,
    // #[arg(long, conflicts_with = "open_connection")]
    // connection_b: Option<ConnectionId>,
    #[arg(long)]
    pub open_channel: bool,
    #[arg(long)]
    pub port_a: Option<PortId>,
    #[arg(long)]
    pub port_b: Option<PortId>,
    #[arg(long)]
    pub channel_version: Option<String>,
    #[arg(
        long,
        value_parser(
            // don't ask questions you don't want the answer to
            |s: &str| serde_json::from_value::<channel::order::Order>(<serde_json::Value as From<&str>>::from(s))
        ),
    )]
    pub channel_ordering: Option<channel::order::Order>,
    #[arg(
        long,
        value_parser(
            // don't ask questions you don't want the answer to
            |s: &str| serde_json::from_value::<channel::order::Order>(<serde_json::Value as From<&str>>::from(s))
        ),
    )]
    pub connection_ordering: Option<Vec<channel::order::Order>>,
}

impl HandshakeRaw {
    pub(crate) fn assert_empty(&self) -> clap::error::Result<()> {
        macro_rules! none_or_err {
            ($($field:ident,)+) => {
                $(
                    if self.$field.is_some() {
                        let mut err = clap::Error::new(ErrorKind::UnknownArgument);
                        err.insert(
                            ContextKind::InvalidArg,
                            ContextValue::String(format!("--{}", stringify!($field).replace('_', "-"))),
                        );
                        return Err(err)
                    }
                )+
            };
        }

        none_or_err! {
            client_a_config,
            client_b_config,

            client_a,
            client_b,

            connection_a,
            port_a,
            port_b,
            channel_version,
            channel_ordering,
            connection_ordering,
        }

        Ok(())
    }
}
