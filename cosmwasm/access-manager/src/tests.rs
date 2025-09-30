use access_manager_types::{
    manager::{
        error::AccessManagerError,
        event::{
            RoleGrantDelayChanged, RoleGranted, RoleGuardianChanged, RoleRevoked, TargetClosed,
            TargetFunctionRoleUpdated,
        },
        msg::{ExecuteMsg, QueryMsg},
    },
    CanCall, HasRole, RoleId, Selector,
};
use cosmwasm_std::{testing::message_info, Addr, Response};
use serde_json::value::to_raw_value;

use crate::{
    error::ContractError,
    execute, min_setback,
    tests::utils::{assert_query_result, setup, ACCOUNT_1, ACCOUNT_2, ADMIN, TARGET_1, TARGET_2},
};

pub mod utils;

#[test]
fn grant_revoke_role_works() {
    let (mut deps, env) = setup();

    let info = message_info(&ADMIN, &[]);

    let grantee = Addr::unchecked("grantee");

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            to_raw_value(&ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
                execution_delay: 0,
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(RoleGranted {
            role_id: RoleId::new(1),
            account: &grantee,
            delay: 0,
            since: env.block.time.seconds(),
            new_member: true
        })
    );

    // grantee has role 1, no execution delay
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::new(1),
            account: grantee.clone(),
        },
        &HasRole {
            is_member: true,
            execution_delay: 0,
        },
    );

    // grantee doesn't have role 2
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::new(2),
            account: grantee.clone(),
        },
        &HasRole {
            is_member: false,
            execution_delay: 0,
        },
    );

    // grant grantee role 2 with execution delay
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            to_raw_value(&ExecuteMsg::GrantRole {
                role_id: RoleId::new(2),
                account: grantee.clone(),
                execution_delay: 10,
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(RoleGranted {
            role_id: RoleId::new(2),
            account: &grantee,
            delay: 10,
            since: env.block.time.seconds(),
            new_member: true
        })
    );

    // grantee has role 2 with execution delay
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::new(2),
            account: grantee.clone(),
        },
        &HasRole {
            is_member: true,
            execution_delay: 10,
        },
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            to_raw_value(&ExecuteMsg::RevokeRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(RoleRevoked {
            role_id: RoleId::new(1),
            account: &grantee
        }),
    );

    // grantee no longer has role 1
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::new(0),
            account: grantee.clone(),
        },
        &HasRole {
            is_member: false,
            execution_delay: 0,
        },
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            to_raw_value(&ExecuteMsg::RevokeRole {
                role_id: RoleId::new(2),
                account: grantee.clone(),
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(RoleRevoked {
            role_id: RoleId::new(2),
            account: &grantee
        }),
    );

    // grantee no longer has role 2
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::new(0),
            account: grantee.clone(),
        },
        &HasRole {
            is_member: false,
            execution_delay: 0,
        },
    );

    // revoking again has no effect
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            to_raw_value(&ExecuteMsg::RevokeRole {
                role_id: RoleId::new(2),
                account: grantee.clone(),
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new()
    );
}

#[test]
fn renounce_role_works() {
    let (mut deps, env) = setup();

    let info = message_info(&ADMIN, &[]);

    let grantee = Addr::unchecked("grantee");

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            to_raw_value(&ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
                execution_delay: 0,
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(RoleGranted {
            role_id: RoleId::new(1),
            account: &grantee,
            delay: 0,
            since: env.block.time.seconds(),
            new_member: true
        })
    );

    // grant grantee role 2 with execution delay
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            to_raw_value(&ExecuteMsg::GrantRole {
                role_id: RoleId::new(2),
                account: grantee.clone(),
                execution_delay: 10,
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(RoleGranted {
            role_id: RoleId::new(2),
            account: &grantee,
            delay: 10,
            since: env.block.time.seconds(),
            new_member: true
        })
    );

    // can't renounce with invalid caller_confirmation
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&grantee, &[]),
            to_raw_value(&ExecuteMsg::RenounceRole {
                role_id: RoleId::new(1),
                caller_confirmation: ADMIN.clone(),
            })
            .unwrap(),
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerBadConfirmation),
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&grantee, &[]),
            to_raw_value(&ExecuteMsg::RenounceRole {
                role_id: RoleId::new(1),
                caller_confirmation: grantee.clone(),
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(RoleRevoked {
            role_id: RoleId::new(1),
            account: &grantee
        }),
    );

    // grantee no longer has role 1
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::new(0),
            account: grantee.clone(),
        },
        &HasRole {
            is_member: false,
            execution_delay: 0,
        },
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&grantee, &[]),
            to_raw_value(&ExecuteMsg::RenounceRole {
                role_id: RoleId::new(2),
                caller_confirmation: grantee.clone(),
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(RoleRevoked {
            role_id: RoleId::new(2),
            account: &grantee
        }),
    );

    // grantee no longer has role 2
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::new(0),
            account: grantee.clone(),
        },
        &HasRole {
            is_member: false,
            execution_delay: 0,
        },
    );

    // renouncing again has no effect
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&grantee, &[]),
            to_raw_value(&ExecuteMsg::RenounceRole {
                role_id: RoleId::new(2),
                caller_confirmation: grantee.clone(),
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new()
    );
}

