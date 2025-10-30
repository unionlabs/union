use cosmwasm_std::Addr;
use serde::{Deserialize, Serialize};
use unionlabs_primitives::H256;

#[cfg(doc)]
use crate::manager::event::*;
use crate::{RoleId, Selector};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub struct InitMsg {
    pub initial_admin: Addr,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// The identifier of the admin role. Required to perform most configuration operations
    /// including other roles' management and target restrictions.
    ///
    /// ```solidity
    /// uint64 public constant ADMIN_ROLE = type(uint64).min; // 0
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L104>
    AdminRole {},

    /// The identifier of the public role. Automatically granted to all addresses with no delay.
    ///
    /// ```solidity
    /// uint64 public constant PUBLIC_ROLE = type(uint64).max; // 2**64-1
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L109>
    PublicRole {},

    /// Expiration delay for scheduled proposals. Defaults to 1 week.
    ///
    /// IMPORTANT: Avoid overriding the expiration with 0. Otherwise every contract proposal will
    /// be expired immediately, disabling any scheduling usage.
    ///
    /// ```solidity
    /// function expiration() public view virtual returns (uint32)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L158>
    Expiration {},

    /// Minimum setback for all delay updates, with the exception of execution delays. It can be
    /// increased without setback (and reset via [`ExecuteMsg::RevokeRole`] in the case event of an
    /// accidental increase). Defaults to 5 days.
    ///
    /// ```solidity
    /// function minSetback() public view virtual returns (uint32)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L163>
    MinSetback {},

    /// Check if an address (`caller`) is authorized to call a given function on a given contract
    /// directly (with no restriction). Additionally, it returns the delay needed to perform the
    /// call indirectly through the [`ExecuteMsg::Schedule`] & [`ExecuteMsg::Execute`] workflow.
    ///
    /// This function is usually called by the targeted contract to control immediate execution of
    /// restricted functions. Therefore we only return true if the call can be performed without
    /// any delay. If the call is subject to a previously set delay (not zero), then the function
    /// should return false and the caller should schedule the operation for future execution.
    ///
    /// If `allowed` is true, the delay can be disregarded and the operation can be immediately
    /// executed, otherwise the operation can be executed if and only if delay is greater than 0.
    ///
    /// NOTE: This function does not report the permissions of the admin functions in the manager
    /// itself. These are defined by the [`access-manager`][crate] documentation.
    ///
    /// ```solidity
    /// function canCall(
    ///     address caller,
    ///     address target,
    ///     bytes4 selector
    /// ) public view virtual returns (bool immediate, uint32 delay)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L139>
    CanCall {
        selector: Box<Selector>,
        target: Addr,
        caller: Addr,
    },

    /// Get whether the contract is closed disabling any access. Otherwise role permissions are
    /// applied.
    ///
    /// NOTE: When the manager itself is closed, admin functions are still accessible to avoid
    /// locking the contract.
    ///
    /// ```solidity
    /// function isTargetClosed(address target) public view virtual returns (bool)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L168>
    IsTargetClosed { target: Addr },

    /// Get the role required to call a function.
    ///
    /// ```solidity
    /// function getTargetFunctionRole(address target, bytes4 selector) public view virtual returns (uint64)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L173>
    GetTargetFunctionRole {
        target: Addr,
        selector: Box<Selector>,
    },

    /// Get the admin delay for a target contract. Changes to contract configuration are subject to
    /// this delay.
    ///
    /// ```solidity
    /// function getTargetAdminDelay(address target) public view virtual returns (uint32)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L178>
    GetTargetAdminDelay { target: Addr },

    /// Get the id of the role that acts as an admin for the given role.
    ///
    /// The admin permission is required to grant the role, revoke the role and update the
    /// execution delay to execute an operation that is restricted to this role.
    ///
    /// ```solidity
    /// function getRoleAdmin(uint64 roleId) public view virtual returns (uint64)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L183>
    GetRoleAdmin { role_id: RoleId },

    /// Get the role that acts as a guardian for a given role.
    ///
    /// The guardian permission allows canceling operations that have been scheduled under the
    /// role.
    ///
    /// ```solidity
    /// function getRoleGuardian(uint64 roleId) public view virtual returns (uint64)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L188>
    GetRoleGuardian { role_id: RoleId },

    /// Get the role current grant delay.
    ///
    /// Its value may change at any point without an event emitted following a call to
    /// [`ExecuteMsg::SetGrantDelay`]. Changes to this value, including effect timepoint are
    /// notified in advance by the [`RoleGrantDelayChanged`] event.
    ///
    /// ```solidity
    /// function getRoleGrantDelay(uint64 roleId) public view virtual returns (uint32)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L193>
    GetRoleGrantDelay { role_id: RoleId },

    /// Get the access details for a given account for a given role. These details include the
    /// timepoint at which membership becomes active, and the delay applied to all operation by
    /// this user that requires this permission level.
    ///
    /// ```solidity
    /// function getAccess(
    ///     uint64 roleId,
    ///     address account
    /// ) public view virtual returns (uint48 since, uint32 currentDelay, uint32 pendingDelay, uint48 effect)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L198>
    GetAccess { role_id: RoleId, account: Addr },

    /// Check if a given account currently has the permission level corresponding to a given role.
    /// Note that this permission might be associated with an execution delay.
    /// [`QueryMsg::GetAccess`] can provide more details.
    ///
    /// ```solidity
    /// function hasRole(
    ///     uint64 roleId,
    ///     address account
    /// ) public view virtual returns (bool isMember, uint32 executionDelay)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L211>
    HasRole { role_id: RoleId, account: Addr },

    /// Return the timepoint at which a scheduled operation will be ready for execution. This
    /// returns 0 if the operation is not yet scheduled, has expired, was executed, or was
    /// canceled.
    ///
    /// ```solidity
    /// function getSchedule(bytes32 id) public view virtual returns (uint48)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L429>
    GetSchedule { id: H256 },

    /// Return the nonce for the latest scheduled operation with a given id. Returns 0 if the
    /// operation has never been scheduled.
    ///
    /// ```solidity
    /// function getNonce(bytes32 id) public view virtual returns (uint32)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L435>
    GetNonce { id: H256 },

    /// Hashing function for execute protection.
    ///
    /// ```solidity
    /// function hashOperation(address caller, address target, bytes calldata data) public view virtual returns (bytes32)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L582>
    HashOperation {
        caller: Addr,
        target: Addr,
        data: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Give a label to a role, for improved role discoverability by UIs.
    ///
    /// Requirements:
    ///
    /// - the caller must be a global admin
    ///
    /// Emits a [`RoleLabel`] event.
    ///
    /// ```solidity
    /// function labelRole(uint64 roleId, string calldata label) public virtual onlyAuthorized
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L225>
    LabelRole { role_id: RoleId, label: String },

    /// Add `account` to `role_id`, or change its execution delay.
    ///
    /// This gives the account the authorization to call any function that is restricted to this
    /// role. An optional execution delay (in seconds) can be set. If that delay is non 0, the user
    /// is required to schedule any operation that is restricted to members of this role. The user
    /// will only be able to execute the operation after the delay has passed, before it has
    /// expired. During this period, admin and guardians can cancel the operation (see
    /// [`ExecuteMsg::Cancel`]).
    ///
    /// If the account has already been granted this role, the execution delay will be updated.
    /// This update is not immediate and follows the delay rules. For example, if a user currently
    /// has a delay of 3 hours, and this is called to reduce that delay to 1 hour, the new delay
    /// will take some time to take effect, enforcing that any operation executed in the 3 hours
    /// that follows this update was indeed scheduled before this update.
    ///
    /// Requirements:
    ///
    /// - the caller must be an admin for the role (see [`QueryMsg::GetRoleAdmin`])
    /// - granted role must not be the [`RoleId::PUBLIC_ROLE`]
    ///
    /// Emits a [`RoleGranted`] event.
    ///
    /// ```solidity
    /// function grantRole(uint64 roleId, address account, uint32 executionDelay) public virtual onlyAuthorized
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L233>
    GrantRole {
        role_id: RoleId,
        account: Addr,
        execution_delay: u32,
    },

    /// Remove an account from a role, with immediate effect. If the account does not have the
    /// role, this call has no effect.
    ///
    /// Requirements:
    ///
    /// - the caller must be an admin for the role (see [`QueryMsg::GetRoleAdmin`])
    /// - revoked role must not be the [`RoleId::PUBLIC_ROLE`]
    ///
    /// Emits a [`RoleRevoked`] event if the account had the role.
    ///
    /// ```solidity
    /// function revokeRole(uint64 roleId, address account) public virtual onlyAuthorized
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L238>
    RevokeRole { role_id: RoleId, account: Addr },

    /// Renounce role permissions for the calling account with immediate effect. If the sender is
    /// not in the role this call has no effect.
    ///
    /// Requirements:
    ///
    /// - the caller must be `caller_confirmation`.
    ///
    /// Emits a [`RoleRevoked`] event if the account had the role.
    ///
    /// ```solidity
    /// function renounceRole(uint64 roleId, address callerConfirmation) public virtual
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L243>
    RenounceRole {
        role_id: RoleId,
        caller_confirmation: Addr,
    },

    /// Change admin role for a given role.
    ///
    /// Requirements:
    ///
    /// - the caller must be a global admin
    ///
    /// Emits a [`RoleAdminChanged`] event.
    ///
    /// ```solidity
    /// function setRoleAdmin(uint64 roleId, uint64 admin) public virtual onlyAuthorized
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L251>
    SetRoleAdmin { role_id: RoleId, admin: RoleId },

    /// Change guardian role for a given role.
    ///
    /// Requirements:
    ///
    /// - the caller must be a global admin
    ///
    /// Emits a [`RoleGuardianChanged`] event.
    ///
    /// ```solidity
    /// function setRoleGuardian(uint64 roleId, uint64 guardian) public virtual onlyAuthorized
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L256>
    SetRoleGuardian { role_id: RoleId, guardian: RoleId },

    /// Update the delay for granting a `roleId`.
    ///
    /// Requirements:
    ///
    /// - the caller must be a global admin
    ///
    /// Emits a [`RoleGrantDelayChanged`] event.
    ///
    /// ```solidity
    /// function setGrantDelay(uint64 roleId, uint32 newDelay) public virtual onlyAuthorized
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L261>
    SetGrantDelay { role_id: RoleId, grant_delay: u32 },

    /// Set the delay for changing the configuration of a given target contract.
    ///
    /// Requirements:
    ///
    /// - the caller must be a global admin
    ///
    /// Emits a [`TargetAdminDelayUpdated`] event.
    ///
    /// ```solidity
    /// function setTargetAdminDelay(address target, uint32 newDelay) public virtual onlyAuthorized
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L395>
    SetTargetAdminDelay { target: Addr, new_delay: u32 },

    /// Set the closed flag for a contract.
    ///
    /// Closing the manager itself won't disable access to admin methods to avoid locking the
    /// contract.
    ///
    /// Requirements:
    ///
    /// - the caller must be a global admin
    ///
    /// Emits a [`TargetClosed`] event.
    ///
    /// ```solidity
    /// function setTargetClosed(address target, bool closed) public virtual onlyAuthorized
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L413>
    SetTargetClosed { target: Addr, closed: bool },

    /// Set the role required to call functions identified by the `selectors` in the `target`
    /// contract.
    ///
    /// Requirements:
    ///
    /// - the caller must be a global admin
    ///
    /// Emits a [`TargetFunctionRoleUpdated`] event per selector.
    ///
    /// ```solidity
    /// function setTargetFunctionRole(
    ///     address target,
    ///     bytes4[] calldata selectors,
    ///     uint64 roleId
    /// ) public virtual onlyAuthorized {
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L374>
    SetTargetFunctionRole {
        target: Addr,
        selectors: Vec<Box<Selector>>,
        role_id: RoleId,
    },

    /// Changes the authority of a target managed by this manager instance.
    ///
    /// Requirements:
    ///
    /// - the caller must be a global admin
    ///
    /// ```solidity
    /// function updateAuthority(address target, address newAuthority) public virtual onlyAuthorized
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L588>
    UpdateAuthority { target: Addr, new_authority: Addr },

    /// Schedule a delayed operation for future execution, and return the operation identifier. It
    /// is possible to choose the timestamp at which the operation becomes executable as long as it
    /// satisfies the execution delays required for the caller. The special value zero will
    /// automatically set the earliest possible time.
    ///
    /// Returns the `operationId` that was scheduled. Since this value is a hash of the parameters,
    /// it can reoccur when the same parameters are used; if this is relevant, the returned `nonce`
    /// can be used to uniquely identify this scheduled operation from other occurrences of the
    /// same `operationId` in invocations of [`ExecuteMsg::Execute`] and [`ExecuteMsg::Cancel`].
    ///
    /// Emits an [`OperationScheduled`] event.
    ///
    /// NOTE: It is not possible to concurrently schedule more than one operation with the same
    /// `target` and `data`. If this is necessary, additional whitespace can be appended to the
    /// message to act as a salt, since whitespace is insignificant for JSON decoding.
    ///
    /// ```solidity
    /// function schedule(
    ///     address target,
    ///     bytes calldata data,
    ///     uint48 when
    /// ) public virtual returns (bytes32 operationId, uint32 nonce)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L440>
    Schedule {
        target: Addr,
        data: String,
        when: u64,
    },

    /// Cancel a scheduled (delayed) operation. Returns the nonce that identifies the previously
    /// scheduled operation that is cancelled.
    ///
    /// Requirements:
    ///
    /// - the caller must be the proposer, a guardian of the targeted function, or a global admin
    ///
    /// Emits an [`OperationCanceled`] event.
    ///
    /// ```solidity
    /// function cancel(address caller, address target, bytes calldata data) external returns (uint32)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L526>
    Cancel {
        caller: Addr,
        target: Addr,
        data: String,
    },

    /// Execute a function that is delay restricted, provided it was properly scheduled beforehand,
    /// or the execution delay is 0.
    ///
    /// Returns the nonce that identifies the previously scheduled operation that is executed, or 0
    /// if the operation wasn't previously scheduled (if the caller doesn't have an execution
    /// delay).
    ///
    /// Emits an [`OperationExecuted`] event only if the call was scheduled and delayed.
    ///
    /// ```solidity
    /// function execute(address target, bytes calldata data) public payable virtual returns (uint32)
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L492>
    Execute { target: Addr, data: String },

    /// Consume a scheduled operation targeting the caller. If such an operation exists, mark it as
    /// consumed (emit an [`OperationExecuted`] event and clean the state). Otherwise, throw an
    /// error.
    ///
    /// This is useful for contract that want to enforce that calls targeting them were scheduled
    /// on the manager, with all the verifications that it implies.
    ///
    /// Emits an [`OperationExecuted`] event.
    ///
    /// ```solidity
    /// function consumeScheduledOp(address caller, bytes calldata data) public virtual
    /// ```
    ///
    /// <https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.4.0/contracts/access/manager/AccessManager.sol#L550>
    ConsumeScheduledOp { caller: Addr, data: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[serde(rename_all = "snake_case")]
pub struct MigrateMsg {}
