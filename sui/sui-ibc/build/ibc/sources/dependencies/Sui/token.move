// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

/// The Token module which implements a Closed Loop Token with a configurable
/// policy. The policy is defined by a set of rules that must be satisfied for
/// an action to be performed on the token.
///
/// The module is designed to be used with a `TreasuryCap` to allow for minting
/// and burning of the `Token`s. And can act as a replacement / extension or a
/// companion to existing open-loop (`Coin`) systems.
///
/// ```
/// Module:      sui::balance       sui::coin             sui::token
/// Main type:   Balance<T>         Coin<T>               Token<T>
/// Capability:  Supply<T>  <---->  TreasuryCap<T> <----> TreasuryCap<T>
/// Abilities:   store              key + store           key
/// ```
///
/// The Token system allows for fine-grained control over the actions performed
/// on the token. And hence it is highly suitable for applications that require
/// control over the currency which a simple open-loop system can't provide.
module sui::token;

use std::{string::String, type_name::{Self, TypeName}};
use sui::{
    balance::{Self, Balance},
    coin::{Coin, TreasuryCap},
    dynamic_field as df,
    event,
    vec_map::{Self, VecMap},
    vec_set::{Self, VecSet}
};

/// The action is not allowed (defined) in the policy.
const EUnknownAction: u64 = 0;
/// The rule was not approved.
const ENotApproved: u64 = 1;
/// Trying to perform an admin action with a wrong cap.
const ENotAuthorized: u64 = 2;
/// The balance is too low to perform the action.
const EBalanceTooLow: u64 = 3;
/// The balance is not zero.
const ENotZero: u64 = 4;
/// The balance is not zero when trying to confirm with `TransferPolicyCap`.
const ECantConsumeBalance: u64 = 5;
/// Rule is trying to access a missing config (with type).
const ENoConfig: u64 = 6;
/// Using `confirm_request_mut` without `spent_balance`. Immutable version
/// of the function must be used instead.
const EUseImmutableConfirm: u64 = 7;

// === Protected Actions ===

/// A Tag for the `spend` action.
const SPEND: vector<u8> = b"spend";
/// A Tag for the `transfer` action.
const TRANSFER: vector<u8> = b"transfer";
/// A Tag for the `to_coin` action.
const TO_COIN: vector<u8> = b"to_coin";
/// A Tag for the `from_coin` action.
const FROM_COIN: vector<u8> = b"from_coin";

/// A single `Token` with `Balance` inside. Can only be owned by an address,
/// and actions performed on it must be confirmed in a matching `TokenPolicy`.
public struct Token<phantom T> has key {
    id: UID,
    /// The Balance of the `Token`.
    balance: Balance<T>,
}

/// A Capability that manages a single `TokenPolicy` specified in the `for`
/// field. Created together with `TokenPolicy` in the `new` function.
public struct TokenPolicyCap<phantom T> has key, store { id: UID, `for`: ID }

/// `TokenPolicy` represents a set of rules that define what actions can be
/// performed on a `Token` and which `Rules` must be satisfied for the
/// action to succeed.
///
/// - For the sake of availability, `TokenPolicy` is a `key`-only object.
/// - Each `TokenPolicy` is managed by a matching `TokenPolicyCap`.
/// - For an action to become available, there needs to be a record in the
/// `rules` VecMap. To allow an action to be performed freely, there's an
/// `allow` function that can be called by the `TokenPolicyCap` owner.
public struct TokenPolicy<phantom T> has key {
    id: UID,
    /// The balance that is effectively spent by the user on the "spend"
    /// action. However, actual decrease of the supply can only be done by
    /// the `TreasuryCap` owner when `flush` is called.
    ///
    /// This balance is effectively spent and cannot be accessed by anyone
    /// but the `TreasuryCap` owner.
    spent_balance: Balance<T>,
    /// The set of rules that define what actions can be performed on the
    /// token. For each "action" there's a set of Rules that must be
    /// satisfied for the `ActionRequest` to be confirmed.
    rules: VecMap<String, VecSet<TypeName>>,
}

