use std::{fmt::Display, path::PathBuf};

use access_manager_types::{
    RoleId, Selector,
    managed::error::AccessManagedError,
    manager::{self, error::AccessManagerError},
};
use anyhow::{Context, Result, anyhow, bail};
use bip39::Mnemonic;
use clap::Parser;
use cometbft_rpc::rpc_types::{BlockResponse, GrpcAbciQueryError};
use cosmos_client::{
    BroadcastTxCommitError, TxClient,
    gas::GasFillerT,
    rpc::{Rpc, RpcT},
    wallet::{LocalSigner, WalletT},
};
use cosmwasm_std::{Addr, to_json_string};
use serde::Serialize;
use tracing::{info, instrument};
use unionlabs::{
    cosmwasm::wasm::{
        msg_execute_contract::MsgExecuteContract,
        msg_instantiate_contract2::MsgInstantiateContract2, msg_store_code::MsgStoreCode,
        msg_update_admin::MsgUpdateAdmin,
    },
    google::protobuf::any::Any,
    primitives::{Bech32, H256},
};

use crate::gas::{GasFillerArgs, any_gas_filler_from_args};

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

const INCREMENT: RoleId = RoleId::new(1);
const INCREMENT_IN_REPLY: RoleId = RoleId::new(3);
const DECREMENT: RoleId = RoleId::new(2);

const INCREMENT_GUARDIAN: RoleId = RoleId::new(4);
const DECREMENT_GUARDIAN: RoleId = RoleId::new(5);

const GRANT_ROLE: RoleId = RoleId::new(6);

const DELEGATE_SCHEDULE: RoleId = RoleId::new(7);
const DELEGATE_EXECUTE: RoleId = RoleId::new(8);

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

async fn execute(
    signer: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    contract: &Bech32<H256>,
    msgs: impl IntoIterator<Item: Serialize>,
) -> Result<()> {
    signer
        .broadcast_tx_commit(
            msgs.into_iter()
                .map(|msg| {
                    Any(MsgExecuteContract {
                        sender: signer.wallet().address().map_data(|d| d.into_bytes()),
                        contract: contract.clone(),
                        msg: serde_json::to_vec(&msg).unwrap().into(),
                        funds: vec![],
                    })
                })
                .collect::<Vec<_>>(),
            "",
            true,
        )
        .await?;

    Ok(())
}

// #[track_caller]
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
        BroadcastTxCommitError::Query(GrpcAbciQueryError { log, .. })
        | BroadcastTxCommitError::TxFailed { log, .. } => {
            if log.contains(&expected_error.to_string()) {
                Ok(())
            } else {
                Err(anyhow!(err).context(format!(
                    "failed with unexpected error: expected log to contain '{expected_error}'"
                )))
            }
        }
        _ => bail!(err),
    }
}

fn hash_operation(caller: impl Display, target: impl Display, data: impl Serialize) -> H256 {
    use sha2::Digest;

    sha2::Sha256::digest(format!(
        "{caller}/{target}/{}",
        serde_json::to_string(&data).unwrap()
    ))
    .into()
}

