use std::num::NonZero;

use access_manager_types::{
    CanCall, HasRole, Nonce, RoleId, RoleLabelsResponse, Selector, Timepoint,
    manager::{
        error::AccessManagerError,
        event::{
            OperationExecuted, OperationScheduled, RoleGrantDelayChanged, RoleGranted,
            RoleGuardianChanged, RoleLabel, RoleRevoked, TargetAdminDelayUpdated, TargetClosed,
            TargetFunctionRoleUpdated,
        },
        msg::{ExecuteMsg, QueryMsg},
    },
};
use cosmwasm_std::{Addr, Response, StdError, SubMsg, WasmMsg, testing::message_info};
use hex_literal::hex;
use unionlabs_primitives::H256;

use crate::{
    contract::expiration,
    error::ContractError,
    execute, min_setback, query,
    tests::utils::{
        ACCOUNT_1, ACCOUNT_2, ACCOUNT_3, ADMIN, TARGET_1, TARGET_2, assert_query_result, setup,
    },
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
            ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
                execution_delay: 0,
            },
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
        &HasRole::Yes {
            execution_delay: None,
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
        &HasRole::No {},
    );

    // grant grantee role 2 with execution delay
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::GrantRole {
                role_id: RoleId::new(2),
                account: grantee.clone(),
                execution_delay: 10,
            },
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
        &HasRole::Yes {
            execution_delay: Some(NonZero::new(10).unwrap()),
        },
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::RevokeRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
            },
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
        &HasRole::No {},
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::RevokeRole {
                role_id: RoleId::new(2),
                account: grantee.clone(),
            },
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
        &HasRole::No {},
    );

    // revoking again has no effect
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::RevokeRole {
                role_id: RoleId::new(2),
                account: grantee.clone(),
            },
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

    // can't renounce public role
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&grantee, &[]),
            ExecuteMsg::RenounceRole {
                role_id: RoleId::PUBLIC_ROLE,
                caller_confirmation: grantee.clone(),
            },
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerLockedRole(
            RoleId::PUBLIC_ROLE
        )),
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
                execution_delay: 0,
            },
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
            ExecuteMsg::GrantRole {
                role_id: RoleId::new(2),
                account: grantee.clone(),
                execution_delay: 10,
            },
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
            ExecuteMsg::RenounceRole {
                role_id: RoleId::new(1),
                caller_confirmation: ADMIN.clone(),
            },
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerBadConfirmation),
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&grantee, &[]),
            ExecuteMsg::RenounceRole {
                role_id: RoleId::new(1),
                caller_confirmation: grantee.clone(),
            },
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
        &HasRole::No {},
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&grantee, &[]),
            ExecuteMsg::RenounceRole {
                role_id: RoleId::new(2),
                caller_confirmation: grantee.clone(),
            },
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
        &HasRole::No {},
    );

    // renouncing again has no effect
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&grantee, &[]),
            ExecuteMsg::RenounceRole {
                role_id: RoleId::new(2),
                caller_confirmation: grantee.clone(),
            },
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
            ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
                execution_delay: 0,
            },
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
        ExecuteMsg::SetRoleAdmin {
            role_id: RoleId::new(1),
            admin: RoleId::new(2),
        },
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
        ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: grantee.clone(),
            execution_delay: 0,
        },
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
        ExecuteMsg::GrantRole {
            role_id: RoleId::new(2),
            account: ACCOUNT_1.clone(),
            execution_delay: 0,
        },
    )
    .unwrap();

    // granter is role admin, able to grant role
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ACCOUNT_1, &[]).clone(),
        ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: grantee.clone(),
            execution_delay: 0,
        },
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
        &HasRole::Yes {
            execution_delay: None,
        },
    );

    // must be role admin to revoke role
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_2, &[]).clone(),
            ExecuteMsg::RevokeRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
            },
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
        ExecuteMsg::RevokeRole {
            role_id: RoleId::new(1),
            account: grantee.clone(),
        },
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
        &HasRole::No {},
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
            ExecuteMsg::GrantRole {
                role_id: RoleId::PUBLIC_ROLE,
                account: ACCOUNT_1.clone(),
                execution_delay: 0,
            },
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
            ExecuteMsg::RevokeRole {
                role_id: RoleId::PUBLIC_ROLE,
                account: ACCOUNT_1.clone(),
            },
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
            ExecuteMsg::SetRoleGuardian {
                role_id: RoleId::PUBLIC_ROLE,
                guardian: RoleId::new(1)
            },
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
            ExecuteMsg::SetRoleGuardian {
                role_id: RoleId::ADMIN_ROLE,
                guardian: RoleId::new(1)
            },
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
            ExecuteMsg::SetRoleAdmin {
                role_id: RoleId::PUBLIC_ROLE,
                admin: RoleId::new(1)
            },
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
            ExecuteMsg::SetRoleAdmin {
                role_id: RoleId::ADMIN_ROLE,
                admin: RoleId::new(1)
            },
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
            ExecuteMsg::SetGrantDelay {
                role_id: RoleId::PUBLIC_ROLE,
                grant_delay: 0
            },
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
            ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
                execution_delay: 0,
            },
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
        &HasRole::Yes {
            execution_delay: None,
        },
    );

    // re-grant role, no execution delay
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
                execution_delay: 0,
            },
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
        &HasRole::Yes {
            execution_delay: None,
        },
    );

    // re-grant role, with execution delay
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
                execution_delay: 10,
            },
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
        &HasRole::Yes {
            execution_delay: Some(NonZero::new(10).unwrap()),
        },
    );

    // re-grant role again, with new execution delay
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            info.clone(),
            ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: grantee.clone(),
                execution_delay: 6,
            },
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
        &HasRole::Yes {
            execution_delay: Some(NonZero::new(10).unwrap()),
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
        &HasRole::Yes {
            execution_delay: Some(NonZero::new(10).unwrap()),
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
        &HasRole::Yes {
            execution_delay: Some(NonZero::new(6).unwrap()),
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
            ExecuteMsg::SetGrantDelay {
                role_id: RoleId::new(1),
                grant_delay: 10
            },
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
            ExecuteMsg::SetRoleGuardian {
                role_id: RoleId::new(1),
                guardian: RoleId::new(2),
            },
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
        ExecuteMsg::GrantRole {
            role_id: RoleId::new(2),
            account: ACCOUNT_1.clone(),
            execution_delay: 0,
        },
    )
    .unwrap();

    // guardian is not able to grant or revoke role
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]).clone(),
            ExecuteMsg::GrantRole {
                role_id: RoleId::new(1),
                account: ACCOUNT_2.clone(),
                execution_delay: 0,
            },
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
            ExecuteMsg::RevokeRole {
                role_id: RoleId::new(1),
                account: ACCOUNT_2.clone(),
            },
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
        &HasRole::Yes {
            execution_delay: None,
        },
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::PUBLIC_ROLE,
            account: ACCOUNT_1.clone(),
        },
        &HasRole::Yes {
            execution_delay: None,
        },
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::PUBLIC_ROLE,
            account: ACCOUNT_2.clone(),
        },
        &HasRole::Yes {
            execution_delay: None,
        },
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::HasRole {
            role_id: RoleId::PUBLIC_ROLE,
            account: Addr::unchecked(""),
        },
        &HasRole::Yes {
            execution_delay: None,
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
            ExecuteMsg::SetTargetFunctionRole {
                target: TARGET_1.clone(),
                selectors: vec![Selector::new("a").to_owned(), Selector::new("b").to_owned()],
                role_id: RoleId::new(1),
            },
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
        ExecuteMsg::SetTargetFunctionRole {
            target: TARGET_1.clone(),
            selectors: vec![Selector::new("a").to_owned()],
            role_id: RoleId::new(1),
        },
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
        &CanCall::Unauthorized {},
    );

    // grant role 1 to account-1
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_1.clone(),
            execution_delay: 0,
        },
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
        &CanCall::Immediate {},
    );

    // grant role 1 to account-2 with an execution delay
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_2.clone(),
            execution_delay: 10,
        },
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
        &CanCall::WithDelay {
            delay: NonZero::new(10).unwrap(),
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
        &CanCall::Unauthorized {},
    );
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            target: TARGET_1.clone(),
            selector: Selector::new("b").to_owned(),
            caller: ACCOUNT_2.clone(),
        },
        &CanCall::Unauthorized {},
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
        ExecuteMsg::SetTargetFunctionRole {
            target: TARGET_1.clone(),
            selectors: vec![Selector::new("a").to_owned()],
            role_id: RoleId::new(1),
        },
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
        &CanCall::Unauthorized {},
    );

    // grant role 1 to account-1
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_1.clone(),
            execution_delay: 0,
        },
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
        &CanCall::Immediate {},
    );

    // grant role 1 to account-2 with an execution delay
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_2.clone(),
            execution_delay: 10,
        },
    )
    .unwrap();

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            ExecuteMsg::SetTargetClosed {
                target: TARGET_1.clone(),
                closed: true,
            },
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
        &CanCall::Unauthorized {},
    );
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            target: TARGET_1.clone(),
            selector: Selector::new("b").to_owned(),
            caller: ACCOUNT_2.clone(),
        },
        &CanCall::Unauthorized {},
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
        &CanCall::Unauthorized {},
    );
}