/// A request to perform an "Action" on a token. Stores the information
/// about the action to be performed and must be consumed by the `confirm_request`
/// or `confirm_request_mut` functions when the Rules are satisfied.
public struct ActionRequest<phantom T> {
    /// Name of the Action to look up in the Policy. Name can be one of the
    /// default actions: `transfer`, `spend`, `to_coin`, `from_coin` or a
    /// custom action.
    name: String,
    /// Amount is present in all of the txs
    amount: u64,
    /// Sender is a permanent field always
    sender: address,
    /// Recipient is only available in `transfer` action.
    recipient: Option<address>,
    /// The balance to be "spent" in the `TokenPolicy`, only available
    /// in the `spend` action.
    spent_balance: Option<Balance<T>>,
    /// Collected approvals (stamps) from completed `Rules`. They're matched
    /// against `TokenPolicy.rules` to determine if the request can be
    /// confirmed.
    approvals: VecSet<TypeName>,
}

/// Dynamic field key for the `TokenPolicy` to store the `Config` for a
/// specific action `Rule`. There can be only one configuration per
/// `Rule` per `TokenPolicy`.
public struct RuleKey<phantom T> has store, copy, drop { is_protected: bool }

/// An event emitted when a `TokenPolicy` is created and shared. Because
/// `TokenPolicy` can only be shared (and potentially frozen in the future),
/// we emit this event in the `share_policy` function and mark it as mutable.
public struct TokenPolicyCreated<phantom T> has copy, drop {
    /// ID of the `TokenPolicy` that was created.
    id: ID,
    /// Whether the `TokenPolicy` is "shared" (mutable) or "frozen"
    /// (immutable) - TBD.
    is_mutable: bool,
}

/// Create a new `TokenPolicy` and a matching `TokenPolicyCap`.
/// The `TokenPolicy` must then be shared using the `share_policy` method.
///
/// `TreasuryCap` guarantees full ownership over the currency, and is unique,
/// hence it is safe to use it for authorization.
public fun new_policy<T>(
    _treasury_cap: &TreasuryCap<T>,
    ctx: &mut TxContext,
): (TokenPolicy<T>, TokenPolicyCap<T>) {
    let policy = TokenPolicy {
        id: object::new(ctx),
        spent_balance: balance::zero(),
        rules: vec_map::empty(),
    };

    let cap = TokenPolicyCap {
        id: object::new(ctx),
        `for`: object::id(&policy),
    };

    (policy, cap)
}

/// Share the `TokenPolicy`. Due to `key`-only restriction, it must be
/// shared after initialization.
public fun share_policy<T>(policy: TokenPolicy<T>) {
    event::emit(TokenPolicyCreated<T> {
        id: object::id(&policy),
        is_mutable: true,
    });

    transfer::share_object(policy)
}

// === Protected Actions ===

/// Transfer a `Token` to a `recipient`. Creates an `ActionRequest` for the
/// "transfer" action. The `ActionRequest` contains the `recipient` field
/// to be used in verification.
public fun transfer<T>(t: Token<T>, recipient: address, ctx: &mut TxContext): ActionRequest<T> {
    let amount = t.balance.value();
    transfer::transfer(t, recipient);

    new_request(
        transfer_action(),
        amount,
        option::some(recipient),
        option::none(),
        ctx,
    )
}

/// Spend a `Token` by unwrapping it and storing the `Balance` in the
/// `ActionRequest` for the "spend" action. The `ActionRequest` contains
/// the `spent_balance` field to be used in verification.
///
/// Spend action requires `confirm_request_mut` to be called to confirm the
/// request and join the spent balance with the `TokenPolicy.spent_balance`.
public fun spend<T>(t: Token<T>, ctx: &mut TxContext): ActionRequest<T> {
    let Token { id, balance } = t;
    id.delete();

    new_request(
        spend_action(),
        balance.value(),
        option::none(),
        option::some(balance),
        ctx,
    )
}

/// Convert `Token` into an open `Coin`. Creates an `ActionRequest` for the
/// "to_coin" action.
public fun to_coin<T>(t: Token<T>, ctx: &mut TxContext): (Coin<T>, ActionRequest<T>) {
    let Token { id, balance } = t;
    let amount = balance.value();
    id.delete();

    (
        balance.into_coin(ctx),
        new_request(
            to_coin_action(),
            amount,
            option::none(),
            option::none(),
            ctx,
        ),
    )
}

/// Convert an open `Coin` into a `Token`. Creates an `ActionRequest` for
/// the "from_coin" action.
public fun from_coin<T>(coin: Coin<T>, ctx: &mut TxContext): (Token<T>, ActionRequest<T>) {
    let amount = coin.value();
    let token = Token {
        id: object::new(ctx),
        balance: coin.into_balance(),
    };

    (
        token,
        new_request(
            from_coin_action(),
            amount,
            option::none(),
            option::none(),
            ctx,
        ),
    )
}

