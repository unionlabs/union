use cosmwasm_std::{
    BankMsg, Event, Order, OwnedDeps,
    testing::{MockApi, MockQuerier, MockStorage, message_info, mock_dependencies, mock_env},
    to_json_vec,
};
use frissitheto::UpgradeMsg;
use ibc_union_spec::ChannelId;
use ucs03_zkgmable::{OnIntentZkgm, OnZkgm};
use unionlabs_primitives::{Bytes, U256};

use super::*;

const ADMIN: &str = "admin";
const ZKGM: &str = "zkgm";

const INITIAL_ADMIN_PATH: U256 = U256::ZERO;
const INITIAL_ADMIN_CHANNEL_ID: ChannelId = ChannelId!(1);
const INITIAL_ADMIN_SENDER: Bytes = <Bytes>::new_static(b"sender");

fn setup_local() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();
    let admin = Addr::unchecked(ADMIN);

    let res = init(
        deps.as_mut(),
        InitMsg::Local {
            admin: admin.clone(),
        },
    );

    assert_eq!(res, Response::new());

    // only local admin is written to storage
    assert_eq!(
        deps.storage
            .iter::<Admins>(Order::Ascending)
            .map(|r| r.unwrap().0)
            .collect::<Vec<_>>(),
        &[Admin::Local(LocalAdmin {
            address: admin.to_string(),
        })]
    );

    // zkgm does not exist after a local admin init
    assert!(deps.storage.maybe_read_item::<Zkgm>().unwrap().is_none());

    deps
}

fn setup_zkgm() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies();

    let zkgm = Addr::unchecked(ZKGM);

    let res = init(
        deps.as_mut(),
        InitMsg::Zkgm {
            zkgm: zkgm.clone(),
            path: INITIAL_ADMIN_PATH,
            channel_id: INITIAL_ADMIN_CHANNEL_ID,
            sender: INITIAL_ADMIN_SENDER,
        },
    );

    assert_eq!(res, Response::new());

    // only remote admin is written to storage
    assert_eq!(
        deps.storage
            .iter::<Admins>(Order::Ascending)
            .map(|r| r.unwrap().0)
            .collect::<Vec<_>>(),
        &[Admin::Remote(RemoteAdmin {
            path: U256::ZERO,
            channel_id: ChannelId!(1),
            address: b"sender".into(),
        })]
    );

    // zkgm address is set after remote admin init
    assert_eq!(deps.storage.read_item::<Zkgm>().unwrap(), zkgm);

    deps
}

#[test]
fn init_local() {
    let _ = setup_local();
}

#[test]
fn init_zkgm() {
    let _ = setup_zkgm();
}

#[test]
fn reentrant_allowed() {
    let mut deps = mock_dependencies();
    let env = mock_env();
    let info = message_info(&env.contract.address, &[]);

    let res = execute(
        deps.as_mut(),
        env,
        info.clone(),
        ExecuteMsg::AddAdmin(Admin::Local(LocalAdmin {
            address: "new-admin".to_owned(),
        })),
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new().add_event(
            Event::new("add_admin")
                .add_attribute("new_admin", "local:new-admin")
                .add_attribute("admin", "self")
        )
    );
}

#[test]
fn only_admin() {
    let mut deps = setup_zkgm();

    let non_admin = Addr::unchecked("non-admin");
    let info = message_info(&non_admin, &[]);

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::Dispatch(vec![]),
        )
        .unwrap_err(),
        ContractError::OnlyAdmin {
            sender: Admin::Local(LocalAdmin {
                address: non_admin.to_string()
            })
        }
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::AddAdmin(Admin::Local(LocalAdmin {
                address: "new-admin".to_owned()
            })),
        )
        .unwrap_err(),
        ContractError::OnlyAdmin {
            sender: Admin::Local(LocalAdmin {
                address: non_admin.to_string()
            })
        }
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::RemoveAdmin(Admin::Local(LocalAdmin {
                address: "new-admin".to_owned()
            })),
        )
        .unwrap_err(),
        ContractError::OnlyAdmin {
            sender: Admin::Local(LocalAdmin {
                address: non_admin.to_string()
            })
        }
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::SetZkgm(Addr::unchecked(ZKGM)),
        )
        .unwrap_err(),
        ContractError::OnlyAdmin {
            sender: Admin::Local(LocalAdmin {
                address: non_admin.to_string()
            })
        }
    );
}

