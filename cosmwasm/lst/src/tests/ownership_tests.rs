// License text copyright (c) 2020 MariaDB Corporation Ab, All Rights Reserved.
// "Business Source License" is a trademark of MariaDB Corporation Ab.
//
// Parameters
//
// Licensor:             Union.fi, Labs Inc.
// Licensed Work:        All files under https://github.com/unionlabs/union's cosmwasm/lst subdirectory
//                       The Licensed Work is (c) 2025 Union.fi, Labs Inc.
// Change Date:          Four years from the date the Licensed Work is published.
// Change License:       Apache-2.0
//
//
// For information about alternative licensing arrangements for the Licensed Work,
// please contact info@union.build.
//
// Notice
//
// Business Source License 1.1
//
// Terms
//
// The Licensor hereby grants you the right to copy, modify, create derivative
// works, redistribute, and make non-production use of the Licensed Work. The
// Licensor may make an Additional Use Grant, above, permitting limited production use.
//
// Effective on the Change Date, or the fourth anniversary of the first publicly
// available distribution of a specific version of the Licensed Work under this
// License, whichever comes first, the Licensor hereby grants you rights under
// the terms of the Change License, and the rights granted in the paragraph
// above terminate.
//
// If your use of the Licensed Work does not comply with the requirements
// currently in effect as described in this License, you must purchase a
// commercial license from the Licensor, its affiliated entities, or authorized
// resellers, or you must refrain from using the Licensed Work.
//
// All copies of the original and modified Licensed Work, and derivative works
// of the Licensed Work, are subject to this License. This License applies
// separately for each version of the Licensed Work and the Change Date may vary
// for each version of the Licensed Work released by Licensor.
//
// You must conspicuously display this License on each original or modified copy
// of the Licensed Work. If you receive the Licensed Work in original or
// modified form from a third party, the terms and conditions set forth in this
// License apply to your use of that work.
//
// Any use of the Licensed Work in violation of this License will automatically
// terminate your rights under this License for the current and all other
// versions of the Licensed Work.
//
// This License does not grant you any right in any trademark or logo of
// Licensor or its affiliates (provided that you may use a trademark or logo of
// Licensor as expressly required by this License).
//
// TO THE EXTENT PERMITTED BY APPLICABLE LAW, THE LICENSED WORK IS PROVIDED ON
// AN "AS IS" BASIS. LICENSOR HEREBY DISCLAIMS ALL WARRANTIES AND CONDITIONS,
// EXPRESS OR IMPLIED, INCLUDING (WITHOUT LIMITATION) WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE, NON-INFRINGEMENT, AND
// TITLE.

use cosmwasm_std::{
    testing::{message_info, mock_env},
    Addr, Event,
};
use depolama::StorageExt;

use crate::{
    contract::execute,
    error::ContractError,
    execute::OWNERSHIP_CLAIM_DELAY_PERIOD_SECONDS,
    msg::ExecuteMsg,
    state::{Admin, PendingOwnerStore},
    tests::test_helper::{setup, ADMIN, UNION1, UNION2, UNION_MONITOR_1},
    types::PendingOwner,
};

const NEW_OWNER: &str = "union1newowner";
const NEW_OWNER2: &str = "union1newowner2";

#[test]
fn ownership_transfer_works() {
    let mut deps = setup();

    let mut env = mock_env();
    env.block.time = Default::default();

    let res = execute(
        deps.as_mut(),
        env,
        message_info(&Addr::unchecked(ADMIN), &[]),
        ExecuteMsg::TransferOwnership {
            new_owner: NEW_OWNER.to_owned(),
        },
    )
    .unwrap();

    assert!(res.attributes.is_empty());
    assert!(res.messages.is_empty());
    assert_eq!(
        res.events,
        [Event::new("transfer_ownership")
            .add_attribute("new_owner", NEW_OWNER)
            .add_attribute("previous_owner", ADMIN)]
    );

    assert_eq!(
        deps.storage.read_item::<PendingOwnerStore>().unwrap(),
        PendingOwner {
            address: NEW_OWNER.to_owned(),
            owner_transfer_min_time_seconds: OWNERSHIP_CLAIM_DELAY_PERIOD_SECONDS
        }
    );
}

#[test]
fn ownership_transfer_overwrites() {
    let mut deps = setup();

    let mut env = mock_env();
    env.block.time = Default::default();

    let res = execute(
        deps.as_mut(),
        env.clone(),
        message_info(&Addr::unchecked(ADMIN), &[]),
        ExecuteMsg::TransferOwnership {
            new_owner: NEW_OWNER.to_owned(),
        },
    )
    .unwrap();

    assert!(res.attributes.is_empty());
    assert!(res.messages.is_empty());
    assert_eq!(
        res.events,
        [Event::new("transfer_ownership")
            .add_attribute("new_owner", NEW_OWNER)
            .add_attribute("previous_owner", ADMIN)]
    );

    assert_eq!(
        deps.storage.read_item::<PendingOwnerStore>().unwrap(),
        PendingOwner {
            address: NEW_OWNER.to_owned(),
            owner_transfer_min_time_seconds: OWNERSHIP_CLAIM_DELAY_PERIOD_SECONDS
        }
    );

    let res = execute(
        deps.as_mut(),
        env,
        message_info(&Addr::unchecked(ADMIN), &[]),
        ExecuteMsg::TransferOwnership {
            new_owner: NEW_OWNER2.to_owned(),
        },
    )
    .unwrap();

    assert!(res.attributes.is_empty());
    assert!(res.messages.is_empty());
    assert_eq!(
        res.events,
        [Event::new("transfer_ownership")
            .add_attribute("new_owner", NEW_OWNER2)
            // previous owner is still ADMIN
            .add_attribute("previous_owner", ADMIN)]
    );

    // correctly overwrites
    assert_eq!(
        deps.storage.read_item::<PendingOwnerStore>().unwrap(),
        PendingOwner {
            address: NEW_OWNER2.to_owned(),
            owner_transfer_min_time_seconds: OWNERSHIP_CLAIM_DELAY_PERIOD_SECONDS
        }
    );
}