// === Public Actions ===

/// Join two `Token`s into one, always available.
public fun join<T>(token: &mut Token<T>, another: Token<T>) {
    let Token { id, balance } = another;
    token.balance.join(balance);
    id.delete();
}

/// Split a `Token` with `amount`.
/// Aborts if the `Token.balance` is lower than `amount`.
public fun split<T>(token: &mut Token<T>, amount: u64, ctx: &mut TxContext): Token<T> {
    assert!(token.balance.value() >= amount, EBalanceTooLow);
    Token {
        id: object::new(ctx),
        balance: token.balance.split(amount),
    }
}

/// Create a zero `Token`.
public fun zero<T>(ctx: &mut TxContext): Token<T> {
    Token {
        id: object::new(ctx),
        balance: balance::zero(),
    }
}

/// Destroy an empty `Token`, fails if the balance is non-zero.
/// Aborts if the `Token.balance` is not zero.
public fun destroy_zero<T>(token: Token<T>) {
    let Token { id, balance } = token;
    assert!(balance.value() == 0, ENotZero);
    balance.destroy_zero();
    id.delete();
}

#[allow(lint(self_transfer))]
/// Transfer the `Token` to the transaction sender.
public fun keep<T>(token: Token<T>, ctx: &mut TxContext) {
    transfer::transfer(token, ctx.sender())
}

// === Request Handling ===

/// Create a new `ActionRequest`.
/// Publicly available method to allow for custom actions.
public fun new_request<T>(
    name: String,
    amount: u64,
    recipient: Option<address>,
    spent_balance: Option<Balance<T>>,
    ctx: &TxContext,
): ActionRequest<T> {
    ActionRequest {
        name,
        amount,
        recipient,
        spent_balance,
        sender: ctx.sender(),
        approvals: vec_set::empty(),
    }
}

/// Confirm the request against the `TokenPolicy` and return the parameters
/// of the request: (Name, Amount, Sender, Recipient).
///
/// Cannot be used for `spend` and similar actions that deliver `spent_balance`
/// to the `TokenPolicy`. For those actions use `confirm_request_mut`.
///
/// Aborts if:
/// - the action is not allowed (missing record in `rules`)
/// - action contains `spent_balance` (use `confirm_request_mut`)
/// - the `ActionRequest` does not meet the `TokenPolicy` rules for the action
public fun confirm_request<T>(
    policy: &TokenPolicy<T>,
    request: ActionRequest<T>,
    _ctx: &mut TxContext,
): (String, u64, address, Option<address>) {
    assert!(request.spent_balance.is_none(), ECantConsumeBalance);
    assert!(policy.rules.contains(&request.name), EUnknownAction);

    let ActionRequest {
        name,
        approvals,
        spent_balance,
        amount,
        sender,
        recipient,
    } = request;

    spent_balance.destroy_none();

    let rules = &(*policy.rules.get(&name)).into_keys();
    let rules_len = rules.length();
    let mut i = 0;

    while (i < rules_len) {
        let rule = &rules[i];
        assert!(approvals.contains(rule), ENotApproved);
        i = i + 1;
    };

    (name, amount, sender, recipient)
}

/// Confirm the request against the `TokenPolicy` and return the parameters
/// of the request: (Name, Amount, Sender, Recipient).
///
/// Unlike `confirm_request` this function requires mutable access to the
/// `TokenPolicy` and must be used on `spend` action. After dealing with the
/// spent balance it calls `confirm_request` internally.
///
/// See `confirm_request` for the list of abort conditions.
public fun confirm_request_mut<T>(
    policy: &mut TokenPolicy<T>,
    mut request: ActionRequest<T>,
    ctx: &mut TxContext,
): (String, u64, address, Option<address>) {
    assert!(policy.rules.contains(&request.name), EUnknownAction);
    assert!(request.spent_balance.is_some(), EUseImmutableConfirm);

    policy.spent_balance.join(request.spent_balance.extract());

    confirm_request(policy, request, ctx)
}