#[test]
fn set_target_closed_works() {
    let (mut deps, env) = setup();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::SetTargetClosed {
            target: TARGET_1.clone(),
            closed: true,
        },
    )
    .unwrap();

    // closing again acts the same as if it was not closed previously
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            ExecuteMsg::SetTargetClosed {
                target: TARGET_1.clone(),
                closed: true,
            },
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
            ExecuteMsg::SetTargetClosed {
                target: TARGET_1.clone(),
                closed: false,
            },
        )
        .unwrap(),
        Response::new().add_event(TargetClosed {
            target: &TARGET_1,
            closed: false,
        }),
    );
}

#[test]
fn schedule_works() {
    let (mut deps, mut env) = setup();

    let operation_id = H256::new(hex!(
        "7c99107b1d6b31f7b0c08fece541ea567a76154de0d91f62d2f2022d09004b0e"
    ));

    let data = r#"{"a":{}}"#;

    // operation has never been scheduled, nonce and timepoint are default values
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetNonce { id: operation_id },
        &Nonce::new(0),
    );
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetSchedule { id: operation_id },
        &None::<()>,
    );

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_1.clone(),
            execution_delay: 10,
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_2.clone(),
            execution_delay: 0,
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::SetTargetFunctionRole {
            role_id: RoleId::new(1),
            target: TARGET_1.clone(),
            selectors: vec![Selector::new("a").to_owned()],
        },
    )
    .unwrap();

    // account-2 has no execution delay, so it cannot schedule
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_2, &[]),
            ExecuteMsg::Schedule {
                target: TARGET_1.clone(),
                data: data.to_owned(),
                when: NonZero::new(env.block.time.seconds() + 5).unwrap()
            },
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerUnauthorizedCall {
            caller: ACCOUNT_2.clone(),
            target: TARGET_1.clone(),
            selector: Selector::new("a").to_owned()
        })
    );

    // attempt to schedule too soon
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            ExecuteMsg::Schedule {
                target: TARGET_1.clone(),
                data: data.to_owned(),
                when: NonZero::new(env.block.time.seconds() + 5).unwrap()
            },
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerUnauthorizedCall {
            caller: ACCOUNT_1.clone(),
            target: TARGET_1.clone(),
            selector: Selector::new("a").to_owned()
        })
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            ExecuteMsg::Schedule {
                target: TARGET_1.clone(),
                data: data.to_owned(),
                when: NonZero::new(env.block.time.seconds() + 10).unwrap()
            },
        )
        .unwrap(),
        Response::new().add_event(OperationScheduled {
            operation_id,
            nonce: Nonce::new(1),
            schedule: NonZero::new(env.block.time.seconds() + 10).unwrap(),
            caller: &ACCOUNT_1,
            target: &TARGET_1,
            data,
        }),
    );

    // operation has been scheduled, nonce is now 1 and timepoint has been set
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetNonce { id: operation_id },
        &Nonce::new(1),
    );
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetSchedule { id: operation_id },
        &Some(NonZero::new(env.block.time.seconds() + 10).unwrap()),
    );

    // can't schedule the exact same operation again before the currently scheduled operation has either expired or been executed
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            ExecuteMsg::Schedule {
                target: TARGET_1.clone(),
                data: data.to_owned(),
                when: NonZero::new(env.block.time.seconds() + 10).unwrap()
            },
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerAlreadyScheduled(
            operation_id
        ))
    );

    env.block.time = env.block.time.plus_seconds(u64::from(expiration()) + 10);

    // scheduled operation expires correctly; nonce does not increase on expiry
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetNonce { id: operation_id },
        &Nonce::new(1),
    );
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetSchedule { id: operation_id },
        &None::<()>,
    );

    // scheduling again increases the nonce and sets the timepoint
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            ExecuteMsg::Schedule {
                target: TARGET_1.clone(),
                data: data.to_owned(),
                when: NonZero::new(env.block.time.seconds() + 10).unwrap()
            },
        )
        .unwrap(),
        Response::new().add_event(OperationScheduled {
            operation_id,
            nonce: Nonce::new(2),
            schedule: NonZero::new(env.block.time.seconds() + 10).unwrap(),
            caller: &ACCOUNT_1,
            target: &TARGET_1,
            data,
        }),
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetNonce { id: operation_id },
        &Nonce::new(2),
    );
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetSchedule { id: operation_id },
        &Some(NonZero::new(env.block.time.seconds() + 10).unwrap()),
    );
}

