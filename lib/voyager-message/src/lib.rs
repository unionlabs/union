#![feature(trait_alias)]

use std::{fmt::Debug, marker::PhantomData};

use futures::Future;
use jsonrpsee::types::error::METHOD_NOT_FOUND_CODE;
use macros::apply;
use queue_msg::{aggregation::SubsetOf, queue_msg, QueueError, QueueMessage};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
use tracing::{info, trace};
use unionlabs::{never::Never, traits::Member};

use crate::{
    call::Call, callback::Callback, context::Context, data::Data, plugin::PluginModuleServer,
};

pub mod call;
pub mod callback;
pub mod data;

pub mod context;
pub mod pass;
pub mod plugin;

pub mod rpc;

pub use reth_ipc;

pub struct VoyagerMessage<D = Value, F = Value, A = Value> {
    #[allow(clippy::type_complexity)] // it's a phantom data bro fight me
    __marker: PhantomData<fn() -> (D, F, A)>,
    __unconstructable: Never,
}

impl<D: Member, C: Member, Cb: Member> QueueMessage for VoyagerMessage<D, C, Cb> {
    type Call = Call<C>;
    type Data = Data<D>;
    type Callback = Callback<Cb>;

    type Context = Context;
}

pub const FATAL_JSONRPC_ERROR_CODE: i32 = -0xBADBEEF;

pub fn json_rpc_error_to_queue_error(value: jsonrpsee::core::client::Error) -> QueueError {
    match value {
        jsonrpsee::core::client::Error::Call(ref error)
            if error.code() == FATAL_JSONRPC_ERROR_CODE
                || error.code() == METHOD_NOT_FOUND_CODE =>
        {
            QueueError::Fatal(Box::new(value))
        }
        value => QueueError::Retry(Box::new(value)),
    }
}

macro_rules! str_newtype {
    (
        $(#[doc = $doc:literal])+
        $vis:vis struct $Struct:ident;
    ) => {
        #[derive(macros::Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        // I tested this and apparently it's not required (newtype is automatically transparent?) but
        // keeping it here for clarity
        #[serde(transparent)]
        #[debug("{}({:?})", stringify!($Struct), self.0)]
        $vis struct $Struct<'a>(::std::borrow::Cow<'a, str>);

        impl<'a> ::core::fmt::Display for $Struct<'a> {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::fmt::Display::fmt(&self.0, f)
            }
        }

        #[allow(unused)]
        impl $Struct<'static> {
            pub const fn new_static(ibc_interface: &'static str) -> Self {
                Self(::std::borrow::Cow::Borrowed(ibc_interface))
            }
        }


        #[allow(unused)]
        impl<'a> $Struct<'a> {
            pub fn new(s: impl Into<::std::borrow::Cow<'a, str>>) -> Self {
                Self(s.into())
            }

            pub fn into_static(self) -> $Struct<'static> {
                $Struct(match self.0 {
                    ::std::borrow::Cow::Borrowed(x) => ::std::borrow::Cow::Owned(x.to_owned()),
                    ::std::borrow::Cow::Owned(x) => ::std::borrow::Cow::Owned(x),
                })
            }

            pub fn as_str(&self) -> &str {
                self.0.as_ref()
            }
        }
    };
}

/// Represents the IBC interface of a chain. Since multiple chains with
/// different consensus mechanisms can have the same execution environment, this
/// value is used to describe how the IBC state is stored on-chain and how the
/// IBC stack is to be interacted with.
#[apply(str_newtype)]
pub struct IbcInterface;

