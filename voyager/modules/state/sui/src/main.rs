use std::fmt::Debug;

use ibc_union_spec::{
    path::StorePath, query::Query, Channel, ClientId, Connection, ConnectionState, IbcUnion
};
use jsonrpsee::{
    core::{async_trait, RpcResult},
    types::ErrorObject,
    Extensions,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sui_sdk::{
    rpc_types::{CheckpointId, SuiObjectDataOptions, SuiTypeTag},
    types::{
        base_types::{ObjectID, SuiAddress},
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        transaction::{Argument, CallArg, Command, ObjectArg, TransactionKind},
        Identifier,
    },
    SuiClient, SuiClientBuilder,
};
use tracing::{debug, instrument, trace};
use unionlabs::{
    encoding::{Bcs, DecodeAs as _},
    ibc::core::client::height::Height,
    primitives::Bytes,
    ErrorReporter,
};
use voyager_message::{
    into_value,
    module::{StateModuleInfo, StateModuleServer},
    primitives::{ChainId, ClientInfo, ClientType, IbcInterface, Timestamp},
    StateModule,
};
use voyager_vm::BoxDynError;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    Module::run().await
}

#[derive(clap::Subcommand)]
pub enum Cmd {
    ChainId,
    VaultAddress,
    SubmitTx,
    FetchAbi,
}

#[derive(Clone)]
pub struct Module {
    pub chain_id: ChainId,

    pub rpc_url: String,

    pub sui_client: sui_sdk::SuiClient,

    pub ibc_store: ObjectID,

    pub ibc_contract: ObjectID,
}

impl StateModule<IbcUnion> for Module {
    type Config = Config;

    async fn new(config: Self::Config, info: StateModuleInfo) -> Result<Self, BoxDynError> {
        let sui_client = SuiClientBuilder::default().build(&config.rpc_url).await?;

        let chain_id = sui_client.read_api().get_chain_identifier().await?;

        info.ensure_chain_id(chain_id.to_string())?;

        Ok(Self {
            chain_id: ChainId::new(chain_id.to_string()),
            sui_client,
            rpc_url: config.rpc_url,
            // TODO(aeryz): can't we derive or get this from `ibc_contract`?
            ibc_store: config.ibc_store,
            ibc_contract: config.ibc_contract,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub rpc_url: String,
    pub ibc_store: ObjectID,
    pub ibc_contract: ObjectID,
}

impl Module {
    #[must_use]
    pub fn make_height(&self, height: u64) -> Height {
        Height::new(height)
    }

    /// Query the latest finalized height of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    pub async fn query_latest_height(&self, _: &Extensions) -> RpcResult<Height> {
        match self
            .sui_client
            .read_api()
            .get_latest_checkpoint_sequence_number()
            .await
        {
            Ok(height) => {
                trace!(height, "latest height");

                Ok(self.make_height(height))
            }
            Err(err) => Err(ErrorObject::owned(
                -1,
                ErrorReporter(err).to_string(),
                None::<()>,
            )),
        }
    }

    /// Query the latest finalized timestamp of this chain.
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    pub async fn query_latest_timestamp(&self, e: &Extensions) -> RpcResult<Timestamp> {
        let latest_height = self.query_latest_height(e).await?;

        match self
            .sui_client
            .read_api()
            .get_checkpoint(CheckpointId::SequenceNumber(latest_height.height()))
            .await
        {
            Ok(checkpoint) => {
                let timestamp = checkpoint.timestamp_ms * 1_000_000;

                debug!(%timestamp, %latest_height, "latest timestamp");

                Ok(Timestamp::from_nanos(timestamp))
            }
            Err(err) => Err(ErrorObject::owned(
                -1,
                ErrorReporter(err).to_string(),
                None::<()>,
            )),
        }
    }
}

#[async_trait]
impl StateModuleServer<IbcUnion> for Module {
    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn query(&self, _: &Extensions, query: Query) -> RpcResult<Value> {
        match query {
            Query::PacketByHash(_packet_by_hash) => todo!(),
            Query::PacketsByBatchHash(_packets_by_batch_hash) => todo!(),
        }
    }

    #[instrument(skip_all, fields(chain_id = %self.chain_id))]
    async fn client_info(&self, _: &Extensions, _: ClientId) -> RpcResult<ClientInfo> {
        Ok(ClientInfo {
            // TODO(aeryz): make this queryable
            client_type: ClientType::new("cometbls"),
            ibc_interface: IbcInterface::new(IbcInterface::IBC_MOVE_APTOS),
            metadata: Default::default(),
        })
    }

    async fn query_ibc_state(
        &self,
        _: &Extensions,
        _: Height,
        path: StorePath,
    ) -> RpcResult<Value> {
        let query = SuiQuery::new(&self.sui_client, self.ibc_store).await;

        Ok(match path {
            StorePath::Connection(path) => {
                let res = query
                    .add_param(path.connection_id.raw())
                    .call(self.ibc_contract, "get_connection")
                    .await
                    .unwrap();

                if res.len() != 1 {
                    panic!("expected a single encoded connection end")
                }

                into_value(convert_connection(
                    ConnectionEnd::decode_as::<Bcs>(&res[0].0).unwrap(),
                ))
            }
            StorePath::Channel(path) => {
                let res = query
                    .add_param(path.channel_id.raw())
                    .call(self.ibc_contract, "get_channel")
                    .await
                    .unwrap();

                if res.len() != 1 {
                    panic!("expected a single encoded connection end")
                }

                into_value(convert_connection(
                    Channel::decode_as::<Bcs>(&res[0].0).unwrap(),
                ))
            }
            StorePath::ClientState(path) => {
                let res = query
                    .add_param(path.client_id.raw())
                    .call(self.ibc_contract, "get_client_state")
                    .await
                    .unwrap();

                if res.len() != 1 {
                    panic!("was expecting a single encoded client state");
                }

                // Doing 1.. here since the return data is bcs encoded vector<u8> which is
                // just `prefix + vector<u8>`
                let client_state_bytes: Bytes = res[0].clone().0[1..].into();

                into_value(Some(client_state_bytes))
            }
            StorePath::ConsensusState(path) => {
                let res = query
                    .add_param(path.client_id.raw())
                    .add_param(path.height)
                    .call(self.ibc_contract, "get_consensus_state")
                    .await
                    .unwrap();

                if res.len() != 1 {
                    panic!("was expecting a single encoded consensus state");
                }

                // Doing 1.. here since the return data is bcs encoded vector<u8> which is
                // just `prefix + vector<u8>`
                let consensus_state_bytes: Bytes = res[0].clone().0[1..].into();

                into_value(consensus_state_bytes)
            }
            _ => todo!(),
        })
    }
}

struct SuiQuery<'a> {
    client: &'a SuiClient,
    params: Vec<CallArg>,
}

impl<'a> SuiQuery<'a> {
    async fn new(client: &'a SuiClient, ibc_store_id: ObjectID) -> Self {
        let object_ref = client
            .read_api()
            .get_object_with_options(ibc_store_id, SuiObjectDataOptions::new())
            .await
            .unwrap()
            .object_ref_if_exists()
            .unwrap();
        Self {
            client,
            params: vec![CallArg::Object(ObjectArg::ImmOrOwnedObject(object_ref))],
        }
    }