#[test]
fn schedule_invalid_data_fails() {
    let (mut deps, env) = setup();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_1.clone(),
            execution_delay: 10,
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::SetTargetFunctionRole {
            role_id: RoleId::new(1),
            target: TARGET_1.clone(),
            selectors: vec![Selector::new("a").to_owned()],
        },
    )
    .unwrap();

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            ExecuteMsg::Schedule {
                target: TARGET_1.clone(),
                data: r"this ain't json bro".to_owned(),
                when: NonZero::new(env.block.time.seconds() + 10).unwrap()
            },
        )
        .unwrap_err(),
        ContractError::Std(StdError::generic_err(
            "error extracting selector: expected ident at line 1 column 2"
        )),
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            ExecuteMsg::Schedule {
                target: TARGET_1.clone(),
                data: r#"{"too":"many","keys":"man"}"#.to_owned(),
                when: NonZero::new(env.block.time.seconds() + 10).unwrap()
            },
        )
        .unwrap_err(),
        ContractError::Std(StdError::generic_err(
            "error extracting selector: multiple keys found at line 1 column 20"
        )),
    );
}

#[test]
fn schedule_reentrant_works() {
    const GRANT_ROLE: RoleId = RoleId::new(6);

    let (mut deps, env) = setup();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::SetTargetFunctionRole {
            target: env.contract.address.clone(),
            selectors: vec![Selector::new("grant_role").to_owned()],
            role_id: GRANT_ROLE,
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::GrantRole {
            account: ACCOUNT_1.clone(),
            role_id: GRANT_ROLE,
            execution_delay: 10,
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::GrantRole {
            account: ACCOUNT_1.clone(),
            role_id: RoleId::new(11),
            execution_delay: 20,
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::SetRoleAdmin {
            role_id: RoleId::new(10),
            admin: RoleId::new(11),
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::SetTargetFunctionRole {
            role_id: GRANT_ROLE,
            target: env.contract.address.clone(),
            selectors: vec![Selector::new("grant_role").to_owned()],
        },
    )
    .unwrap();

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            ExecuteMsg::Schedule {
                target: env.contract.address.clone(),
                data: serde_json_wasm::to_string(&ExecuteMsg::GrantRole {
                    role_id: RoleId::new(10),
                    account: ACCOUNT_2.clone(),
                    execution_delay: 0
                })
                .unwrap(),
                when: NonZero::new(env.block.time.seconds() + 20).unwrap()
            },
        )
        .unwrap(),
        Response::new().add_event(OperationScheduled {
            operation_id: H256::new(hex!(
                "8851fd1669d010b077f22bf956ea2ae240fe964d0f9d30e46f702b6e950278b5"
            )),
            nonce: Nonce::new(1),
            schedule: NonZero::new(env.block.time.seconds() + 20).unwrap(),
            caller: &ACCOUNT_1,
            target: &env.contract.address,
            data: r#"{"grant_role":{"role_id":"10","account":"account-2","execution_delay":0}}"#,
        }),
    );
}

#[test]
fn target_role_internal_selector_fails() {
    let (mut deps, env) = setup();

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]).clone(),
            ExecuteMsg::SetTargetFunctionRole {
                selectors: vec![Selector::new("$$internal_method").to_owned()],
                role_id: RoleId::new(1),
                target: TARGET_1.clone(),
            },
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::InternalSelector(
            Selector::new("$$internal_method").to_owned()
        ))
    );

    assert_eq!(
        query(
            deps.as_ref(),
            env.clone(),
            QueryMsg::GetTargetFunctionRole {
                target: TARGET_1.clone(),
                selector: Selector::new("$$also_internal").to_owned(),
            },
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::InternalSelector(
            Selector::new("$$also_internal").to_owned()
        ))
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            selector: Selector::new("$$internal_yet_again").to_owned(),
            target: TARGET_1.clone(),
            caller: ACCOUNT_1.clone(),
        },
        &CanCall::Immediate {},
    );
}

