---
sidebar_position: 1
---

# `x/gov`

## Abstract

This paper specifies the Governance module of the Cosmos SDK, which was first
described in the [Cosmos Whitepaper](https://cosmos.network/about/whitepaper) in
June 2016.

The module enables Cosmos SDK based blockchain to support an on-chain governance
system. In this system, holders of the native staking token of the chain can vote
on proposals on a 1 token 1 vote basis. Next is a list of features the module
currently supports:

* **Proposal submission:** Users can submit proposals with a deposit. Once the
minimum deposit is reached, the proposal enters voting period. The minimum deposit can be reached by collecting deposits from different users (including proposer) within deposit period.
* **Vote:** Participants can vote on proposals that reached MinDeposit and entered voting period.
* **Inheritance and penalties:** Delegators, by default, inherit their validator's vote if they don't vote themselves.
* **Claiming deposit:** Users that deposited on proposals can recover their
deposits if the proposal was accepted or rejected. If the proposal was vetoed, or never entered voting period (minimum deposit not reached within deposit period), the deposit is burned (or refunded depending on the gov parameters).

## Contents

The following specification uses *ATOM* as the native staking token. The module
can be adapted to any Proof-Of-Stake blockchain by replacing *ATOM* with the native
staking token of the chain.

* [Concepts](#concepts)
    * [Proposal submission](#proposal-submission)
    * [Deposit](#deposit)
    * [Vote](#vote)
* [State](#state)
    * [Proposals](#proposals)
    * [Parameters and base types](#parameters-and-base-types)
    * [Deposit](#deposit-1)
    * [ValidatorGovInfo](#validatorgovinfo)
    * [Legacy Proposal](#legacy-proposal)
* [Messages](#messages)
    * [Proposal Submission](#proposal-submission-1)
    * [Deposit](#deposit-2)
    * [Vote](#vote-1)
* [Events](#events)
    * [EndBlocker](#endblocker)
    * [Handlers](#handlers)
* [Parameters](#parameters)
* [Client](#client)
    * [CLI](#cli)
    * [gRPC](#grpc)
    * [REST](#rest)
* [Metadata](#metadata)
    * [Proposal](#proposal-3)
    * [Vote](#vote-5)

## Concepts

The governance process is divided into a few steps that are outlined below:

* **Proposal submission:** Proposal is submitted to the blockchain with a
  deposit.
* **Vote:** Once deposit reaches a certain value (`MinDeposit`), proposal is
  confirmed and vote opens. Bonded Atom holders can then send `TxGovVote`
  transactions to vote on the proposal.
* **Execution** After a period of time, the votes are tallied and depending
  on the result, the messages in the proposal will be executed.

### Proposal submission

#### Right to submit a proposal

Every account can submit proposals by sending a `MsgSubmitProposal` transaction.
Once a proposal is submitted, it is identified by its unique `proposalID`.

#### Proposal Messages

A proposal includes an array of `sdk.Msg`s which are executed automatically if the
proposal passes. The messages are executed by the governance `ModuleAccount` itself. Modules
such as `x/upgrade`, that want to allow certain messages to be executed by governance
only should add a whitelist within the respective msg server, granting the governance
module the right to execute the message once a quorum has been reached. The governance
module uses the core `router.Service` to check that these messages are correctly constructed
and have a respective path to execute on but do not perform a full validity check.

:::warning
Ultimately, governance is able to execute any proposal, even if they weren't meant to be executed by governance (ie. no authority present).
Messages without authority are messages meant to be executed by users. Using the `MsgSudoExec` message in a proposal, let governance be able to execute any message, effectively acting as super user.
:::

### Deposit

To prevent spam, proposals must be submitted with a deposit in the coins defined by
the `MinDeposit` param.

When a proposal is submitted, it has to be accompanied with a deposit that must be
strictly positive, but can be inferior to `MinDeposit`. The submitter doesn't need
to pay for the entire deposit on their own. The newly created proposal is stored in
an *inactive proposal queue* and stays there until its deposit passes the `MinDeposit`.
Other token holders can increase the proposal's deposit by sending a `Deposit`
transaction. If a proposal doesn't pass the `MinDeposit` before the deposit end time
(the time when deposits are no longer accepted), the proposal will be destroyed: the
proposal will be removed from state and the deposit will be burned (see x/gov `EndBlocker`).
When a proposal deposit passes the `MinDeposit` threshold (even during the proposal
submission) before the deposit end time, the proposal will be moved into the
*active proposal queue* and the voting period will begin.

The deposit is kept in escrow and held by the governance `ModuleAccount` until the
proposal is finalized (passed or rejected).

#### Deposit refund and burn

When a proposal is finalized, the coins from the deposit are either refunded or burned
according to the final tally of the proposal and the governance module parameters:

* All refunded or burned deposits are removed from the state. Events are issued when
  burning or refunding a deposit.
* If the proposal is approved or rejected but *not* vetoed, each deposit will be
  automatically refunded to its respective depositor (transferred from the governance
  `ModuleAccount`).
* If the proposal is marked as Spam, the deposit will be burned.

For other cases, they are three parameters that define if the deposit of a proposal should be burned or returned to the depositors.

* `BurnVoteVeto` burns the proposal deposit if the proposal gets vetoed.
* `BurnVoteQuorum` burns the proposal deposit if the vote does not reach quorum.
* `BurnProposalDepositPrevote` burns the proposal deposit if it does not enter the voting phase.

> Note: These parameters are modifiable via governance.

### Vote

#### Participants

*Participants* are users that have the right to vote on proposals. On the
Cosmos Hub, participants are bonded Atom holders. Unbonded Atom holders and
other users do not get the right to participate in governance. However, they
can submit and deposit on proposals.

Note that when *participants* have bonded and unbonded Atoms, their voting power is calculated from their bonded Atom holdings only.

#### Voting period

Once a proposal reaches `MinDeposit`, it immediately enters `Voting period`. We
define `Voting period` as the interval between the moment the vote opens and
the moment the vote closes. The default value of `Voting period` is 2 weeks but is modifiable at genesis or governance.

#### Option set

The option set of a proposal refers to the set of choices a participant can
choose from when casting its vote.

The initial option set includes the following options:

* `Yes` / `Option 1`
* `Abstain` / `Option 2`
* `No` / `Option 3`
* `NoWithVeto` / `Option 4`
* `Spam` / `Option Spam`

`NoWithVeto` counts as `No` but also adds a `Veto` vote. `Abstain` option
allows voters to signal that they do not intend to vote in favor or against the
proposal but accept the result of the vote.

#### Weighted Votes

[ADR-037](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-037-gov-split-vote.md) introduces the weighted vote feature which allows a staker to split their votes into several voting options. For example, it could use 70% of its voting power to vote Yes and 30% of its voting power to vote No.

Often times the entity owning that address might not be a single individual. For example, a company might have different stakeholders who want to vote differently, and so it makes sense to allow them to split their voting power. Currently, it is not possible for them to do "passthrough voting" and giving their users voting rights over their tokens. However, with this system, exchanges can poll their users for voting preferences, and then vote on-chain proportionally to the results of the poll.

To represent weighted vote on chain, we use the following Protobuf message.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/gov/proto/cosmos/gov/v1/gov.proto#L56-L63
```

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/gov/proto/cosmos/gov/v1/gov.proto#L202-L219
```

For a weighted vote to be valid, the `options` field must not contain duplicate vote options, and the sum of weights of all options must be equal to 1.

The maximum number of weighted vote options can be limited by the developer via a config parameter, named `MaxVoteOptionsLen`, which gets passed into the gov keeper.

### Quorum

Quorum is defined as the minimum percentage of voting power that needs to be
cast on a proposal for the result to be valid.

### Expedited Quorum

Expedited Quorum is defined as the minimum percentage of voting power that needs to be
cast on an **expedited** proposal for the result to be valid.

### Yes Quorum

Yes quorum is a more restrictive quorum that is used to determine if a proposal passes.
It is defined as the minimum percentage of voting power that needs to have voted `Yes` for the proposal to pass.
It differs from `Threshold` as it takes the whole voting power into account, not only `Yes` and `No` votes.
By default, `YesQuorum` is set to 0, which means no minimum.

### Proposal Types

Proposal types have been introduced in [ADR-069](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-069-gov-improvements.md).

#### Standard proposal

A standard proposal is a proposal that can contain any messages. The proposal follows the standard governance flow and governance parameters.

#### Expedited Proposal

A proposal can be expedited, making the proposal use shorter voting duration and a higher tally threshold by its default. If an expedited proposal fails to meet the threshold within the scope of shorter voting duration, the expedited proposal is then converted to a regular proposal and restarts voting under regular voting conditions.

#### Optimistic Proposal

An optimistic proposal is a proposal that passes unless a threshold of NO votes is reached.
Voter can only vote NO on the proposal. If the NO threshold is reached, the optimistic proposal is converted to a standard proposal.

That threshold is defined by the `optimistic_rejected_threshold` governance parameter.
A chain can optionally set a list of authorized addresses that can submit optimistic proposals using the `optimistic_authorized_addresses` governance parameter.

#### Multiple Choice Proposals

A multiple choice proposal is a proposal where the voting options can be defined by the proposer.
The number of voting options is limited to a maximum of **4**.
Multiple choice proposals, contrary to any other proposal type, cannot have messages to execute. They are only text proposals.

### Threshold

Threshold is defined as the minimum proportion of `Yes` votes (excluding `Abstain` votes) for the proposal to be accepted.

Initially, the threshold is set at 50% of `Yes` votes, excluding `Abstain` votes.
A possibility to veto exists if more than 1/3rd of all votes are `NoWithVeto` votes.  
Note, both of these values are derived from the `Params`
on-chain parameter, which is modifiable by governance.
This means that proposals are accepted iff:

* There exist bonded tokens.
* Quorum has been achieved.
* The proportion of `Abstain` votes is inferior to 1/1.
* The proportion of `NoWithVeto` votes is inferior to 1/3, including
  `Abstain` votes.
* The proportion of `Yes` votes, excluding `Abstain` votes, at the end of
  the voting period is superior to 1/2.

For expedited proposals, by default, the threshold is higher than with a *normal proposal*, namely, 66.7%.

### Inheritance

If a delegator does not vote, by default, it will inherit its validator vote.

* If the delegator votes before its validator, it will not inherit from the validator's vote.
* If the delegator votes after its validator, it will override its validator  vote with its own.
  If the proposal is urgent, it is possible that the vote will close before delegators have a chance to react and
  override their validator's vote. This is not a problem, as proposals require more than 2/3rd of the total voting power to pass, when tallied at the end of the voting period. Because as little as 1/3 + 1 validation power could collude to censor transactions, non-collusion is already assumed for ranges exceeding this threshold.

This behavior can be changed by passing a custom tally calculation function to the governance module.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/gov/keeper/config.go#L33-L35
```

#### Validator’s punishment for non-voting

At present, validators are not punished for failing to vote.

#### Execution

Execution is the process of executing the messages contained in a proposal. The execution phase will commence after the proposal has been accepted by the network. The messages contained in the proposal will be executed in the order they were submitted. All messages must be executed successfully for the proposal to be considered successful. I

If a proposal passes but fails to execute, the proposal will be marked as `StatusFailed`. This status is different from `StatusRejected`, which is used when a proposal fails to pass.

Execution has an upper limit on how much gas can be consumed in a single block. This limit is defined by the `ProposalExecutionGas` parameter.

## State

The governance module uses [collections](https://docs.cosmos.network/v0.50/build/packages/collections) for state management.

### Constitution

`Constitution` is found in the genesis state.  It is a string field intended to be used to describe the purpose of a particular blockchain, and its expected norms.  A few examples of how the constitution field can be used:

* define the purpose of the chain, laying a foundation for its future development
* set expectations for delegators
* set expectations for validators
* define the chain's relationship to "meatspace" entities, like a foundation or corporation

Since this is more of a social feature than a technical feature, we'll now get into some items that may have been useful to have in a genesis constitution:

* What limitations on governance exist, if any?
    * is it okay for the community to slash the wallet of a whale that they no longer feel that they want around? (viz: Juno Proposal 4 and 16)
    * can governance "socially slash" a validator who is using unapproved MEV? (viz: commonwealth.im/osmosis)
    * In the event of an economic emergency, what should validators do?
        * Terra crash of May, 2022, saw validators choose to run a new binary with code that had not been approved by governance, because the governance token had been inflated to nothing.
* What is the purpose of the chain, specifically?
    * best example of this is the Cosmos hub, where different founding groups, have different interpretations of the purpose of the network.

This genesis entry, "constitution" hasn't been designed for existing chains, who should likely just ratify a constitution using their governance system.  Instead, this is for new chains. It will allow for validators to have a much clearer idea of purpose and the expectations placed on them while operating their nodes. Likewise, for community members, the constitution will give them some idea of what to expect from both the "chain team" and the validators, respectively.

This constitution is designed to be immutable, and placed only in genesis, though that could change over time by a pull request to the cosmos-sdk that allows for the constitution to be changed by governance. Communities wishing to make amendments to their original constitution should use the governance mechanism and a "signaling proposal" to do exactly that.

**Ideal use scenario for a cosmos chain constitution**

As a chain developer, you decide that you'd like to provide clarity to your key user groups:

* validators
* token holders
* developers (yourself)

You use the constitution to immutably store some Markdown in genesis, so that when difficult questions come up, the constitution can provide guidance to the community.

### Proposals

`Proposal` objects are used to tally votes and generally track the proposal's state.
They contain an array of arbitrary `sdk.Msg`'s which the governance module will attempt
to resolve and then execute if the proposal passes. `Proposal`'s are identified by a
unique id and contains a series of timestamps: `submit_time`, `deposit_end_time`,
`voting_start_time`, `voting_end_time` which track the lifecycle of a proposal

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/gov/proto/cosmos/gov/v1/gov.proto#L78-L134
```

A proposal will generally require more than just a set of messages to explain its
purpose but need some greater justification and allow a means for interested participants
to discuss and debate the proposal.
In most cases, **it is encouraged to have an off-chain system that supports the on-chain governance process**.
To accommodate for this, a proposal contains a special **`metadata`** field, a string,
which can be used to add context to the proposal. The `metadata` field allows custom use for networks,
however, it is expected that the field contains a URL or some form of CID using a system such as
[IPFS](https://docs.ipfs.io/concepts/content-addressing/). To support the case of
interoperability across networks, the SDK recommends that the `metadata` represents
the following `JSON` template:

```json
{
  "title": "...",
  "description": "...",
  "forum": "...", // a link to the discussion platform (i.e. Discord)
  "other": "..." // any extra data that doesn't correspond to the other fields
}
```

This makes it far easier for clients to support multiple networks.

Fields metadata, title and summary have a maximum length that is chosen by the app developer, and
passed into the gov keeper as a config (`x/gov/keeper/config`).

The default maximum length are:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/gov/keeper/config.go#L38-L47
```

#### Writing a module that uses governance

There are many aspects of a chain, or of the individual modules that you may want to
use governance to perform such as changing various parameters. This is very simple
to do. First, write out your message types and `MsgServer` implementation. Add an
`authority` field to the keeper which will be populated in the constructor with the
governance module account: `govKeeper.GetGovernanceAccount().GetAddress()`. Then for
the methods in the `msg_server.go`, perform a check on the message that the signer
matches `authority`. This will prevent any user from executing that message.

:::warning
Note, any message can be executed by governance if embedded in `MsgSudoExec`.
:::

### Parameters and base types

`Params` define the rules according to which votes are run. If governance wants to change a
parameter it can do so by submitting a gov `MsgUpdateParams` governance proposal.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/gov/proto/cosmos/gov/v1/gov.proto#L259-L348
```

Parameters are stored in the `gov` store under the key `ParamsKey`.

Additionally, we introduce some basic types:

```go
type ProposalStatus byte

const (
    StatusNil           ProposalStatus = 0x00
    StatusDepositPeriod ProposalStatus = 0x01  // Proposal is submitted. Participants can deposit on it but not vote
    StatusVotingPeriod  ProposalStatus = 0x02  // MinDeposit is reached, participants can vote
    StatusPassed        ProposalStatus = 0x03  // Proposal passed and successfully executed
    StatusRejected      ProposalStatus = 0x04  // Proposal has been rejected
    StatusFailed        ProposalStatus = 0x05  // Proposal passed but failed execution
)
```

### Deposit

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/gov/proto/cosmos/gov/v1/gov.proto#L65-L76
```

### ValidatorGovInfo

This type is used in a temp map when tallying

```go
  type ValidatorGovInfo struct {
    Minus     sdk.Dec
    Vote      Vote
  }
```

### Legacy Proposal

:::warning
Legacy proposals (`gov/v1beta1`) are deprecated. Use the new proposal flow by granting the governance module the right to execute the message.
:::

## Messages

### Proposal Submission

Proposals can be submitted by any account via a `MsgSubmitProposal` or a `MsgSubmitMultipleChoiceProposal` transaction.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/gov/proto/cosmos/gov/v1/tx.proto#L64-L102
```

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/gov/proto/cosmos/gov/v1/tx.proto#L229-L256
```

:::tip
A multiple choice proposal is a proposal where the voting options can be defined by the proposer.
It cannot have messages to execute. It is only a text proposal.
This means submitting a multiple choice proposal using `MsgSubmitProposal` is invalid, as vote options cannot be defined.
:::

All `sdk.Msgs` passed into the `messages` field of a `MsgSubmitProposal` message
must be registered in the app's message router. Each of these messages must
have one signer, namely the gov module account. And finally, the metadata length
must not be larger than the `maxMetadataLen` config passed into the gov keeper.
The `initialDeposit` must be strictly positive and conform to the accepted denom of the `MinDeposit` param.

### Deposit

Once a proposal is submitted, if `Proposal.TotalDeposit < GovParams.MinDeposit`, Atom holders can send
`MsgDeposit` transactions to increase the proposal's deposit.

A deposit is accepted iff:

* The proposal exists
* The proposal is not in the voting period
* The deposited coins are conform to the accepted denom from the `MinDeposit` param

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/gov/proto/cosmos/gov/v1/tx.proto#L167-L180
```

### Vote

Once `GovParams.MinDeposit` is reached, voting period starts. From there,
bonded Atom holders are able to send `MsgVote` transactions to cast their
vote on the proposal.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/gov/proto/cosmos/gov/v1/tx.proto#L125-L141
```

## Events

The governance module emits the following events:

### EndBlocker

| Type              | Attribute Key   | Attribute Value  |
| ----------------- | --------------- | ---------------- |
| inactive_proposal | proposal_id     | {proposalID}     |
| inactive_proposal | proposal_result | {proposalResult} |
| active_proposal   | proposal_id     | {proposalID}     |
| active_proposal   | proposal_result | {proposalResult} |

### Handlers

#### MsgSubmitProposal, MsgSubmitMultipleChoiceProposal

| Type                | Attribute Key       | Attribute Value |
| ------------------- | ------------------- | --------------- |
| submit_proposal     | proposal_id         | {proposalID}    |
| submit_proposal [0] | voting_period_start | {proposalID}    |
| proposal_deposit    | amount              | {depositAmount} |
| proposal_deposit    | proposal_id         | {proposalID}    |
| message             | module              | governance      |
| message             | action              | submit_proposal |
| message             | sender              | {senderAddress} |

* [0] Event only emitted if the voting period starts during the submission.

#### MsgVote

| Type          | Attribute Key | Attribute Value |
| ------------- | ------------- | --------------- |
| proposal_vote | option        | {voteOption}    |
| proposal_vote | proposal_id   | {proposalID}    |
| message       | module        | governance      |
| message       | action        | vote            |
| message       | sender        | {senderAddress} |

#### MsgVoteWeighted

| Type          | Attribute Key | Attribute Value       |
| ------------- | ------------- | --------------------- |
| proposal_vote | option        | {weightedVoteOptions} |
| proposal_vote | proposal_id   | {proposalID}          |
| message       | module        | governance            |
| message       | action        | vote                  |
| message       | sender        | {senderAddress}       |

#### MsgDeposit

| Type                 | Attribute Key       | Attribute Value |
| -------------------- | ------------------- | --------------- |
| proposal_deposit     | amount              | {depositAmount} |
| proposal_deposit     | proposal_id         | {proposalID}    |
| proposal_deposit [0] | voting_period_start | {proposalID}    |
| message              | module              | governance      |
| message              | action              | deposit         |
| message              | sender              | {senderAddress} |

* [0] Event only emitted if the voting period starts during the submission.

## Parameters

The governance module contains the following parameters:

| Key                             | Type              | Example                                 |
| ------------------------------- | ----------------- | --------------------------------------- |
| min_deposit                     | array (coins)     | [{"denom":"uatom","amount":"10000000"}] |
| max_deposit_period              | string (time ns)  | "172800000000000" (17280s)              |
| voting_period                   | string (time ns)  | "172800000000000" (17280s)              |
| quorum                          | string (dec)      | "0.334000000000000000"                  |
| yes_quorum                      | string (dec)      | "0.4"                                   |
| threshold                       | string (dec)      | "0.500000000000000000"                  |
| veto                            | string (dec)      | "0.334000000000000000"                  |
| expedited_threshold             | string (time ns)  | "0.667000000000000000"                  |
| expedited_voting_period         | string (time ns)  | "86400000000000" (8600s)                |
| expedited_min_deposit           | array (coins)     | [{"denom":"uatom","amount":"50000000"}] |
| expedited_quorum                | string (dec)      | "0.5"                                   |
| burn_proposal_deposit_prevote   | bool              | false                                   |
| burn_vote_quorum                | bool              | false                                   |
| burn_vote_veto                  | bool              | true                                    |
| min_initial_deposit_ratio       | string            | "0.1"                                   |
| proposal_cancel_ratio           | string (dec)      | "0.5"                                   |
| proposal_cancel_dest            | string (address)  | "cosmos1.." or empty for burn           |
| proposal_cancel_max_period      | string (dec)      | "0.5"                                   |
| optimistic_rejected_threshold   | string (dec)      | "0.1"                                   |
| optimistic_authorized_addresses | array (addresses) | []                                      |

**NOTE**: The governance module contains parameters that are objects unlike other
modules. If only a subset of parameters are desired to be changed, only they need
to be included and not the entire parameter object structure.

### Message Based Parameters

In addition to the parameters above, the governance module can also be configured to have different parameters for a given proposal message.

| Key           | Type             | Example                    |
| ------------- | ---------------- | -------------------------- |
| voting_period | string (time ns) | "172800000000000" (17280s) |
| yes_quorum    | string (dec)     | "0.4"                      |
| quorum        | string (dec)     | "0.334000000000000000"     |
| threshold     | string (dec)     | "0.500000000000000000"     |
| veto          | string (dec)     | "0.334000000000000000"     |

If configured, these params will take precedence over the global params for a specific proposal.

:::warning
Currently, messaged based parameters limit the number of messages that can be included in a proposal.
Only messages that have the same message parameters can be included in the same proposal.
:::

## Metadata

The gov module has two locations for metadata where users can provide further context about the on-chain actions they are taking. By default all metadata fields have a 255 character length field where metadata can be stored in json format, either on-chain or off-chain depending on the amount of data required. Here we provide a recommendation for the json structure and where the data should be stored. There are two important factors in making these recommendations. First, that the gov and group modules are consistent with one another, note the number of proposals made by all groups may be quite large. Second, that client applications such as block explorers and governance interfaces have confidence in the consistency of metadata structure across chains.

### Proposal

Location: off-chain as json object stored on IPFS (mirrors [group proposal](../group/README.md#metadata))

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

Location: on-chain as json within 255 character limit (mirrors [group vote](../group/README.md#metadata))

```json
{
  "justification": "",
}
```

## Client

### CLI

A user can query and interact with the `gov` module using the CLI.

#### Query

The `query` commands allow users to query `gov` state.

```bash
simd query gov --help
```

##### deposit

The `deposit` command allows users to query a deposit for a given proposal from a given depositor.

```bash
simd query gov deposit [proposal-id] [depositor-addr] [flags]
```

Example:

```bash
simd query gov deposit 1 cosmos1..
```

Example Output:

```bash
amount:
- amount: "100"
  denom: stake
depositor: cosmos1..
proposal_id: "1"
```

##### deposits

The `deposits` command allows users to query all deposits for a given proposal.

```bash
simd query gov deposits [proposal-id] [flags]
```

Example:

```bash
simd query gov deposits 1
```

Example Output:

```bash
deposits:
- amount:
  - amount: "100"
    denom: stake
  depositor: cosmos1..
  proposal_id: "1"
pagination:
  next_key: null
  total: "0"
```

##### params

The `params` command allows users to query all parameters for the `gov` module.

```bash
simd query gov params [flags]
```

Example:

```bash
simd query gov params
```

Example Output:

```bash
params:
  expedited_min_deposit:
  - amount: "50000000"
    denom: stake
  expedited_threshold: "0.670000000000000000"
  expedited_voting_period: 86400s
  max_deposit_period: 172800s
  min_deposit:
  - amount: "10000000"
    denom: stake
  min_initial_deposit_ratio: "0.000000000000000000"
  proposal_cancel_burn_rate: "0.500000000000000000"
  quorum: "0.334000000000000000"
  threshold: "0.500000000000000000"
  veto_threshold: "0.334000000000000000"
  voting_period: 172800s
```

##### proposal

The `proposal` command allows users to query a given proposal.

```bash
simd query gov proposal [proposal-id] [flags]
```

Example:

```bash
simd query gov proposal 1
```

Example Output:

```bash
deposit_end_time: "2022-03-30T11:50:20.819676256Z"
final_tally_result:
  abstain_count: "0"
  no_count: "0"
  no_with_veto_count: "0"
  yes_count: "0"
id: "1"
messages:
- '@type': /cosmos.bank.v1beta1.MsgSend
  amount:
  - amount: "10"
    denom: stake
  from_address: cosmos1..
  to_address: cosmos1..
metadata: AQ==
status: PROPOSAL_STATUS_DEPOSIT_PERIOD
submit_time: "2022-03-28T11:50:20.819676256Z"
total_deposit:
- amount: "10"
  denom: stake
voting_end_time: null
voting_start_time: null
```

##### proposals

The `proposals` command allows users to query all proposals with optional filters.

```bash
simd query gov proposals [flags]
```

Example:

```bash
simd query gov proposals
```

Example Output:

```bash
pagination:
  next_key: null
  total: "0"
proposals:
- deposit_end_time: "2022-03-30T11:50:20.819676256Z"
  final_tally_result:
    abstain_count: "0"
    no_count: "0"
    no_with_veto_count: "0"
    yes_count: "0"
  id: "1"
  messages:
  - '@type': /cosmos.bank.v1beta1.MsgSend
    amount:
    - amount: "10"
      denom: stake
    from_address: cosmos1..
    to_address: cosmos1..
  metadata: AQ==
  status: PROPOSAL_STATUS_DEPOSIT_PERIOD
  submit_time: "2022-03-28T11:50:20.819676256Z"
  total_deposit:
  - amount: "10"
    denom: stake
  voting_end_time: null
  voting_start_time: null
- deposit_end_time: "2022-03-30T14:02:41.165025015Z"
  final_tally_result:
    abstain_count: "0"
    no_count: "0"
    no_with_veto_count: "0"
    yes_count: "0"
  id: "2"
  messages:
  - '@type': /cosmos.bank.v1beta1.MsgSend
    amount:
    - amount: "10"
      denom: stake
    from_address: cosmos1..
    to_address: cosmos1..
  metadata: AQ==
  status: PROPOSAL_STATUS_DEPOSIT_PERIOD
  submit_time: "2022-03-28T14:02:41.165025015Z"
  total_deposit:
  - amount: "10"
    denom: stake
  voting_end_time: null
  voting_start_time: null
```

##### proposer

The `proposer` command allows users to query the proposer for a given proposal.

```bash
simd query gov proposer [proposal-id] [flags]
```

Example:

```bash
simd query gov proposer 1
```

Example Output:

```bash
proposal_id: "1"
proposer: cosmos1..
```

##### tally

The `tally` command allows users to query the tally of a given proposal vote.

```bash
simd query gov tally [proposal-id] [flags]
```

Example:

```bash
simd query gov tally 1
```

Example Output:

```bash
abstain: "0"
"no": "0"
no_with_veto: "0"
"yes": "1"
```

##### vote

The `vote` command allows users to query a vote for a given proposal.

```bash
simd query gov vote [proposal-id] [voter-addr] [flags]
```

Example:

```bash
simd query gov vote 1 cosmos1..
```

Example Output:

```bash
option: VOTE_OPTION_YES
options:
- option: VOTE_OPTION_YES
  weight: "1.000000000000000000"
proposal_id: "1"
voter: cosmos1..
```

##### votes

The `votes` command allows users to query all votes for a given proposal.

```bash
simd query gov votes [proposal-id] [flags]
```

Example:

```bash
simd query gov votes 1
```

Example Output:

```bash
pagination:
  next_key: null
  total: "0"
votes:
- option: VOTE_OPTION_YES
  options:
  - option: VOTE_OPTION_YES
    weight: "1.000000000000000000"
  proposal_id: "1"
  voter: cosmos1..
```

#### Transactions

The `tx` commands allow users to interact with the `gov` module.

```bash
simd tx gov --help
```

##### deposit

The `deposit` command allows users to deposit tokens for a given proposal.

```bash
simd tx gov deposit [proposal-id] [deposit] [flags]
```

Example:

```bash
simd tx gov deposit 1 10000000stake --from cosmos1..
```

##### draft-proposal

The `draft-proposal` command allows users to draft any type of proposal.
The command returns a `draft_proposal.json`, to be used by `submit-proposal` after being completed.
The `draft_metadata.json` is meant to be uploaded to [IPFS](#metadata).

```bash
simd tx gov draft-proposal
```

##### submit-proposal

The `submit-proposal` command allows users to submit a governance proposal along with some messages and metadata.
Messages, metadata and deposit are defined in a JSON file.

```bash
simd tx gov submit-proposal [path-to-proposal-json] [flags]
```

Example:

```bash
simd tx gov submit-proposal /path/to/proposal.json --from cosmos1..
```

where `proposal.json` contains:

```json
{
  "messages": [
    {
      "@type": "/cosmos.bank.v1beta1.MsgSend",
      "from_address": "cosmos1...", // The gov module module address
      "to_address": "cosmos1...",
      "amount":[{"denom": "stake","amount": "10"}]
    }
  ],
  "metadata": "AQ==",
  "deposit": "10stake",
  "title": "Proposal Title",
  "summary": "Proposal Summary"
}
```

:::note
By default the metadata, summary and title are both limited by 255 characters, this can be overridden by the application developer.
:::

:::tip
When metadata is not specified, the title is limited to 255 characters and the summary 40x the title length.
:::

##### cancel-proposal

Once proposal is canceled, from the deposits of proposal `deposits * proposal_cancel_ratio` will be burned or sent to `ProposalCancelDest` address , if `ProposalCancelDest` is empty then deposits will be burned. The `remaining deposits` will be sent to depositors.

```bash
simd tx gov cancel-proposal [proposal-id] [flags]
```

Example:

```bash
simd tx gov cancel-proposal 1 --from cosmos1...
```

##### vote

The `vote` command allows users to submit a vote for a given governance proposal.

```bash
simd tx gov vote [command] [flags]
```

Example:

```bash
simd tx gov vote 1 yes --from cosmos1..
```

##### weighted-vote

The `weighted-vote` command allows users to submit a weighted vote for a given governance proposal.

```bash
simd tx gov weighted-vote [proposal-id] [weighted-options] [flags]
```

Example:

```bash
simd tx gov weighted-vote 1 yes=0.5,no=0.5 --from cosmos1..
```

### gRPC

A user can query the `gov` module using gRPC endpoints.

#### Proposal

The `Proposal` endpoint allows users to query a given proposal.

```bash
cosmos.gov.v1.Query/Proposal
```

Example:

```bash
grpcurl -plaintext \
    -d '{"proposal_id":"1"}' \
    localhost:9090 \
    cosmos.gov.v1.Query/Proposal
```

#### Proposals

The `Proposals` endpoint allows users to query all proposals with optional filters.

```bash
cosmos.gov.v1.Query/Proposals
```

Example:

```bash
grpcurl -plaintext \
    localhost:9090 \
    cosmos.gov.v1.Query/Proposals
```

#### Vote

The `Vote` endpoint allows users to query a vote for a given proposal.

```bash
cosmos.gov.v1.Query/Vote
```

Example:

```bash
grpcurl -plaintext \
    -d '{"proposal_id":"1","voter":"cosmos1.."}' \
    localhost:9090 \
    cosmos.gov.v1.Query/Vote
```

#### Votes

The `Votes` endpoint allows users to query all votes for a given proposal.

```bash
cosmos.gov.v1.Query/Votes
```

Example:

```bash
grpcurl -plaintext \
    -d '{"proposal_id":"1"}' \
    localhost:9090 \
    cosmos.gov.v1.Query/Votes
```

#### Params

The `Params` endpoint allows users to query all parameters for the `gov` module.

```bash
cosmos.gov.v1.Query/Params
```

Example:

```bash
grpcurl -plaintext \
    -d '{"params_type":"voting"}' \
    localhost:9090 \
    cosmos.gov.v1.Query/Params
```

#### Deposit

The `Deposit` endpoint allows users to query a deposit for a given proposal from a given depositor.

```bash
cosmos.gov.v1.Query/Deposit
```

Example:

```bash
grpcurl -plaintext \
    '{"proposal_id":"1","depositor":"cosmos1.."}' \
    localhost:9090 \
    cosmos.gov.v1.Query/Deposit
```

#### deposits

The `Deposits` endpoint allows users to query all deposits for a given proposal.

```bash
cosmos.gov.v1.Query/Deposits
```

Example:

```bash
grpcurl -plaintext \
    -d '{"proposal_id":"1"}' \
    localhost:9090 \
    cosmos.gov.v1.Query/Deposits
```

#### TallyResult

The `TallyResult` endpoint allows users to query the tally of a given proposal.

```bash
cosmos.gov.v1.Query/TallyResult
```

Example:

```bash
grpcurl -plaintext \
    -d '{"proposal_id":"1"}' \
    localhost:9090 \
    cosmos.gov.v1.Query/TallyResult
```

### REST

A user can query the `gov` module using REST endpoints.

#### proposal

The `proposals` endpoint allows users to query a given proposal.

```bash
/cosmos/gov/v1/proposals/{proposal_id}
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/proposals/1
```

#### proposals

The `proposals` endpoint also allows users to query all proposals with optional filters.


```bash
/cosmos/gov/v1/proposals
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/proposals
```

#### voter vote

The `votes` endpoint allows users to query a vote for a given proposal.


```bash
/cosmos/gov/v1/proposals/{proposal_id}/votes/{voter}
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/proposals/1/votes/cosmos1..
```

#### votes

The `votes` endpoint allows users to query all votes for a given proposal.

```bash
/cosmos/gov/v1/proposals/{proposal_id}/votes
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/proposals/1/votes
```

#### params

The `params` endpoint allows users to query all parameters for the `gov` module.

```bash
/cosmos/gov/v1/params/{params_type}
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/params/voting
```

Note: `params_type` are deprecated in v1 since all params are stored in Params.

#### deposits

The `deposits` endpoint allows users to query a deposit for a given proposal from a given depositor.

```bash
/cosmos/gov/v1/proposals/{proposal_id}/deposits/{depositor}
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/proposals/1/deposits/cosmos1..
```

#### proposal deposits

The `deposits` endpoint allows users to query all deposits for a given proposal.

```bash
/cosmos/gov/v1/proposals/{proposal_id}/deposits
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/proposals/1/deposits
```

#### tally

The `tally` endpoint allows users to query the tally of a given proposal.

```bash
/cosmos/gov/v1/proposals/{proposal_id}/tally
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/proposals/1/tally
```