#[test]
fn grant_revoke_role_requires_role_admin() {
    let (mut deps, env) = setup();

    let grantee = Addr::unchecked("grantee");

    // no role admin configured, granter must be global admin
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]).clone(),
            to_raw_value(&ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
                execution_delay: 0,
            })
            .unwrap(),
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerUnauthorizedAccount {
            msg_sender: ACCOUNT_1.clone(),
            required_role_id: RoleId::new(0)
        })
    );

    // role 2 is now admin of role 1
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        to_raw_value(&ExecuteMsg::SetRoleAdmin {
            role_id: RoleId::new(1),
            admin: RoleId::new(2),
        })
        .unwrap(),
    )
    .unwrap();

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetRoleAdmin {
            role_id: RoleId::new(1),
        },
        &RoleId::new(2),
    );

    // role admin configured, granter must be role admin
    let res = execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ACCOUNT_1, &[]).clone(),
        to_raw_value(&ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: grantee.clone(),
            execution_delay: 0,
        })
        .unwrap(),
    )
    .unwrap_err();

    assert_eq!(
        res,
        ContractError::AccessManager(AccessManagerError::AccessManagerUnauthorizedAccount {
            msg_sender: ACCOUNT_1.clone(),
            required_role_id: RoleId::new(2)
        })
    );

    // grant role 2 to non-admin
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]).clone(),
        to_raw_value(&ExecuteMsg::GrantRole {
            role_id: RoleId::new(2),
            account: ACCOUNT_1.clone(),
            execution_delay: 0,
        })
        .unwrap(),
    )
    .unwrap();

    // granter is role admin, able to grant role
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ACCOUNT_1, &[]).clone(),
        to_raw_value(&ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: grantee.clone(),
            execution_delay: 0,
        })
        .unwrap(),
    )
    .unwrap();

    // grantee now has role 1, as granted by the role admin
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::new(1),
            account: grantee.clone(),
        },
        &HasRole {
            is_member: true,
            execution_delay: 0,
        },
    );

    // must be role admin to revoke role
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_2, &[]).clone(),
            to_raw_value(&ExecuteMsg::RevokeRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
            })
            .unwrap(),
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerUnauthorizedAccount {
            msg_sender: ACCOUNT_2.clone(),
            required_role_id: RoleId::new(2)
        })
    );

    // granter is role admin, able to revoke role
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ACCOUNT_1, &[]).clone(),
        to_raw_value(&ExecuteMsg::RevokeRole {
            role_id: RoleId::new(1),
            account: grantee.clone(),
        })
        .unwrap(),
    )
    .unwrap();

    // grantee now has role 1, as granted by the role admin
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::new(1),
            account: grantee.clone(),
        },
        &HasRole {
            is_member: false,
            execution_delay: 0,
        },
    );
}