#[test]
fn cancel_works() {
    let init = || {
        let (mut deps, env) = setup();

        let role = RoleId::new(1);
        let guardian = RoleId::new(2);
        let admin = RoleId::new(3);

        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            ExecuteMsg::GrantRole {
                role_id: role,
                account: ACCOUNT_1.clone(),
                execution_delay: 1,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            ExecuteMsg::SetRoleGuardian {
                role_id: role,
                guardian,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            ExecuteMsg::GrantRole {
                role_id: guardian,
                account: ACCOUNT_2.clone(),
                execution_delay: 1,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            ExecuteMsg::SetRoleAdmin {
                role_id: role,
                admin,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            ExecuteMsg::GrantRole {
                role_id: admin,
                account: ACCOUNT_3.clone(),
                execution_delay: 1,
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            ExecuteMsg::SetTargetFunctionRole {
                role_id: role,
                target: TARGET_1.clone(),
                selectors: vec![Selector::new("a").to_owned()],
            },
        )
        .unwrap();

        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            ExecuteMsg::Schedule {
                target: TARGET_1.clone(),
                data: r#"{"a":{}}"#.to_owned(),
                when: NonZero::new(env.block.time.seconds() + 5).unwrap(),
            },
        )
        .unwrap();

        (deps, env)
    };

    // original scheduler can cancel
    {
        let (mut deps, env) = init();

        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            ExecuteMsg::Cancel {
                caller: ACCOUNT_1.clone(),
                target: TARGET_1.clone(),
                data: r#"{"a":{}}"#.to_owned(),
            },
        )
        .unwrap();
    }

    // role guardian can cancel
    {
        let (mut deps, env) = init();

        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_2, &[]),
            ExecuteMsg::Cancel {
                caller: ACCOUNT_1.clone(),
                target: TARGET_1.clone(),
                data: r#"{"a":{}}"#.to_owned(),
            },
        )
        .unwrap();
    }

    // role admin can *not* cancel
    {
        let (mut deps, env) = init();

        assert_eq!(
            execute(
                deps.as_mut(),
                env.clone(),
                message_info(&ACCOUNT_3, &[]),
                ExecuteMsg::Cancel {
                    caller: ACCOUNT_1.clone(),
                    target: TARGET_1.clone(),
                    data: r#"{"a":{}}"#.to_owned(),
                },
            )
            .unwrap_err(),
            ContractError::AccessManager(AccessManagerError::AccessManagerUnauthorizedCancel {
                msg_sender: ACCOUNT_3.clone(),
                caller: ACCOUNT_1.clone(),
                target: TARGET_1.clone(),
                selector: Selector::new("a").to_owned()
            })
        );
    }

    // global admin can cancel
    {
        let (mut deps, env) = init();

        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            ExecuteMsg::Cancel {
                caller: ACCOUNT_1.clone(),
                target: TARGET_1.clone(),
                data: r#"{"a":{}}"#.to_owned(),
            },
        )
        .unwrap();
    }
}