async fn wait_for_finalized_block_with(
    rpc: &cometbft_rpc::Client,
    f: impl Fn(BlockResponse) -> bool,
) -> Result<()> {
    loop {
        let commit = rpc.commit(None).await?;
        let block = rpc
            .block(Some(
                commit
                    .signed_header
                    .header
                    .height
                    .add(&if commit.canonical { 0 } else { -1 })
                    .inner()
                    .try_into()
                    .unwrap(),
            ))
            .await?;

        if f(block) {
            return Ok(());
        }
    }
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

    info!("bob can't call noop since there's no target function role set for this selector");
    execute_expect_error(
        &bob_client,
        &managed,
        &access_managed_example::msg::ExecuteMsg::Noop {},
        AccessManagedError::AccessManagedUnauthorized {
            caller: Addr::unchecked(bob_client.wallet().address().to_string()),
        },
    )
    .await?;

    info!("set noop target role");
    execute(
        &alice_client,
        &manager,
        [manager::msg::ExecuteMsg::SetTargetFunctionRole {
            target: Addr::unchecked(managed.to_string()),
            selectors: vec![Selector::new("noop").to_owned()],
            role_id: RoleId::PUBLIC_ROLE,
        }],
    )
    .await?;

    info!("bob can now call noop since it can be called by the public role");
    execute(
        &bob_client,
        &managed,
        [access_managed_example::msg::ExecuteMsg::Noop {}],
    )
    .await?;

    info!("set increment target role");
    info!("set decrement target role");
    info!("set increment_in_reply target role");
    info!("set decrement_in_sub_msg target role");
    info!("set INCREMENT role guardian");
    info!("set DECREMENT role guardian");
    info!("set INCREMENT_IN_REPLY role guardian");
    info!("grant INCREMENT to bob with an execution delay");

    execute(
        &alice_client,
        &manager,
        [
            &manager::msg::ExecuteMsg::SetTargetFunctionRole {
                target: Addr::unchecked(managed.to_string()),
                selectors: vec![Selector::new("increment").to_owned()],
                role_id: INCREMENT,
            },
            &manager::msg::ExecuteMsg::SetTargetFunctionRole {
                target: Addr::unchecked(managed.to_string()),
                selectors: vec![Selector::new("decrement").to_owned()],
                role_id: DECREMENT,
            },
            &manager::msg::ExecuteMsg::SetTargetFunctionRole {
                target: Addr::unchecked(managed.to_string()),
                selectors: vec![Selector::new("increment_in_reply").to_owned()],
                role_id: INCREMENT_IN_REPLY,
            },
            &manager::msg::ExecuteMsg::SetRoleGuardian {
                role_id: INCREMENT,
                guardian: INCREMENT_GUARDIAN,
            },
            &manager::msg::ExecuteMsg::SetRoleGuardian {
                role_id: DECREMENT,
                guardian: DECREMENT_GUARDIAN,
            },
            &manager::msg::ExecuteMsg::SetRoleGuardian {
                role_id: INCREMENT_IN_REPLY,
                guardian: INCREMENT_GUARDIAN,
            },
            &manager::msg::ExecuteMsg::GrantRole {
                role_id: INCREMENT,
                account: Addr::unchecked(bob_client.wallet().address().to_string()),
                execution_delay: 20,
            },
            &manager::msg::ExecuteMsg::GrantRole {
                role_id: DECREMENT,
                account: Addr::unchecked(bob_client.wallet().address().to_string()),
                execution_delay: 20,
            },
            &manager::msg::ExecuteMsg::GrantRole {
                role_id: INCREMENT_IN_REPLY,
                account: Addr::unchecked(bob_client.wallet().address().to_string()),
                execution_delay: 20,
            },
            &manager::msg::ExecuteMsg::GrantRole {
                role_id: INCREMENT_GUARDIAN,
                account: Addr::unchecked(charlie_client.wallet().address().to_string()),
                execution_delay: 20,
            },
            &manager::msg::ExecuteMsg::GrantRole {
                role_id: DECREMENT_GUARDIAN,
                account: Addr::unchecked(charlie_client.wallet().address().to_string()),
                execution_delay: 20,
            },
        ],
    )
    .await?;

    schedule_increment(&alice_client, &bob_client, &manager, &managed).await?;

    schedule_decrement_in_sub_msg(&alice_client, &bob_client, &manager, &managed).await?;

    schedule_increment_in_reply(&alice_client, &bob_client, &manager, &managed).await?;

    schedule_reentrant(&alice_client, &bob_client, &charlie_client, &manager).await?;

    execute_reentrant(
        &alice_client,
        &bob_client,
        &charlie_client,
        &manager,
        &managed,
    )
    .await?;

    Ok(())
}

