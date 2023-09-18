# ADR 042: Group Module

## Changelog

* 2020/04/09: Initial Draft

## Status

Draft

## Abstract

This ADR defines the `x/group` module which allows the creation and management of on-chain multi-signature accounts and enables voting for message execution based on configurable decision policies.

## Context

The legacy amino multi-signature mechanism of the Cosmos SDK has certain limitations:

* Key rotation is not possible, although this can be solved with [account rekeying](adr-034-account-rekeying.md).
* Thresholds can't be changed.
* UX is cumbersome for non-technical users ([#5661](https://github.com/cosmos/cosmos-sdk/issues/5661)).
* It requires `legacy_amino` sign mode ([#8141](https://github.com/cosmos/cosmos-sdk/issues/8141)).

While the group module is not meant to be a total replacement for the current multi-signature accounts, it provides a solution to the limitations described above, with a more flexible key management system where keys can be added, updated or removed, as well as configurable thresholds.
It's meant to be used with other access control modules such as [`x/feegrant`](./adr-029-fee-grant-module.md) ans [`x/authz`](adr-030-authz-module.md) to simplify key management for individuals and organizations.

The proof of concept of the group module can be found in https://github.com/regen-network/regen-ledger/tree/master/proto/regen/group/v1alpha1 and https://github.com/regen-network/regen-ledger/tree/master/x/group.

## Decision

We propose merging the `x/group` module with its supporting [ORM/Table Store package](https://github.com/regen-network/regen-ledger/tree/master/orm) ([#7098](https://github.com/cosmos/cosmos-sdk/issues/7098)) into the Cosmos SDK and continuing development here. There will be a dedicated ADR for the ORM package.

### Group

A group is a composition of accounts with associated weights. It is not
an account and doesn't have a balance. It doesn't in and of itself have any
sort of voting or decision weight.
Group members can create proposals and vote on them through group accounts using different decision policies.

It has an `admin` account which can manage members in the group, update the group
metadata and set a new admin.

```protobuf
message GroupInfo {

    // group_id is the unique ID of this group.
    uint64 group_id = 1;

    // admin is the account address of the group's admin.
    string admin = 2;

    // metadata is any arbitrary metadata to attached to the group.
    bytes metadata = 3;

    // version is used to track changes to a group's membership structure that
    // would break existing proposals. Whenever a member weight has changed,
    // or any member is added or removed, the version is incremented and will
    // invalidate all proposals from older versions.
    uint64 version = 4;

    // total_weight is the sum of the group members' weights.
    string total_weight = 5;
}
```

```protobuf
message GroupMember {

    // group_id is the unique ID of the group.
    uint64 group_id = 1;

    // member is the member data.
    Member member = 2;
}

// Member represents a group member with an account address,
// non-zero weight and metadata.
message Member {

    // address is the member's account address.
    string address = 1;

    // weight is the member's voting weight that should be greater than 0.
    string weight = 2;

    // metadata is any arbitrary metadata to attached to the member.
    bytes metadata = 3;
}
```

### Group Account

A group account is an account associated with a group and a decision policy.
A group account does have a balance.

Group accounts are abstracted from groups because a single group may have
multiple decision policies for different types of actions. Managing group
membership separately from decision policies results in the least overhead
and keeps membership consistent across different policies. The pattern that
is recommended is to have a single master group account for a given group,
and then to create separate group accounts with different decision policies
and delegate the desired permissions from the master account to
those "sub-accounts" using the [`x/authz` module](adr-030-authz-module.md).

```protobuf
message GroupAccountInfo {

    // address is the group account address.
    string address = 1;

    // group_id is the ID of the Group the GroupAccount belongs to.
    uint64 group_id = 2;

    // admin is the account address of the group admin.
    string admin = 3;

    // metadata is any arbitrary metadata of this group account.
    bytes metadata = 4;

    // version is used to track changes to a group's GroupAccountInfo structure that
    // invalidates active proposal from old versions.
    uint64 version = 5;

    // decision_policy specifies the group account's decision policy.
    google.protobuf.Any decision_policy = 6 [(cosmos_proto.accepts_interface) = "cosmos.group.v1.DecisionPolicy"];
}
```

Similarly to a group admin, a group account admin can update its metadata, decision policy or set a new group account admin.

A group account can also be an admin or a member of a group.
For instance, a group admin could be another group account which could "elects" the members or it could be the same group that elects itself.

### Decision Policy

A decision policy is the mechanism by which members of a group can vote on
proposals.

All decision policies should have a minimum and maximum voting window.
The minimum voting window is the minimum duration that must pass in order
for a proposal to potentially pass, and it may be set to 0. The maximum voting
window is the maximum time that a proposal may be voted on and executed if
it reached enough support before it is closed.
Both of these values must be less than a chain-wide max voting window parameter.

We define the `DecisionPolicy` interface that all decision policies must implement:

```go
type DecisionPolicy interface {
	codec.ProtoMarshaler

	ValidateBasic() error
	GetTimeout() types.Duration
	Allow(tally Tally, totalPower string, votingDuration time.Duration) (DecisionPolicyResult, error)
	Validate(g GroupInfo) error
}

type DecisionPolicyResult struct {
	Allow bool
	Final bool
}
```

#### Threshold decision policy

A threshold decision policy defines a minimum support votes (_yes_), based on a tally
of voter weights, for a proposal to pass. For
this decision policy, abstain and veto are treated as no support (_no_).

```protobuf
message ThresholdDecisionPolicy {

    // threshold is the minimum weighted sum of support votes for a proposal to succeed.
    string threshold = 1;

    // voting_period is the duration from submission of a proposal to the end of voting period
    // Within this period, votes and exec messages can be submitted.
    google.protobuf.Duration voting_period = 2 [(gogoproto.nullable) = false];
}
```

### Proposal

Any member of a group can submit a proposal for a group account to decide upon.
A proposal consists of a set of `sdk.Msg`s that will be executed if the proposal
passes as well as any metadata associated with the proposal. These `sdk.Msg`s get validated as part of the `Msg/CreateProposal` request validation. They should also have their signer set as the group account.

Internally, a proposal also tracks:

* its current `Status`: submitted, closed or aborted
* its `Result`: unfinalized, accepted or rejected
* its `VoteState` in the form of a `Tally`, which is calculated on new votes and when executing the proposal.

```protobuf
// Tally represents the sum of weighted votes.
message Tally {
    option (gogoproto.goproto_getters) = false;

    // yes_count is the weighted sum of yes votes.
    string yes_count = 1;

    // no_count is the weighted sum of no votes.
    string no_count = 2;

    // abstain_count is the weighted sum of abstainers.
    string abstain_count = 3;

    // veto_count is the weighted sum of vetoes.
    string veto_count = 4;
}
```

### Voting

Members of a group can vote on proposals. There are four choices to choose while voting - yes, no, abstain and veto. Not
all decision policies will support them. Votes can contain some optional metadata.
In the current implementation, the voting window begins as soon as a proposal
is submitted.

Voting internally updates the proposal `VoteState` as well as `Status` and `Result` if needed.

### Executing Proposals

Proposals will not be automatically executed by the chain in this current design,
but rather a user must submit a `Msg/Exec` transaction to attempt to execute the
proposal based on the current votes and decision policy. A future upgrade could
automate this and have the group account (or a fee granter) pay.

#### Changing Group Membership

In the current implementation, updating a group or a group account after submitting a proposal will make it invalid. It will simply fail if someone calls `Msg/Exec` and will eventually be garbage collected.

### Notes on current implementation

This section outlines the current implementation used in the proof of concept of the group module but this could be subject to changes and iterated on.

#### ORM

The [ORM package](https://github.com/cosmos/cosmos-sdk/discussions/9156) defines tables, sequences and secondary indexes which are used in the group module.

Groups are stored in state as part of a `groupTable`, the `group_id` being an auto-increment integer. Group members are stored in a `groupMemberTable`.

Group accounts are stored in a `groupAccountTable`. The group account address is generated based on an auto-increment integer which is used to derive the group module `RootModuleKey` into a `DerivedModuleKey`, as stated in [ADR-033](adr-033-protobuf-inter-module-comm.md#modulekeys-and-moduleids). The group account is added as a new `ModuleAccount` through `x/auth`.

Proposals are stored as part of the `proposalTable` using the `Proposal` type. The `proposal_id` is an auto-increment integer.

Votes are stored in the `voteTable`. The primary key is based on the vote's `proposal_id` and `voter` account address.

#### ADR-033 to route proposal messages

Inter-module communication introduced by [ADR-033](adr-033-protobuf-inter-module-comm.md) can be used to route a proposal's messages using the `DerivedModuleKey` corresponding to the proposal's group account.

## Consequences

### Positive

* Improved UX for multi-signature accounts allowing key rotation and custom decision policies.

### Negative

### Neutral

* It uses ADR 033 so it will need to be implemented within the Cosmos SDK, but this doesn't imply necessarily any large refactoring of existing Cosmos SDK modules.
* The current implementation of the group module uses the ORM package.

## Further Discussions

* Convergence of `/group` and `x/gov` as both support proposals and voting: https://github.com/cosmos/cosmos-sdk/discussions/9066
* `x/group` possible future improvements:
    * Execute proposals on submission (https://github.com/regen-network/regen-ledger/issues/288)
    * Withdraw a proposal (https://github.com/regen-network/cosmos-modules/issues/41)
    * Make `Tally` more flexible and support non-binary choices

## References

* Initial specification:
    * https://gist.github.com/aaronc/b60628017352df5983791cad30babe56#group-module
    * [#5236](https://github.com/cosmos/cosmos-sdk/pull/5236)
* Proposal to add `x/group` into the Cosmos SDK: [#7633](https://github.com/cosmos/cosmos-sdk/issues/7633)