#[test]
fn role_label_works() {
    let (mut deps, env) = setup();

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            ExecuteMsg::LabelRole {
                role_id: RoleId::new(1),
                label: "role".to_owned(),
            },
        )
        .unwrap(),
        Response::new().add_event(RoleLabel {
            role_id: RoleId::new(1),
            label: "role",
        }),
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetRoleLabels {
            role_ids: vec![RoleId::new(1), RoleId::new(2)],
        },
        &RoleLabelsResponse(
            [
                (RoleId::new(1), Some("role".to_string())),
                (RoleId::new(2), None),
            ]
            .into_iter()
            .collect(),
        ),
    );
}

// TODO: Link audit report once it's published
#[test]
fn locked_roles_not_labelable() {
    {
        let (mut deps, env) = setup();

        assert_eq!(
            execute(
                deps.as_mut(),
                env.clone(),
                message_info(&ADMIN, &[]),
                ExecuteMsg::LabelRole {
                    role_id: RoleId::ADMIN_ROLE,
                    label: "ADMIN".to_owned(),
                },
            )
            .unwrap_err(),
            ContractError::AccessManager(AccessManagerError::AccessManagerLockedRole(
                RoleId::ADMIN_ROLE
            ))
        );
    }

    {
        let (mut deps, env) = setup();

        assert_eq!(
            execute(
                deps.as_mut(),
                env.clone(),
                message_info(&ADMIN, &[]),
                ExecuteMsg::LabelRole {
                    role_id: RoleId::PUBLIC_ROLE,
                    label: "PUBLIC".to_owned(),
                },
            )
            .unwrap_err(),
            ContractError::AccessManager(AccessManagerError::AccessManagerLockedRole(
                RoleId::PUBLIC_ROLE
            ))
        );
    }
}

