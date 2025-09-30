use std::{fmt::Display, path::PathBuf};

use access_manager_types::{managed::error::AccessManagedError, manager, RoleId, Selector};
use anyhow::{anyhow, bail, Context, Result};
use bip39::Mnemonic;
use clap::Parser;
use cometbft_rpc::rpc_types::GrpcAbciQueryError;
use cosmos_client::{
    gas::GasFillerT,
    rpc::{Rpc, RpcT},
    wallet::{LocalSigner, WalletT},
    BroadcastTxCommitError, TxClient,
};
use cosmwasm_std::Addr;
use serde::Serialize;
use tracing::{info, instrument};
use unionlabs::{
    cosmwasm::wasm::{
        msg_execute_contract::MsgExecuteContract,
        msg_instantiate_contract2::MsgInstantiateContract2, msg_store_code::MsgStoreCode,
    },
    google::protobuf::any::Any,
    primitives::{Bech32, H256},
};

use crate::gas::{any_gas_filler_from_args, GasFillerArgs};

pub mod gas;

#[derive(Debug, Parser)]
struct App {
    #[arg(long)]
    manager_bytecode: PathBuf,
    #[arg(long)]
    managed_bytecode: PathBuf,
    #[arg(long, short = 'r')]
    rpc_url: String,
    #[command(flatten)]
    gas_config: GasFillerArgs,
    #[arg(long, default_value = "")]
    salt: String,
}

const ALICE: &str = "wine parrot nominee girl exchange element pudding grow area twenty next junior come render shadow evidence sentence start rough debate feed all limb real";
const BOB: &str = "gun more barrel helmet velvet people alter depth bargain use isolate pear before frown already limb sweet response legal invest stand barrel stone conduct";
const CHARLIE: &str = "young soup enroll tornado mercy athlete tray resist limit spare address license cargo quantum panda useful clog autumn shoot observe input next across movie";

const NOOP_ROLE_ID: RoleId = RoleId::new(1);

fn mk_wallet(mnemonic: &str, bech32_prefix: &str) -> LocalSigner {
    LocalSigner::new(
        tiny_hderive::bip32::ExtendedPrivKey::derive(
            &mnemonic.parse::<Mnemonic>().unwrap().to_seed(""),
            // this is the default cosmossdk hd path
            "m/44'/118'/0'/0/0",
        )
        .unwrap()
        .secret()
        .into(),
        bech32_prefix.to_owned(),
    )
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();

    let app = App::parse();

    info!("salt is {}", app.salt);

    let client = cometbft_rpc::Client::new(&app.rpc_url).await?;

    let bech32_prefix = client
        .grpc_abci_query::<_, protos::cosmos::auth::v1beta1::Bech32PrefixResponse>(
            "/cosmos.auth.v1beta1.Query/Bech32Prefix",
            &protos::cosmos::auth::v1beta1::Bech32PrefixRequest {},
            None,
            false,
        )
        .await
        .context("querying bech32 prefix")?
        .into_result()?
        .unwrap()
        .bech32_prefix;

    let alice_client = TxClient::new(
        mk_wallet(ALICE, &bech32_prefix),
        Rpc::new(app.rpc_url.clone()).await?,
        any_gas_filler_from_args(app.gas_config.clone(), app.rpc_url.clone()).await?,
    );

    let bob_client = TxClient::new(
        mk_wallet(BOB, &bech32_prefix),
        Rpc::new(app.rpc_url.clone()).await?,
        any_gas_filler_from_args(app.gas_config.clone(), app.rpc_url.clone()).await?,
    );

    let charlie_client = TxClient::new(
        mk_wallet(CHARLIE, &bech32_prefix),
        Rpc::new(app.rpc_url.clone()).await?,
        any_gas_filler_from_args(app.gas_config.clone(), app.rpc_url.clone()).await?,
    );

    let (manager, managed) = setup_contracts(&alice_client, &app).await?;

    // bob can't call noop since there's no target function role set for this selector
    execute_expect_error(
        &bob_client,
        &managed,
        &access_managed_example::msg::ExecuteMsg::Noop {},
        AccessManagedError::AccessManagedUnauthorized {
            caller: Addr::unchecked(bob_client.wallet().address().to_string()),
        },
    )
    .await?;

    // set noop target role
    execute(
        &alice_client,
        &manager,
        &manager::msg::ExecuteMsg::SetTargetFunctionRole {
            target: Addr::unchecked(managed.to_string()),
            selectors: vec![Selector::new("noop").to_owned()],
            role_id: NOOP_ROLE_ID,
        },
    )
    .await?;

    // bob still can't execute since they don't have the noop role yet
    execute_expect_error(
        &bob_client,
        &managed,
        &access_managed_example::msg::ExecuteMsg::Noop {},
        AccessManagedError::AccessManagedUnauthorized {
            caller: Addr::unchecked(bob_client.wallet().address().to_string()),
        },
    )
    .await?;

    // grant bob the noop role
    execute(
        &alice_client,
        &manager,
        &manager::msg::ExecuteMsg::GrantRole {
            role_id: NOOP_ROLE_ID,
            account: Addr::unchecked(bob_client.wallet().address().to_string()),
            execution_delay: 0,
        },
    )
    .await?;

    // bob can now call noop since they have the required role
    execute(
        &bob_client,
        &managed,
        &access_managed_example::msg::ExecuteMsg::Noop {},
    )
    .await?;

    Ok(())
}

