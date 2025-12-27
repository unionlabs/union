use move_core_types::identifier::IdentStr;
use sui_sdk::{
    SuiClient,
    rpc_types::SuiTypeTag,
    types::{
        base_types::{ObjectID, SequenceNumber, SuiAddress},
        programmable_transaction_builder::ProgrammableTransactionBuilder,
        transaction::{
            Argument, CallArg, Command, ObjectArg, SharedObjectMutability, TransactionKind,
        },
    },
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    SuiSdk(#[from] sui_sdk::error::Error),
    #[error("query error ({0})")]
    Query(String),
}

/// Utility for calling a function in [dev-inspect](https://docs.sui.io/sui-api-ref#sui_devinspecttransactionblock) mode.
pub struct SuiQuery<'a> {
    client: &'a SuiClient,
    package: ObjectID,
    params: Vec<CallArg>,
}

impl<'a> SuiQuery<'a> {
    /// When the move function start with a store, use this initializer to assign the first param to that object.
    /// Eg.
    /// ```move
    /// public fun get_channel(ibc_store: &IBCStore, channel_id: u32): Channel;
    /// ```
    pub async fn new_with_store(
        sui_client: &'a SuiClient,
        package: ObjectID,
        store_object_id: ObjectID,
        store_initial_seq: SequenceNumber,
    ) -> Self {
        Self {
            client: sui_client,
            package,
            params: vec![CallArg::Object(ObjectArg::SharedObject {
                id: store_object_id,
                initial_shared_version: store_initial_seq,
                mutability: SharedObjectMutability::Immutable,
            })],
        }
    }

    /// Creates a new [`SuiQuery`] context.
    pub async fn new(sui_client: &'a SuiClient, package: ObjectID) -> Self {
        Self {
            client: sui_client,
            package,
            params: vec![],
        }
    }

    /// Adds a parameter to the function call. `param` is `bcs` encoded.
    pub fn add_param<T>(mut self, param: T) -> Self
    where
        T: serde::Serialize,
    {
        self.params.push(CallArg::Pure(
            bcs::to_bytes(&param).expect("serialization works"),
        ));
        self
    }

    /// Calls the given `package::module::function` via [dev-inspect](https://docs.sui.io/sui-api-ref#sui_devinspecttransactionblock) and returns it's raw output.
    pub async fn call(
        self,
        module: &IdentStr,
        function: &IdentStr,
    ) -> Result<Vec<(Vec<u8>, SuiTypeTag)>, Error> {
        let mut ptb = ProgrammableTransactionBuilder::new();
        ptb.command(Command::move_call(
            self.package,
            module.into(),
            function.into(),
            vec![],
            self.params
                .iter()
                .enumerate()
                .map(|(i, _)| Argument::Input(i as u16))
                .collect(),
        ));

        for arg in self.params {
            ptb.input(arg)
                .expect("adding a new input shouldn't be a problem");
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
            .await?;

        match (res.results, res.error) {
            (Some(res), _) => Ok(res[0].clone().return_values),
            (_, Some(err)) => Err(Error::Query(err)),
            _ => panic!("invalid state"),
        }
    }
}