#[test]
fn target_admin_delay_works() {
    let (mut deps, mut env) = setup();

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            ExecuteMsg::SetTargetAdminDelay {
                target: TARGET_1.clone(),
                new_delay: 100
            },
        )
        .unwrap(),
        Response::new().add_event(TargetAdminDelayUpdated {
            target: &TARGET_1,
            delay: 100,
            since: env.block.time.seconds() + u64::from(min_setback()),
        }),
    );

    env.block.time = env.block.time.plus_seconds(3);

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            ExecuteMsg::SetTargetAdminDelay {
                target: TARGET_1.clone(),
                new_delay: min_setback() * 10
            },
        )
        .unwrap(),
        Response::new().add_event(TargetAdminDelayUpdated {
            target: &TARGET_1,
            delay: min_setback() * 10,
            since: env.block.time.seconds() + u64::from(min_setback()),
        }),
    );

    env.block.time = env.block.time.plus_seconds(min_setback().into());

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ADMIN, &[]),
            ExecuteMsg::SetTargetAdminDelay {
                target: TARGET_1.clone(),
                new_delay: 100
            },
        )
        .unwrap(),
        Response::new().add_event(TargetAdminDelayUpdated {
            target: &TARGET_1,
            delay: 100,
            since: env.block.time.seconds() + u64::from(min_setback() * 10) - 100,
        }),
    );
}

#[test]
fn admin_operations_check_admin() {
    for msg in [
        ExecuteMsg::SetRoleGuardian {
            role_id: RoleId::new(1),
            guardian: RoleId::new(2),
        },
        ExecuteMsg::SetRoleAdmin {
            role_id: RoleId::new(1),
            admin: RoleId::new(2),
        },
        ExecuteMsg::SetGrantDelay {
            role_id: RoleId::new(1),
            grant_delay: 0,
        },
        ExecuteMsg::SetTargetFunctionRole {
            target: TARGET_1.clone(),
            selectors: vec![Selector::new("a").to_owned()],
            role_id: RoleId::new(1),
        },
        ExecuteMsg::SetTargetAdminDelay {
            target: TARGET_1.clone(),
            new_delay: 0,
        },
        ExecuteMsg::SetTargetClosed {
            target: TARGET_1.clone(),
            closed: true,
        },
        ExecuteMsg::LabelRole {
            role_id: RoleId::new(1),
            label: "role".to_owned(),
        },
    ] {
        let (mut deps, env) = setup();
        assert_eq!(
            execute(
                deps.as_mut(),
                env.clone(),
                message_info(&ACCOUNT_1, &[]),
                msg,
            )
            .unwrap_err(),
            ContractError::AccessManager(AccessManagerError::AccessManagerUnauthorizedAccount {
                msg_sender: ACCOUNT_1.clone(),
                required_role_id: RoleId::ADMIN_ROLE,
            }),
        );
    }
}