#[test]
fn public_role_locked() {
    let (mut deps, env) = setup();

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]).clone(),
            to_raw_value(&ExecuteMsg::GrantRole {
                role_id: RoleId::PUBLIC_ROLE,
                account: ACCOUNT_1.clone(),
                execution_delay: 0,
            })
            .unwrap(),
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerLockedRole(
            RoleId::PUBLIC_ROLE
        ))
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]).clone(),
            to_raw_value(&ExecuteMsg::RevokeRole {
                role_id: RoleId::PUBLIC_ROLE,
                account: ACCOUNT_1.clone(),
            })
            .unwrap(),
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerLockedRole(
            RoleId::PUBLIC_ROLE
        ))
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]).clone(),
            to_raw_value(&ExecuteMsg::SetRoleGuardian {
                role_id: RoleId::PUBLIC_ROLE,
                guardian: RoleId::new(1)
            })
            .unwrap(),
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerLockedRole(
            RoleId::PUBLIC_ROLE
        ))
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]).clone(),
            to_raw_value(&ExecuteMsg::SetRoleGuardian {
                role_id: RoleId::ADMIN_ROLE,
                guardian: RoleId::new(1)
            })
            .unwrap(),
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerLockedRole(
            RoleId::ADMIN_ROLE
        ))
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]).clone(),
            to_raw_value(&ExecuteMsg::SetRoleAdmin {
                role_id: RoleId::PUBLIC_ROLE,
                admin: RoleId::new(1)
            })
            .unwrap(),
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerLockedRole(
            RoleId::PUBLIC_ROLE
        ))
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]).clone(),
            to_raw_value(&ExecuteMsg::SetRoleAdmin {
                role_id: RoleId::ADMIN_ROLE,
                admin: RoleId::new(1)
            })
            .unwrap(),
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerLockedRole(
            RoleId::ADMIN_ROLE
        ))
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]).clone(),
            to_raw_value(&ExecuteMsg::SetGrantDelay {
                role_id: RoleId::PUBLIC_ROLE,
                grant_delay: 0
            })
            .unwrap(),
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerLockedRole(
            RoleId::PUBLIC_ROLE
        ))
    );
}

#[test]
fn re_grant_role() {
    let (mut deps, mut env) = setup();

    let info = message_info(&ADMIN, &[]);

    let grantee = Addr::unchecked("grantee");

    // grant role, no execution delay
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            to_raw_value(&ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
                execution_delay: 0,
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(RoleGranted {
            role_id: RoleId::new(1),
            account: &grantee,
            delay: 0,
            since: env.block.time.seconds(),
            new_member: true
        })
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::new(1),
            account: grantee.clone(),
        },
        &HasRole {
            is_member: true,
            execution_delay: 0,
        },
    );

    // re-grant role, no execution delay
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            to_raw_value(&ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
                execution_delay: 0,
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(RoleGranted {
            role_id: RoleId::new(1),
            account: &grantee,
            delay: 0,
            since: env.block.time.seconds(),
            new_member: false
        })
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::new(1),
            account: grantee.clone(),
        },
        &HasRole {
            is_member: true,
            execution_delay: 0,
        },
    );

    // re-grant role, with execution delay
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            to_raw_value(&ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
                execution_delay: 10,
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(RoleGranted {
            role_id: RoleId::new(1),
            account: &grantee,
            delay: 10,
            since: env.block.time.seconds(),
            new_member: false
        }),
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::new(1),
            account: grantee.clone(),
        },
        &HasRole {
            is_member: true,
            execution_delay: 10,
        },
    );

    // re-grant role again, with new execution delay
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            to_raw_value(&ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
                execution_delay: 6,
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(RoleGranted {
            role_id: RoleId::new(1),
            account: &grantee,
            delay: 6,
            since: env.block.time.seconds() + 4, /* previous execution delay (10) - new execution
                                                  * delay (6) */
            new_member: false,
        }),
    );

    // still has old execution delay
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::new(1),
            account: grantee.clone(),
        },
        &HasRole {
            is_member: true,
            execution_delay: 10,
        },
    );

    // still has old execution delay after 3 seconds (1 less than the difference between the old and
    // new delay)
    env.block.time = env.block.time.plus_seconds(3);
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::new(1),
            account: grantee.clone(),
        },
        &HasRole {
            is_member: true,
            execution_delay: 10,
        },
    );

    // still has old execution delay after 4 seconds total (difference between the old and new
    // delay)
    env.block.time = env.block.time.plus_seconds(1);
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::new(1),
            account: grantee.clone(),
        },
        &HasRole {
            is_member: true,
            execution_delay: 6,
        },
    );
}

#[test]
fn grant_delay_works() {
    let (mut deps, mut env) = setup();

    // set grant delay for role 1
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            to_raw_value(&ExecuteMsg::SetGrantDelay {
                role_id: RoleId::new(1),
                grant_delay: 10
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(RoleGrantDelayChanged {
            role_id: RoleId::new(1),
            delay: 10,
            since: env.block.time.seconds() + u64::from(min_setback()),
        }),
    );

    // new delay doesn't come into effect until the min_setback
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetRoleGrantDelay {
            role_id: RoleId::new(1),
        },
        &0,
    );

    env.block.time = env.block.time.plus_seconds((min_setback() - 1).into());
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetRoleGrantDelay {
            role_id: RoleId::new(1),
        },
        &0,
    );

    env.block.time = env.block.time.plus_seconds(1);
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetRoleGrantDelay {
            role_id: RoleId::new(1),
        },
        &10,
    );
}