#[instrument(skip_all, fields(salt = app.salt))]
async fn setup_contracts(
    alice_client: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    app: &App,
) -> Result<(Bech32<H256>, Bech32<H256>)> {
    let salt = if app.salt.is_empty() {
        String::new()
    } else {
        format!("-{}", app.salt)
    };

    let code_id = alice_client
        .tx(
            MsgStoreCode {
                sender: alice_client.wallet().address().map_data(|d| d.into_bytes()),
                wasm_byte_code: std::fs::read(&app.manager_bytecode)?.into(),
                instantiate_permission: None,
            },
            "",
            true,
        )
        .await?
        .1
        .code_id;

    let label = format!("manager{salt}");
    let manager_address = alice_client
        .tx(
            MsgInstantiateContract2 {
                sender: alice_client.wallet().address().map_data(|d| d.into_bytes()),
                admin: alice_client.wallet().address().map_data(|d| d.into_bytes()),
                code_id,
                label: label.clone(),
                msg: serde_json::to_vec(&access_manager_types::manager::msg::InitMsg {
                    initial_admin: Addr::unchecked(alice_client.wallet().address().to_string()),
                })
                .unwrap()
                .into(),
                funds: vec![],
                salt: label.into_bytes().into(),
                fix_msg: false,
            },
            "",
            true,
        )
        .await?
        .1
        .address;

    let code_id = alice_client
        .tx(
            MsgStoreCode {
                sender: alice_client.wallet().address().map_data(|d| d.into_bytes()),
                wasm_byte_code: std::fs::read(&app.managed_bytecode)?.into(),
                instantiate_permission: None,
            },
            "",
            true,
        )
        .await?
        .1
        .code_id;

    let label = format!("managed{salt}");
    let managed_address = alice_client
        .tx(
            MsgInstantiateContract2 {
                sender: alice_client.wallet().address().map_data(|d| d.into_bytes()),
                admin: alice_client.wallet().address().map_data(|d| d.into_bytes()),
                code_id,
                label: label.clone(),
                msg: serde_json::to_vec(&access_manager_types::managed::msg::InitMsg {
                    initial_authority: Addr::unchecked(manager_address.to_string()),
                })
                .unwrap()
                .into(),
                funds: vec![],
                salt: label.into_bytes().into(),
                fix_msg: false,
            },
            "",
            true,
        )
        .await?
        .1
        .address;

    info!(%managed_address, %manager_address, "contracts set up");

    Ok((manager_address, managed_address))
}

async fn execute(
    signer: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    contract: &Bech32<H256>,
    msg: &impl Serialize,
) -> Result<()> {
    signer
        .tx(
            MsgExecuteContract {
                sender: signer.wallet().address().map_data(|d| d.into_bytes()),
                contract: contract.clone(),
                msg: serde_json::to_vec(msg).unwrap().into(),
                funds: vec![],
            },
            "",
            true,
        )
        .await?;

    Ok(())
}

async fn execute_expect_error(
    signer: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    contract: &Bech32<H256>,
    msg: &impl Serialize,
    expected_error: impl Display,
) -> Result<()> {
    let err = signer
        .simulate_tx(
            [Any(MsgExecuteContract {
                sender: signer.wallet().address().map_data(|d| d.into_bytes()),
                contract: contract.clone(),
                msg: serde_json::to_vec(msg).unwrap().into(),
                funds: vec![],
            })],
            "",
        )
        .await
        .unwrap_err();

    match &err {
        BroadcastTxCommitError::Query(GrpcAbciQueryError {
            error_code,
            codespace,
            log,
        })
        | BroadcastTxCommitError::TxFailed {
            codespace,
            error_code,
            log,
        } => {
            if log.contains(&expected_error.to_string()) {
                Ok(())
            } else {
                Err(anyhow!(err).context(
                    "failed with unexpected error: expected log to contain '{expected_error}'",
                ))
            }
        }
        _ => bail!(err),
    }
}