// TODO: Link audit report once it's published
#[test]
fn execute_with_delay_without_schedule_must_fail() {
    let (mut deps, env) = setup();

    let operation_id = H256::new(hex!(
        "7c99107b1d6b31f7b0c08fece541ea567a76154de0d91f62d2f2022d09004b0e"
    ));

    let data = r#"{"a":{}}"#;

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_1.clone(),
            execution_delay: 10,
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::SetTargetFunctionRole {
            role_id: RoleId::new(1),
            target: TARGET_1.clone(),
            selectors: vec![Selector::new("a").to_owned()],
        },
    )
    .unwrap();

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            selector: Selector::new("a").to_owned(),
            target: TARGET_1.clone(),
            caller: ACCOUNT_1.clone(),
        },
        &CanCall::WithDelay {
            delay: const { NonZero::new(10).unwrap() },
        },
    );

    // Try to execute directly WITHOUT calling `Schedule` first.
    //
    // OpenZeppelin behaviour:
    //
    // ```
    // _canCallExtended(..) -> (immediate = false, setback = 10)
    // if (setback != 0 || getSchedule(operationId) != 0) {
    //     _consumeScheduledOp(operationId); // timepoint == 0 -> AccessManagerNotScheduled
    // }
    // ```
    //
    // So OZ AccessManager MUST revert with AccessManagerNotScheduled(operationId).
    //
    // This test encodes that behaviour. If our port allows this call to succeed
    // without a prior schedule, the test will panic (and that’s the PoC).
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            ExecuteMsg::Execute {
                target: TARGET_1.clone(),
                data: data.to_owned(),
            },
        )
        .unwrap_err(),
        ContractError::AccessManager(AccessManagerError::AccessManagerNotScheduled(operation_id))
    );

    // account-2 has the required role but with no execution delay, so it should be able to execute immediately, without an already scheduled operation
    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_2.clone(),
            execution_delay: 0,
        },
    )
    .unwrap();

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_2, &[]),
            ExecuteMsg::Execute {
                target: TARGET_1.clone(),
                data: data.to_owned(),
            },
        )
        .unwrap(),
        // no event is emitted since this was not a scheduled operation
        Response::new().add_submessage(SubMsg::reply_on_success(
            WasmMsg::Execute {
                contract_addr: TARGET_1.to_string(),
                msg: data.as_bytes().into(),
                funds: vec![]
            },
            1
        )),
    );

    // no scheduled operation was executed, all values should still be default
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetNonce { id: operation_id },
        &Nonce::new(0),
    );
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetSchedule { id: operation_id },
        &None::<Timepoint>,
    );
}

#[test]
fn execute_works() {
    let (mut deps, mut env) = setup();

    let operation_id = H256::new(hex!(
        "7c99107b1d6b31f7b0c08fece541ea567a76154de0d91f62d2f2022d09004b0e"
    ));

    let data = r#"{"a":{}}"#;

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::GrantRole {
            role_id: RoleId::new(1),
            account: ACCOUNT_1.clone(),
            execution_delay: 10,
        },
    )
    .unwrap();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&ADMIN, &[]),
        ExecuteMsg::SetTargetFunctionRole {
            role_id: RoleId::new(1),
            target: TARGET_1.clone(),
            selectors: vec![Selector::new("a").to_owned()],
        },
    )
    .unwrap();

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::CanCall {
            selector: Selector::new("a").to_owned(),
            target: TARGET_1.clone(),
            caller: ACCOUNT_1.clone(),
        },
        &CanCall::WithDelay {
            delay: const { NonZero::new(10).unwrap() },
        },
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            ExecuteMsg::Schedule {
                target: TARGET_1.clone(),
                data: data.to_owned(),
                when: NonZero::new(env.block.time.seconds() + 10).unwrap()
            },
        )
        .unwrap(),
        Response::new().add_event(OperationScheduled {
            operation_id,
            nonce: Nonce::new(1),
            schedule: NonZero::new(env.block.time.seconds() + 10).unwrap(),
            caller: &ACCOUNT_1,
            target: &TARGET_1,
            data,
        }),
    );

    // operation has been scheduled, nonce is now 1 and timepoint has been set
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetNonce { id: operation_id },
        &Nonce::new(1),
    );
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetSchedule { id: operation_id },
        &Some(NonZero::new(env.block.time.seconds() + 10).unwrap()),
    );

    env.block.time = env.block.time.plus_seconds(10);

    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&ACCOUNT_1, &[]),
            ExecuteMsg::Execute {
                target: TARGET_1.clone(),
                data: data.to_owned(),
            },
        )
        .unwrap(),
        Response::new()
            .add_submessage(SubMsg::reply_on_success(
                WasmMsg::Execute {
                    contract_addr: TARGET_1.to_string(),
                    msg: data.as_bytes().into(),
                    funds: vec![]
                },
                1
            ))
            .add_event(OperationExecuted {
                operation_id,
                nonce: Nonce::new(1),
            }),
    );

    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetNonce { id: operation_id },
        &Nonce::new(1),
    );
    assert_query_result(
        deps.as_ref(),
        &env,
        QueryMsg::GetSchedule { id: operation_id },
        &None::<Timepoint>,
    );
}