#[instrument(skip_all)]
async fn schedule_increment(
    alice_client: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    bob_client: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    manager: &Bech32<H256>,
    managed: &Bech32<H256>,
) -> Result<()> {
    info!(
        "bob can't call increment since they have an execution delay, they must schedule the call"
    );
    let operation_id = hash_operation(
        bob_client.wallet().address(),
        managed,
        access_managed_example::msg::ExecuteMsg::Increment { by: 1 },
    );
    execute_expect_error(
        bob_client,
        managed,
        &access_managed_example::msg::ExecuteMsg::Increment { by: 1 },
        AccessManagerError::AccessManagerNotScheduled(operation_id),
    )
    .await?;

    info!("schedule increment call too soon");
    let data =
        to_json_string(&access_managed_example::msg::ExecuteMsg::Increment { by: 1 }).unwrap();
    execute_expect_error(
        bob_client,
        manager,
        &manager::msg::ExecuteMsg::Schedule {
            target: Addr::unchecked(managed.to_string()),
            data: data.clone(),
            when: now() + 5,
        },
        AccessManagerError::AccessManagerUnauthorizedCall {
            caller: Addr::unchecked(bob_client.wallet().address().to_string()),
            target: Addr::unchecked(managed.to_string()),
            selector: Selector::new("increment").to_owned(),
        },
    )
    .await?;

    info!("schedule increment call with correct delay");
    let when = now() + 25;
    execute(
        bob_client,
        manager,
        [manager::msg::ExecuteMsg::Schedule {
            target: Addr::unchecked(managed.to_string()),
            data: data.clone(),
            when,
        }],
    )
    .await?;

    info!("execute scheduled call too soon");
    execute_expect_error(
        bob_client,
        manager,
        &manager::msg::ExecuteMsg::Execute {
            target: Addr::unchecked(managed.to_string()),
            data: data.clone(),
        },
        AccessManagerError::AccessManagerNotReady(operation_id),
    )
    .await?;

    info!("scheduled op not ready");
    execute_expect_error(
        bob_client,
        managed,
        &access_managed_example::msg::ExecuteMsg::Increment { by: 1 },
        AccessManagerError::AccessManagerNotReady(operation_id),
    )
    .await?;

    wait_for_finalized_block_with(alice_client.rpc().client(), |block| {
        block.block.header.time.seconds.inner() as u64 > when
    })
    .await?;

    info!("cannot execute ready op through managed contract, must execute through manager");
    execute_expect_error(
        bob_client,
        managed,
        &access_managed_example::msg::ExecuteMsg::Increment { by: 1 },
        AccessManagedError::AccessManagedUnauthorized {
            caller: Addr::unchecked(managed.to_string()),
        },
    )
    .await?;

    info!("execute scheduled call once ready");
    execute(
        bob_client,
        manager,
        [manager::msg::ExecuteMsg::Execute {
            target: Addr::unchecked(managed.to_string()),
            data: data.clone(),
        }],
    )
    .await?;

    Ok(())
}

#[instrument(skip_all)]
async fn schedule_decrement_in_sub_msg(
    alice_client: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    bob_client: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    manager: &Bech32<H256>,
    managed: &Bech32<H256>,
) -> Result<()> {
    let msg = access_managed_example::msg::ExecuteMsg::Decrement {
        by: 1,
        in_sub_msg: true,
    };
    let data = to_json_string(&msg).unwrap();
    let operation_id = hash_operation(bob_client.wallet().address(), managed, &msg);

    info!(
        "bob can't call decrement since they have an execution delay, they must schedule the call"
    );
    execute_expect_error(
        bob_client,
        managed,
        &msg,
        AccessManagerError::AccessManagerNotScheduled(operation_id),
    )
    .await?;

    info!("schedule decrement call too soon");
    execute_expect_error(
        bob_client,
        manager,
        &manager::msg::ExecuteMsg::Schedule {
            target: Addr::unchecked(managed.to_string()),
            data: data.clone(),
            when: now() + 5,
        },
        AccessManagerError::AccessManagerUnauthorizedCall {
            caller: Addr::unchecked(bob_client.wallet().address().to_string()),
            target: Addr::unchecked(managed.to_string()),
            selector: Selector::new("decrement").to_owned(),
        },
    )
    .await?;

    info!("schedule decrement call with correct delay");
    let when = now() + 25;
    execute(
        bob_client,
        manager,
        [manager::msg::ExecuteMsg::Schedule {
            target: Addr::unchecked(managed.to_string()),
            data: data.clone(),
            when,
        }],
    )
    .await?;

    info!("execute scheduled call too soon");
    execute_expect_error(
        bob_client,
        manager,
        &manager::msg::ExecuteMsg::Execute {
            target: Addr::unchecked(managed.to_string()),
            data: data.clone(),
        },
        AccessManagerError::AccessManagerNotReady(operation_id),
    )
    .await?;

    info!("scheduled op not ready");
    execute_expect_error(
        bob_client,
        managed,
        &msg,
        AccessManagerError::AccessManagerNotReady(operation_id),
    )
    .await?;

    wait_for_finalized_block_with(alice_client.rpc().client(), |block| {
        block.block.header.time.seconds.inner() as u64 > when
    })
    .await?;

    info!("cannot execute ready op through managed contract, must execute through manager");
    execute_expect_error(
        bob_client,
        managed,
        &msg,
        AccessManagedError::AccessManagedUnauthorized {
            caller: Addr::unchecked(managed.to_string()),
        },
    )
    .await?;

    info!("execute scheduled call once ready");
    execute(
        bob_client,
        manager,
        [manager::msg::ExecuteMsg::Execute {
            target: Addr::unchecked(managed.to_string()),
            data: data.clone(),
        }],
    )
    .await?;

    Ok(())
}