/// Confirm an `ActionRequest` as the `TokenPolicyCap` owner. This function
/// allows `TokenPolicy` owner to perform Capability-gated actions ignoring
/// the ruleset specified in the `TokenPolicy`.
///
/// Aborts if request contains `spent_balance` due to inability of the
/// `TokenPolicyCap` to decrease supply. For scenarios like this a
/// `TreasuryCap` is required (see `confirm_with_treasury_cap`).
public fun confirm_with_policy_cap<T>(
    _policy_cap: &TokenPolicyCap<T>,
    request: ActionRequest<T>,
    _ctx: &mut TxContext,
): (String, u64, address, Option<address>) {
    assert!(request.spent_balance.is_none(), ECantConsumeBalance);

    let ActionRequest {
        name,
        amount,
        sender,
        recipient,
        approvals: _,
        spent_balance,
    } = request;

    spent_balance.destroy_none();

    (name, amount, sender, recipient)
}

/// Confirm an `ActionRequest` as the `TreasuryCap` owner. This function
/// allows `TreasuryCap` owner to perform Capability-gated actions ignoring
/// the ruleset specified in the `TokenPolicy`.
///
/// Unlike `confirm_with_policy_cap` this function allows `spent_balance`
/// to be consumed, decreasing the `total_supply` of the `Token`.
public fun confirm_with_treasury_cap<T>(
    treasury_cap: &mut TreasuryCap<T>,
    request: ActionRequest<T>,
    _ctx: &mut TxContext,
): (String, u64, address, Option<address>) {
    let ActionRequest {
        name,
        amount,
        sender,
        recipient,
        approvals: _,
        spent_balance,
    } = request;

    if (spent_balance.is_some()) {
        treasury_cap.supply_mut().decrease_supply(spent_balance.destroy_some());
    } else {
        spent_balance.destroy_none();
    };

    (name, amount, sender, recipient)
}

// === Rules API ===

/// Add an "approval" to the `ActionRequest` by providing a Witness.
/// Intended to be used by Rules to add their own approvals, however, can
/// be used to add arbitrary approvals to the request (not only the ones
/// required by the `TokenPolicy`).
public fun add_approval<T, W: drop>(_t: W, request: &mut ActionRequest<T>, _ctx: &mut TxContext) {
    request.approvals.insert(type_name::get<W>())
}

/// Add a `Config` for a `Rule` in the `TokenPolicy`. Rule configuration is
/// independent from the `TokenPolicy.rules` and needs to be managed by the
/// Rule itself. Configuration is stored per `Rule` and not per `Rule` per
/// `Action` to allow reuse in different actions.
///
/// - Rule witness guarantees that the `Config` is approved by the Rule.
/// - `TokenPolicyCap` guarantees that the `Config` setup is initiated by
/// the `TokenPolicy` owner.
public fun add_rule_config<T, Rule: drop, Config: store>(
    _rule: Rule,
    self: &mut TokenPolicy<T>,
    cap: &TokenPolicyCap<T>,
    config: Config,
    _ctx: &mut TxContext,
) {
    assert!(object::id(self) == cap.`for`, ENotAuthorized);
    df::add(&mut self.id, key<Rule>(), config)
}

/// Get a `Config` for a `Rule` in the `TokenPolicy`. Requires `Rule`
/// witness, hence can only be read by the `Rule` itself. This requirement
/// guarantees safety of the stored `Config` and allows for simpler dynamic
/// field management inside the Rule Config (custom type keys are not needed
/// for access gating).
///
/// Aborts if the Config is not present.
public fun rule_config<T, Rule: drop, Config: store>(_rule: Rule, self: &TokenPolicy<T>): &Config {
    assert!(has_rule_config_with_type<T, Rule, Config>(self), ENoConfig);
    df::borrow(&self.id, key<Rule>())
}

/// Get mutable access to the `Config` for a `Rule` in the `TokenPolicy`.
/// Requires `Rule` witness, hence can only be read by the `Rule` itself,
/// as well as `TokenPolicyCap` to guarantee that the `TokenPolicy` owner
/// is the one who initiated the `Config` modification.
///
/// Aborts if:
/// - the Config is not present
/// - `TokenPolicyCap` is not matching the `TokenPolicy`
public fun rule_config_mut<T, Rule: drop, Config: store>(
    _rule: Rule,
    self: &mut TokenPolicy<T>,
    cap: &TokenPolicyCap<T>,
): &mut Config {
    assert!(has_rule_config_with_type<T, Rule, Config>(self), ENoConfig);
    assert!(object::id(self) == cap.`for`, ENotAuthorized);
    df::borrow_mut(&mut self.id, key<Rule>())
}

