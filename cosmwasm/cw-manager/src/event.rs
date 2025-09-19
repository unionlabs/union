use cosmwasm_std::{Addr, Event};
use unionlabs_primitives::{Bytes, H256};

use crate::types::{Selector, RoleId};

/// A delayed operation was scheduled.
///
/// ```solidity
/// event OperationScheduled(
///     bytes32 indexed operationId,
///     uint32 indexed nonce,
///     uint48 schedule,
///     address caller,
///     address target,
///     bytes data
/// );
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L7-L17>
pub(crate) fn operation_scheduled(
    operation_id: H256,
    nonce: u32,
    schedule: u64,
    caller: &Addr,
    target: &Addr,
    data: Bytes,
) -> Event {
    Event::new("operation_schedule")
        .add_attribute("operation_id", operation_id.to_string())
        .add_attribute("nonce", nonce.to_string())
        .add_attribute("schedule", schedule.to_string())
        .add_attribute("caller", caller)
        .add_attribute("target", target)
        .add_attribute("data", data.to_string())
}

/// A scheduled operation was executed.
///
/// ```solidity
/// event OperationExecuted(bytes32 indexed operationId, uint32 indexed nonce);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L19-L22>
pub(crate) fn operation_executed(operation_id: H256, nonce: u32) -> Event {
    Event::new("operation_executed")
        .add_attribute("operation_id", operation_id.to_string())
        .add_attribute("nonce", nonce.to_string())
}

/// A scheduled operation was canceled.
///
/// ```solidity
/// event OperationCanceled(bytes32 indexed operationId, uint32 indexed nonce);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L24-L27>
pub(crate) fn operation_canceled(operation_id: H256, nonce: u32) -> Event {
    Event::new("operation_canceled")
        .add_attribute("operation_id", operation_id.to_string())
        .add_attribute("nonce", nonce.to_string())
}

/// Informational labelling for a roleId.
///
/// ```solidity
/// event RoleLabel(uint64 indexed roleId, string label);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L29-L32>
pub(crate) fn role_label(role_id: RoleId, label: &str) -> Event {
    Event::new("role_label")
        .add_attribute("role_id", role_id.to_string())
        .add_attribute("label", label)
}

/// Emitted when `account` is granted `roleId`.
///
/// NOTE: The meaning of the `since` argument depends on the `newMember` argument.
/// If the role is granted to a new member, the `since` argument indicates when the account becomes a member of the role, otherwise it indicates the execution delay for this account and roleId is updated.
///
/// ```solidity
/// event RoleGranted(uint64 indexed roleId, address indexed account, uint32 delay, uint48 since, bool newMember);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L34-L41>
pub(crate) fn role_granted(
    role_id: RoleId,
    account: &Addr,
    delay: u32,
    since: u64,
    new_member: bool,
) -> Event {
    Event::new("role_granted")
        .add_attribute("role_id", role_id.to_string())
        .add_attribute("account", account)
        .add_attribute("delay", delay.to_string())
        .add_attribute("since", since.to_string())
        .add_attribute("new_member", new_member.to_string())
}

/// Emitted when `account` membership or `roleId` is revoked. Unlike granting, revoking is instantaneous.
///
/// ```solidity
/// event RoleRevoked(uint64 indexed roleId, address indexed account);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L43-L46>
pub(crate) fn role_revoked(role_id: RoleId, account: &Addr) -> Event {
    Event::new("role_revoked")
        .add_attribute("role_id", role_id.to_string())
        .add_attribute("account", account)
}

/// Role acting as admin over a given `roleId` is updated.
///
/// ```solidity
/// event RoleAdminChanged(uint64 indexed roleId, uint64 indexed admin);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L48-L51>
pub(crate) fn role_admin_changed(role_id: RoleId, admin: RoleId) -> Event {
    Event::new("role_admin_changed")
        .add_attribute("role_id", role_id.to_string())
        .add_attribute("admin", admin.to_string())
}

/// Role acting as guardian over a given `roleId` is updated.
///
/// ```solidity
/// event RoleGuardianChanged(uint64 indexed roleId, uint64 indexed guardian);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L53-L56>
pub(crate) fn role_guardian_changed(role_id: RoleId, guardian: RoleId) -> Event {
    Event::new("role_guardian_changed")
        .add_attribute("role_id", role_id.to_string())
        .add_attribute("guardian", guardian.to_string())
}

/// Grant delay for a given `roleId` will be updated to `delay` when `since` is reached.
///
/// ```solidity
/// event RoleGrantDelayChanged(uint64 indexed roleId, uint32 delay, uint48 since);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L58-L61>
pub(crate) fn role_grant_delay_changed(role_id: RoleId, delay: u32, since: u64) -> Event {
    Event::new("role_grant_delay_changed")
        .add_attribute("role_id", role_id.to_string())
        .add_attribute("delay", delay.to_string())
        .add_attribute("since", since.to_string())
}

/// Target mode is updated (true = closed, false = open).
///
/// ```solidity
/// event TargetClosed(address indexed target, bool closed);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L63-L66>
pub(crate) fn target_closed(target: &Addr, closed: bool) -> Event {
    Event::new("target_closed")
        .add_attribute("target", target)
        .add_attribute("closed", closed.to_string())
}

/// Role required to invoke `selector` on `target` is updated to `roleId`.
///
/// ```solidity
/// event TargetFunctionRoleUpdated(address indexed target, bytes4 selector, uint64 indexed roleId);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L68-L71>
pub(crate) fn target_function_role_updated(
    target: &Addr,
    method: &Selector,
    role_id: RoleId,
) -> Event {
    Event::new("target_function_role_updated")
        .add_attribute("target", target)
        .add_attribute("selector", method.to_string())
        .add_attribute("role_id", role_id.to_string())
}

/// Admin delay for a given `target` will be updated to `delay` when `since` is reached.
///
/// ```solidity
/// event TargetAdminDelayUpdated(address indexed target, uint32 delay, uint48 since);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L73-L76>
pub(crate) fn target_admin_delay_updated(target: &Addr, delay: u32, since: u64) -> Event {
    Event::new("target_admin_delay_updated")
        .add_attribute("target", target)
        .add_attribute("delay", delay.to_string())
        .add_attribute("since", since.to_string())
}