#[instrument(skip_all)]
async fn schedule_increment_in_reply(
    alice_client: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    bob_client: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    manager: &Bech32<H256>,
    managed: &Bech32<H256>,
) -> Result<()> {
    let msg = access_managed_example::msg::ExecuteMsg::IncrementInReply { by: 1 };
    let data = to_json_string(&msg).unwrap();
    let operation_id = hash_operation(bob_client.wallet().address(), managed, &msg);

    info!(
        "bob can't call increment_in_reply since they have an execution delay, they must schedule the call"
    );
    execute_expect_error(
        bob_client,
        managed,
        &msg,
        AccessManagerError::AccessManagerNotScheduled(operation_id),
    )
    .await?;

    info!("schedule increment_in_reply call too soon");
    execute_expect_error(
        bob_client,
        manager,
        &manager::msg::ExecuteMsg::Schedule {
            target: Addr::unchecked(managed.to_string()),
            data: data.clone(),
            when: now() + 5,
        },
        AccessManagerError::AccessManagerUnauthorizedCall {
            caller: Addr::unchecked(bob_client.wallet().address().to_string()),
            target: Addr::unchecked(managed.to_string()),
            selector: Selector::new("increment_in_reply").to_owned(),
        },
    )
    .await?;

    info!("schedule increment_in_reply call with correct delay");
    let when = now() + 25;
    execute(
        bob_client,
        manager,
        [manager::msg::ExecuteMsg::Schedule {
            target: Addr::unchecked(managed.to_string()),
            data: data.clone(),
            when,
        }],
    )
    .await?;

    info!("execute scheduled call too soon");
    execute_expect_error(
        bob_client,
        manager,
        &manager::msg::ExecuteMsg::Execute {
            target: Addr::unchecked(managed.to_string()),
            data: data.clone(),
        },
        AccessManagerError::AccessManagerNotReady(operation_id),
    )
    .await?;

    info!("scheduled op not ready");
    execute_expect_error(
        bob_client,
        managed,
        &msg,
        AccessManagerError::AccessManagerNotReady(operation_id),
    )
    .await?;

    wait_for_finalized_block_with(alice_client.rpc().client(), |block| {
        block.block.header.time.seconds.inner() as u64 > when
    })
    .await?;

    info!("cannot execute ready op through managed contract, must execute through manager");
    execute_expect_error(
        bob_client,
        managed,
        &msg,
        AccessManagedError::AccessManagedUnauthorized {
            caller: Addr::unchecked(managed.to_string()),
        },
    )
    .await?;

    info!("execute scheduled call once ready");
    execute(
        bob_client,
        manager,
        [manager::msg::ExecuteMsg::Execute {
            target: Addr::unchecked(managed.to_string()),
            data: data.clone(),
        }],
    )
    .await?;

    Ok(())
}