/// Remove a `Config` for a `Rule` in the `TokenPolicy`.
/// Unlike the `add_rule_config`, this function does not require a `Rule`
/// witness, hence can be performed by the `TokenPolicy` owner on their own.
///
/// Rules need to make sure that the `Config` is present when performing
/// verification of the `ActionRequest`.
///
/// Aborts if:
/// - the Config is not present
/// - `TokenPolicyCap` is not matching the `TokenPolicy`
public fun remove_rule_config<T, Rule, Config: store>(
    self: &mut TokenPolicy<T>,
    cap: &TokenPolicyCap<T>,
    _ctx: &mut TxContext,
): Config {
    assert!(has_rule_config_with_type<T, Rule, Config>(self), ENoConfig);
    assert!(object::id(self) == cap.`for`, ENotAuthorized);
    df::remove(&mut self.id, key<Rule>())
}

/// Check if a config for a `Rule` is set in the `TokenPolicy` without
/// checking the type of the `Config`.
public fun has_rule_config<T, Rule>(self: &TokenPolicy<T>): bool {
    df::exists_<RuleKey<Rule>>(&self.id, key<Rule>())
}

/// Check if a `Config` for a `Rule` is set in the `TokenPolicy` and that
/// it matches the type provided.
public fun has_rule_config_with_type<T, Rule, Config: store>(self: &TokenPolicy<T>): bool {
    df::exists_with_type<RuleKey<Rule>, Config>(&self.id, key<Rule>())
}

// === Protected: Setting Rules ===

/// Allows an `action` to be performed on the `Token` freely by adding an
/// empty set of `Rules` for the `action`.
///
/// Aborts if the `TokenPolicyCap` is not matching the `TokenPolicy`.
public fun allow<T>(
    self: &mut TokenPolicy<T>,
    cap: &TokenPolicyCap<T>,
    action: String,
    _ctx: &mut TxContext,
) {
    assert!(object::id(self) == cap.`for`, ENotAuthorized);
    self.rules.insert(action, vec_set::empty());
}

/// Completely disallows an `action` on the `Token` by removing the record
/// from the `TokenPolicy.rules`.
///
/// Aborts if the `TokenPolicyCap` is not matching the `TokenPolicy`.
public fun disallow<T>(
    self: &mut TokenPolicy<T>,
    cap: &TokenPolicyCap<T>,
    action: String,
    _ctx: &mut TxContext,
) {
    assert!(object::id(self) == cap.`for`, ENotAuthorized);
    self.rules.remove(&action);
}

/// Adds a Rule for an action with `name` in the `TokenPolicy`.
///
/// Aborts if the `TokenPolicyCap` is not matching the `TokenPolicy`.
public fun add_rule_for_action<T, Rule: drop>(
    self: &mut TokenPolicy<T>,
    cap: &TokenPolicyCap<T>,
    action: String,
    ctx: &mut TxContext,
) {
    assert!(object::id(self) == cap.`for`, ENotAuthorized);
    if (!self.rules.contains(&action)) {
        allow(self, cap, action, ctx);
    };

    self.rules.get_mut(&action).insert(type_name::get<Rule>())
}

/// Removes a rule for an action with `name` in the `TokenPolicy`. Returns
/// the config object to be handled by the sender (or a Rule itself).
///
/// Aborts if the `TokenPolicyCap` is not matching the `TokenPolicy`.
public fun remove_rule_for_action<T, Rule: drop>(
    self: &mut TokenPolicy<T>,
    cap: &TokenPolicyCap<T>,
    action: String,
    _ctx: &mut TxContext,
) {
    assert!(object::id(self) == cap.`for`, ENotAuthorized);

    self.rules.get_mut(&action).remove(&type_name::get<Rule>())
}

// === Protected: Treasury Management ===

/// Mint a `Token` with a given `amount` using the `TreasuryCap`.
public fun mint<T>(cap: &mut TreasuryCap<T>, amount: u64, ctx: &mut TxContext): Token<T> {
    let balance = cap.supply_mut().increase_supply(amount);
    Token { id: object::new(ctx), balance }
}

/// Burn a `Token` using the `TreasuryCap`.
public fun burn<T>(cap: &mut TreasuryCap<T>, token: Token<T>) {
    let Token { id, balance } = token;
    cap.supply_mut().decrease_supply(balance);
    id.delete();
}