#[test]
fn local_admin() {
    let mut deps = setup_local();

    let admin = Addr::unchecked(ADMIN);
    let info = message_info(&admin, &[]);

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::Dispatch(vec![]),
        )
        .unwrap(),
        Response::new().add_event(Event::new("dispatch").add_attribute("admin", "local:admin"))
    );

    // initial admin can add new admin
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::AddAdmin(Admin::Local(LocalAdmin {
                address: "new-admin".to_owned()
            })),
        )
        .unwrap(),
        Response::new().add_event(
            Event::new("add_admin")
                .add_attribute("new_admin", "local:new-admin")
                .add_attribute("admin", "local:admin")
        )
    );

    // both admins now exist in storage
    assert_eq!(
        deps.storage
            .iter::<Admins>(Order::Ascending)
            .map(|r| r.unwrap().0)
            .collect::<Vec<_>>(),
        &[
            Admin::Local(LocalAdmin {
                address: admin.to_string(),
            }),
            Admin::Local(LocalAdmin {
                address: "new-admin".to_string(),
            })
        ]
    );

    let remote_admin = RemoteAdmin {
        address: <Bytes>::new_static(&[0x00]),
        channel_id: ChannelId!(1),
        path: U256::ZERO,
    };

    // new admin can add another new admin
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::AddAdmin(Admin::Remote(remote_admin.clone())),
        )
        .unwrap(),
        Response::new().add_event(
            Event::new("add_admin")
                .add_attribute("new_admin", "remote:0/1/0x00")
                .add_attribute("admin", "local:admin")
        )
    );

    // all 3 admins now exist in storage
    assert_eq!(
        deps.storage
            .iter::<Admins>(Order::Ascending)
            .map(|r| r.unwrap().0)
            .collect::<Vec<_>>(),
        &[
            Admin::Local(LocalAdmin {
                address: admin.to_string(),
            }),
            Admin::Local(LocalAdmin {
                address: "new-admin".to_string(),
            }),
            Admin::Remote(remote_admin.clone())
        ]
    );

    // new admin can remove old admin
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("new-admin"), &[]),
            ExecuteMsg::RemoveAdmin(Admin::Local(LocalAdmin {
                address: "admin".to_owned()
            })),
        )
        .unwrap(),
        Response::new().add_event(
            Event::new("remove_admin")
                .add_attribute("removed_admin", "local:admin")
                .add_attribute("admin", "local:new-admin")
        )
    );

    // the original admin has been removed from storage
    assert_eq!(
        deps.storage
            .iter::<Admins>(Order::Ascending)
            .map(|r| r.unwrap().0)
            .collect::<Vec<_>>(),
        &[
            Admin::Local(LocalAdmin {
                address: "new-admin".to_string(),
            }),
            Admin::Remote(remote_admin.clone())
        ]
    );

    // removing it again doesn't emit the event
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("new-admin"), &[]),
            ExecuteMsg::RemoveAdmin(Admin::Local(LocalAdmin {
                address: "admin".to_owned()
            })),
        )
        .unwrap(),
        Response::new()
    );

    // storage doesn't change after the noop remove admin call
    assert_eq!(
        deps.storage
            .iter::<Admins>(Order::Ascending)
            .map(|r| r.unwrap().0)
            .collect::<Vec<_>>(),
        &[
            Admin::Local(LocalAdmin {
                address: "new-admin".to_string(),
            }),
            Admin::Remote(remote_admin.clone())
        ]
    );

    // an admin can remove itself as an admin
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked("new-admin"), &[]),
            ExecuteMsg::RemoveAdmin(Admin::Local(LocalAdmin {
                address: "new-admin".to_owned()
            })),
        )
        .unwrap(),
        Response::new().add_event(
            Event::new("remove_admin")
                .add_attribute("removed_admin", "local:new-admin")
                .add_attribute("admin", "local:new-admin")
        )
    );

    // only the remote admin exists in storage now
    assert_eq!(
        deps.storage
            .iter::<Admins>(Order::Ascending)
            .map(|r| r.unwrap().0)
            .collect::<Vec<_>>(),
        &[Admin::Remote(remote_admin.clone())]
    );
}