/// Well-known IBC interfaces, defined as constants for reusability and to allow
/// for pattern matching.
impl IbcInterface<'static> {
    /// Native light clients in ibc-go, through the client v1 router. This
    /// entrypoint uses protobuf [`Any`] wrapping to route to the correct
    /// module, such as "/ibc.lightclients.tendermint.v1.ClientState" for native
    /// 07-tendermint clients.
    ///
    /// [`Any`]: https://protobuf.dev/programming-guides/proto3/#any
    pub const IBC_GO_V8_NATIVE: &'static str = "ibc-go-v8/native";

    /// 08-wasm light clients in ibc-go, through the client v1 router. Similar
    /// to the ibc-go-v8/native entrypoint, this module also uses [`Any`]
    /// wrapping for client routing, however, there is another level of
    /// indirection, since the `Any` routing only routes to the wasm module. All
    /// state for wasm clients is [wrapped](wasm-protos), with the internal
    /// state being opaque bytes to be interpreted by the light client.
    ///
    /// [`Any`]: https://protobuf.dev/programming-guides/proto3/#any
    /// [wasm-protos]: https://github.com/cosmos/ibc-go/blob/release/v8.4.x/proto/ibc/lightclients/wasm/v1/wasm.proto
    pub const IBC_GO_V8_08_WASM: &'static str = "ibc-go-v8/08-wasm";

    /// Solidity light clients, run via Union's IBC solidity stack. This stack
    /// is fully virtualized in the EVM, and as such can be run on any chain
    /// running the EVM as part of their execution layer (ethereum, ethereum
    /// L2s, berachain, etc).
    pub const IBC_SOLIDITY: &'static str = "ibc-solidity";

    // lots more to come - near, move, fuel - stay tuned
}

/// Newtype for client types. Clients of the same type have the same client
/// state, consensus state, and header (client update) types.
#[apply(str_newtype)]
pub struct ClientType;

/// Well-known client types, defined as constants for reusability and to allow
/// for pattern matching.
impl ClientType<'static> {
    /// A client tracking CometBLS consensus.
    pub const COMETBLS: &'static str = "cometbls";

    /// A client tracking vanilla Tendermint (CometBFT).
    pub const TENDERMINT: &'static str = "tendermint";

    /// A client tracking the Ethereum beacon chain consensus, with the mainnet
    /// configuration.
    pub const ETHEREUM_MAINNET: &'static str = "ethereum-mainnet";

    /// A client tracking the Ethereum beacon chain consensus, with the minimal
    /// configuration.
    pub const ETHEREUM_MINIMAL: &'static str = "ethereum-minimal";

    /// A client tracking the state of the Scroll zkevm L2, settling on
    /// Ethereum.
    pub const SCROLL: &'static str = "scroll";

    /// A client tracking the state of the Arbitrum optimistic L2, settling on
    /// Ethereum.
    pub const ARBITRUM: &'static str = "arbitrum";

    /// A client tracking the state of a BeaconKit chain.
    pub const BEACON_KIT: &'static str = "beacon-kit";

    // lots more to come - near, movement, linea, polygon - stay tuned
}

/// Identifier used to uniquely indentify the chain, as provided by the chain itself.
///
/// # Examples
///
/// 1 -> ethereum mainnet
/// 11155111 -> ethereum sepolia testnet
/// union-testnet-8 -> union testnet
/// stargaze-1 -> stargaze mainnet
#[apply(str_newtype)]
pub struct ChainId;

#[queue_msg]
pub struct PluginMessage<T = serde_json::Value> {
    pub plugin: String,
    pub message: T,
}

impl<T, U> SubsetOf<Data<T>> for PluginMessage<U>
where
    U: SubsetOf<T>,
{
    fn try_from_super(data: Data<T>) -> Result<Self, Data<T>> {
        match data {
            Data::Plugin(PluginMessage { plugin, message }) => match U::try_from_super(message) {
                Ok(message) => Ok(PluginMessage { plugin, message }),
                Err(message) => Err(Data::plugin(plugin, message)),
            },
            data => Err(data),
        }
    }

    fn into_super(self) -> Data<T> {
        Data::<T>::plugin(self.plugin, self.message.into_super())
    }
}