/// Flush the `TokenPolicy.spent_balance` into the `TreasuryCap`. This
/// action is only available to the `TreasuryCap` owner.
public fun flush<T>(
    self: &mut TokenPolicy<T>,
    cap: &mut TreasuryCap<T>,
    _ctx: &mut TxContext,
): u64 {
    let amount = self.spent_balance.value();
    let balance = self.spent_balance.split(amount);
    cap.supply_mut().decrease_supply(balance)
}

// === Getters: `TokenPolicy` and `Token` ===

/// Check whether an action is present in the rules VecMap.
public fun is_allowed<T>(self: &TokenPolicy<T>, action: &String): bool {
    self.rules.contains(action)
}

/// Returns the rules required for a specific action.
public fun rules<T>(self: &TokenPolicy<T>, action: &String): VecSet<TypeName> {
    *self.rules.get(action)
}

/// Returns the `spent_balance` of the `TokenPolicy`.
public fun spent_balance<T>(self: &TokenPolicy<T>): u64 {
    self.spent_balance.value()
}

/// Returns the `balance` of the `Token`.
public fun value<T>(t: &Token<T>): u64 {
    t.balance.value()
}

// === Action Names ===

/// Name of the Transfer action.
public fun transfer_action(): String {
    let transfer_str = TRANSFER;
    transfer_str.to_string()
}

/// Name of the `Spend` action.
public fun spend_action(): String {
    let spend_str = SPEND;
    spend_str.to_string()
}

/// Name of the `ToCoin` action.
public fun to_coin_action(): String {
    let to_coin_str = TO_COIN;
    to_coin_str.to_string()
}

/// Name of the `FromCoin` action.
public fun from_coin_action(): String {
    let from_coin_str = FROM_COIN;
    from_coin_str.to_string()
}

// === Action Request Fields ==

/// The Action in the `ActionRequest`.
public fun action<T>(self: &ActionRequest<T>): String { self.name }

/// Amount of the `ActionRequest`.
public fun amount<T>(self: &ActionRequest<T>): u64 { self.amount }

/// Sender of the `ActionRequest`.
public fun sender<T>(self: &ActionRequest<T>): address { self.sender }

/// Recipient of the `ActionRequest`.
public fun recipient<T>(self: &ActionRequest<T>): Option<address> {
    self.recipient
}

/// Approvals of the `ActionRequest`.
public fun approvals<T>(self: &ActionRequest<T>): VecSet<TypeName> {
    self.approvals
}

/// Burned balance of the `ActionRequest`.
public fun spent<T>(self: &ActionRequest<T>): Option<u64> {
    if (self.spent_balance.is_some()) {
        option::some(self.spent_balance.borrow().value())
    } else {
        option::none()
    }
}

// === Internal ===

/// Create a new `RuleKey` for a `Rule`. The `is_protected` field is kept
/// for potential future use, if Rules were to have a freely modifiable
/// storage as addition / replacement for the `Config` system.
///
/// The goal of `is_protected` is to potentially allow Rules store a mutable
/// version of their configuration and mutate state on user action.
fun key<Rule>(): RuleKey<Rule> { RuleKey { is_protected: true } }

// === Testing ===

#[test_only]
public fun new_policy_for_testing<T>(ctx: &mut TxContext): (TokenPolicy<T>, TokenPolicyCap<T>) {
    let policy = TokenPolicy {
        id: object::new(ctx),
        rules: vec_map::empty(),
        spent_balance: balance::zero(),
    };
    let cap = TokenPolicyCap {
        id: object::new(ctx),
        `for`: object::id(&policy),
    };

    (policy, cap)
}

#[test_only]
public fun burn_policy_for_testing<T>(policy: TokenPolicy<T>, cap: TokenPolicyCap<T>) {
    let TokenPolicyCap { id: cap_id, `for`: _ } = cap;
    let TokenPolicy { id, rules: _, spent_balance } = policy;
    spent_balance.destroy_for_testing();
    cap_id.delete();
    id.delete();
}

#[test_only]
public fun mint_for_testing<T>(amount: u64, ctx: &mut TxContext): Token<T> {
    let balance = balance::create_for_testing(amount);
    Token { id: object::new(ctx), balance }
}

#[test_only]
public fun burn_for_testing<T>(token: Token<T>) {
    let Token { id, balance } = token;
    balance.destroy_for_testing();
    id.delete();
}