#[test]
fn require_one_admin() {
    let mut deps = setup_local();

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN), &[]),
            ExecuteMsg::RemoveAdmin(Admin::Local(LocalAdmin {
                address: ADMIN.to_owned()
            }))
        )
        .unwrap_err(),
        ContractError::OneAdminRequired
    );
}

#[test]
fn only_zkgm() {
    let mut deps = setup_zkgm();

    let relayer = Addr::unchecked("relayer");
    let info = message_info(&relayer, &[]);

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::Zkgmable(Zkgmable::OnZkgm(OnZkgm {
                caller: relayer.clone(),
                path: U256::ZERO,
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(1),
                sender: b"uhoh".into(),
                message: b"".into(),
                relayer: relayer.clone(),
                relayer_msg: b"".into()
            })),
        )
        .unwrap_err(),
        ContractError::OnlyZkgm {
            sender: relayer.clone()
        }
    );

    // only remote admins can call on_zkgm
    let zkgm = Addr::unchecked(ZKGM);
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&zkgm, &[]),
            ExecuteMsg::Zkgmable(Zkgmable::OnZkgm(OnZkgm {
                caller: relayer.clone(),
                path: U256::ZERO,
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(1),
                sender: b"uhoh".into(),
                message: b"".into(),
                relayer: relayer.clone(),
                relayer_msg: b"".into()
            })),
        )
        .unwrap_err(),
        ContractError::OnlyAdmin {
            sender: Admin::Remote(RemoteAdmin {
                address: b"uhoh".into(),
                channel_id: ChannelId!(1),
                path: U256::ZERO
            })
        }
    );

    // intents are not supported for cw-account
    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            ExecuteMsg::Zkgmable(Zkgmable::OnIntentZkgm(OnIntentZkgm {
                caller: relayer.clone(),
                path: U256::ZERO,
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(1),
                sender: b"uhoh".into(),
                message: b"".into(),
                market_maker: relayer.clone(),
                market_maker_msg: b"".into()
            })),
        )
        .unwrap_err(),
        ContractError::IntentsNotSupported
    );
}

#[test]
fn zkgmable() {
    let mut deps = setup_zkgm();
    let env = mock_env();
    let info = message_info(&Addr::unchecked(ZKGM), &[]);

    let relayer = Addr::unchecked("relayer");

    let message = <CosmosMsg>::Bank(BankMsg::Send {
        to_address: "to_address".to_owned(),
        amount: vec![],
    });

    let res = execute(
        deps.as_mut(),
        env,
        info.clone(),
        ExecuteMsg::Zkgmable(Zkgmable::OnZkgm(OnZkgm {
            caller: relayer.clone(),
            path: INITIAL_ADMIN_PATH,
            source_channel_id: ChannelId!(1),
            destination_channel_id: INITIAL_ADMIN_CHANNEL_ID,
            sender: INITIAL_ADMIN_SENDER,
            message: to_json_vec(&[&message]).unwrap().into(),
            relayer: relayer.clone(),
            relayer_msg: b"".into(),
        })),
    )
    .unwrap();

    assert_eq!(
        res,
        Response::new().add_message(message).add_event(
            Event::new("remote_execute")
                .add_attribute("sender", INITIAL_ADMIN_SENDER.to_string())
                .add_attribute("channel_id", INITIAL_ADMIN_CHANNEL_ID.to_string())
                .add_attribute("path", INITIAL_ADMIN_PATH.to_string())
        )
    );
}

