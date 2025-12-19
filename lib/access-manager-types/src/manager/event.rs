use std::num::NonZero;

use cosmwasm_event::Event;
use cosmwasm_std::Addr;
use unionlabs_primitives::H256;

use crate::{Nonce, RoleId, Selector};

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
#[derive(Event)]
#[event("operation_scheduled")]
pub struct OperationScheduled<'a> {
    pub operation_id: H256,
    pub nonce: Nonce,
    pub schedule: NonZero<u64>,
    pub caller: &'a Addr,
    pub target: &'a Addr,
    pub data: &'a str,
}

/// A scheduled operation was executed.
///
/// ```solidity
/// event OperationExecuted(bytes32 indexed operationId, uint32 indexed nonce);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L22>
#[derive(Event)]
#[event("operation_executed")]
pub struct OperationExecuted {
    pub operation_id: H256,
    pub nonce: Nonce,
}

/// A scheduled operation was canceled.
///
/// ```solidity
/// event OperationCanceled(bytes32 indexed operationId, uint32 indexed nonce);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L27>
#[derive(Event)]
#[event("operation_canceled")]
pub struct OperationCanceled {
    pub operation_id: H256,
    pub nonce: Nonce,
}

/// Informational labeling for a role id.
///
/// ```solidity
/// event RoleLabel(uint64 indexed roleId, string label);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L32>
#[derive(Event)]
#[event("role_label")]
pub struct RoleLabel<'a> {
    pub role_id: RoleId,
    pub label: &'a str,
}

/// Emitted when `account` is granted `role_id`.
///
/// NOTE: The meaning of the `since` argument depends on the `new_member` argument. If the role is
/// granted to a new member, the `since` argument indicates when the account becomes a member of the
/// role, otherwise it indicates the execution delay for this account and role id is updated.
///
/// ```solidity
/// event RoleGranted(uint64 indexed roleId, address indexed account, uint32 delay, uint48 since, bool newMember);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L41>
#[derive(Event)]
#[event("role_granted")]
pub struct RoleGranted<'a> {
    pub role_id: RoleId,
    pub account: &'a Addr,
    pub delay: u32,
    pub since: u64,
    pub new_member: bool,
}

/// Emitted when `account` membership or `role_id` is revoked. Unlike granting, revoking is
/// instantaneous.
///
/// ```solidity
/// event RoleRevoked(uint64 indexed roleId, address indexed account);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L46>
#[derive(Event)]
#[event("role_revoked")]
pub struct RoleRevoked<'a> {
    pub role_id: RoleId,
    pub account: &'a Addr,
}

/// Role acting as admin over a given `role_id` is updated.
///
/// ```solidity
/// event RoleAdminChanged(uint64 indexed roleId, uint64 indexed admin);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L51>
#[derive(Event)]
#[event("role_admin_changed")]
pub struct RoleAdminChanged {
    pub role_id: RoleId,
    pub admin: RoleId,
}

/// Role acting as guardian over a given `role_id` is updated.
///
/// ```solidity
/// event RoleGuardianChanged(uint64 indexed roleId, uint64 indexed guardian);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L56>
#[derive(Event)]
#[event("role_guardian_changed")]
pub struct RoleGuardianChanged {
    pub role_id: RoleId,
    pub guardian: RoleId,
}

/// Grant delay for a given `role_id` will be updated to `delay` when `since` is reached.
///
/// ```solidity
/// event RoleGrantDelayChanged(uint64 indexed roleId, uint32 delay, uint48 since);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L61>
#[derive(Event)]
#[event("role_grant_delay_changed")]
pub struct RoleGrantDelayChanged {
    pub role_id: RoleId,
    pub delay: u32,
    pub since: u64,
}

/// Target mode is updated (true = closed, false = open).
///
/// ```solidity
/// event TargetClosed(address indexed target, bool closed);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L66>
#[derive(Event)]
#[event("target_closed")]
pub struct TargetClosed<'a> {
    pub target: &'a Addr,
    pub closed: bool,
}

/// Role required to invoke `selector` on `target` is updated to `role_id`.
///
/// ```solidity
/// event TargetFunctionRoleUpdated(address indexed target, bytes4 selector, uint64 indexed roleId);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L71>
#[derive(Event)]
#[event("target_function_role_updated")]
pub struct TargetFunctionRoleUpdated<'a> {
    pub target: &'a Addr,
    pub selector: &'a Selector,
    pub role_id: RoleId,
}

/// Admin delay for a given `target` will be updated to `delay` when `since` is reached.
///
/// ```solidity
/// event TargetAdminDelayUpdated(address indexed target, uint32 delay, uint48 since);
/// ```
///
/// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/IAccessManager.sol#L76>
#[derive(Event)]
#[event("target_admin_delay_updated")]
pub struct TargetAdminDelayUpdated<'a> {
    pub target: &'a Addr,
    pub delay: u32,
    pub since: u64,
}