#[test]
fn role_guardian_works() {
    let (mut deps, env) = setup();

    // set guardian of role 1
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            to_raw_value(&ExecuteMsg::SetRoleGuardian {
                role_id: RoleId::new(1),
                guardian: RoleId::new(2),
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(RoleGuardianChanged {
            role_id: RoleId::new(1),
            guardian: RoleId::new(2),
        }),
    );

    // new delay doesn't come into effect until the min_setback
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetRoleGuardian {
            role_id: RoleId::new(1),
        },
        &RoleId::new(2),
    );

    // grant guardian role to account-1
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]).clone(),
        to_raw_value(&ExecuteMsg::GrantRole {
            role_id: RoleId::new(2),
            account: ACCOUNT_1.clone(),
            execution_delay: 0,
        })
        .unwrap(),
    )
    .unwrap();

    // guardian is not able to grant or revoke role
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]).clone(),
            to_raw_value(&ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: ACCOUNT_2.clone(),
                execution_delay: 0,
            })
            .unwrap(),
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerUnauthorizedAccount {
            msg_sender: ACCOUNT_1.clone(),
            required_role_id: RoleId::new(0)
        })
    );
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]).clone(),
            to_raw_value(&ExecuteMsg::RevokeRole {
                role_id: RoleId::new(1),
                account: ACCOUNT_2.clone(),
            })
            .unwrap(),
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerUnauthorizedAccount {
            msg_sender: ACCOUNT_1.clone(),
            required_role_id: RoleId::new(0)
        })
    );
}

#[test]
fn everyone_has_public_role() {
    let (deps, env) = setup();

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::PUBLIC_ROLE,
            account: ADMIN.clone(),
        },
        &HasRole {
            is_member: true,
            execution_delay: 0,
        },
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::PUBLIC_ROLE,
            account: ACCOUNT_1.clone(),
        },
        &HasRole {
            is_member: true,
            execution_delay: 0,
        },
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::PUBLIC_ROLE,
            account: ACCOUNT_2.clone(),
        },
        &HasRole {
            is_member: true,
            execution_delay: 0,
        },
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::PUBLIC_ROLE,
            account: Addr::unchecked(""),
        },
        &HasRole {
            is_member: true,
            execution_delay: 0,
        },
    );
}

#[test]
fn target_function_role_works() {
    let (mut deps, env) = setup();

    // "a" and "b" on target-1 are callable by role 1
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            to_raw_value(&ExecuteMsg::SetTargetFunctionRole {
                target: TARGET_1.clone(),
                selectors: vec![Selector::new("a").to_owned(), Selector::new("b").to_owned()],
                role_id: RoleId::new(1),
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_events([
            TargetFunctionRoleUpdated {
                target: &TARGET_1,
                selector: Selector::new("a"),
                role_id: RoleId::new(1)
            },
            TargetFunctionRoleUpdated {
                target: &TARGET_1,
                selector: Selector::new("b"),
                role_id: RoleId::new(1)
            },
        ]),
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetTargetFunctionRole {
            target: TARGET_1.clone(),
            selector: Selector::new("a").to_owned(),
        },
        &RoleId::new(1),
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetTargetFunctionRole {
            target: TARGET_1.clone(),
            selector: Selector::new("b").to_owned(),
        },
        &RoleId::new(1),
    );

    // defaults to admin if not set
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetTargetFunctionRole {
            // target not configured yet
            target: TARGET_2.clone(),
            selector: Selector::new("a").to_owned(),
        },
        &RoleId::new(0),
    );
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetTargetFunctionRole {
            target: TARGET_1.clone(),
            // target configured with other selectors but not this one
            selector: Selector::new("c").to_owned(),
        },
        &RoleId::new(0),
    );
}