#[test]
fn requires_admin() {
    let mut deps = setup();

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(UNION1), &[]),
            ExecuteMsg::TransferOwnership {
                new_owner: UNION2.to_owned(),
            },
        )
        .unwrap_err(),
        ContractError::Unauthorized {
            sender: Addr::unchecked(UNION1)
        }
    );

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            // monitor is also not allowed to transfer ownership, *only* admin
            message_info(&Addr::unchecked(UNION_MONITOR_1), &[]),
            ExecuteMsg::TransferOwnership {
                new_owner: UNION2.to_owned(),
            },
        )
        .unwrap_err(),
        ContractError::Unauthorized {
            sender: Addr::unchecked(UNION_MONITOR_1)
        }
    );
}

#[test]
fn revoke_requires_admin() {
    let mut deps = setup();

    execute(
        deps.as_mut(),
        mock_env(),
        message_info(&Addr::unchecked(ADMIN), &[]),
        ExecuteMsg::TransferOwnership {
            new_owner: NEW_OWNER.to_owned(),
        },
    )
    .unwrap();

    assert_eq!(
        execute(
            deps.as_mut(),
            mock_env(),
            message_info(&Addr::unchecked(UNION1), &[]),
            ExecuteMsg::RevokeOwnershipTransfer {},
        )
        .unwrap_err(),
        ContractError::Unauthorized {
            sender: Addr::unchecked(UNION1)
        }
    );
}

#[test]
fn revoke_works() {
    let mut deps = setup();

    let mut env = mock_env();
    env.block.time = Default::default();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&Addr::unchecked(ADMIN), &[]),
        ExecuteMsg::TransferOwnership {
            new_owner: NEW_OWNER.to_owned(),
        },
    )
    .unwrap();

    let res = execute(
        deps.as_mut(),
        env,
        message_info(&Addr::unchecked(ADMIN), &[]),
        ExecuteMsg::RevokeOwnershipTransfer {},
    )
    .unwrap();

    assert!(res.attributes.is_empty());
    assert!(res.messages.is_empty());
    assert_eq!(res.events, [Event::new("revoke_ownership_transfer")]);

    assert!(deps
        .storage
        .maybe_read_item::<PendingOwnerStore>()
        .unwrap()
        .is_none(),);
    assert_eq!(
        deps.storage.read_item::<Admin>().unwrap(),
        Addr::unchecked(ADMIN)
    );
}

#[test]
fn revoke_works_after_delay_period() {
    let mut deps = setup();

    let mut env = mock_env();
    env.block.time = Default::default();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&Addr::unchecked(ADMIN), &[]),
        ExecuteMsg::TransferOwnership {
            new_owner: NEW_OWNER.to_owned(),
        },
    )
    .unwrap();

    env.block.time = env
        .block
        .time
        .plus_seconds(OWNERSHIP_CLAIM_DELAY_PERIOD_SECONDS);

    let res = execute(
        deps.as_mut(),
        env,
        message_info(&Addr::unchecked(ADMIN), &[]),
        ExecuteMsg::RevokeOwnershipTransfer {},
    )
    .unwrap();

    assert!(res.attributes.is_empty());
    assert!(res.messages.is_empty());
    assert_eq!(res.events, [Event::new("revoke_ownership_transfer")]);

    assert!(deps
        .storage
        .maybe_read_item::<PendingOwnerStore>()
        .unwrap()
        .is_none(),);
    assert_eq!(
        deps.storage.read_item::<Admin>().unwrap(),
        Addr::unchecked(ADMIN)
    );
}

#[test]
fn claim_ownership_works() {
    let mut deps = setup();

    let mut env = mock_env();
    env.block.time = Default::default();

    execute(
        deps.as_mut(),
        env.clone(),
        message_info(&Addr::unchecked(ADMIN), &[]),
        ExecuteMsg::TransferOwnership {
            new_owner: NEW_OWNER.to_owned(),
        },
    )
    .unwrap();

    // can't claim yet
    assert_eq!(
        execute(
            deps.as_mut(),
            env.clone(),
            message_info(&Addr::unchecked(NEW_OWNER), &[]),
            ExecuteMsg::AcceptOwnership {},
        )
        .unwrap_err(),
        ContractError::OwnershipTransferNotReady {
            claimable_at_seconds: OWNERSHIP_CLAIM_DELAY_PERIOD_SECONDS,
            now_seconds: 0
        }
    );

    env.block.time = env
        .block
        .time
        .plus_seconds(OWNERSHIP_CLAIM_DELAY_PERIOD_SECONDS);

    let res = execute(
        deps.as_mut(),
        env,
        message_info(&Addr::unchecked(NEW_OWNER), &[]),
        ExecuteMsg::AcceptOwnership {},
    )
    .unwrap();

    assert!(res.attributes.is_empty());
    assert!(res.messages.is_empty());
    assert_eq!(
        res.events,
        [Event::new("accept_ownership").add_attribute("new_owner", NEW_OWNER)]
    );

    assert!(deps
        .storage
        .maybe_read_item::<PendingOwnerStore>()
        .unwrap()
        .is_none(),);
    assert_eq!(
        deps.storage.read_item::<Admin>().unwrap(),
        Addr::unchecked(NEW_OWNER)
    );
}
