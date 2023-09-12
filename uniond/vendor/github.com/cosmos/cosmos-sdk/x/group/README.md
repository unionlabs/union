---
sidebar_position: 1
---

# `x/group`

## Abstract

The following documents specify the group module.

This module allows the creation and management of on-chain multisig accounts and enables voting for message execution based on configurable decision policies.

## Contents

* [Concepts](#concepts)
    * [Group](#group)
    * [Group Policy](#group-policy)
    * [Decision Policy](#decision-policy)
    * [Proposal](#proposal)
    * [Pruning](#pruning)
* [State](#state)
    * [Group Table](#group-table)
    * [Group Member Table](#group-member-table)
    * [Group Policy Table](#group-policy-table)
    * [Proposal Table](#proposal-table)
    * [Vote Table](#vote-table)
* [Msg Service](#msg-service)
    * [Msg/CreateGroup](#msgcreategroup)
    * [Msg/UpdateGroupMembers](#msgupdategroupmembers)
    * [Msg/UpdateGroupAdmin](#msgupdategroupadmin)
    * [Msg/UpdateGroupMetadata](#msgupdategroupmetadata)
    * [Msg/CreateGroupPolicy](#msgcreategrouppolicy)
    * [Msg/CreateGroupWithPolicy](#msgcreategroupwithpolicy)
    * [Msg/UpdateGroupPolicyAdmin](#msgupdategrouppolicyadmin)
    * [Msg/UpdateGroupPolicyDecisionPolicy](#msgupdategrouppolicydecisionpolicy)
    * [Msg/UpdateGroupPolicyMetadata](#msgupdategrouppolicymetadata)
    * [Msg/SubmitProposal](#msgsubmitproposal)
    * [Msg/WithdrawProposal](#msgwithdrawproposal)
    * [Msg/Vote](#msgvote)
    * [Msg/Exec](#msgexec)
    * [Msg/LeaveGroup](#msgleavegroup)
* [Events](#events)
    * [EventCreateGroup](#eventcreategroup)
    * [EventUpdateGroup](#eventupdategroup)
    * [EventCreateGroupPolicy](#eventcreategrouppolicy)
    * [EventUpdateGroupPolicy](#eventupdategrouppolicy)
    * [EventCreateProposal](#eventcreateproposal)
    * [EventWithdrawProposal](#eventwithdrawproposal)
    * [EventVote](#eventvote)
    * [EventExec](#eventexec)
    * [EventLeaveGroup](#eventleavegroup)
    * [EventProposalPruned](#eventproposalpruned)
* [Client](#client)
    * [CLI](#cli)
    * [gRPC](#grpc)
    * [REST](#rest)
* [Metadata](#metadata)

## Concepts

### Group

A group is simply an aggregation of accounts with associated weights. It is not
an account and doesn't have a balance. It doesn't in and of itself have any
sort of voting or decision weight. It does have an "administrator" which has
the ability to add, remove and update members in the group. Note that a
group policy account could be an administrator of a group, and that the
administrator doesn't necessarily have to be a member of the group.

### Group Policy

A group policy is an account associated with a group and a decision policy.
Group policies are abstracted from groups because a single group may have
multiple decision policies for different types of actions. Managing group
membership separately from decision policies results in the least overhead
and keeps membership consistent across different policies. The pattern that
is recommended is to have a single master group policy for a given group,
and then to create separate group policies with different decision policies
and delegate the desired permissions from the master account to
those "sub-accounts" using the `x/authz` module.

### Decision Policy

A decision policy is the mechanism by which members of a group can vote on
proposals, as well as the rules that dictate whether a proposal should pass
or not based on its tally outcome.

All decision policies generally would have a mininum execution period and a
maximum voting window. The minimum execution period is the minimum amount of time
that must pass after submission in order for a proposal to potentially be executed, and it may
be set to 0. The maximum voting window is the maximum time after submission that a proposal may
be voted on before it is tallied.

The chain developer also defines an app-wide maximum execution period, which is
the maximum amount of time after a proposal's voting period end where users are
allowed to execute a proposal.

The current group module comes shipped with two decision policies: threshold
and percentage. Any chain developer can extend upon these two, by creating
custom decision policies, as long as they adhere to the `DecisionPolicy`
interface:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/group/types.go#L27-L45
```

#### Threshold decision policy

A threshold decision policy defines a threshold of yes votes (based on a tally
of voter weights) that must be achieved in order for a proposal to pass. For
this decision policy, abstain and veto are simply treated as no's.

This decision policy also has a VotingPeriod window and a MinExecutionPeriod
window. The former defines the duration after proposal submission where members
are allowed to vote, after which tallying is performed. The latter specifies
the minimum duration after proposal submission where the proposal can be
executed. If set to 0, then the proposal is allowed to be executed immediately
on submission (using the `TRY_EXEC` option). Obviously, MinExecutionPeriod
cannot be greater than VotingPeriod+MaxExecutionPeriod (where MaxExecution is
the app-defined duration that specifies the window after voting ended where a
proposal can be executed).

#### Percentage decision policy

A percentage decision policy is similar to a threshold decision policy, except
that the threshold is not defined as a constant weight, but as a percentage.
It's more suited for groups where the group members' weights can be updated, as
the percentage threshold stays the same, and doesn't depend on how those member
weights get updated.

Same as the Threshold decision policy, the percentage decision policy has the
two VotingPeriod and MinExecutionPeriod parameters.

### Proposal

Any member(s) of a group can submit a proposal for a group policy account to decide upon.
A proposal consists of a set of messages that will be executed if the proposal
passes as well as any metadata associated with the proposal.

#### Voting

There are four choices to choose while voting - yes, no, abstain and veto. Not
all decision policies will take the four choices into account. Votes can contain some optional metadata.
In the current implementation, the voting window begins as soon as a proposal
is submitted, and the end is defined by the group policy's decision policy.

#### Withdrawing Proposals

Proposals can be withdrawn any time before the voting period end, either by the
admin of the group policy or by one of the proposers. Once withdrawn, it is
marked as `PROPOSAL_STATUS_WITHDRAWN`, and no more voting or execution is
allowed on it.

#### Aborted Proposals

If the group policy is updated during the voting period of the proposal, then
the proposal is marked as `PROPOSAL_STATUS_ABORTED`, and no more voting or
execution is allowed on it. This is because the group policy defines the rules
of proposal voting and execution, so if those rules change during the lifecycle
of a proposal, then the proposal should be marked as stale.

#### Tallying

Tallying is the counting of all votes on a proposal. It happens only once in
the lifecycle of a proposal, but can be triggered by two factors, whichever
happens first:

* either someone tries to execute the proposal (see next section), which can
  happen on a `Msg/Exec` transaction, or a `Msg/{SubmitProposal,Vote}`
  transaction with the `Exec` field set. When a proposal execution is attempted,
  a tally is done first to make sure the proposal passes.
* or on `EndBlock` when the proposal's voting period end just passed.

If the tally result passes the decision policy's rules, then the proposal is
marked as `PROPOSAL_STATUS_ACCEPTED`, or else it is marked as
`PROPOSAL_STATUS_REJECTED`. In any case, no more voting is allowed anymore, and the tally
result is persisted to state in the proposal's `FinalTallyResult`.

#### Executing Proposals

Proposals are executed only when the tallying is done, and the group account's
decision policy allows the proposal to pass based on the tally outcome. They
are marked by the status `PROPOSAL_STATUS_ACCEPTED`. Execution must happen
before a duration of `MaxExecutionPeriod` (set by the chain developer) after
each proposal's voting period end.

Proposals will not be automatically executed by the chain in this current design,
but rather a user must submit a `Msg/Exec` transaction to attempt to execute the
proposal based on the current votes and decision policy. Any user (not only the
group members) can execute proposals that have been accepted, and execution fees are
paid by the proposal executor.
It's also possible to try to execute a proposal immediately on creation or on
new votes using the `Exec` field of `Msg/SubmitProposal` and `Msg/Vote` requests.
In the former case, proposers signatures are considered as yes votes.
In these cases, if the proposal can't be executed (i.e. it didn't pass the
decision policy's rules), it will still be opened for new votes and
could be tallied and executed later on.

A successful proposal execution will have its `ExecutorResult` marked as
`PROPOSAL_EXECUTOR_RESULT_SUCCESS`. The proposal will be automatically pruned
after execution. On the other hand, a failed proposal execution will be marked
as `PROPOSAL_EXECUTOR_RESULT_FAILURE`. Such a proposal can be re-executed
multiple times, until it expires after `MaxExecutionPeriod` after voting period
end.

### Pruning

Proposals and votes are automatically pruned to avoid state bloat.

Votes are pruned:

* either after a successful tally, i.e. a tally whose result passes the decision
  policy's rules, which can be trigged by a `Msg/Exec` or a
  `Msg/{SubmitProposal,Vote}` with the `Exec` field set,
* or on `EndBlock` right after the proposal's voting period end. This applies to proposals with status `aborted` or `withdrawn` too.

whichever happens first.

Proposals are pruned:

* on `EndBlock` whose proposal status is `withdrawn` or `aborted` on proposal's voting period end before tallying,
* and either after a successful proposal execution,
* or on `EndBlock` right after the proposal's `voting_period_end` +
  `max_execution_period` (defined as an app-wide configuration) is passed,

whichever happens first.

## State

The `group` module uses the `orm` package which provides table storage with support for
primary keys and secondary indexes. `orm` also defines `Sequence` which is a persistent unique key generator based on a counter that can be used along with `Table`s.

Here's the list of tables and associated sequences and indexes stored as part of the `group` module.

### Group Table

The `groupTable` stores `GroupInfo`: `0x0 | BigEndian(GroupId) -> ProtocolBuffer(GroupInfo)`.

#### groupSeq

The value of `groupSeq` is incremented when creating a new group and corresponds to the new `GroupId`: `0x1 | 0x1 -> BigEndian`.

The second `0x1` corresponds to the ORM `sequenceStorageKey`.

#### groupByAdminIndex

`groupByAdminIndex` allows to retrieve groups by admin address:
`0x2 | len([]byte(group.Admin)) | []byte(group.Admin) | BigEndian(GroupId) -> []byte()`.

### Group Member Table

The `groupMemberTable` stores `GroupMember`s: `0x10 | BigEndian(GroupId) | []byte(member.Address) -> ProtocolBuffer(GroupMember)`.

The `groupMemberTable` is a primary key table and its `PrimaryKey` is given by
`BigEndian(GroupId) | []byte(member.Address)` which is used by the following indexes.

#### groupMemberByGroupIndex

`groupMemberByGroupIndex` allows to retrieve group members by group id:
`0x11 | BigEndian(GroupId) | PrimaryKey -> []byte()`.

#### groupMemberByMemberIndex

`groupMemberByMemberIndex` allows to retrieve group members by member address:
`0x12 | len([]byte(member.Address)) | []byte(member.Address) | PrimaryKey -> []byte()`.

### Group Policy Table

The `groupPolicyTable` stores `GroupPolicyInfo`: `0x20 | len([]byte(Address)) | []byte(Address) -> ProtocolBuffer(GroupPolicyInfo)`.

The `groupPolicyTable` is a primary key table and its `PrimaryKey` is given by
`len([]byte(Address)) | []byte(Address)` which is used by the following indexes.

#### groupPolicySeq

The value of `groupPolicySeq` is incremented when creating a new group policy and is used to generate the new group policy account `Address`:
`0x21 | 0x1 -> BigEndian`.

The second `0x1` corresponds to the ORM `sequenceStorageKey`.

#### groupPolicyByGroupIndex

`groupPolicyByGroupIndex` allows to retrieve group policies by group id:
`0x22 | BigEndian(GroupId) | PrimaryKey -> []byte()`.

#### groupPolicyByAdminIndex

`groupPolicyByAdminIndex` allows to retrieve group policies by admin address:
`0x23 | len([]byte(Address)) | []byte(Address) | PrimaryKey -> []byte()`.

### Proposal Table

The `proposalTable` stores `Proposal`s: `0x30 | BigEndian(ProposalId) -> ProtocolBuffer(Proposal)`.

#### proposalSeq

The value of `proposalSeq` is incremented when creating a new proposal and corresponds to the new `ProposalId`: `0x31 | 0x1 -> BigEndian`.

The second `0x1` corresponds to the ORM `sequenceStorageKey`.

#### proposalByGroupPolicyIndex

`proposalByGroupPolicyIndex` allows to retrieve proposals by group policy account address:
`0x32 | len([]byte(account.Address)) | []byte(account.Address) | BigEndian(ProposalId) -> []byte()`.

#### ProposalsByVotingPeriodEndIndex

`proposalsByVotingPeriodEndIndex` allows to retrieve proposals sorted by chronological `voting_period_end`:
`0x33 | sdk.FormatTimeBytes(proposal.VotingPeriodEnd) | BigEndian(ProposalId) -> []byte()`.

This index is used when tallying the proposal votes at the end of the voting period, and for pruning proposals at `VotingPeriodEnd + MaxExecutionPeriod`.

### Vote Table

The `voteTable` stores `Vote`s: `0x40 | BigEndian(ProposalId) | []byte(voter.Address) -> ProtocolBuffer(Vote)`.

The `voteTable` is a primary key table and its `PrimaryKey` is given by
`BigEndian(ProposalId) | []byte(voter.Address)` which is used by the following indexes.

#### voteByProposalIndex

`voteByProposalIndex` allows to retrieve votes by proposal id:
`0x41 | BigEndian(ProposalId) | PrimaryKey -> []byte()`.

#### voteByVoterIndex

`voteByVoterIndex` allows to retrieve votes by voter address:
`0x42 | len([]byte(voter.Address)) | []byte(voter.Address) | PrimaryKey -> []byte()`.

## Msg Service

### Msg/CreateGroup

A new group can be created with the `MsgCreateGroup`, which has an admin address, a list of members and some optional metadata.

The metadata has a maximum length that is chosen by the app developer, and
passed into the group keeper as a config.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/group/v1/tx.proto#L67-L80
```

It's expected to fail if

* metadata length is greater than `MaxMetadataLen` config
* members are not correctly set (e.g. wrong address format, duplicates, or with 0 weight).

### Msg/UpdateGroupMembers

Group members can be updated with the `UpdateGroupMembers`.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/group/v1/tx.proto#L88-L102
```

In the list of `MemberUpdates`, an existing member can be removed by setting its weight to 0.

It's expected to fail if:

* the signer is not the admin of the group.
* for any one of the associated group policies, if its decision policy's `Validate()` method fails against the updated group.

### Msg/UpdateGroupAdmin

The `UpdateGroupAdmin` can be used to update a group admin.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/group/v1/tx.proto#L107-L120
```

It's expected to fail if the signer is not the admin of the group.

### Msg/UpdateGroupMetadata

The `UpdateGroupMetadata` can be used to update a group metadata.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/group/v1/tx.proto#L125-L138
```

It's expected to fail if:

* new metadata length is greater than `MaxMetadataLen` config.
* the signer is not the admin of the group.

### Msg/CreateGroupPolicy

A new group policy can be created with the `MsgCreateGroupPolicy`, which has an admin address, a group id, a decision policy and some optional metadata.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/group/v1/tx.proto#L147-L165
```

It's expected to fail if:

* the signer is not the admin of the group.
* metadata length is greater than `MaxMetadataLen` config.
* the decision policy's `Validate()` method doesn't pass against the group.

### Msg/CreateGroupWithPolicy

A new group with policy can be created with the `MsgCreateGroupWithPolicy`, which has an admin address, a list of members, a decision policy, a `group_policy_as_admin` field to optionally set group and group policy admin with group policy address and some optional metadata for group and group policy.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/group/v1/tx.proto#L191-L215
```

It's expected to fail for the same reasons as `Msg/CreateGroup` and `Msg/CreateGroupPolicy`.

### Msg/UpdateGroupPolicyAdmin

The `UpdateGroupPolicyAdmin` can be used to update a group policy admin.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/group/v1/tx.proto#L173-L186
```

It's expected to fail if the signer is not the admin of the group policy.

### Msg/UpdateGroupPolicyDecisionPolicy

The `UpdateGroupPolicyDecisionPolicy` can be used to update a decision policy.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/group/v1/tx.proto#L226-L241
```

It's expected to fail if:

* the signer is not the admin of the group policy.
* the new decision policy's `Validate()` method doesn't pass against the group.

### Msg/UpdateGroupPolicyMetadata

The `UpdateGroupPolicyMetadata` can be used to update a group policy metadata.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/group/v1/tx.proto#L246-L259
```

It's expected to fail if:

* new metadata length is greater than `MaxMetadataLen` config.
* the signer is not the admin of the group.

### Msg/SubmitProposal

A new proposal can be created with the `MsgSubmitProposal`, which has a group policy account address, a list of proposers addresses, a list of messages to execute if the proposal is accepted and some optional metadata.
An optional `Exec` value can be provided to try to execute the proposal immediately after proposal creation. Proposers signatures are considered as yes votes in this case.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/group/v1/tx.proto#L281-L315
```

It's expected to fail if:

* metadata, title, or summary length is greater than `MaxMetadataLen` config.
* if any of the proposers is not a group member.

### Msg/WithdrawProposal

A proposal can be withdrawn using `MsgWithdrawProposal` which has an `address` (can be either a proposer or the group policy admin) and a `proposal_id` (which has to be withdrawn).

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/group/v1/tx.proto#L323-L333
```

It's expected to fail if:

* the signer is neither the group policy admin nor proposer of the proposal.
* the proposal is already closed or aborted.

### Msg/Vote

A new vote can be created with the `MsgVote`, given a proposal id, a voter address, a choice (yes, no, veto or abstain) and some optional metadata.
An optional `Exec` value can be provided to try to execute the proposal immediately after voting.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/group/v1/tx.proto#L338-L358
```

It's expected to fail if:

* metadata length is greater than `MaxMetadataLen` config.
* the proposal is not in voting period anymore.

### Msg/Exec

A proposal can be executed with the `MsgExec`.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/group/v1/tx.proto#L363-L373
```

The messages that are part of this proposal won't be executed if:

* the proposal has not been accepted by the group policy.
* the proposal has already been successfully executed.

### Msg/LeaveGroup

The `MsgLeaveGroup` allows group member to leave a group.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/group/v1/tx.proto#L381-L391
```

It's expected to fail if:

* the group member is not part of the group.
* for any one of the associated group policies, if its decision policy's `Validate()` method fails against the updated group.

## Events

The group module emits the following events:

### EventCreateGroup

| Type                             | Attribute Key | Attribute Value                  |
| -------------------------------- | ------------- | -------------------------------- |
| message                          | action        | /cosmos.group.v1.Msg/CreateGroup |
| cosmos.group.v1.EventCreateGroup | group_id      | {groupId}                        |

### EventUpdateGroup

| Type                             | Attribute Key | Attribute Value                                            |
| -------------------------------- | ------------- | ---------------------------------------------------------- |
| message                          | action        | /cosmos.group.v1.Msg/UpdateGroup{Admin\|Metadata\|Members} |
| cosmos.group.v1.EventUpdateGroup | group_id      | {groupId}                                                  |

### EventCreateGroupPolicy

| Type                                   | Attribute Key | Attribute Value                        |
| -------------------------------------- | ------------- | -------------------------------------- |
| message                                | action        | /cosmos.group.v1.Msg/CreateGroupPolicy |
| cosmos.group.v1.EventCreateGroupPolicy | address       | {groupPolicyAddress}                   |

### EventUpdateGroupPolicy

| Type                                   | Attribute Key | Attribute Value                                                         |
| -------------------------------------- | ------------- | ----------------------------------------------------------------------- |
| message                                | action        | /cosmos.group.v1.Msg/UpdateGroupPolicy{Admin\|Metadata\|DecisionPolicy} |
| cosmos.group.v1.EventUpdateGroupPolicy | address       | {groupPolicyAddress}                                                    |

### EventCreateProposal

| Type                                | Attribute Key | Attribute Value                     |
| ----------------------------------- | ------------- | ----------------------------------- |
| message                             | action        | /cosmos.group.v1.Msg/CreateProposal |
| cosmos.group.v1.EventCreateProposal | proposal_id   | {proposalId}                        |

### EventWithdrawProposal

| Type                                  | Attribute Key | Attribute Value                       |
| ------------------------------------- | ------------- | ------------------------------------- |
| message                               | action        | /cosmos.group.v1.Msg/WithdrawProposal |
| cosmos.group.v1.EventWithdrawProposal | proposal_id   | {proposalId}                          |

### EventVote

| Type                      | Attribute Key | Attribute Value           |
| ------------------------- | ------------- | ------------------------- |
| message                   | action        | /cosmos.group.v1.Msg/Vote |
| cosmos.group.v1.EventVote | proposal_id   | {proposalId}              |

## EventExec

| Type                      | Attribute Key | Attribute Value           |
| ------------------------- | ------------- | ------------------------- |
| message                   | action        | /cosmos.group.v1.Msg/Exec |
| cosmos.group.v1.EventExec | proposal_id   | {proposalId}              |
| cosmos.group.v1.EventExec | logs          | {logs_string}             |

### EventLeaveGroup

| Type                            | Attribute Key | Attribute Value                 |
| ------------------------------- | ------------- | ------------------------------- |
| message                         | action        | /cosmos.group.v1.Msg/LeaveGroup |
| cosmos.group.v1.EventLeaveGroup | proposal_id   | {proposalId}                    |
| cosmos.group.v1.EventLeaveGroup | address       | {address}                       |

### EventProposalPruned

| Type                                | Attribute Key | Attribute Value                 |
|-------------------------------------|---------------|---------------------------------|
| message                             | action        | /cosmos.group.v1.Msg/LeaveGroup |
| cosmos.group.v1.EventProposalPruned | proposal_id   | {proposalId}                    |
| cosmos.group.v1.EventProposalPruned | status        | {ProposalStatus}                |
| cosmos.group.v1.EventProposalPruned | tally_result  | {TallyResult}                   |


## Client

### CLI

A user can query and interact with the `group` module using the CLI.

#### Query

The `query` commands allow users to query `group` state.

```bash
simd query group --help
```

##### group-info

The `group-info` command allows users to query for group info by given group id.

```bash
simd query group group-info [id] [flags]
```

Example:

```bash
simd query group group-info 1
```

Example Output:

```bash
admin: cosmos1..
group_id: "1"
metadata: AQ==
total_weight: "3"
version: "1"
```

##### group-policy-info

The `group-policy-info` command allows users to query for group policy info by account address of group policy .

```bash
simd query group group-policy-info [group-policy-account] [flags]
```

Example:

```bash
simd query group group-policy-info cosmos1..
```

Example Output:

```bash
address: cosmos1..
admin: cosmos1..
decision_policy:
  '@type': /cosmos.group.v1.ThresholdDecisionPolicy
  threshold: "1"
  windows:
      min_execution_period: 0s
      voting_period: 432000s
group_id: "1"
metadata: AQ==
version: "1"
```

##### group-members

The `group-members` command allows users to query for group members by group id with pagination flags.

```bash
simd query group group-members [id] [flags]
```

Example:

```bash
simd query group group-members 1
```

Example Output:

```bash
members:
- group_id: "1"
  member:
    address: cosmos1..
    metadata: AQ==
    weight: "2"
- group_id: "1"
  member:
    address: cosmos1..
    metadata: AQ==
    weight: "1"
pagination:
  next_key: null
  total: "2"
```

##### groups-by-admin

The `groups-by-admin` command allows users to query for groups by admin account address with pagination flags.

```bash
simd query group groups-by-admin [admin] [flags]
```

Example:

```bash
simd query group groups-by-admin cosmos1..
```

Example Output:

```bash
groups:
- admin: cosmos1..
  group_id: "1"
  metadata: AQ==
  total_weight: "3"
  version: "1"
- admin: cosmos1..
  group_id: "2"
  metadata: AQ==
  total_weight: "3"
  version: "1"
pagination:
  next_key: null
  total: "2"
```

##### group-policies-by-group

The `group-policies-by-group` command allows users to query for group policies by group id with pagination flags.

```bash
simd query group group-policies-by-group [group-id] [flags]
```

Example:

```bash
simd query group group-policies-by-group 1
```

Example Output:

```bash
group_policies:
- address: cosmos1..
  admin: cosmos1..
  decision_policy:
    '@type': /cosmos.group.v1.ThresholdDecisionPolicy
    threshold: "1"
    windows:
      min_execution_period: 0s
      voting_period: 432000s
  group_id: "1"
  metadata: AQ==
  version: "1"
- address: cosmos1..
  admin: cosmos1..
  decision_policy:
    '@type': /cosmos.group.v1.ThresholdDecisionPolicy
    threshold: "1"
    windows:
      min_execution_period: 0s
      voting_period: 432000s
  group_id: "1"
  metadata: AQ==
  version: "1"
pagination:
  next_key: null
  total: "2"
```

##### group-policies-by-admin

The `group-policies-by-admin` command allows users to query for group policies by admin account address with pagination flags.

```bash
simd query group group-policies-by-admin [admin] [flags]
```

Example:

```bash
simd query group group-policies-by-admin cosmos1..
```

Example Output:

```bash
group_policies:
- address: cosmos1..
  admin: cosmos1..
  decision_policy:
    '@type': /cosmos.group.v1.ThresholdDecisionPolicy
    threshold: "1"
    windows:
      min_execution_period: 0s
      voting_period: 432000s
  group_id: "1"
  metadata: AQ==
  version: "1"
- address: cosmos1..
  admin: cosmos1..
  decision_policy:
    '@type': /cosmos.group.v1.ThresholdDecisionPolicy
    threshold: "1"
    windows:
      min_execution_period: 0s
      voting_period: 432000s
  group_id: "1"
  metadata: AQ==
  version: "1"
pagination:
  next_key: null
  total: "2"
```

##### proposal

The `proposal` command allows users to query for proposal by id.

```bash
simd query group proposal [id] [flags]
```

Example:

```bash
simd query group proposal 1
```

Example Output:

```bash
proposal:
  address: cosmos1..
  executor_result: EXECUTOR_RESULT_NOT_RUN
  group_policy_version: "1"
  group_version: "1"
  metadata: AQ==
  msgs:
  - '@type': /cosmos.bank.v1beta1.MsgSend
    amount:
    - amount: "100000000"
      denom: stake
    from_address: cosmos1..
    to_address: cosmos1..
  proposal_id: "1"
  proposers:
  - cosmos1..
  result: RESULT_UNFINALIZED
  status: STATUS_SUBMITTED
  submitted_at: "2021-12-17T07:06:26.310638964Z"
  windows:
    min_execution_period: 0s
    voting_period: 432000s
  vote_state:
    abstain_count: "0"
    no_count: "0"
    veto_count: "0"
    yes_count: "0"
  summary: "Summary"
  title: "Title"
```

##### proposals-by-group-policy

The `proposals-by-group-policy` command allows users to query for proposals by account address of group policy with pagination flags.

```bash
simd query group proposals-by-group-policy [group-policy-account] [flags]
```

Example:

```bash
simd query group proposals-by-group-policy cosmos1..
```

Example Output:

```bash
pagination:
  next_key: null
  total: "1"
proposals:
- address: cosmos1..
  executor_result: EXECUTOR_RESULT_NOT_RUN
  group_policy_version: "1"
  group_version: "1"
  metadata: AQ==
  msgs:
  - '@type': /cosmos.bank.v1beta1.MsgSend
    amount:
    - amount: "100000000"
      denom: stake
    from_address: cosmos1..
    to_address: cosmos1..
  proposal_id: "1"
  proposers:
  - cosmos1..
  result: RESULT_UNFINALIZED
  status: STATUS_SUBMITTED
  submitted_at: "2021-12-17T07:06:26.310638964Z"
  windows:
    min_execution_period: 0s
    voting_period: 432000s
  vote_state:
    abstain_count: "0"
    no_count: "0"
    veto_count: "0"
    yes_count: "0"
  summary: "Summary"
  title: "Title"
```

##### vote

The `vote` command allows users to query for vote by proposal id and voter account address.

```bash
simd query group vote [proposal-id] [voter] [flags]
```

Example:

```bash
simd query group vote 1 cosmos1..
```

Example Output:

```bash
vote:
  choice: CHOICE_YES
  metadata: AQ==
  proposal_id: "1"
  submitted_at: "2021-12-17T08:05:02.490164009Z"
  voter: cosmos1..
```

##### votes-by-proposal

The `votes-by-proposal` command allows users to query for votes by proposal id with pagination flags.

```bash
simd query group votes-by-proposal [proposal-id] [flags]
```

Example:

```bash
simd query group votes-by-proposal 1
```

Example Output:

```bash
pagination:
  next_key: null
  total: "1"
votes:
- choice: CHOICE_YES
  metadata: AQ==
  proposal_id: "1"
  submitted_at: "2021-12-17T08:05:02.490164009Z"
  voter: cosmos1..
```

##### votes-by-voter

The `votes-by-voter` command allows users to query for votes by voter account address with pagination flags.

```bash
simd query group votes-by-voter [voter] [flags]
```

Example:

```bash
simd query group votes-by-voter cosmos1..
```

Example Output:

```bash
pagination:
  next_key: null
  total: "1"
votes:
- choice: CHOICE_YES
  metadata: AQ==
  proposal_id: "1"
  submitted_at: "2021-12-17T08:05:02.490164009Z"
  voter: cosmos1..
```

### Transactions

The `tx` commands allow users to interact with the `group` module.

```bash
simd tx group --help
```

#### create-group

The `create-group` command allows users to create a group which is an aggregation of member accounts with associated weights and
an administrator account.

```bash
simd tx group create-group [admin] [metadata] [members-json-file]
```

Example:

```bash
simd tx group create-group cosmos1.. "AQ==" members.json
```

#### update-group-admin

The `update-group-admin` command allows users to update a group's admin.

```bash
simd tx group update-group-admin [admin] [group-id] [new-admin] [flags]
```

Example:

```bash
simd tx group update-group-admin cosmos1.. 1 cosmos1..
```

#### update-group-members

The `update-group-members` command allows users to update a group's members.

```bash
simd tx group update-group-members [admin] [group-id] [members-json-file] [flags]
```

Example:

```bash
simd tx group update-group-members cosmos1.. 1 members.json
```

#### update-group-metadata

The `update-group-metadata` command allows users to update a group's metadata.

```bash
simd tx group update-group-metadata [admin] [group-id] [metadata] [flags]
```

Example:

```bash
simd tx group update-group-metadata cosmos1.. 1 "AQ=="
```

#### create-group-policy

The `create-group-policy` command allows users to create a group policy which is an account associated with a group and a decision policy.

```bash
simd tx group create-group-policy [admin] [group-id] [metadata] [decision-policy] [flags]
```

Example:

```bash
simd tx group create-group-policy cosmos1.. 1 "AQ==" '{"@type":"/cosmos.group.v1.ThresholdDecisionPolicy", "threshold":"1", "windows": {"voting_period": "120h", "min_execution_period": "0s"}}'
```

#### create-group-with-policy

The `create-group-with-policy` command allows users to create a group which is an aggregation of member accounts with associated weights and an administrator account with decision policy. If the `--group-policy-as-admin` flag is set to `true`, the group policy address becomes the group and group policy admin.

```bash
simd tx group create-group-with-policy [admin] [group-metadata] [group-policy-metadata] [members-json-file] [decision-policy] [flags]
```

Example:

```bash
simd tx group create-group-with-policy cosmos1.. "AQ==" "AQ==" members.json '{"@type":"/cosmos.group.v1.ThresholdDecisionPolicy", "threshold":"1", "windows": {"voting_period": "120h", "min_execution_period": "0s"}}'
```

#### update-group-policy-admin

The `update-group-policy-admin` command allows users to update a group policy admin.

```bash
simd tx group update-group-policy-admin [admin] [group-policy-account] [new-admin] [flags]
```

Example:

```bash
simd tx group update-group-policy-admin cosmos1.. cosmos1.. cosmos1..
```

#### update-group-policy-metadata

The `update-group-policy-metadata` command allows users to update a group policy metadata.

```bash
simd tx group update-group-policy-metadata [admin] [group-policy-account] [new-metadata] [flags]
```

Example:

```bash
simd tx group update-group-policy-metadata cosmos1.. cosmos1.. "AQ=="
```

#### update-group-policy-decision-policy

The `update-group-policy-decision-policy` command allows users to update a group policy's decision policy.

```bash
simd  tx group update-group-policy-decision-policy [admin] [group-policy-account] [decision-policy] [flags]
```

Example:

```bash
simd tx group update-group-policy-decision-policy cosmos1.. cosmos1.. '{"@type":"/cosmos.group.v1.ThresholdDecisionPolicy", "threshold":"2", "windows": {"voting_period": "120h", "min_execution_period": "0s"}}'
```

#### submit-proposal

The `submit-proposal` command allows users to submit a new proposal.

```bash
simd tx group submit-proposal [group-policy-account] [proposer[,proposer]*] [msg_tx_json_file] [metadata] [flags]
```

Example:

```bash
simd tx group submit-proposal cosmos1.. cosmos1.. msg_tx.json "AQ=="
```

#### withdraw-proposal

The `withdraw-proposal` command allows users to withdraw a proposal.

```bash
simd tx group withdraw-proposal [proposal-id] [group-policy-admin-or-proposer]
```

Example:

```bash
simd tx group withdraw-proposal 1 cosmos1..
```

#### vote

The `vote` command allows users to vote on a proposal.

```bash
simd tx group vote proposal-id] [voter] [choice] [metadata] [flags]
```

Example:

```bash
simd tx group vote 1 cosmos1.. CHOICE_YES "AQ=="
```

#### exec

The `exec` command allows users to execute a proposal.

```bash
simd tx group exec [proposal-id] [flags]
```

Example:

```bash
simd tx group exec 1
```

#### leave-group

The `leave-group` command allows group member to leave the group.

```bash
simd tx group leave-group [member-address] [group-id]
```

Example:

```bash
simd tx group leave-group cosmos1... 1
```

### gRPC

A user can query the `group` module using gRPC endpoints.

#### GroupInfo

The `GroupInfo` endpoint allows users to query for group info by given group id.

```bash
cosmos.group.v1.Query/GroupInfo
```

Example:

```bash
grpcurl -plaintext \
    -d '{"group_id":1}' localhost:9090 cosmos.group.v1.Query/GroupInfo
```

Example Output:

```bash
{
  "info": {
    "groupId": "1",
    "admin": "cosmos1..",
    "metadata": "AQ==",
    "version": "1",
    "totalWeight": "3"
  }
}
```

#### GroupPolicyInfo

The `GroupPolicyInfo` endpoint allows users to query for group policy info by account address of group policy.

```bash
cosmos.group.v1.Query/GroupPolicyInfo
```

Example:

```bash
grpcurl -plaintext \
    -d '{"address":"cosmos1.."}'  localhost:9090 cosmos.group.v1.Query/GroupPolicyInfo
```

Example Output:

```bash
{
  "info": {
    "address": "cosmos1..",
    "groupId": "1",
    "admin": "cosmos1..",
    "version": "1",
    "decisionPolicy": {"@type":"/cosmos.group.v1.ThresholdDecisionPolicy","threshold":"1","windows": {"voting_period": "120h", "min_execution_period": "0s"}},
  }
}
```

#### GroupMembers

The `GroupMembers` endpoint allows users to query for group members by group id with pagination flags.

```bash
cosmos.group.v1.Query/GroupMembers
```

Example:

```bash
grpcurl -plaintext \
    -d '{"group_id":"1"}'  localhost:9090 cosmos.group.v1.Query/GroupMembers
```

Example Output:

```bash
{
  "members": [
    {
      "groupId": "1",
      "member": {
        "address": "cosmos1..",
        "weight": "1"
      }
    },
    {
      "groupId": "1",
      "member": {
        "address": "cosmos1..",
        "weight": "2"
      }
    }
  ],
  "pagination": {
    "total": "2"
  }
}
```

#### GroupsByAdmin

The `GroupsByAdmin` endpoint allows users to query for groups by admin account address with pagination flags.

```bash
cosmos.group.v1.Query/GroupsByAdmin
```

Example:

```bash
grpcurl -plaintext \
    -d '{"admin":"cosmos1.."}'  localhost:9090 cosmos.group.v1.Query/GroupsByAdmin
```

Example Output:

```bash
{
  "groups": [
    {
      "groupId": "1",
      "admin": "cosmos1..",
      "metadata": "AQ==",
      "version": "1",
      "totalWeight": "3"
    },
    {
      "groupId": "2",
      "admin": "cosmos1..",
      "metadata": "AQ==",
      "version": "1",
      "totalWeight": "3"
    }
  ],
  "pagination": {
    "total": "2"
  }
}
```

#### GroupPoliciesByGroup

The `GroupPoliciesByGroup` endpoint allows users to query for group policies by group id with pagination flags.

```bash
cosmos.group.v1.Query/GroupPoliciesByGroup
```

Example:

```bash
grpcurl -plaintext \
    -d '{"group_id":"1"}'  localhost:9090 cosmos.group.v1.Query/GroupPoliciesByGroup
```

Example Output:

```bash
{
  "GroupPolicies": [
    {
      "address": "cosmos1..",
      "groupId": "1",
      "admin": "cosmos1..",
      "version": "1",
      "decisionPolicy": {"@type":"/cosmos.group.v1.ThresholdDecisionPolicy","threshold":"1","windows":{"voting_period": "120h", "min_execution_period": "0s"}},
    },
    {
      "address": "cosmos1..",
      "groupId": "1",
      "admin": "cosmos1..",
      "version": "1",
      "decisionPolicy": {"@type":"/cosmos.group.v1.ThresholdDecisionPolicy","threshold":"1","windows":{"voting_period": "120h", "min_execution_period": "0s"}},
    }
  ],
  "pagination": {
    "total": "2"
  }
}
```

#### GroupPoliciesByAdmin

The `GroupPoliciesByAdmin` endpoint allows users to query for group policies by admin account address with pagination flags.

```bash
cosmos.group.v1.Query/GroupPoliciesByAdmin
```

Example:

```bash
grpcurl -plaintext \
    -d '{"admin":"cosmos1.."}'  localhost:9090 cosmos.group.v1.Query/GroupPoliciesByAdmin
```

Example Output:

```bash
{
  "GroupPolicies": [
    {
      "address": "cosmos1..",
      "groupId": "1",
      "admin": "cosmos1..",
      "version": "1",
      "decisionPolicy": {"@type":"/cosmos.group.v1.ThresholdDecisionPolicy","threshold":"1","windows":{"voting_period": "120h", "min_execution_period": "0s"}},
    },
    {
      "address": "cosmos1..",
      "groupId": "1",
      "admin": "cosmos1..",
      "version": "1",
      "decisionPolicy": {"@type":"/cosmos.group.v1.ThresholdDecisionPolicy","threshold":"1","windows":{"voting_period": "120h", "min_execution_period": "0s"}},
    }
  ],
  "pagination": {
    "total": "2"
  }
}
```

#### Proposal

The `Proposal` endpoint allows users to query for proposal by id.

```bash
cosmos.group.v1.Query/Proposal
```

Example:

```bash
grpcurl -plaintext \
    -d '{"proposal_id":"1"}'  localhost:9090 cosmos.group.v1.Query/Proposal
```

Example Output:

```bash
{
  "proposal": {
    "proposalId": "1",
    "address": "cosmos1..",
    "proposers": [
      "cosmos1.."
    ],
    "submittedAt": "2021-12-17T07:06:26.310638964Z",
    "groupVersion": "1",
    "GroupPolicyVersion": "1",
    "status": "STATUS_SUBMITTED",
    "result": "RESULT_UNFINALIZED",
    "voteState": {
      "yesCount": "0",
      "noCount": "0",
      "abstainCount": "0",
      "vetoCount": "0"
    },
    "windows": {
      "min_execution_period": "0s",
      "voting_period": "432000s"
    },
    "executorResult": "EXECUTOR_RESULT_NOT_RUN",
    "messages": [
      {"@type":"/cosmos.bank.v1beta1.MsgSend","amount":[{"denom":"stake","amount":"100000000"}],"fromAddress":"cosmos1..","toAddress":"cosmos1.."}
    ],
    "title": "Title",
    "summary": "Summary",
  }
}
```

#### ProposalsByGroupPolicy

The `ProposalsByGroupPolicy` endpoint allows users to query for proposals by account address of group policy with pagination flags.

```bash
cosmos.group.v1.Query/ProposalsByGroupPolicy
```

Example:

```bash
grpcurl -plaintext \
    -d '{"address":"cosmos1.."}'  localhost:9090 cosmos.group.v1.Query/ProposalsByGroupPolicy
```

Example Output:

```bash
{
  "proposals": [
    {
      "proposalId": "1",
      "address": "cosmos1..",
      "proposers": [
        "cosmos1.."
      ],
      "submittedAt": "2021-12-17T08:03:27.099649352Z",
      "groupVersion": "1",
      "GroupPolicyVersion": "1",
      "status": "STATUS_CLOSED",
      "result": "RESULT_ACCEPTED",
      "voteState": {
        "yesCount": "1",
        "noCount": "0",
        "abstainCount": "0",
        "vetoCount": "0"
      },
      "windows": {
        "min_execution_period": "0s",
        "voting_period": "432000s"
      },
      "executorResult": "EXECUTOR_RESULT_NOT_RUN",
      "messages": [
        {"@type":"/cosmos.bank.v1beta1.MsgSend","amount":[{"denom":"stake","amount":"100000000"}],"fromAddress":"cosmos1..","toAddress":"cosmos1.."}
      ],
      "title": "Title",
      "summary": "Summary",
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```

#### VoteByProposalVoter

The `VoteByProposalVoter` endpoint allows users to query for vote by proposal id and voter account address.

```bash
cosmos.group.v1.Query/VoteByProposalVoter
```

Example:

```bash
grpcurl -plaintext \
    -d '{"proposal_id":"1","voter":"cosmos1.."}'  localhost:9090 cosmos.group.v1.Query/VoteByProposalVoter
```

Example Output:

```bash
{
  "vote": {
    "proposalId": "1",
    "voter": "cosmos1..",
    "choice": "CHOICE_YES",
    "submittedAt": "2021-12-17T08:05:02.490164009Z"
  }
}
```

#### VotesByProposal

The `VotesByProposal` endpoint allows users to query for votes by proposal id with pagination flags.

```bash
cosmos.group.v1.Query/VotesByProposal
```

Example:

```bash
grpcurl -plaintext \
    -d '{"proposal_id":"1"}'  localhost:9090 cosmos.group.v1.Query/VotesByProposal
```

Example Output:

```bash
{
  "votes": [
    {
      "proposalId": "1",
      "voter": "cosmos1..",
      "choice": "CHOICE_YES",
      "submittedAt": "2021-12-17T08:05:02.490164009Z"
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```

#### VotesByVoter

The `VotesByVoter` endpoint allows users to query for votes by voter account address with pagination flags.

```bash
cosmos.group.v1.Query/VotesByVoter
```

Example:

```bash
grpcurl -plaintext \
    -d '{"voter":"cosmos1.."}'  localhost:9090 cosmos.group.v1.Query/VotesByVoter
```

Example Output:

```bash
{
  "votes": [
    {
      "proposalId": "1",
      "voter": "cosmos1..",
      "choice": "CHOICE_YES",
      "submittedAt": "2021-12-17T08:05:02.490164009Z"
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```

### REST

A user can query the `group` module using REST endpoints.

#### GroupInfo

The `GroupInfo` endpoint allows users to query for group info by given group id.

```bash
/cosmos/group/v1/group_info/{group_id}
```

Example:

```bash
curl localhost:1317/cosmos/group/v1/group_info/1
```

Example Output:

```bash
{
  "info": {
    "id": "1",
    "admin": "cosmos1..",
    "metadata": "AQ==",
    "version": "1",
    "total_weight": "3"
  }
}
```

#### GroupPolicyInfo

The `GroupPolicyInfo` endpoint allows users to query for group policy info by account address of group policy.

```bash
/cosmos/group/v1/group_policy_info/{address}
```

Example:

```bash
curl localhost:1317/cosmos/group/v1/group_policy_info/cosmos1..
```

Example Output:

```bash
{
  "info": {
    "address": "cosmos1..",
    "group_id": "1",
    "admin": "cosmos1..",
    "metadata": "AQ==",
    "version": "1",
    "decision_policy": {
      "@type": "/cosmos.group.v1.ThresholdDecisionPolicy",
      "threshold": "1",
      "windows": {
        "voting_period": "120h",
        "min_execution_period": "0s"
      }
    },
  }
}
```

#### GroupMembers

The `GroupMembers` endpoint allows users to query for group members by group id with pagination flags.

```bash
/cosmos/group/v1/group_members/{group_id}
```

Example:

```bash
curl localhost:1317/cosmos/group/v1/group_members/1
```

Example Output:

```bash
{
  "members": [
    {
      "group_id": "1",
      "member": {
        "address": "cosmos1..",
        "weight": "1",
        "metadata": "AQ=="
      }
    },
    {
      "group_id": "1",
      "member": {
        "address": "cosmos1..",
        "weight": "2",
        "metadata": "AQ=="
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "2"
  }
}
```

#### GroupsByAdmin

The `GroupsByAdmin` endpoint allows users to query for groups by admin account address with pagination flags.

```bash
/cosmos/group/v1/groups_by_admin/{admin}
```

Example:

```bash
curl localhost:1317/cosmos/group/v1/groups_by_admin/cosmos1..
```

Example Output:

```bash
{
  "groups": [
    {
      "id": "1",
      "admin": "cosmos1..",
      "metadata": "AQ==",
      "version": "1",
      "total_weight": "3"
    },
    {
      "id": "2",
      "admin": "cosmos1..",
      "metadata": "AQ==",
      "version": "1",
      "total_weight": "3"
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "2"
  }
}
```

#### GroupPoliciesByGroup

The `GroupPoliciesByGroup` endpoint allows users to query for group policies by group id with pagination flags.

```bash
/cosmos/group/v1/group_policies_by_group/{group_id}
```

Example:

```bash
curl localhost:1317/cosmos/group/v1/group_policies_by_group/1
```

Example Output:

```bash
{
  "group_policies": [
    {
      "address": "cosmos1..",
      "group_id": "1",
      "admin": "cosmos1..",
      "metadata": "AQ==",
      "version": "1",
      "decision_policy": {
        "@type": "/cosmos.group.v1.ThresholdDecisionPolicy",
        "threshold": "1",
        "windows": {
          "voting_period": "120h",
          "min_execution_period": "0s"
      }
      },
    },
    {
      "address": "cosmos1..",
      "group_id": "1",
      "admin": "cosmos1..",
      "metadata": "AQ==",
      "version": "1",
      "decision_policy": {
        "@type": "/cosmos.group.v1.ThresholdDecisionPolicy",
        "threshold": "1",
        "windows": {
          "voting_period": "120h",
          "min_execution_period": "0s"
      }
      },
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "2"
  }
}
```

#### GroupPoliciesByAdmin

The `GroupPoliciesByAdmin` endpoint allows users to query for group policies by admin account address with pagination flags.

```bash
/cosmos/group/v1/group_policies_by_admin/{admin}
```

Example:

```bash
curl localhost:1317/cosmos/group/v1/group_policies_by_admin/cosmos1..
```

Example Output:

```bash
{
  "group_policies": [
    {
      "address": "cosmos1..",
      "group_id": "1",
      "admin": "cosmos1..",
      "metadata": "AQ==",
      "version": "1",
      "decision_policy": {
        "@type": "/cosmos.group.v1.ThresholdDecisionPolicy",
        "threshold": "1",
        "windows": {
          "voting_period": "120h",
          "min_execution_period": "0s"
      } 
      },
    },
    {
      "address": "cosmos1..",
      "group_id": "1",
      "admin": "cosmos1..",
      "metadata": "AQ==",
      "version": "1",
      "decision_policy": {
        "@type": "/cosmos.group.v1.ThresholdDecisionPolicy",
        "threshold": "1",
        "windows": {
          "voting_period": "120h",
          "min_execution_period": "0s"
      }
      },
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "2"
  }
```

#### Proposal

The `Proposal` endpoint allows users to query for proposal by id.

```bash
/cosmos/group/v1/proposal/{proposal_id}
```

Example:

```bash
curl localhost:1317/cosmos/group/v1/proposal/1
```

Example Output:

```bash
{
  "proposal": {
    "proposal_id": "1",
    "address": "cosmos1..",
    "metadata": "AQ==",
    "proposers": [
      "cosmos1.."
    ],
    "submitted_at": "2021-12-17T07:06:26.310638964Z",
    "group_version": "1",
    "group_policy_version": "1",
    "status": "STATUS_SUBMITTED",
    "result": "RESULT_UNFINALIZED",
    "vote_state": {
      "yes_count": "0",
      "no_count": "0",
      "abstain_count": "0",
      "veto_count": "0"
    },
    "windows": {
      "min_execution_period": "0s",
      "voting_period": "432000s"
    },
    "executor_result": "EXECUTOR_RESULT_NOT_RUN",
    "messages": [
      {
        "@type": "/cosmos.bank.v1beta1.MsgSend",
        "from_address": "cosmos1..",
        "to_address": "cosmos1..",
        "amount": [
          {
            "denom": "stake",
            "amount": "100000000"
          }
        ]
      }
    ],
    "title": "Title",
    "summary": "Summary",
  }
}
```

#### ProposalsByGroupPolicy

The `ProposalsByGroupPolicy` endpoint allows users to query for proposals by account address of group policy with pagination flags.

```bash
/cosmos/group/v1/proposals_by_group_policy/{address}
```

Example:

```bash
curl localhost:1317/cosmos/group/v1/proposals_by_group_policy/cosmos1..
```

Example Output:

```bash
{
  "proposals": [
    {
      "id": "1",
      "group_policy_address": "cosmos1..",
      "metadata": "AQ==",
      "proposers": [
        "cosmos1.."
      ],
      "submit_time": "2021-12-17T08:03:27.099649352Z",
      "group_version": "1",
      "group_policy_version": "1",
      "status": "STATUS_CLOSED",
      "result": "RESULT_ACCEPTED",
      "vote_state": {
        "yes_count": "1",
        "no_count": "0",
        "abstain_count": "0",
        "veto_count": "0"
      },
      "windows": {
        "min_execution_period": "0s",
        "voting_period": "432000s"
      },
      "executor_result": "EXECUTOR_RESULT_NOT_RUN",
      "messages": [
        {
          "@type": "/cosmos.bank.v1beta1.MsgSend",
          "from_address": "cosmos1..",
          "to_address": "cosmos1..",
          "amount": [
            {
              "denom": "stake",
              "amount": "100000000"
            }
          ]
        }
      ]
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "1"
  }
}
```

#### VoteByProposalVoter

The `VoteByProposalVoter` endpoint allows users to query for vote by proposal id and voter account address.

```bash
/cosmos/group/v1/vote_by_proposal_voter/{proposal_id}/{voter}
```

Example:

```bash
curl localhost:1317/cosmos/group/v1beta1/vote_by_proposal_voter/1/cosmos1..
```

Example Output:

```bash
{
  "vote": {
    "proposal_id": "1",
    "voter": "cosmos1..",
    "choice": "CHOICE_YES",
    "metadata": "AQ==",
    "submitted_at": "2021-12-17T08:05:02.490164009Z"
  }
}
```

#### VotesByProposal

The `VotesByProposal` endpoint allows users to query for votes by proposal id with pagination flags.

```bash
/cosmos/group/v1/votes_by_proposal/{proposal_id}
```

Example:

```bash
curl localhost:1317/cosmos/group/v1/votes_by_proposal/1
```

Example Output:

```bash
{
  "votes": [
    {
      "proposal_id": "1",
      "voter": "cosmos1..",
      "option": "CHOICE_YES",
      "metadata": "AQ==",
      "submit_time": "2021-12-17T08:05:02.490164009Z"
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "1"
  }
}
```

#### VotesByVoter

The `VotesByVoter` endpoint allows users to query for votes by voter account address with pagination flags.

```bash
/cosmos/group/v1/votes_by_voter/{voter}
```

Example:

```bash
curl localhost:1317/cosmos/group/v1/votes_by_voter/cosmos1..
```

Example Output:

```bash
{
  "votes": [
    {
      "proposal_id": "1",
      "voter": "cosmos1..",
      "choice": "CHOICE_YES",
      "metadata": "AQ==",
      "submitted_at": "2021-12-17T08:05:02.490164009Z"
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "1"
  }
}
```

## Metadata

The group module has four locations for metadata where users can provide further context about the on-chain actions they are taking. By default all metadata fields have a 255 character length field where metadata can be stored in json format, either on-chain or off-chain depending on the amount of data required. Here we provide a recommendation for the json structure and where the data should be stored. There are two important factors in making these recommendations. First, that the group and gov modules are consistent with one another, note the number of proposals made by all groups may be quite large. Second, that client applications such as block explorers and governance interfaces have confidence in the consistency of metadata structure across chains.

### Proposal

Location: off-chain as json object stored on IPFS (mirrors [gov proposal](../gov/README.md#metadata))

```json
{
  "title": "",
  "authors": [""],
  "summary": "",
  "details": "",
  "proposal_forum_url": "",
  "vote_option_context": "",
}
```

:::note
The `authors` field is an array of strings, this is to allow for multiple authors to be listed in the metadata.
In v0.46, the `authors` field is a comma-separated string. Frontends are encouraged to support both formats for backwards compatibility.
:::

### Vote

Location: on-chain as json within 255 character limit (mirrors [gov vote](../gov/README.md#metadata))

```json
{
  "justification": "",
}
```

### Group

Location: off-chain as json object stored on IPFS

```json
{
  "name": "",
  "description": "",
  "group_website_url": "",
  "group_forum_url": "",
}
```

### Decision policy

Location: on-chain as json within 255 character limit

```json
{
  "name": "",
  "description": "",
}
```