macro_rules! top_level_identifiable_enum {
    (
        $(#[$meta:meta])*
        pub enum $Enum:ident$(<$Inner:ident = serde_json::Value>)? {
            $(
                $(#[$inner_meta:meta])*
                $Variant:ident($VariantInner:ty$(,)?),
            )*
        }
    ) => {
        $(#[$meta])*
        pub enum $Enum$(<$Inner = serde_json::Value>)? {
            $(
                $(#[$inner_meta])*
                $Variant($VariantInner),
            )*
        }

        $(
            impl<$Inner> $Enum<$Inner> {
                pub fn plugin(plugin: impl Into<String>, message: impl Into<$Inner>) -> $Enum<$Inner> {
                    Self::Plugin(PluginMessage { plugin: plugin.into(), message: message.into() }).into()
                }
            }
        )*
    };
}
pub(crate) use top_level_identifiable_enum;

#[derive(clap::Subcommand)]
pub enum DefaultCmd {}

pub async fn default_subcommand_handler<T>(_: T, cmd: DefaultCmd) -> Result<(), Never> {
    match cmd {}
}

pub async fn run_module_server<
    D: Member,
    C: Member,
    Cb: Member,
    Config: DeserializeOwned,
    M: PluginModuleServer<D, C, Cb> + Clone,
    Fut: Future<Output = Result<M, impl Debug>>,
    Cmd: clap::Subcommand,
    CmdFut: Future<Output = Result<(), impl Debug>>,
>(
    new_fn: fn(Config) -> Fut,
    into_rpc_fn: fn(M) -> jsonrpsee::RpcModule<M>,
    do_cmd: fn(Config, Cmd) -> CmdFut,
) {
    #[derive(clap::Parser)]
    enum Args<Cmd: clap::Subcommand> {
        Run {
            socket: String,
            config: String,
        },
        Cmd {
            #[command(subcommand)]
            cmd: Cmd,
            #[arg(long)]
            config: String,
        },
    }

    let app = <Args<Cmd> as clap::Parser>::parse();

    match app {
        Args::Run { socket, config } => {
            let config = serde_json::from_str(&config).expect("unable to parse config");

            let module = new_fn(config)
                .await
                .expect("error instantiating client module");

            let server = reth_ipc::server::Builder::default().build(socket);

            let addr = server.endpoint();

            let mut rpcs = PluginModuleServer::into_rpc(module.clone());
            rpcs.merge(into_rpc_fn(module)).unwrap();

            trace!(methods = ?*rpcs, "registered methods");

            let server_handle = server.start(rpcs).await.unwrap();

            info!("listening on {addr}");

            tokio::spawn(server_handle.stopped()).await.unwrap();
        }
        Args::Cmd { cmd, config } => do_cmd(
            serde_json::from_str(&config).expect("unable to parse config"),
            cmd,
        )
        .await
        .unwrap(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn jaq_filter() {
        use jaq_interpret::{Ctx, FilterT, ParseCtx, RcIter, Val};
        use serde_json::json;

        let input = json!({
          "@type": "data",
          "@value": {
            "@type": "ibc_event",
            "@value": {
              "chain_id": "union-testnet-8",
              "client_type": "beacon-kit",
              "tx_hash": "0x76043797c55e9a626c370d5b2869c726804023bdb4925077641fb5d4e852f098",
              "height": {
                "revision_number": 8,
                "revision_height": 2184293
              },
              "event": {
                "@type": "acknowledge_packet",
                "@value": {
                  "packet_timeout_height": {
                    "revision_number": 0,
                    "revision_height": 0
                  },
                  "packet_timeout_timestamp": 1722981156460696801_u64,
                  "packet_sequence": 39433,
                  "packet_src_port": "wasm.union1m87a5scxnnk83wfwapxlufzm58qe2v65985exff70z95a2yr86yq7hl08h",
                  "packet_src_channel": "channel-86",
                  "packet_dst_port": "0x6f270608fb562133777af0f71f6386ffc1737c30",
                  "packet_dst_channel": "channel-3",
                  "packet_data_hex": "0x00",
                  "packet_channel_ordering": "unordered",
                  "connection_id": "connection-28"
                }
              }
            }
          }
        });

        // let filter = r#"."@type" == "fetch" and ."@value"."@type" ==
        // "update_headers""#;
        let filter = r#"if false then "a" else false end"#;

        let mut ctx = ParseCtx::new(["PLUGIN_NAME".to_owned()].into());

        ctx.insert_natives(jaq_core::core());
        ctx.insert_defs(jaq_std::std());

        // parse the filter
        let f = jaq_syn::parse(filter, |p| p.module(|p| p.term()))
            .unwrap()
            .conv(filter);

        // dbg!(&f);

        // compile the filter in the context of the given definitions
        let f = ctx.compile(f);
        assert!(ctx.errs.is_empty());

        let inputs = RcIter::new(core::iter::empty());

        // iterator over the output values
        let out = f.run((
            Ctx::new([Val::str("oogabooga".to_owned())], &inputs),
            Val::from(input),
        ));

        for out in out {
            let out = out.unwrap();
            println!("{out}");
        }
    }
}
