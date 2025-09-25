use cosmwasm_std::{Addr, Event};
use unionlabs_primitives::H256;

use crate::types::{RoleId, Selector};

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
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L17>
pub struct OperationScheduled<'a> {
    pub operation_id: H256,
    pub nonce: u32,
    pub schedule: u64,
    pub caller: &'a Addr,
    pub target: &'a Addr,
    pub data: &'a str,
}

impl From<OperationScheduled<'_>> for Event {
    fn from(val: OperationScheduled<'_>) -> Self {
        Event::new("operation_schedule")
            .add_attribute("operation_id", val.operation_id.to_string())
            .add_attribute("nonce", val.nonce.to_string())
            .add_attribute("schedule", val.schedule.to_string())
            .add_attribute("caller", val.caller)
            .add_attribute("target", val.target)
            .add_attribute("data", val.data)
    }
}

/// A scheduled operation was executed.
///
/// ```solidity
/// event OperationExecuted(bytes32 indexed operationId, uint32 indexed nonce);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L22>
pub struct OperationExecuted {
    pub operation_id: H256,
    pub nonce: u32,
}

impl From<OperationExecuted> for Event {
    fn from(val: OperationExecuted) -> Self {
        Event::new("operation_executed")
            .add_attribute("operation_id", val.operation_id.to_string())
            .add_attribute("nonce", val.nonce.to_string())
    }
}

/// A scheduled operation was canceled.
///
/// ```solidity
/// event OperationCanceled(bytes32 indexed operationId, uint32 indexed nonce);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L27>
pub struct OperationCanceled {
    pub operation_id: H256,
    pub nonce: u32,
}

impl From<OperationCanceled> for Event {
    fn from(val: OperationCanceled) -> Self {
        Event::new("operation_canceled")
            .add_attribute("operation_id", val.operation_id.to_string())
            .add_attribute("nonce", val.nonce.to_string())
    }
}

/// Informational labeling for a role id.
///
/// ```solidity
/// event RoleLabel(uint64 indexed roleId, string label);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L32>
pub struct RoleLabel<'a> {
    pub role_id: RoleId,
    pub label: &'a str,
}

impl From<RoleLabel<'_>> for Event {
    fn from(val: RoleLabel<'_>) -> Self {
        Event::new("role_label")
            .add_attribute("role_id", val.role_id.to_string())
            .add_attribute("label", val.label)
    }
}

/// Emitted when `account` is granted `role_id`.
///
/// NOTE: The meaning of the `since` argument depends on the `new_member` argument. If the role is granted to a new member, the `since` argument indicates when the account becomes a member of the role, otherwise it indicates the execution delay for this account and role id is updated.
///
/// ```solidity
/// event RoleGranted(uint64 indexed roleId, address indexed account, uint32 delay, uint48 since, bool newMember);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L41>
pub struct RoleGranted<'a> {
    pub role_id: RoleId,
    pub account: &'a Addr,
    pub delay: u32,
    pub since: u64,
    pub new_member: bool,
}

impl From<RoleGranted<'_>> for Event {
    fn from(val: RoleGranted<'_>) -> Self {
        Event::new("role_granted")
            .add_attribute("role_id", val.role_id.to_string())
            .add_attribute("account", val.account)
            .add_attribute("delay", val.delay.to_string())
            .add_attribute("since", val.since.to_string())
            .add_attribute("new_member", val.new_member.to_string())
    }
}

/// Emitted when `account` membership or `role_id` is revoked. Unlike granting, revoking is
/// instantaneous.
///
/// ```solidity
/// event RoleRevoked(uint64 indexed roleId, address indexed account);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L46>
pub struct RoleRevoked<'a> {
    pub role_id: RoleId,
    pub account: &'a Addr,
}

impl From<RoleRevoked<'_>> for Event {
    fn from(val: RoleRevoked<'_>) -> Self {
        Event::new("role_revoked")
            .add_attribute("role_id", val.role_id.to_string())
            .add_attribute("account", val.account)
    }
}

/// Role acting as admin over a given `role_id` is updated.
///
/// ```solidity
/// event RoleAdminChanged(uint64 indexed roleId, uint64 indexed admin);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L51>
pub struct RoleAdminChanged {
    pub role_id: RoleId,
    pub admin: RoleId,
}

impl From<RoleAdminChanged> for Event {
    fn from(val: RoleAdminChanged) -> Self {
        Event::new("role_admin_changed")
            .add_attribute("role_id", val.role_id.to_string())
            .add_attribute("admin", val.admin.to_string())
    }
}

/// Role acting as guardian over a given `role_id` is updated.
///
/// ```solidity
/// event RoleGuardianChanged(uint64 indexed roleId, uint64 indexed guardian);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L56>
pub struct RoleGuardianChanged {
    pub role_id: RoleId,
    pub guardian: RoleId,
}

impl From<RoleGuardianChanged> for Event {
    fn from(val: RoleGuardianChanged) -> Self {
        Event::new("role_guardian_changed")
            .add_attribute("role_id", val.role_id.to_string())
            .add_attribute("guardian", val.guardian.to_string())
    }
}

/// Grant delay for a given `role_id` will be updated to `delay` when `since` is reached.
///
/// ```solidity
/// event RoleGrantDelayChanged(uint64 indexed roleId, uint32 delay, uint48 since);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L61>
pub struct RoleGrantDelayChanged {
    pub role_id: RoleId,
    pub delay: u32,
    pub since: u64,
}

impl From<RoleGrantDelayChanged> for Event {
    fn from(val: RoleGrantDelayChanged) -> Self {
        Event::new("role_grant_delay_changed")
            .add_attribute("role_id", val.role_id.to_string())
            .add_attribute("delay", val.delay.to_string())
            .add_attribute("since", val.since.to_string())
    }
}

/// Target mode is updated (true = closed, false = open).
///
/// ```solidity
/// event TargetClosed(address indexed target, bool closed);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L66>
pub struct TargetClosed<'a> {
    pub target: &'a Addr,
    pub closed: bool,
}

impl From<TargetClosed<'_>> for Event {
    fn from(val: TargetClosed<'_>) -> Self {
        Event::new("target_closed")
            .add_attribute("target", val.target)
            .add_attribute("closed", val.closed.to_string())
    }
}

/// Role required to invoke `selector` on `target` is updated to `role_id`.
///
/// ```solidity
/// event TargetFunctionRoleUpdated(address indexed target, bytes4 selector, uint64 indexed roleId);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L71>
pub struct TargetFunctionRoleUpdated<'a> {
    pub target: &'a Addr,
    pub selector: &'a Selector,
    pub role_id: RoleId,
}

impl From<TargetFunctionRoleUpdated<'_>> for Event {
    fn from(val: TargetFunctionRoleUpdated<'_>) -> Self {
        Event::new("target_function_role_updated")
            .add_attribute("target", val.target)
            .add_attribute("selector", val.selector.to_string())
            .add_attribute("role_id", val.role_id.to_string())
    }
}

/// Admin delay for a given `target` will be updated to `delay` when `since` is reached.
///
/// ```solidity
/// event TargetAdminDelayUpdated(address indexed target, uint32 delay, uint48 since);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L76>
pub struct TargetAdminDelayUpdated<'a> {
    pub target: &'a Addr,
    pub delay: u32,
    pub since: u64,
}

impl From<TargetAdminDelayUpdated<'_>> for Event {
    fn from(val: TargetAdminDelayUpdated<'_>) -> Self {
        Event::new("target_admin_delay_updated")
            .add_attribute("target", val.target)
            .add_attribute("delay", val.delay.to_string())
            .add_attribute("since", val.since.to_string())
    }
}