    fn add_param<T>(mut self, param: T) -> Self
    where
        T: serde::Serialize,
    {
        self.params
            .push(CallArg::Pure(bcs::to_bytes(&param).unwrap()));
        self
    }

    async fn call(
        self,
        package: ObjectID,
        function: &str,
    ) -> Result<Vec<(Vec<u8>, SuiTypeTag)>, String> {
        let mut ptb = ProgrammableTransactionBuilder::new();
        ptb.command(Command::move_call(
            package,
            Identifier::new("ibc").unwrap(),
            Identifier::new(function).unwrap(),
            vec![],
            self.params
                .iter()
                .enumerate()
                .map(|(i, _)| Argument::Input(i as u16))
                .collect(),
        ));

        for arg in self.params {
            ptb.input(arg).unwrap();
        }

        let res = self
            .client
            .read_api()
            .dev_inspect_transaction_block(
                SuiAddress::ZERO,
                TransactionKind::ProgrammableTransaction(ptb.finish()),
                None,
                None,
                None,
            )
            .await
            .unwrap();

        match (res.results, res.error) {
            (Some(res), _) => Ok(res[0].clone().return_values),
            (_, Some(err)) => Err(err),
            _ => panic!("invalid"),
        }
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectionEnd {
    pub state: u8,
    pub client_id: u32,
    pub counterparty_client_id: u32,
    pub counterparty_connection_id: u32,
}

fn convert_connection(connection: ConnectionEnd) -> Connection {
    Connection {
        state: match connection.state {
            1 => ConnectionState::Init,
            2 => ConnectionState::TryOpen,
            3 => ConnectionState::Open,
            _ => panic!("connection state must be 1..=3"),
        },
        client_id: connection.client_id.try_into().unwrap(),
        counterparty_client_id: connection.counterparty_client_id.try_into().unwrap(),
        counterparty_connection_id: connection.counterparty_connection_id.try_into().ok(),
    }
}