#[test]
fn can_call_works() {
    let (mut deps, env) = setup();

    // "a" and "b" on target-1 are callable by role 1
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        to_raw_value(&ExecuteMsg::SetTargetFunctionRole {
            target: TARGET_1.clone(),
            selectors: vec![Selector::new("a").to_owned()],
            role_id: RoleId::new(1),
        })
        .unwrap(),
    )
    .unwrap();

    // account-1 doesn't yet have the required role so it is not allowed to call
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            target: TARGET_1.clone(),
            selector: Selector::new("a").to_owned(),
            caller: ACCOUNT_1.clone(),
        },
        &CanCall {
            allowed: false,
            delay: 0,
        },
    );

    // grant role 1 to account-1
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        to_raw_value(&ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_1.clone(),
            execution_delay: 0,
        })
        .unwrap(),
    )
    .unwrap();

    // account-1 can call now, with no delay
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            target: TARGET_1.clone(),
            selector: Selector::new("a").to_owned(),
            caller: ACCOUNT_1.clone(),
        },
        &CanCall {
            allowed: true,
            delay: 0,
        },
    );

    // grant role 1 to account-2 with an execution delay
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        to_raw_value(&ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_2.clone(),
            execution_delay: 10,
        })
        .unwrap(),
    )
    .unwrap();

    // account-2 can't call, but may go through the schedule and execute flow to queue an operation
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            target: TARGET_1.clone(),
            selector: Selector::new("a").to_owned(),
            caller: ACCOUNT_2.clone(),
        },
        &CanCall {
            allowed: false,
            delay: 10,
        },
    );

    // neither account-1 nor account-2 can call "b"
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            target: TARGET_1.clone(),
            selector: Selector::new("b").to_owned(),
            caller: ACCOUNT_1.clone(),
        },
        &CanCall {
            allowed: false,
            delay: 0,
        },
    );
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            target: TARGET_1.clone(),
            selector: Selector::new("b").to_owned(),
            caller: ACCOUNT_2.clone(),
        },
        &CanCall {
            allowed: false,
            delay: 0,
        },
    );
}

#[test]
fn closed_target_not_callable() {
    let (mut deps, env) = setup();

    // "a" and "b" on target-1 are callable by role 1
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        to_raw_value(&ExecuteMsg::SetTargetFunctionRole {
            target: TARGET_1.clone(),
            selectors: vec![Selector::new("a").to_owned()],
            role_id: RoleId::new(1),
        })
        .unwrap(),
    )
    .unwrap();

    // account-1 doesn't yet have the required role so it is not allowed to call
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            target: TARGET_1.clone(),
            selector: Selector::new("a").to_owned(),
            caller: ACCOUNT_1.clone(),
        },
        &CanCall {
            allowed: false,
            delay: 0,
        },
    );

    // grant role 1 to account-1
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        to_raw_value(&ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_1.clone(),
            execution_delay: 0,
        })
        .unwrap(),
    )
    .unwrap();

    // account-1 can call now, with no delay
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            target: TARGET_1.clone(),
            selector: Selector::new("a").to_owned(),
            caller: ACCOUNT_1.clone(),
        },
        &CanCall {
            allowed: true,
            delay: 0,
        },
    );

    // grant role 1 to account-2 with an execution delay
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        to_raw_value(&ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_2.clone(),
            execution_delay: 10,
        })
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            to_raw_value(&ExecuteMsg::SetTargetClosed {
                target: TARGET_1.clone(),
                closed: true,
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(TargetClosed {
            target: &TARGET_1,
            closed: true,
        }),
    );

    // neither account-1 nor account-2 can call anything on target-1
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            target: TARGET_1.clone(),
            selector: Selector::new("a").to_owned(),
            caller: ACCOUNT_1.clone(),
        },
        &CanCall {
            allowed: false,
            delay: 0,
        },
    );
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            target: TARGET_1.clone(),
            selector: Selector::new("b").to_owned(),
            caller: ACCOUNT_2.clone(),
        },
        &CanCall {
            allowed: false,
            delay: 0,
        },
    );
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            target: TARGET_1.clone(),
            // unconfigured selector, must be uncallable when target is closed
            selector: Selector::new("c").to_owned(),
            caller: ACCOUNT_2.clone(),
        },
        &CanCall {
            allowed: false,
            delay: 0,
        },
    );
}

#[test]
fn set_target_closed_works() {
    let (mut deps, env) = setup();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        to_raw_value(&ExecuteMsg::SetTargetClosed {
            target: TARGET_1.clone(),
            closed: true,
        })
        .unwrap(),
    )
    .unwrap();

    // closing again acts the same as if it was not closed previously
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            to_raw_value(&ExecuteMsg::SetTargetClosed {
                target: TARGET_1.clone(),
                closed: true,
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(TargetClosed {
            target: &TARGET_1,
            closed: true,
        }),
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            to_raw_value(&ExecuteMsg::SetTargetClosed {
                target: TARGET_1.clone(),
                closed: false,
            })
            .unwrap(),
        )
        .unwrap(),
        Response::new().add_event(TargetClosed {
            target: &TARGET_1,
            closed: false,
        }),
    );
}
