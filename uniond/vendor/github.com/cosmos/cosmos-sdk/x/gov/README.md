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
* **Inheritance and penalties:** Delegators inherit their validator's vote if
they don't vote themselves.
* **Claiming deposit:** Users that deposited on proposals can recover their
deposits if the proposal was accepted or rejected. If the proposal was vetoed, or never entered voting period (minimum deposit not reached within deposit period), the deposit is burned.

This module is in use on the Cosmos Hub (a.k.a [gaia](https://github.com/cosmos/gaia)).
Features that may be added in the future are described in [Future Improvements](#future-improvements).

## Contents

The following specification uses *ATOM* as the native staking token. The module
can be adapted to any Proof-Of-Stake blockchain by replacing *ATOM* with the native
staking token of the chain.

* [Concepts](#concepts)
    * [Proposal submission](#proposal-submission)
    * [Deposit](#deposit)
    * [Vote](#vote)
    * [Software Upgrade](#software-upgrade)
* [State](#state)
    * [Proposals](#proposals)
    * [Parameters and base types](#parameters-and-base-types)
    * [Deposit](#deposit-1)
    * [ValidatorGovInfo](#validatorgovinfo)
    * [Stores](#stores)
    * [Proposal Processing Queue](#proposal-processing-queue)
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
* [Future Improvements](#future-improvements)

## Concepts

*Disclaimer: This is work in progress. Mechanisms are susceptible to change.*

The governance process is divided in a few steps that are outlined below:

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
module uses the `MsgServiceRouter` to check that these messages are correctly constructed
and have a respective path to execute on but do not perform a full validity check.

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
according to the final tally of the proposal:

* If the proposal is approved or rejected but *not* vetoed, each deposit will be
  automatically refunded to its respective depositor (transferred from the governance
  `ModuleAccount`).
* When the proposal is vetoed with greater than 1/3, deposits will be burned from the
  governance `ModuleAccount` and the proposal information along with its deposit
  information will be removed from state.
* All refunded or burned deposits are removed from the state. Events are issued when
  burning or refunding a deposit.

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
the moment the vote closes. The initial value of `Voting period` is 2 weeks.

#### Option set

The option set of a proposal refers to the set of choices a participant can
choose from when casting its vote.

The initial option set includes the following options:

* `Yes`
* `No`
* `NoWithVeto`
* `Abstain`

`NoWithVeto` counts as `No` but also adds a `Veto` vote. `Abstain` option
allows voters to signal that they do not intend to vote in favor or against the
proposal but accept the result of the vote.

*Note: from the UI, for urgent proposals we should maybe add a ‘Not Urgent’ option that casts a `NoWithVeto` vote.*

#### Weighted Votes

[ADR-037](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-037-gov-split-vote.md) introduces the weighted vote feature which allows a staker to split their votes into several voting options. For example, it could use 70% of its voting power to vote Yes and 30% of its voting power to vote No.

Often times the entity owning that address might not be a single individual. For example, a company might have different stakeholders who want to vote differently, and so it makes sense to allow them to split their voting power. Currently, it is not possible for them to do "passthrough voting" and giving their users voting rights over their tokens. However, with this system, exchanges can poll their users for voting preferences, and then vote on-chain proportionally to the results of the poll.

To represent weighted vote on chain, we use the following Protobuf message.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/gov/v1beta1/gov.proto#L34-L47
```

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/gov/v1beta1/gov.proto#L181-L201
```

For a weighted vote to be valid, the `options` field must not contain duplicate vote options, and the sum of weights of all options must be equal to 1.

### Quorum

Quorum is defined as the minimum percentage of voting power that needs to be
cast on a proposal for the result to be valid.

### Expedited Proposals

A proposal can be expedited, making the proposal use shorter voting duration and a higher tally threshold by its default. If an expedited proposal fails to meet the threshold within the scope of shorter voting duration, the expedited proposal is then converted to a regular proposal and restarts voting under regular voting conditions.

#### Threshold

Threshold is defined as the minimum proportion of `Yes` votes (excluding
`Abstain` votes) for the proposal to be accepted.

Initially, the threshold is set at 50% of `Yes` votes, excluding `Abstain`
votes. A possibility to veto exists if more than 1/3rd of all votes are
`NoWithVeto` votes.  Note, both of these values are derived from the `TallyParams`
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

#### Inheritance

If a delegator does not vote, it will inherit its validator vote.

* If the delegator votes before its validator, it will not inherit from the
  validator's vote.
* If the delegator votes after its validator, it will override its validator
  vote with its own. If the proposal is urgent, it is possible
  that the vote will close before delegators have a chance to react and
  override their validator's vote. This is not a problem, as proposals require more than 2/3rd of the total voting power to pass, when tallied at the end of the voting period. Because as little as 1/3 + 1 validation power could collude to censor transactions, non-collusion is already assumed for ranges exceeding this threshold.

#### Validator’s punishment for non-voting

At present, validators are not punished for failing to vote.

#### Governance address

Later, we may add permissioned keys that could only sign txs from certain modules. For the MVP, the `Governance address` will be the main validator address generated at account creation. This address corresponds to a different PrivKey than the CometBFT PrivKey which is responsible for signing consensus messages. Validators thus do not have to sign governance transactions with the sensitive CometBFT PrivKey.

#### Burnable Params

There are three parameters that define if the deposit of a proposal should be burned or returned to the depositors. 

* `BurnVoteVeto` burns the proposal deposit if the proposal gets vetoed. 
* `BurnVoteQuorum` burns the proposal deposit if the proposal deposit if the vote does not reach quorum.
* `BurnProposalDepositPrevote` burns the proposal deposit if it does not enter the voting phase. 

> Note: These parameters are modifiable via governance. 

## State

### Constitution

`Constitution` is found in the genesis state.  It is a string field intended to be used to descibe the purpose of a particular blockchain, and its expected norms.  A few examples of how the constitution field can be used:

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
    * best example of this is the Cosmos hub, where different founding groups, have different interpertations of the purpose of the network.

This genesis entry, "constitution" hasn't been designed for existing chains, who should likely just ratify a constitution using their governance system.  Instead, this is for new chains.  It will allow for validators to have a much clearer idea of purpose and the expecations placed on them while operating thier nodes.  Likewise, for community members, the constitution will give them some idea of what to expect from both the "chain team" and the validators, respectively.

This constitution is designed to be immutable, and placed only in genesis, though that could change over time by a pull request to the cosmos-sdk that allows for the constitution to be changed by governance.  Communities whishing to make amendments to their original constitution should use the governance mechanism and a "signaling proposal" to do exactly that.

**Ideal use scenario for a cosmos chain constitution**

As a chain developer, you decide that you'd like to provide clarity to your key user groups:

* validators
* token holders
* developers (yourself)

You use the constitution to immutably store some Markdown in genesis, so that when difficult questions come up, the constutituon can provide guidance to the community.

### Proposals

`Proposal` objects are used to tally votes and generally track the proposal's state.
They contain an array of arbitrary `sdk.Msg`'s which the governance module will attempt
to resolve and then execute if the proposal passes. `Proposal`'s are identified by a
unique id and contains a series of timestamps: `submit_time`, `deposit_end_time`,
`voting_start_time`, `voting_end_time` which track the lifecycle of a proposal

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/gov/v1/gov.proto#L51-L99
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

The metadata has a maximum length that is chosen by the app developer, and
passed into the gov keeper as a config. The default maximum length in the SDK is 255 characters.

#### Writing a module that uses governance

There are many aspects of a chain, or of the individual modules that you may want to
use governance to perform such as changing various parameters. This is very simple
to do. First, write out your message types and `MsgServer` implementation. Add an
`authority` field to the keeper which will be populated in the constructor with the
governance module account: `govKeeper.GetGovernanceAccount().GetAddress()`. Then for
the methods in the `msg_server.go`, perform a check on the message that the signer
matches `authority`. This will prevent any user from executing that message.

### Parameters and base types

`Parameters` define the rules according to which votes are run. There can only
be one active parameter set at any given time. If governance wants to change a
parameter set, either to modify a value or add/remove a parameter field, a new
parameter set has to be created and the previous one rendered inactive.

#### DepositParams

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/gov/v1/gov.proto#L152-L162
```

#### VotingParams

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/gov/v1/gov.proto#L164-L168
```

#### TallyParams

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/gov/v1/gov.proto#L170-L182
```

Parameters are stored in a global `GlobalParams` KVStore.

Additionally, we introduce some basic types:

```go
type Vote byte

const (
    VoteYes         = 0x1
    VoteNo          = 0x2
    VoteNoWithVeto  = 0x3
    VoteAbstain     = 0x4
)

type ProposalType  string

const (
    ProposalTypePlainText       = "Text"
    ProposalTypeSoftwareUpgrade = "SoftwareUpgrade"
)

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
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/gov/v1/gov.proto#L38-L49
```

### ValidatorGovInfo

This type is used in a temp map when tallying

```go
  type ValidatorGovInfo struct {
    Minus     sdk.Dec
    Vote      Vote
  }
```

## Stores

:::note
Stores are KVStores in the multi-store. The key to find the store is the first parameter in the list
:::

We will use one KVStore `Governance` to store four mappings:

* A mapping from `proposalID|'proposal'` to `Proposal`.
* A mapping from `proposalID|'addresses'|address` to `Vote`. This mapping allows
  us to query all addresses that voted on the proposal along with their vote by
  doing a range query on `proposalID:addresses`.
* A mapping from `ParamsKey|'Params'` to `Params`. This map allows to query all
  x/gov params.
* A mapping from `VotingPeriodProposalKeyPrefix|proposalID` to a single byte. This allows
  us to know if a proposal is in the voting period or not with very low gas cost.
  
For pseudocode purposes, here are the two function we will use to read or write in stores:

* `load(StoreKey, Key)`: Retrieve item stored at key `Key` in store found at key `StoreKey` in the multistore
* `store(StoreKey, Key, value)`: Write value `Value` at key `Key` in store found at key `StoreKey` in the multistore

### Proposal Processing Queue

**Store:**

* `ProposalProcessingQueue`: A queue `queue[proposalID]` containing all the
  `ProposalIDs` of proposals that reached `MinDeposit`. During each `EndBlock`,
  all the proposals that have reached the end of their voting period are processed.
  To process a finished proposal, the application tallies the votes, computes the
  votes of each validator and checks if every validator in the validator set has
  voted. If the proposal is accepted, deposits are refunded. Finally, the proposal
  content `Handler` is executed.

And the pseudocode for the `ProposalProcessingQueue`:

```go
  in EndBlock do

    for finishedProposalID in GetAllFinishedProposalIDs(block.Time)
      proposal = load(Governance, <proposalID|'proposal'>) // proposal is a const key

      validators = Keeper.getAllValidators()
      tmpValMap := map(sdk.AccAddress)ValidatorGovInfo

      // Initiate mapping at 0. This is the amount of shares of the validator's vote that will be overridden by their delegator's votes
      for each validator in validators
        tmpValMap(validator.OperatorAddr).Minus = 0

      // Tally
      voterIterator = rangeQuery(Governance, <proposalID|'addresses'>) //return all the addresses that voted on the proposal
      for each (voterAddress, vote) in voterIterator
        delegations = stakingKeeper.getDelegations(voterAddress) // get all delegations for current voter

        for each delegation in delegations
          // make sure delegation.Shares does NOT include shares being unbonded
          tmpValMap(delegation.ValidatorAddr).Minus += delegation.Shares
          proposal.updateTally(vote, delegation.Shares)

        _, isVal = stakingKeeper.getValidator(voterAddress)
        if (isVal)
          tmpValMap(voterAddress).Vote = vote

      tallyingParam = load(GlobalParams, 'TallyingParam')

      // Update tally if validator voted
      for each validator in validators
        if tmpValMap(validator).HasVoted
          proposal.updateTally(tmpValMap(validator).Vote, (validator.TotalShares - tmpValMap(validator).Minus))



      // Check if proposal is accepted or rejected
      totalNonAbstain := proposal.YesVotes + proposal.NoVotes + proposal.NoWithVetoVotes
      if (proposal.Votes.YesVotes/totalNonAbstain > tallyingParam.Threshold AND proposal.Votes.NoWithVetoVotes/totalNonAbstain  < tallyingParam.Veto)
        //  proposal was accepted at the end of the voting period
        //  refund deposits (non-voters already punished)
        for each (amount, depositor) in proposal.Deposits
          depositor.AtomBalance += amount

        stateWriter, err := proposal.Handler()
        if err != nil
            // proposal passed but failed during state execution
            proposal.CurrentStatus = ProposalStatusFailed
         else
            // proposal pass and state is persisted
            proposal.CurrentStatus = ProposalStatusAccepted
            stateWriter.save()
      else
        // proposal was rejected
        proposal.CurrentStatus = ProposalStatusRejected

      store(Governance, <proposalID|'proposal'>, proposal)
```

### Legacy Proposal

:::warning
Legacy proposals are deprecated. Use the new proposal flow by granting the governance module the right to execute the message.
:::

A legacy proposal is the old implementation of governance proposal.
Contrary to proposal that can contain any messages, a legacy proposal allows to submit a set of pre-defined proposals.
These proposals are defined by their types and handled by handlers that are registered in the gov v1beta1 router.

More information on how to submit proposals in the [client section](#client).

## Messages

### Proposal Submission

Proposals can be submitted by any account via a `MsgSubmitProposal` transaction.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/gov/v1/tx.proto#L42-L69
```

All `sdk.Msgs` passed into the `messages` field of a `MsgSubmitProposal` message
must be registered in the app's `MsgServiceRouter`. Each of these messages must
have one signer, namely the gov module account. And finally, the metadata length
must not be larger than the `maxMetadataLen` config passed into the gov keeper.
The `initialDeposit` must be strictly positive and conform to the accepted denom of the `MinDeposit` param.

**State modifications:**

* Generate new `proposalID`
* Create new `Proposal`
* Initialise `Proposal`'s attributes
* Decrease balance of sender by `InitialDeposit`
* If `MinDeposit` is reached:
    * Push `proposalID` in `ProposalProcessingQueue`
* Transfer `InitialDeposit` from the `Proposer` to the governance `ModuleAccount`

### Deposit

Once a proposal is submitted, if `Proposal.TotalDeposit < ActiveParam.MinDeposit`, Atom holders can send
`MsgDeposit` transactions to increase the proposal's deposit.

A deposit is accepted iff:

* The proposal exists
* The proposal is not in the voting period
* The deposited coins are conform to the accepted denom from the `MinDeposit` param

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/gov/v1/tx.proto#L134-L147
```

**State modifications:**

* Decrease balance of sender by `deposit`
* Add `deposit` of sender in `proposal.Deposits`
* Increase `proposal.TotalDeposit` by sender's `deposit`
* If `MinDeposit` is reached:
    * Push `proposalID` in `ProposalProcessingQueueEnd`
* Transfer `Deposit` from the `proposer` to the governance `ModuleAccount`

### Vote

Once `ActiveParam.MinDeposit` is reached, voting period starts. From there,
bonded Atom holders are able to send `MsgVote` transactions to cast their
vote on the proposal.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/gov/v1/tx.proto#L92-L108
```

**State modifications:**

* Record `Vote` of sender

:::note
Gas cost for this message has to take into account the future tallying of the vote in EndBlocker.
:::

## Events

The governance module emits the following events:

### EndBlocker

| Type              | Attribute Key   | Attribute Value  |
|-------------------|-----------------|------------------|
| inactive_proposal | proposal_id     | {proposalID}     |
| inactive_proposal | proposal_result | {proposalResult} |
| active_proposal   | proposal_id     | {proposalID}     |
| active_proposal   | proposal_result | {proposalResult} |

### Handlers

#### MsgSubmitProposal

| Type                | Attribute Key       | Attribute Value |
|---------------------|---------------------|-----------------|
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
|---------------|---------------|-----------------|
| proposal_vote | option        | {voteOption}    |
| proposal_vote | proposal_id   | {proposalID}    |
| message       | module        | governance      |
| message       | action        | vote            |
| message       | sender        | {senderAddress} |

#### MsgVoteWeighted

| Type          | Attribute Key | Attribute Value       |
|---------------|---------------|-----------------------|
| proposal_vote | option        | {weightedVoteOptions} |
| proposal_vote | proposal_id   | {proposalID}          |
| message       | module        | governance            |
| message       | action        | vote                  |
| message       | sender        | {senderAddress}       |

#### MsgDeposit

| Type                 | Attribute Key       | Attribute Value |
|----------------------|---------------------|-----------------|
| proposal_deposit     | amount              | {depositAmount} |
| proposal_deposit     | proposal_id         | {proposalID}    |
| proposal_deposit [0] | voting_period_start | {proposalID}    |
| message              | module              | governance      |
| message              | action              | deposit         |
| message              | sender              | {senderAddress} |

* [0] Event only emitted if the voting period starts during the submission.

## Parameters

The governance module contains the following parameters:

| Key                           | Type             | Example                                 |
|-------------------------------|------------------|-----------------------------------------|
| min_deposit                   | array (coins)    | [{"denom":"uatom","amount":"10000000"}] |
| max_deposit_period            | string (time ns) | "172800000000000" (17280s)              |
| voting_period                 | string (time ns) | "172800000000000" (17280s)              |
| quorum                        | string (dec)     | "0.334000000000000000"                  |
| threshold                     | string (dec)     | "0.500000000000000000"                  |
| veto                          | string (dec)     | "0.334000000000000000"                  |
| expedited_threshold           | string (time ns) | "0.667000000000000000"                  |
| expedited_voting_period       | string (time ns) | "86400000000000" (8600s)                |
| expedited_min_deposit         | array (coins)    | [{"denom":"uatom","amount":"50000000"}] |
| burn_proposal_deposit_prevote | bool             | false                                    |
| burn_vote_quorum              | bool             | false                                   |
| burn_vote_veto                | bool             | true                                    |
| min_initial_deposit_ratio                | string             | "0.1"                                    |


**NOTE**: The governance module contains parameters that are objects unlike other
modules. If only a subset of parameters are desired to be changed, only they need
to be included and not the entire parameter object structure.

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
simd query gov deposit [proposal-id] [depositer-addr] [flags]
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

##### param

The `param` command allows users to query a given parameter for the `gov` module.

```bash
simd query gov param [param-type] [flags]
```

Example:

```bash
simd query gov param voting
```

Example Output:

```bash
voting_period: "172800000000000"
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
deposit_params:
  max_deposit_period: 172800s
  min_deposit:
  - amount: "10000000"
    denom: stake
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
tally_params:
  quorum: "0.334000000000000000"
  threshold: "0.500000000000000000"
  veto_threshold: "0.334000000000000000"
voting_params:
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

##### submit-legacy-proposal

The `submit-legacy-proposal` command allows users to submit a governance legacy proposal along with an initial deposit.

```bash
simd tx gov submit-legacy-proposal [command] [flags]
```

Example:

```bash
simd tx gov submit-legacy-proposal --title="Test Proposal" --description="testing" --type="Text" --deposit="100000000stake" --from cosmos1..
```

Example (`param-change`):

```bash
simd tx gov submit-legacy-proposal param-change proposal.json --from cosmos1..
```

```json
{
  "title": "Test Proposal",
  "description": "testing, testing, 1, 2, 3",
  "changes": [
    {
      "subspace": "staking",
      "key": "MaxValidators",
      "value": 100
    }
  ],
  "deposit": "10000000stake"
}
```

#### cancel-proposal

Once proposal is canceled, from the deposits of proposal `deposits * proposal_cancel_ratio` will be burned or sent to `ProposalCancelDest` address , if `ProposalCancelDest` is empty then deposits will be burned. The `remaining deposits` will be sent to depositers.

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

Using legacy v1beta1:

```bash
cosmos.gov.v1beta1.Query/Proposal
```

Example:

```bash
grpcurl -plaintext \
    -d '{"proposal_id":"1"}' \
    localhost:9090 \
    cosmos.gov.v1beta1.Query/Proposal
```

Example Output:

```bash
{
  "proposal": {
    "proposalId": "1",
    "content": {"@type":"/cosmos.gov.v1beta1.TextProposal","description":"testing, testing, 1, 2, 3","title":"Test Proposal"},
    "status": "PROPOSAL_STATUS_VOTING_PERIOD",
    "finalTallyResult": {
      "yes": "0",
      "abstain": "0",
      "no": "0",
      "noWithVeto": "0"
    },
    "submitTime": "2021-09-16T19:40:08.712440474Z",
    "depositEndTime": "2021-09-18T19:40:08.712440474Z",
    "totalDeposit": [
      {
        "denom": "stake",
        "amount": "10000000"
      }
    ],
    "votingStartTime": "2021-09-16T19:40:08.712440474Z",
    "votingEndTime": "2021-09-18T19:40:08.712440474Z",
    "title": "Test Proposal",
    "summary": "testing, testing, 1, 2, 3"
  }
}
```

Using v1:

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

Example Output:

```bash
{
  "proposal": {
    "id": "1",
    "messages": [
      {"@type":"/cosmos.bank.v1beta1.MsgSend","amount":[{"denom":"stake","amount":"10"}],"fromAddress":"cosmos1..","toAddress":"cosmos1.."}
    ],
    "status": "PROPOSAL_STATUS_VOTING_PERIOD",
    "finalTallyResult": {
      "yesCount": "0",
      "abstainCount": "0",
      "noCount": "0",
      "noWithVetoCount": "0"
    },
    "submitTime": "2022-03-28T11:50:20.819676256Z",
    "depositEndTime": "2022-03-30T11:50:20.819676256Z",
    "totalDeposit": [
      {
        "denom": "stake",
        "amount": "10000000"
      }
    ],
    "votingStartTime": "2022-03-28T14:25:26.644857113Z",
    "votingEndTime": "2022-03-30T14:25:26.644857113Z",
    "metadata": "AQ==",
    "title": "Test Proposal",
    "summary": "testing, testing, 1, 2, 3"
  }
}
```

#### Proposals

The `Proposals` endpoint allows users to query all proposals with optional filters.

Using legacy v1beta1:

```bash
cosmos.gov.v1beta1.Query/Proposals
```

Example:

```bash
grpcurl -plaintext \
    localhost:9090 \
    cosmos.gov.v1beta1.Query/Proposals
```

Example Output:

```bash
{
  "proposals": [
    {
      "proposalId": "1",
      "status": "PROPOSAL_STATUS_VOTING_PERIOD",
      "finalTallyResult": {
        "yes": "0",
        "abstain": "0",
        "no": "0",
        "noWithVeto": "0"
      },
      "submitTime": "2022-03-28T11:50:20.819676256Z",
      "depositEndTime": "2022-03-30T11:50:20.819676256Z",
      "totalDeposit": [
        {
          "denom": "stake",
          "amount": "10000000010"
        }
      ],
      "votingStartTime": "2022-03-28T14:25:26.644857113Z",
      "votingEndTime": "2022-03-30T14:25:26.644857113Z"
    },
    {
      "proposalId": "2",
      "status": "PROPOSAL_STATUS_DEPOSIT_PERIOD",
      "finalTallyResult": {
        "yes": "0",
        "abstain": "0",
        "no": "0",
        "noWithVeto": "0"
      },
      "submitTime": "2022-03-28T14:02:41.165025015Z",
      "depositEndTime": "2022-03-30T14:02:41.165025015Z",
      "totalDeposit": [
        {
          "denom": "stake",
          "amount": "10"
        }
      ],
      "votingStartTime": "0001-01-01T00:00:00Z",
      "votingEndTime": "0001-01-01T00:00:00Z"
    }
  ],
  "pagination": {
    "total": "2"
  }
}

```

Using v1:

```bash
cosmos.gov.v1.Query/Proposals
```

Example:

```bash
grpcurl -plaintext \
    localhost:9090 \
    cosmos.gov.v1.Query/Proposals
```

Example Output:

```bash
{
  "proposals": [
    {
      "id": "1",
      "messages": [
        {"@type":"/cosmos.bank.v1beta1.MsgSend","amount":[{"denom":"stake","amount":"10"}],"fromAddress":"cosmos1..","toAddress":"cosmos1.."}
      ],
      "status": "PROPOSAL_STATUS_VOTING_PERIOD",
      "finalTallyResult": {
        "yesCount": "0",
        "abstainCount": "0",
        "noCount": "0",
        "noWithVetoCount": "0"
      },
      "submitTime": "2022-03-28T11:50:20.819676256Z",
      "depositEndTime": "2022-03-30T11:50:20.819676256Z",
      "totalDeposit": [
        {
          "denom": "stake",
          "amount": "10000000010"
        }
      ],
      "votingStartTime": "2022-03-28T14:25:26.644857113Z",
      "votingEndTime": "2022-03-30T14:25:26.644857113Z",
      "metadata": "AQ==",
      "title": "Proposal Title",
      "summary": "Proposal Summary"
    },
    {
      "id": "2",
      "messages": [
        {"@type":"/cosmos.bank.v1beta1.MsgSend","amount":[{"denom":"stake","amount":"10"}],"fromAddress":"cosmos1..","toAddress":"cosmos1.."}
      ],
      "status": "PROPOSAL_STATUS_DEPOSIT_PERIOD",
      "finalTallyResult": {
        "yesCount": "0",
        "abstainCount": "0",
        "noCount": "0",
        "noWithVetoCount": "0"
      },
      "submitTime": "2022-03-28T14:02:41.165025015Z",
      "depositEndTime": "2022-03-30T14:02:41.165025015Z",
      "totalDeposit": [
        {
          "denom": "stake",
          "amount": "10"
        }
      ],
      "metadata": "AQ==",
      "title": "Proposal Title",
      "summary": "Proposal Summary"
    }
  ],
  "pagination": {
    "total": "2"
  }
}
```

#### Vote

The `Vote` endpoint allows users to query a vote for a given proposal.

Using legacy v1beta1:

```bash
cosmos.gov.v1beta1.Query/Vote
```

Example:

```bash
grpcurl -plaintext \
    -d '{"proposal_id":"1","voter":"cosmos1.."}' \
    localhost:9090 \
    cosmos.gov.v1beta1.Query/Vote
```

Example Output:

```bash
{
  "vote": {
    "proposalId": "1",
    "voter": "cosmos1..",
    "option": "VOTE_OPTION_YES",
    "options": [
      {
        "option": "VOTE_OPTION_YES",
        "weight": "1000000000000000000"
      }
    ]
  }
}
```

Using v1:

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

Example Output:

```bash
{
  "vote": {
    "proposalId": "1",
    "voter": "cosmos1..",
    "option": "VOTE_OPTION_YES",
    "options": [
      {
        "option": "VOTE_OPTION_YES",
        "weight": "1.000000000000000000"
      }
    ]
  }
}
```

#### Votes

The `Votes` endpoint allows users to query all votes for a given proposal.

Using legacy v1beta1:

```bash
cosmos.gov.v1beta1.Query/Votes
```

Example:

```bash
grpcurl -plaintext \
    -d '{"proposal_id":"1"}' \
    localhost:9090 \
    cosmos.gov.v1beta1.Query/Votes
```

Example Output:

```bash
{
  "votes": [
    {
      "proposalId": "1",
      "voter": "cosmos1..",
      "options": [
        {
          "option": "VOTE_OPTION_YES",
          "weight": "1000000000000000000"
        }
      ]
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```

Using v1:

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

Example Output:

```bash
{
  "votes": [
    {
      "proposalId": "1",
      "voter": "cosmos1..",
      "options": [
        {
          "option": "VOTE_OPTION_YES",
          "weight": "1.000000000000000000"
        }
      ]
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```

#### Params

The `Params` endpoint allows users to query all parameters for the `gov` module.

<!-- TODO: #10197 Querying governance params outputs nil values -->

Using legacy v1beta1:

```bash
cosmos.gov.v1beta1.Query/Params
```

Example:

```bash
grpcurl -plaintext \
    -d '{"params_type":"voting"}' \
    localhost:9090 \
    cosmos.gov.v1beta1.Query/Params
```

Example Output:

```bash
{
  "votingParams": {
    "votingPeriod": "172800s"
  },
  "depositParams": {
    "maxDepositPeriod": "0s"
  },
  "tallyParams": {
    "quorum": "MA==",
    "threshold": "MA==",
    "vetoThreshold": "MA=="
  }
}
```

Using v1:

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

Example Output:

```bash
{
  "votingParams": {
    "votingPeriod": "172800s"
  }
}
```

#### Deposit

The `Deposit` endpoint allows users to query a deposit for a given proposal from a given depositor.

Using legacy v1beta1:

```bash
cosmos.gov.v1beta1.Query/Deposit
```

Example:

```bash
grpcurl -plaintext \
    '{"proposal_id":"1","depositor":"cosmos1.."}' \
    localhost:9090 \
    cosmos.gov.v1beta1.Query/Deposit
```

Example Output:

```bash
{
  "deposit": {
    "proposalId": "1",
    "depositor": "cosmos1..",
    "amount": [
      {
        "denom": "stake",
        "amount": "10000000"
      }
    ]
  }
}
```

Using v1:

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

Example Output:

```bash
{
  "deposit": {
    "proposalId": "1",
    "depositor": "cosmos1..",
    "amount": [
      {
        "denom": "stake",
        "amount": "10000000"
      }
    ]
  }
}
```

#### deposits

The `Deposits` endpoint allows users to query all deposits for a given proposal.

Using legacy v1beta1:

```bash
cosmos.gov.v1beta1.Query/Deposits
```

Example:

```bash
grpcurl -plaintext \
    -d '{"proposal_id":"1"}' \
    localhost:9090 \
    cosmos.gov.v1beta1.Query/Deposits
```

Example Output:

```bash
{
  "deposits": [
    {
      "proposalId": "1",
      "depositor": "cosmos1..",
      "amount": [
        {
          "denom": "stake",
          "amount": "10000000"
        }
      ]
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```

Using v1:

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

Example Output:

```bash
{
  "deposits": [
    {
      "proposalId": "1",
      "depositor": "cosmos1..",
      "amount": [
        {
          "denom": "stake",
          "amount": "10000000"
        }
      ]
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```

#### TallyResult

The `TallyResult` endpoint allows users to query the tally of a given proposal.

Using legacy v1beta1:

```bash
cosmos.gov.v1beta1.Query/TallyResult
```

Example:

```bash
grpcurl -plaintext \
    -d '{"proposal_id":"1"}' \
    localhost:9090 \
    cosmos.gov.v1beta1.Query/TallyResult
```

Example Output:

```bash
{
  "tally": {
    "yes": "1000000",
    "abstain": "0",
    "no": "0",
    "noWithVeto": "0"
  }
}
```

Using v1:

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

Example Output:

```bash
{
  "tally": {
    "yes": "1000000",
    "abstain": "0",
    "no": "0",
    "noWithVeto": "0"
  }
}
```

### REST

A user can query the `gov` module using REST endpoints.

#### proposal

The `proposals` endpoint allows users to query a given proposal.

Using legacy v1beta1:

```bash
/cosmos/gov/v1beta1/proposals/{proposal_id}
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1beta1/proposals/1
```

Example Output:

```bash
{
  "proposal": {
    "proposal_id": "1",
    "content": null,
    "status": "PROPOSAL_STATUS_VOTING_PERIOD",
    "final_tally_result": {
      "yes": "0",
      "abstain": "0",
      "no": "0",
      "no_with_veto": "0"
    },
    "submit_time": "2022-03-28T11:50:20.819676256Z",
    "deposit_end_time": "2022-03-30T11:50:20.819676256Z",
    "total_deposit": [
      {
        "denom": "stake",
        "amount": "10000000010"
      }
    ],
    "voting_start_time": "2022-03-28T14:25:26.644857113Z",
    "voting_end_time": "2022-03-30T14:25:26.644857113Z"
  }
}
```

Using v1:

```bash
/cosmos/gov/v1/proposals/{proposal_id}
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/proposals/1
```

Example Output:

```bash
{
  "proposal": {
    "id": "1",
    "messages": [
      {
        "@type": "/cosmos.bank.v1beta1.MsgSend",
        "from_address": "cosmos1..",
        "to_address": "cosmos1..",
        "amount": [
          {
            "denom": "stake",
            "amount": "10"
          }
        ]
      }
    ],
    "status": "PROPOSAL_STATUS_VOTING_PERIOD",
    "final_tally_result": {
      "yes_count": "0",
      "abstain_count": "0",
      "no_count": "0",
      "no_with_veto_count": "0"
    },
    "submit_time": "2022-03-28T11:50:20.819676256Z",
    "deposit_end_time": "2022-03-30T11:50:20.819676256Z",
    "total_deposit": [
      {
        "denom": "stake",
        "amount": "10000000"
      }
    ],
    "voting_start_time": "2022-03-28T14:25:26.644857113Z",
    "voting_end_time": "2022-03-30T14:25:26.644857113Z",
    "metadata": "AQ==",
    "title": "Proposal Title",
    "summary": "Proposal Summary"
  }
}
```

#### proposals

The `proposals` endpoint also allows users to query all proposals with optional filters.

Using legacy v1beta1:

```bash
/cosmos/gov/v1beta1/proposals
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1beta1/proposals
```

Example Output:

```bash
{
  "proposals": [
    {
      "proposal_id": "1",
      "content": null,
      "status": "PROPOSAL_STATUS_VOTING_PERIOD",
      "final_tally_result": {
        "yes": "0",
        "abstain": "0",
        "no": "0",
        "no_with_veto": "0"
      },
      "submit_time": "2022-03-28T11:50:20.819676256Z",
      "deposit_end_time": "2022-03-30T11:50:20.819676256Z",
      "total_deposit": [
        {
          "denom": "stake",
          "amount": "10000000"
        }
      ],
      "voting_start_time": "2022-03-28T14:25:26.644857113Z",
      "voting_end_time": "2022-03-30T14:25:26.644857113Z"
    },
    {
      "proposal_id": "2",
      "content": null,
      "status": "PROPOSAL_STATUS_DEPOSIT_PERIOD",
      "final_tally_result": {
        "yes": "0",
        "abstain": "0",
        "no": "0",
        "no_with_veto": "0"
      },
      "submit_time": "2022-03-28T14:02:41.165025015Z",
      "deposit_end_time": "2022-03-30T14:02:41.165025015Z",
      "total_deposit": [
        {
          "denom": "stake",
          "amount": "10"
        }
      ],
      "voting_start_time": "0001-01-01T00:00:00Z",
      "voting_end_time": "0001-01-01T00:00:00Z"
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "2"
  }
}
```

Using v1:

```bash
/cosmos/gov/v1/proposals
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/proposals
```

Example Output:

```bash
{
  "proposals": [
    {
      "id": "1",
      "messages": [
        {
          "@type": "/cosmos.bank.v1beta1.MsgSend",
          "from_address": "cosmos1..",
          "to_address": "cosmos1..",
          "amount": [
            {
              "denom": "stake",
              "amount": "10"
            }
          ]
        }
      ],
      "status": "PROPOSAL_STATUS_VOTING_PERIOD",
      "final_tally_result": {
        "yes_count": "0",
        "abstain_count": "0",
        "no_count": "0",
        "no_with_veto_count": "0"
      },
      "submit_time": "2022-03-28T11:50:20.819676256Z",
      "deposit_end_time": "2022-03-30T11:50:20.819676256Z",
      "total_deposit": [
        {
          "denom": "stake",
          "amount": "10000000010"
        }
      ],
      "voting_start_time": "2022-03-28T14:25:26.644857113Z",
      "voting_end_time": "2022-03-30T14:25:26.644857113Z",
      "metadata": "AQ==",
      "title": "Proposal Title",
      "summary": "Proposal Summary"
    },
    {
      "id": "2",
      "messages": [
        {
          "@type": "/cosmos.bank.v1beta1.MsgSend",
          "from_address": "cosmos1..",
          "to_address": "cosmos1..",
          "amount": [
            {
              "denom": "stake",
              "amount": "10"
            }
          ]
        }
      ],
      "status": "PROPOSAL_STATUS_DEPOSIT_PERIOD",
      "final_tally_result": {
        "yes_count": "0",
        "abstain_count": "0",
        "no_count": "0",
        "no_with_veto_count": "0"
      },
      "submit_time": "2022-03-28T14:02:41.165025015Z",
      "deposit_end_time": "2022-03-30T14:02:41.165025015Z",
      "total_deposit": [
        {
          "denom": "stake",
          "amount": "10"
        }
      ],
      "voting_start_time": null,
      "voting_end_time": null,
      "metadata": "AQ==",
      "title": "Proposal Title",
      "summary": "Proposal Summary"
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "2"
  }
}
```

#### voter vote

The `votes` endpoint allows users to query a vote for a given proposal.

Using legacy v1beta1:

```bash
/cosmos/gov/v1beta1/proposals/{proposal_id}/votes/{voter}
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1beta1/proposals/1/votes/cosmos1..
```

Example Output:

```bash
{
  "vote": {
    "proposal_id": "1",
    "voter": "cosmos1..",
    "option": "VOTE_OPTION_YES",
    "options": [
      {
        "option": "VOTE_OPTION_YES",
        "weight": "1.000000000000000000"
      }
    ]
  }
}
```

Using v1:

```bash
/cosmos/gov/v1/proposals/{proposal_id}/votes/{voter}
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/proposals/1/votes/cosmos1..
```

Example Output:

```bash
{
  "vote": {
    "proposal_id": "1",
    "voter": "cosmos1..",
    "options": [
      {
        "option": "VOTE_OPTION_YES",
        "weight": "1.000000000000000000"
      }
    ],
    "metadata": ""
  }
}
```

#### votes

The `votes` endpoint allows users to query all votes for a given proposal.

Using legacy v1beta1:

```bash
/cosmos/gov/v1beta1/proposals/{proposal_id}/votes
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1beta1/proposals/1/votes
```

Example Output:

```bash
{
  "votes": [
    {
      "proposal_id": "1",
      "voter": "cosmos1..",
      "option": "VOTE_OPTION_YES",
      "options": [
        {
          "option": "VOTE_OPTION_YES",
          "weight": "1.000000000000000000"
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

Using v1:

```bash
/cosmos/gov/v1/proposals/{proposal_id}/votes
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/proposals/1/votes
```

Example Output:

```bash
{
  "votes": [
    {
      "proposal_id": "1",
      "voter": "cosmos1..",
      "options": [
        {
          "option": "VOTE_OPTION_YES",
          "weight": "1.000000000000000000"
        }
      ],
      "metadata": ""
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "1"
  }
}
```

#### params

The `params` endpoint allows users to query all parameters for the `gov` module.

<!-- TODO: #10197 Querying governance params outputs nil values -->

Using legacy v1beta1:

```bash
/cosmos/gov/v1beta1/params/{params_type}
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1beta1/params/voting
```

Example Output:

```bash
{
  "voting_params": {
    "voting_period": "172800s"
  },
  "deposit_params": {
    "min_deposit": [
    ],
    "max_deposit_period": "0s"
  },
  "tally_params": {
    "quorum": "0.000000000000000000",
    "threshold": "0.000000000000000000",
    "veto_threshold": "0.000000000000000000"
  }
}
```

Using v1:

```bash
/cosmos/gov/v1/params/{params_type}
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/params/voting
```

Example Output:

```bash
{
  "voting_params": {
    "voting_period": "172800s"
  },
  "deposit_params": {
    "min_deposit": [
    ],
    "max_deposit_period": "0s"
  },
  "tally_params": {
    "quorum": "0.000000000000000000",
    "threshold": "0.000000000000000000",
    "veto_threshold": "0.000000000000000000"
  }
}
```

#### deposits

The `deposits` endpoint allows users to query a deposit for a given proposal from a given depositor.

Using legacy v1beta1:

```bash
/cosmos/gov/v1beta1/proposals/{proposal_id}/deposits/{depositor}
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1beta1/proposals/1/deposits/cosmos1..
```

Example Output:

```bash
{
  "deposit": {
    "proposal_id": "1",
    "depositor": "cosmos1..",
    "amount": [
      {
        "denom": "stake",
        "amount": "10000000"
      }
    ]
  }
}
```

Using v1:

```bash
/cosmos/gov/v1/proposals/{proposal_id}/deposits/{depositor}
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/proposals/1/deposits/cosmos1..
```

Example Output:

```bash
{
  "deposit": {
    "proposal_id": "1",
    "depositor": "cosmos1..",
    "amount": [
      {
        "denom": "stake",
        "amount": "10000000"
      }
    ]
  }
}
```

#### proposal deposits

The `deposits` endpoint allows users to query all deposits for a given proposal.

Using legacy v1beta1:

```bash
/cosmos/gov/v1beta1/proposals/{proposal_id}/deposits
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1beta1/proposals/1/deposits
```

Example Output:

```bash
{
  "deposits": [
    {
      "proposal_id": "1",
      "depositor": "cosmos1..",
      "amount": [
        {
          "denom": "stake",
          "amount": "10000000"
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

Using v1:

```bash
/cosmos/gov/v1/proposals/{proposal_id}/deposits
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/proposals/1/deposits
```

Example Output:

```bash
{
  "deposits": [
    {
      "proposal_id": "1",
      "depositor": "cosmos1..",
      "amount": [
        {
          "denom": "stake",
          "amount": "10000000"
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

#### tally

The `tally` endpoint allows users to query the tally of a given proposal.

Using legacy v1beta1:

```bash
/cosmos/gov/v1beta1/proposals/{proposal_id}/tally
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1beta1/proposals/1/tally
```

Example Output:

```bash
{
  "tally": {
    "yes": "1000000",
    "abstain": "0",
    "no": "0",
    "no_with_veto": "0"
  }
}
```

Using v1:

```bash
/cosmos/gov/v1/proposals/{proposal_id}/tally
```

Example:

```bash
curl localhost:1317/cosmos/gov/v1/proposals/1/tally
```

Example Output:

```bash
{
  "tally": {
    "yes": "1000000",
    "abstain": "0",
    "no": "0",
    "no_with_veto": "0"
  }
}
```

## Metadata

The gov module has two locations for metadata where users can provide further context about the on-chain actions they are taking. By default all metadata fields have a 255 character length field where metadata can be stored in json format, either on-chain or off-chain depending on the amount of data required. Here we provide a recommendation for the json structure and where the data should be stored. There are two important factors in making these recommendations. First, that the gov and group modules are consistent with one another, note the number of proposals made by all groups may be quite large. Second, that client applications such as block explorers and governance interfaces have confidence in the consistency of metadata structure accross chains.

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

## Future Improvements

The current documentation only describes the minimum viable product for the
governance module. Future improvements may include:

* **`BountyProposals`:** If accepted, a `BountyProposal` creates an open
  bounty. The `BountyProposal` specifies how many Atoms will be given upon
  completion. These Atoms will be taken from the `reserve pool`. After a
  `BountyProposal` is accepted by governance, anybody can submit a
  `SoftwareUpgradeProposal` with the code to claim the bounty. Note that once a
  `BountyProposal` is accepted, the corresponding funds in the `reserve pool`
  are locked so that payment can always be honored. In order to link a
  `SoftwareUpgradeProposal` to an open bounty, the submitter of the
  `SoftwareUpgradeProposal` will use the `Proposal.LinkedProposal` attribute.
  If a `SoftwareUpgradeProposal` linked to an open bounty is accepted by
  governance, the funds that were reserved are automatically transferred to the
  submitter.
* **Complex delegation:** Delegators could choose other representatives than
  their validators. Ultimately, the chain of representatives would always end
  up to a validator, but delegators could inherit the vote of their chosen
  representative before they inherit the vote of their validator. In other
  words, they would only inherit the vote of their validator if their other
  appointed representative did not vote.
* **Better process for proposal review:** There would be two parts to
  `proposal.Deposit`, one for anti-spam (same as in MVP) and an other one to
  reward third party auditors.