#[instrument(skip_all)]
async fn schedule_reentrant(
    alice_client: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    bob_client: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    charlie_client: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    manager: &Bech32<H256>,
) -> Result<()> {
    execute(
        alice_client,
        manager,
        [
            &manager::msg::ExecuteMsg::SetTargetFunctionRole {
                target: Addr::unchecked(manager.to_string()),
                selectors: vec![Selector::new("grant_role").to_owned()],
                role_id: GRANT_ROLE,
            },
            &manager::msg::ExecuteMsg::GrantRole {
                account: Addr::unchecked(charlie_client.wallet().address().to_string()),
                role_id: GRANT_ROLE,
                execution_delay: 10,
            },
            &manager::msg::ExecuteMsg::GrantRole {
                account: Addr::unchecked(charlie_client.wallet().address().to_string()),
                role_id: RoleId::new(11),
                execution_delay: 20,
            },
            &manager::msg::ExecuteMsg::SetRoleAdmin {
                role_id: RoleId::new(10),
                admin: RoleId::new(11),
            },
        ],
    )
    .await?;

    let msg = manager::msg::ExecuteMsg::GrantRole {
        role_id: RoleId::new(10),
        account: Addr::unchecked(bob_client.wallet().address().to_string()),
        execution_delay: 0,
    };
    let data = to_json_string(&msg).unwrap();
    let operation_id = hash_operation(charlie_client.wallet().address(), manager, &msg);

    info!(
        "charlie can't call grant_role since they have an execution delay, they must schedule the call"
    );
    execute_expect_error(
        charlie_client,
        manager,
        &msg,
        AccessManagerError::AccessManagerNotScheduled(operation_id),
    )
    .await?;

    info!("schedule grant_role call too soon");
    execute_expect_error(
        charlie_client,
        manager,
        &manager::msg::ExecuteMsg::Schedule {
            target: Addr::unchecked(manager.to_string()),
            data: data.clone(),
            when: now() + 5,
        },
        AccessManagerError::AccessManagerUnauthorizedCall {
            caller: Addr::unchecked(charlie_client.wallet().address().to_string()),
            target: Addr::unchecked(manager.to_string()),
            selector: Selector::new("grant_role").to_owned(),
        },
    )
    .await?;

    info!("schedule grant_role call with correct delay");
    let when = now() + 25;
    execute(
        charlie_client,
        manager,
        [manager::msg::ExecuteMsg::Schedule {
            target: Addr::unchecked(manager.to_string()),
            data: data.clone(),
            when,
        }],
    )
    .await?;

    info!("execute scheduled call too soon");
    execute_expect_error(
        charlie_client,
        manager,
        &manager::msg::ExecuteMsg::Execute {
            target: Addr::unchecked(manager.to_string()),
            data: data.clone(),
        },
        AccessManagerError::AccessManagerNotReady(operation_id),
    )
    .await?;

    info!("scheduled op not ready");
    execute_expect_error(
        charlie_client,
        manager,
        &msg,
        AccessManagerError::AccessManagerNotReady(operation_id),
    )
    .await?;

    wait_for_finalized_block_with(alice_client.rpc().client(), |block| {
        block.block.header.time.seconds.inner() as u64 > when
    })
    .await?;

    info!("execute scheduled call once ready");
    execute(
        charlie_client,
        manager,
        [manager::msg::ExecuteMsg::Execute {
            target: Addr::unchecked(manager.to_string()),
            data: data.clone(),
        }],
    )
    .await?;

    Ok(())
}

