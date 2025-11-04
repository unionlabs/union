use cosmwasm_std::{
    Addr, from_json,
    testing::{message_info, mock_dependencies},
};

use super::*;

#[test]
fn not_paused_if_never_paused() {
    let deps = mock_dependencies();

    assert!(!is_paused(deps.as_ref()).unwrap());

    assert!(!from_json::<bool>(query(deps.as_ref(), &PausableQuery::Paused {}).unwrap()).unwrap());
}

#[test]
fn pause_unpause_works() {
    let mut deps = mock_dependencies();

    assert_eq!(
        execute(
            deps.as_mut(),
            &message_info(&Addr::unchecked("sender"), &[]),
            &Pausable::Pause {}
        )
        .unwrap(),
        Response::new().add_event(Paused {
            account: &Addr::unchecked("sender")
        })
    );

    assert!(is_paused(deps.as_ref()).unwrap());

    assert!(from_json::<bool>(query(deps.as_ref(), &PausableQuery::Paused {}).unwrap()).unwrap());

    assert_eq!(
        execute(
            deps.as_mut(),
            &message_info(&Addr::unchecked("sender"), &[]),
            &Pausable::Unpause {}
        )
        .unwrap(),
        Response::new().add_event(Unpaused {
            account: &Addr::unchecked("sender")
        })
    );

    assert!(!is_paused(deps.as_ref()).unwrap());

    assert!(!from_json::<bool>(query(deps.as_ref(), &PausableQuery::Paused {}).unwrap()).unwrap());
}

#[test]
fn when_not_paused() {
    let mut deps = mock_dependencies();

    WhenNotPaused(()).ensure_not_paused(deps.as_ref()).unwrap();

    execute(
        deps.as_mut(),
        &message_info(&Addr::unchecked("sender"), &[]),
        &Pausable::Pause {},
    )
    .unwrap();

    assert_eq!(
        WhenNotPaused(())
            .ensure_not_paused(deps.as_ref())
            .unwrap_err(),
        ContractError::ExpectedPause
    );
}

#[test]
fn when_paused() {
    let mut deps = mock_dependencies();

    assert_eq!(
        WhenPaused(()).ensure_paused(deps.as_ref()).unwrap_err(),
        ContractError::EnforcedPause
    );

    execute(
        deps.as_mut(),
        &message_info(&Addr::unchecked("sender"), &[]),
        &Pausable::Pause {},
    )
    .unwrap();

    WhenPaused(()).ensure_paused(deps.as_ref()).unwrap();
}