#[test]
fn zkgmable_invalid_message() {
    let mut deps = setup_zkgm();
    let env = mock_env();
    let info = message_info(&Addr::unchecked(ZKGM), &[]);

    let relayer = Addr::unchecked("relayer");

    let res = execute(
        deps.as_mut(),
        env,
        info.clone(),
        ExecuteMsg::Zkgmable(Zkgmable::OnZkgm(OnZkgm {
            caller: relayer.clone(),
            path: INITIAL_ADMIN_PATH,
            source_channel_id: ChannelId!(1),
            destination_channel_id: INITIAL_ADMIN_CHANNEL_ID,
            sender: INITIAL_ADMIN_SENDER,
            message: b"".into(),
            relayer: relayer.clone(),
            relayer_msg: b"".into(),
        })),
    )
    .unwrap_err();

    assert_eq!(
        res,
        ContractError::Std(StdError::parse_err(
            "alloc::vec::Vec<cosmwasm_std::results::cosmos_msg::CosmosMsg>",
            "EOF while parsing a JSON value."
        ))
    );
}

#[test]
fn set_zkgm() {
    let mut deps = setup_local();

    let zkgm = Addr::unchecked(ZKGM);

    assert!(deps.storage.maybe_read_item::<Zkgm>().unwrap().is_none());

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN), &[]),
            ExecuteMsg::SetZkgm(zkgm.clone())
        )
        .unwrap(),
        Response::new().add_event(
            Event::new("set_zkgm")
                .add_attribute("zkgm", &zkgm)
                .add_attribute("admin", "local:admin")
        )
    );

    assert_eq!(deps.storage.read_item::<Zkgm>().unwrap(), zkgm);

    let new_zkgm = Addr::unchecked("new_zkgm");

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ADMIN), &[]),
            ExecuteMsg::SetZkgm(new_zkgm.clone())
        )
        .unwrap(),
        Response::new().add_event(
            Event::new("set_zkgm")
                .add_attribute("zkgm", &new_zkgm)
                .add_attribute("admin", "local:admin")
        )
    );

    assert_eq!(deps.storage.read_item::<Zkgm>().unwrap(), new_zkgm);
}

#[test]
fn init_and_instantiate_same() {
    let mut init_deps = mock_dependencies();
    let mut instantiate_deps = mock_dependencies();

    let env = mock_env();
    let info = message_info(&Addr::unchecked(""), &[]);

    let msg = InitMsg::Local {
        admin: Addr::unchecked(ADMIN),
    };

    let init_res = migrate(
        init_deps.as_mut(),
        env.clone(),
        UpgradeMsg::Init(msg.clone()),
    )
    .unwrap();
    let instantiate_res = instantiate(instantiate_deps.as_mut(), env, info, msg.clone()).unwrap();

    assert_eq!(init_res, instantiate_res);
    assert_eq!(
        format!("{:?}", init_deps.storage),
        format!("{:?}", instantiate_deps.storage)
    );
}

#[test]
fn ensure_remote_admin_fails_if_zkgm_not_set() {
    let mut deps = setup_local();

    let relayer = Addr::unchecked("relayer");

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(ZKGM), &[]),
            ExecuteMsg::Zkgmable(Zkgmable::OnZkgm(OnZkgm {
                caller: relayer.clone(),
                path: U256::ZERO,
                source_channel_id: ChannelId!(1),
                destination_channel_id: ChannelId!(1),
                sender: b"uhoh".into(),
                message: b"".into(),
                relayer: relayer.clone(),
                relayer_msg: b"".into()
            })),
        )
        .unwrap_err(),
        ContractError::ZkgmNotConfigured
    );
}