#[instrument(skip_all)]
async fn execute_reentrant(
    alice_client: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    bob_client: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    charlie_client: &TxClient<impl WalletT, impl RpcT, impl GasFillerT>,
    manager: &Bech32<H256>,
    managed: &Bech32<H256>,
) -> Result<()> {
    execute(
        alice_client,
        manager,
        [
            &manager::msg::ExecuteMsg::SetTargetFunctionRole {
                target: Addr::unchecked(managed.to_string()),
                selectors: vec![Selector::new("delegate_schedule").to_owned()],
                role_id: DELEGATE_SCHEDULE,
            },
            &manager::msg::ExecuteMsg::SetTargetFunctionRole {
                target: Addr::unchecked(managed.to_string()),
                selectors: vec![Selector::new("delegate_execute").to_owned()],
                role_id: DELEGATE_EXECUTE,
            },
            &manager::msg::ExecuteMsg::GrantRole {
                account: Addr::unchecked(charlie_client.wallet().address().to_string()),
                role_id: DELEGATE_SCHEDULE,
                execution_delay: 0,
            },
            &manager::msg::ExecuteMsg::GrantRole {
                account: Addr::unchecked(charlie_client.wallet().address().to_string()),
                role_id: DELEGATE_EXECUTE,
                execution_delay: 10,
            },
            &manager::msg::ExecuteMsg::SetTargetFunctionRole {
                target: Addr::unchecked(manager.to_string()),
                selectors: vec![Selector::new("grant_role").to_owned()],
                role_id: GRANT_ROLE,
            },
            &manager::msg::ExecuteMsg::GrantRole {
                account: Addr::unchecked(managed.to_string()),
                role_id: GRANT_ROLE,
                execution_delay: 10,
            },
            &manager::msg::ExecuteMsg::GrantRole {
                account: Addr::unchecked(managed.to_string()),
                role_id: RoleId::new(11),
                execution_delay: 20,
            },
            &manager::msg::ExecuteMsg::SetRoleAdmin {
                role_id: RoleId::new(10),
                admin: RoleId::new(11),
            },
        ],
    )
    .await?;

    let grant_role_msg = manager::msg::ExecuteMsg::GrantRole {
        role_id: RoleId::new(10),
        account: Addr::unchecked(bob_client.wallet().address().to_string()),
        execution_delay: 0,
    };
    let grant_role_data = to_json_string(&grant_role_msg).unwrap();
    let _grant_role_operation_id = hash_operation(managed, manager, &grant_role_msg);

    let execute_call_msg = access_managed_example::msg::ExecuteMsg::DelegateExecute {
        target: Addr::unchecked(manager.to_string()),
        data: grant_role_data.clone(),
    };
    let execute_call_data = to_json_string(&execute_call_msg).unwrap();
    let execute_call_operation_id = hash_operation(
        charlie_client.wallet().address(),
        managed,
        &execute_call_msg,
    );

    info!("schedule grant_role call too soon");
    execute_expect_error(
        charlie_client,
        managed,
        &access_managed_example::msg::ExecuteMsg::DelegateSchedule {
            target: Addr::unchecked(manager.to_string()),
            data: grant_role_data.clone(),
            when: now() + 5,
        },
        AccessManagerError::AccessManagerUnauthorizedCall {
            caller: Addr::unchecked(managed.to_string()),
            target: Addr::unchecked(manager.to_string()),
            selector: Selector::new("grant_role").to_owned(),
        },
    )
    .await?;

    info!("schedule grant_role call with correct delay");
    let grant_role_schedule_when = now() + 25;
    execute(
        charlie_client,
        managed,
        [&access_managed_example::msg::ExecuteMsg::DelegateSchedule {
            target: Addr::unchecked(manager.to_string()),
            data: grant_role_data.clone(),
            when: grant_role_schedule_when,
        }],
    )
    .await?;

    info!("schedule execute call");
    let execute_schedule_when = now() + 25;
    execute(
        charlie_client,
        manager,
        [&manager::msg::ExecuteMsg::Schedule {
            target: Addr::unchecked(managed.to_string()),
            data: execute_call_data.clone(),
            when: execute_schedule_when,
        }],
    )
    .await?;

    info!("scheduled op not ready");
    execute_expect_error(
        charlie_client,
        managed,
        &execute_call_msg,
        AccessManagerError::AccessManagerNotReady(execute_call_operation_id),
    )
    .await?;

    wait_for_finalized_block_with(alice_client.rpc().client(), |block| {
        block.block.header.time.seconds.inner() as u64 > grant_role_schedule_when
            && block.block.header.time.seconds.inner() as u64 > execute_schedule_when
    })
    .await?;

    info!("execute scheduled call once ready");
    execute(
        charlie_client,
        manager,
        [manager::msg::ExecuteMsg::Execute {
            target: Addr::unchecked(managed.to_string()),
            data: execute_call_data.clone(),
        }],
    )
    .await?;

    Ok(())
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
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

    alice_client
        .tx(
            MsgUpdateAdmin {
                sender: alice_client.wallet().address().map_data(|d| d.into_bytes()),
                new_admin: managed_address.clone().map_data(Into::into),
                contract: managed_address.clone(),
            },
            "",
            true,
        )
        .await?;

    execute(
        alice_client,
        &managed_address,
        &[upgradable::msg::ExecuteMsg::Upgrade {
            new_code_id: code_id.get(),
            msg: serde_json::to_value(&access_managed_example::msg::MigrateMsg {}).unwrap(),
        }],
    )
    .await?;

    info!(%managed_address, %manager_address, "contracts set up");

    Ok((manager_address, managed_address))
}
