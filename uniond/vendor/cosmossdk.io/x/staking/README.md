---
sidebar_position: 1
---

# `x/staking`

## Abstract

This paper specifies the Staking module of the Cosmos SDK that was first
described in the [Cosmos Whitepaper](https://cosmos.network/about/whitepaper)
in June 2016.

The module enables Cosmos SDK-based blockchain to support an advanced
Proof-of-Stake (PoS) system. In this system, holders of the native staking token of
the chain can become validators and can delegate tokens to validators,
ultimately determining the effective validator set for the system.

This module is used in the Cosmos Hub, the first Hub in the Cosmos
network.

## Contents

* [State](#state)
    * [Pool](#pool)
    * [LastTotalPower](#lasttotalpower)
    * [UnbondingID](#unbondingid)
    * [Params](#params)
    * [Validator](#validator)
    * [Delegation](#delegation)
    * [UnbondingDelegation](#unbondingdelegation)
    * [Redelegation](#redelegation)
    * [Queues](#queues)
    * [ConsPubkeyRotation](#conspubkeyrotation)
* [State Transitions](#state-transitions)
    * [Validators](#validators)
    * [Delegations](#delegations)
    * [Slashing](#slashing)
    * [How Shares are calculated](#how-shares-are-calculated)
* [Messages](#messages)
    * [MsgCreateValidator](#msgcreatevalidator)
    * [MsgEditValidator](#msgeditvalidator)
    * [MsgDelegate](#msgdelegate)
    * [MsgUndelegate](#msgundelegate)
    * [MsgCancelUnbondingDelegation](#msgcancelunbondingdelegation)
    * [MsgBeginRedelegate](#msgbeginredelegate)
    * [MsgUpdateParams](#msgupdateparams)
    * [MsgRotateConsPubkey](#msgrotateconspubkey)
* [End-Block](#end-block)
    * [Validator Set Changes](#validator-set-changes)
    * [Queues](#queues-1)
* [Hooks](#hooks)
* [Events](#events)
    * [EndBlocker](#endblocker)
    * [Msg's](#msgs)
* [Parameters](#parameters)
* [Client](#client)
    * [CLI](#cli)
    * [gRPC](#grpc)
    * [REST](#rest)

## State

### Pool

Pool is used for tracking bonded and not-bonded token supply of the bond denomination.

### LastTotalPower

LastTotalPower tracks the total amounts of bonded tokens recorded during the previous end block.
Store entries prefixed with "Last" must remain unchanged until EndBlock.

* LastTotalPower: `0x12 -> ProtocolBuffer(math.Int)`

### UnbondingID

UnbondingID stores the ID of the latest unbonding operation. It enables creating unique IDs for unbonding operations, i.e., UnbondingID is incremented every time a new unbonding operation (validator unbonding, unbonding delegation, redelegation) is initiated.

* UnbondingID: `0x37 -> uint64`

### Params

The staking module stores its params in state with the prefix of `0x51`,
it can be updated with governance or the address with authority.

* Params: `0x51 | ProtocolBuffer(Params)`

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/staking.proto#L300-L328
```

### Validator

Validators can have one of three statuses

* `Unbonded`: The validator is not in the active set. They cannot sign blocks and do not earn
  rewards. They can receive delegations.
* `Bonded`: Once the validator receives sufficient bonded tokens they automatically join the
  active set during [`EndBlock`](#validator-set-changes) and their status is updated to `Bonded`.
  They are signing blocks and receiving rewards. They can receive further delegations.
  They can be slashed for misbehavior. Delegators to this validator who unbond their delegation
  must wait the duration of the UnbondingTime, a chain-specific param, during which time
  they are still slashable for offences of the source validator if those offences were committed
  during the period of time that the tokens were bonded.
* `Unbonding`: When a validator leaves the active set, either by choice or due to slashing, jailing or
  tombstoning, an unbonding of all their delegations begins. All delegations must then wait the UnbondingTime
  before their tokens are moved to their accounts from the `BondedPool`.

:::warning
Tombstoning is permanent, once tombstoned a validator's consensus key can not be reused within the chain where the tombstoning happened.
:::

Validators objects should be primarily stored and accessed by the
`OperatorAddr`, an SDK validator address for the operator of the validator. Two
additional indices are maintained per validator object in order to fulfill
required lookups for slashing and validator-set updates. A third special index
(`LastValidatorPower`) is also maintained which however remains constant
throughout each block, unlike the first two indices which mirror the validator
records within a block.

* Validators: `0x21 | OperatorAddrLen (1 byte) | OperatorAddr -> ProtocolBuffer(validator)`
* ValidatorsByConsAddr: `0x22 | ConsAddrLen (1 byte) | ConsAddr -> OperatorAddr`
* ValidatorsByPower: `0x23 | BigEndian(ConsensusPower) | OperatorAddrLen (1 byte) | OperatorAddr -> OperatorAddr`
* LastValidatorsPower: `0x11 | OperatorAddrLen (1 byte) | OperatorAddr -> ProtocolBuffer(ConsensusPower)`
* ValidatorsByUnbondingID: `0x38 | UnbondingID ->  0x21 | OperatorAddrLen (1 byte) | OperatorAddr`

`Validators` is the primary index - it ensures that each operator can have only one
associated validator, where the public key of that validator can change in the
future. Delegators can refer to the immutable operator of the validator, without
concern for the changing public key.

`ValidatorsByUnbondingID` is an additional index that enables lookups for
 validators by the unbonding IDs corresponding to their current unbonding.

`ValidatorByConsAddr` is an additional index that enables lookups for slashing.
When CometBFT reports evidence, it provides the validator address, so this
map is needed to find the operator. Note that the `ConsAddr` corresponds to the
address which can be derived from the validator's `ConsPubKey`.

`ValidatorsByPower` is an additional index that provides a sorted list of
potential validators to quickly determine the current active set. Here
ConsensusPower is validator.Tokens/10^6 by default. Note that all validators
where `Jailed` is true are not stored within this index.

`LastValidatorsPower` is a special index that provides a historical list of the
last-block's bonded validators. This index remains constant during a block but
is updated during the validator set update process which takes place in [`EndBlock`](#end-block).

Each validator's state is stored in a `Validator` struct:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/staking.proto#L92-L138
```


The initial commission rates to be used for creating a validator are stored in a `CommissionRates` struct:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/staking.proto#L30-L54
```

### Delegation

Delegations are identified by combining `DelegatorAddr` (the address of the delegator)
with the `ValidatorAddr` Delegators are indexed in the store as follows:

* Delegation: `0x31 | DelegatorAddrLen (1 byte) | DelegatorAddr | ValidatorAddrLen (1 byte) | ValidatorAddr -> ProtocolBuffer(delegation)`

Stake holders may delegate coins to validators; under this circumstance their
funds are held in a `Delegation` data structure. It is owned by one
delegator, and is associated with the shares for one validator. The sender of
the transaction is the owner of the bond.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/staking.proto#L196-L210
```

#### Delegator Shares

When one delegates tokens to a Validator, they are issued a number of delegator shares based on a
dynamic exchange rate, calculated as follows from the total number of tokens delegated to the
validator and the number of shares issued so far:

`Shares per Token = validator.TotalShares() / validator.Tokens()`

Only the number of shares received is stored on the DelegationEntry. When a delegator then
Undelegates, the token amount they receive is calculated from the number of shares they currently
hold and the inverse exchange rate:

`Tokens per Share = validator.Tokens() / validatorShares()`

These `Shares` are simply an accounting mechanism. They are not a fungible asset. The reason for
this mechanism is to simplify the accounting around slashing. Rather than iteratively slashing the
tokens of every delegation entry, instead the Validator's total bonded tokens can be slashed,
effectively reducing the value of each issued delegator share.

### UnbondingDelegation

Shares in a `Delegation` can be unbonded, but they must for some time exist as
an `UnbondingDelegation`, where shares can be reduced if Byzantine behavior is
detected.

`UnbondingDelegation` are indexed in the store as:

* UnbondingDelegation: `0x32 | DelegatorAddrLen (1 byte) | DelegatorAddr | ValidatorAddrLen (1 byte) | ValidatorAddr -> ProtocolBuffer(unbondingDelegation)`
* UnbondingDelegationsFromValidator: `0x33 | ValidatorAddrLen (1 byte) | ValidatorAddr | DelegatorAddrLen (1 byte) | DelegatorAddr -> nil`
* UnbondingDelegationByUnbondingId: `0x38 | UnbondingId -> 0x32 | DelegatorAddrLen (1 byte) | DelegatorAddr | ValidatorAddrLen (1 byte) | ValidatorAddr`
 `UnbondingDelegation` is used in queries, to lookup all unbonding delegations for
 a given delegator.

`UnbondingDelegationsFromValidator` is used in slashing, to lookup all
 unbonding delegations associated with a given validator that need to be
 slashed.

 `UnbondingDelegationByUnbondingId` is an additional index that enables
 lookups for unbonding delegations by the unbonding IDs of the containing
 unbonding delegation entries.


A UnbondingDelegation object is created every time an unbonding is initiated.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/staking.proto#L214-L253
```

### Redelegation

The bonded tokens worth of a `Delegation` may be instantly redelegated from a
source validator to a different validator (destination validator). However when
this occurs they must be tracked in a `Redelegation` object, whereby their
shares can be slashed if their tokens have contributed to a Byzantine fault
committed by the source validator.

`Redelegation` are indexed in the store as:

* Redelegations: `0x34 | DelegatorAddrLen (1 byte) | DelegatorAddr | ValidatorAddrLen (1 byte) | ValidatorSrcAddr | ValidatorDstAddr -> ProtocolBuffer(redelegation)`
* RedelegationsBySrc: `0x35 | ValidatorSrcAddrLen (1 byte) | ValidatorSrcAddr | ValidatorDstAddrLen (1 byte) | ValidatorDstAddr | DelegatorAddrLen (1 byte) | DelegatorAddr -> nil`
* RedelegationsByDst: `0x36 | ValidatorDstAddrLen (1 byte) | ValidatorDstAddr | ValidatorSrcAddrLen (1 byte) | ValidatorSrcAddr | DelegatorAddrLen (1 byte) | DelegatorAddr -> nil`
* RedelegationByUnbondingId: `0x38 | UnbondingId -> 0x34 | DelegatorAddrLen (1 byte) | DelegatorAddr | ValidatorAddrLen (1 byte) | ValidatorSrcAddr | ValidatorDstAddr`

 `Redelegations` is used for queries, to lookup all redelegations for a given
 delegator.

 `RedelegationsBySrc` is used for slashing based on the `ValidatorSrcAddr`.

 `RedelegationsByDst` is used for slashing based on the `ValidatorDstAddr`

The first map here is used for queries, to lookup all redelegations for a given
delegator. The second map is used for slashing based on the `ValidatorSrcAddr`,
while the third map is for slashing based on the `ValidatorDstAddr`.

`RedelegationByUnbondingId` is an additional index that enables
 lookups for redelegations by the unbonding IDs of the containing
 redelegation entries.

A redelegation object is created every time a redelegation occurs. To prevent
"redelegation hopping" redelegations may not occur under the situation that:

* the (re)delegator already has another immature redelegation in progress
  with a destination to a validator (let's call it `Validator X`)
* and, the (re)delegator is attempting to create a _new_ redelegation
  where the source validator for this new redelegation is `Validator X`.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/staking.proto#L256-L298
```

## ConsPubkeyRotation

The `ConsPubkey` of a validator will be instantly rotated to the new `ConsPubkey`. The rotation will be tracked to only allow a limited number of rotations within an unbonding period of time.

`ConsPubkeyRotation` are indexed in the store as:

ValidatorConsPubKeyRotationHistoryKey: `101 | valAddr | rotatedHeight -> ProtocolBuffer(ConsPubKeyRotationHistory)`555682

BlockConsPubKeyRotationHistoryKey (index): `102 | rotatedHeight | valAddr | -> ProtocolBuffer(ConsPubKeyRotationHistory)`

ValidatorConsensusKeyRotationRecordQueueKey: `103 | format(time) -> ProtocolBuffer(ValAddrsOfRotatedConsKeys)`

ValidatorConsensusKeyRotationRecordIndexKey:`104 | valAddr | format(time) -> ProtocolBuffer([]Byte{})`

OldToNewConsAddrMap:`105 | byte(oldConsAddr) -> byte(newConsAddr)`

ConsAddrToValidatorIdentifierMap:`106 | byte(newConsAddr) -> byte(initialConsAddr)`

`ConsPubKeyRotationHistory` is used for querying the rotations of a validator

`ValidatorConsensusKeyRotationRecordQueueKey` is to keep track of the rotation across the unbonding period (waiting period in the queue), this will be pruned after the unbonding period of waiting time.

`ValidatorConsensusKeyRotationRecordIndexKey` is to keep track of a validator that how many rotations were made inside unbonding period. This will be pruned after the unbonding period of waiting time.

A `ConsPubKeyRotationHistory` object is created every time a consensus pubkey rotation occurs.

An entry is added in `OldToNewConsAddrMap` collection for every rotation (Note: this is to handle the evidences when submitted with old cons key).

An entry is added in `ConsAddrToValidatorIdentifierMap` collection for every rotation, this entry is to block the rotation if the validator is rotating to the cons key which is involved in the history.

To prevent the spam: 

* There will only limited number of rotations can be done within unbonding period of time. 
* A non-negligible fee will be deducted for rotating a consensus key.

### Queues

All queue objects are sorted by timestamp. The time used within any queue is
firstly converted to UTC, rounded to the nearest nanosecond then sorted. The sortable time format
used is a slight modification of the RFC3339Nano and uses the format string
`"2006-01-02T15:04:05.000000000"`. Notably this format:

* right pads all zeros
* drops the time zone info (we already use UTC)

In all cases, the stored timestamp represents the maturation time of the queue
element.

#### UnbondingDelegationQueue

For the purpose of tracking progress of unbonding delegations the unbonding
delegations queue is kept.

* UnbondingDelegation: `0x41 | format(time) -> []DVPair`

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/staking.proto#L159-L173
```

#### RedelegationQueue

For the purpose of tracking progress of redelegations the redelegation queue is
kept.

* RedelegationQueue: `0x42 | format(time) -> []DVVTriplet`

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/staking.proto#L175-L191
```

#### ValidatorQueue

For the purpose of tracking progress of unbonding validators the validator
queue is kept.

* ValidatorQueueTime: `0x43 | format(time) -> []sdk.ValAddress`

The stored object by each key is an array of validator operator addresses from
which the validator object can be accessed. Typically it is expected that only
a single validator record will be associated with a given timestamp however it is possible
that multiple validators exist in the queue at the same location.

#### ValidatorConsensusKeyRotationRecordQueueKey

For the purpose of tracking progress or consensus pubkey rotations the `ValidatorConsensusKeyRotationRecordQueueKey` kept.

* ValidatorConsensusKeyRotationRecordQueueKey: `103 | format(time) -> types.ValAddrsOfRotatedConsKeys`

Here timestamp will be the unique identifier in the queue which is of future time 
(which is calculated with the current block time adding with unbonding period),
Whenever the next item with the same waiting time comes to the queue, we will get
the present store info and append the `ValAddress` to the array and set it back in the store.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/staking.proto#L420-L424
```




## State Transitions

### Validators

State transitions in validators are performed on every [`EndBlock`](#validator-set-changes)
in order to check for changes in the active `ValidatorSet`.

A validator can be `Unbonded`, `Unbonding` or `Bonded`. `Unbonded`
and `Unbonding` are collectively called `Not Bonded`. A validator can move
directly between all the states, except for from `Bonded` to `Unbonded`.

#### Not bonded to Bonded

The following transition occurs when a validator's ranking in the `ValidatorPowerIndex` surpasses
that of the `LastValidator`.

* set `validator.Status` to `Bonded`
* send the `validator.Tokens` from the `NotBondedTokens` to the `BondedPool` `ModuleAccount`
* delete the existing record from `ValidatorByPowerIndex`
* add a new updated record to the `ValidatorByPowerIndex`
* update the `Validator` object for this validator
* if it exists, delete any `ValidatorQueue` record for this validator

#### Bonded to Unbonding

When a validator begins the unbonding process the following operations occur:

* send the `validator.Tokens` from the `BondedPool` to the `NotBondedTokens` `ModuleAccount`
* set `validator.Status` to `Unbonding`
* delete the existing record from `ValidatorByPowerIndex`
* add a new updated record to the `ValidatorByPowerIndex`
* update the `Validator` object for this validator
* insert a new record into the `ValidatorQueue` for this validator

#### Unbonding to Unbonded

A validator moves from unbonding to unbonded when the `ValidatorQueue` object
moves from bonded to unbonded

* update the `Validator` object for this validator
* set `validator.Status` to `Unbonded`

#### Jail/Unjail

when a validator is jailed it is effectively removed from the CometBFT set.
this process may be also be reversed. the following operations occur:

* set `Validator.Jailed` and update object
* if jailed delete record from `ValidatorByPowerIndex`
* if unjailed add record to `ValidatorByPowerIndex`

Jailed validators are not present in any of the following stores:

* the power store (from consensus power to address)

### Delegations

#### Delegate

When a delegation occurs both the validator and the delegation objects are affected

* determine the delegators shares based on tokens delegated and the validator's exchange rate
* remove tokens from the sending account
* add shares the delegation object or add them to a created validator object
* add new delegator shares and update the `Validator` object
* transfer the `delegation.Amount` from the delegator's account to the `BondedPool` or the `NotBondedPool` `ModuleAccount` depending if the `validator.Status` is `Bonded` or not
* delete the existing record from `ValidatorByPowerIndex`
* add an new updated record to the `ValidatorByPowerIndex`

#### Begin Unbonding

As a part of the Undelegate and Complete Unbonding state transitions Unbond
Delegation may be called.

* subtract the unbonded shares from delegator
* add the unbonded tokens to an `UnbondingDelegationEntry`
* update the delegation or remove the delegation if there are no more shares
* if the delegation is the operator of the validator and no more shares exist then trigger a jail validator
* update the validator with removed the delegator shares and associated coins
* if the validator state is `Bonded`, transfer the `Coins` worth of the unbonded
  shares from the `BondedPool` to the `NotBondedPool` `ModuleAccount`
* remove the validator if it is unbonded and there are no more delegation shares.
* remove the validator if it is unbonded and there are no more delegation shares
* get a unique `unbondingId` and map it to the `UnbondingDelegationEntry` in `UnbondingDelegationByUnbondingId`
* call the `AfterUnbondingInitiated(unbondingId)` hook
* add the unbonding delegation to `UnbondingDelegationQueue` with the completion time set to `UnbondingTime`

#### Cancel an `UnbondingDelegation` Entry

When a `cancel unbond delegation` occurs both the `validator`, the `delegation` and an `UnbondingDelegationQueue` state will be updated.

* if cancel unbonding delegation amount equals to the `UnbondingDelegation` entry `balance`, then the `UnbondingDelegation` entry deleted from `UnbondingDelegationQueue`.
* if the `cancel unbonding delegation amount is less than the `UnbondingDelegation` entry balance, then the `UnbondingDelegation` entry will be updated with new balance in the `UnbondingDelegationQueue`.
* cancel `amount` is [Delegated](#delegations) back to  the original `validator`.

#### Complete Unbonding

For undelegations which do not complete immediately, the following operations
occur when the unbonding delegation queue element matures:

* remove the entry from the `UnbondingDelegation` object
* transfer the tokens from the `NotBondedPool` `ModuleAccount` to the delegator `Account`

#### Begin Redelegation

Redelegations affect the delegation, source and destination validators.

* perform an `unbond` delegation from the source validator to retrieve the tokens worth of the unbonded shares
* using the unbonded tokens, `Delegate` them to the destination validator
* if the `sourceValidator.Status` is `Bonded`, and the `destinationValidator` is not,
  transfer the newly delegated tokens from the `BondedPool` to the `NotBondedPool` `ModuleAccount`
* otherwise, if the `sourceValidator.Status` is not `Bonded`, and the `destinationValidator`
  is `Bonded`, transfer the newly delegated tokens from the `NotBondedPool` to the `BondedPool` `ModuleAccount`
* record the token amount in an new entry in the relevant `Redelegation`

From when a redelegation begins until it completes, the delegator is in a state of "pseudo-unbonding", and can still be
slashed for infractions that occurred before the redelegation began.

#### Complete Redelegation

When a redelegations complete the following occurs:

* remove the entry from the `Redelegation` object

#### Consensus pubkey rotation

When a `ConsPubkeyRotation` occurs the validator and the `ValidatorConsensusKeyRotationRecordQueueKey` are updated:

* the old consensus pubkey address will be removed from state and new consensus pubkey address will be added in place.
* transfers the voting power to the new consensus pubkey address.
* and triggers the hooks to update the `signing-info` in the `slashing` module 
* and triggers the hooks to add the deducted fee to the `community pool` funds

### Slashing

#### Slash Validator

When a Validator is slashed, the following occurs:

* The total `slashAmount` is calculated as the `slashFactor` (a chain parameter) \* `TokensFromConsensusPower`,
  the total number of tokens bonded to the validator at the time of the infraction.
* Every unbonding delegation and pseudo-unbonding redelegation such that the infraction occurred before the unbonding or
  redelegation began from the validator are slashed by the `slashFactor` percentage of the initialBalance.
* Each amount slashed from redelegations and unbonding delegations is subtracted from the
  total slash amount.
* The `remaingSlashAmount` is then slashed from the validator's tokens in the `BondedPool` or
  `NonBondedPool` depending on the validator's status. This reduces the total supply of tokens.

In the case of a slash due to any infraction that requires evidence to submitted (for example double-sign), the slash
occurs at the block where the evidence is included, not at the block where the infraction occurred.
Put otherwise, validators are not slashed retroactively, only when they are caught.

#### Slash Unbonding Delegation

When a validator is slashed, so are those unbonding delegations from the validator that began unbonding
after the time of the infraction. Every entry in every unbonding delegation from the validator
is slashed by `slashFactor`. The amount slashed is calculated from the `InitialBalance` of the
delegation and is capped to prevent a resulting negative balance. Completed (or mature) unbondings are not slashed.

#### Slash Redelegation

When a validator is slashed, so are all redelegations from the validator that began after the
infraction. Redelegations are slashed by `slashFactor`.
Redelegations that began before the infraction are not slashed.
The amount slashed is calculated from the `InitialBalance` of the delegation and is capped to
prevent a resulting negative balance.
Mature redelegations (that have completed pseudo-unbonding) are not slashed.

### How Shares are calculated

At any given point in time, each validator has a number of tokens, `T`, and has a number of shares issued, `S`.
Each delegator, `i`, holds a number of shares, `S_i`.
The number of tokens is the sum of all tokens delegated to the validator, plus the rewards, minus the slashes.

The delegator is entitled to a portion of the underlying tokens proportional to their proportion of shares.
So delegator `i` is entitled to `T * S_i / S` of the validator's tokens.

When a delegator delegates new tokens to the validator, they receive a number of shares proportional to their contribution.
So when delegator `j` delegates `T_j` tokens, they receive `S_j = S * T_j / T` shares.
The total number of tokens is now `T + T_j`, and the total number of shares is `S + S_j`.
`j`s proportion of the shares is the same as their proportion of the total tokens contributed: `(S + S_j) / S = (T + T_j) / T`.

A special case is the initial delegation, when `T = 0` and `S = 0`, so `T_j / T` is undefined.
For the initial delegation, delegator `j` who delegates `T_j` tokens receive `S_j = T_j` shares.
So a validator that hasn't received any rewards and has not been slashed will have `T = S`.

## Messages

In this section we describe the processing of the staking messages and the corresponding updates to the state. All created/modified state objects specified by each message are defined within the [state](#state) section.

### MsgCreateValidator

A validator is created using the `MsgCreateValidator` message.
The validator must be created with an initial delegation from the operator.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L20-L21
```

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L57-L80
```

This message is expected to fail if:

* another validator with this operator address is already registered
* another validator with this pubkey is already registered
* the initial self-delegation tokens are of a denom not specified as the bonding denom
* the commission parameters are faulty, namely:
    * `MaxRate` is either > 1 or < 0
    * the initial `Rate` is either negative or > `MaxRate`
    * the initial `MaxChangeRate` is either negative or > `MaxRate`
* the description fields are too large

This message creates and stores the `Validator` object at appropriate indexes.
Additionally a self-delegation is made with the initial tokens delegation
tokens `Delegation`. The validator always starts as unbonded but may be bonded
in the first end-block.

### MsgEditValidator

The `Description`, `CommissionRate` of a validator can be updated using the
`MsgEditValidator` message.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L23-L24
```

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L85-L104
```

This message is expected to fail if:

* the initial `CommissionRate` is either negative or > `MaxRate`
* the `CommissionRate` has already been updated within the previous 24 hours
* the `CommissionRate` is > `MaxChangeRate`
* the description fields are too large

This message stores the updated `Validator` object.

### MsgDelegate

Within this message the delegator provides coins, and in return receives
some amount of their validator's (newly created) delegator-shares that are
assigned to `Delegation.Shares`.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L26-L28
```

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L109-L121
```

This message is expected to fail if:

* the validator does not exist
* the `Amount` `Coin` has a denomination different than one defined by `params.BondDenom`
* the exchange rate is invalid, meaning the validator has no tokens (due to slashing) but there are outstanding shares
* the amount delegated is less than the minimum allowed delegation

If an existing `Delegation` object for provided addresses does not already
exist then it is created as part of this message otherwise the existing
`Delegation` is updated to include the newly received shares.

The delegator receives newly minted shares at the current exchange rate.
The exchange rate is the number of existing shares in the validator divided by
the number of currently delegated tokens.

The validator is updated in the `ValidatorByPower` index, and the delegation is
tracked in validator object in the `Validators` index.

It is possible to delegate to a jailed validator, the only difference being it
will not be added to the power index until it is unjailed.

![Delegation sequence](https://raw.githubusercontent.com/cosmos/cosmos-sdk/release/v0.46.x/docs/uml/svg/delegation_sequence.svg)

### MsgUndelegate

The `MsgUndelegate` message allows delegators to undelegate their tokens from
validator.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L34-L36
```

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L147-L159
```

This message returns a response containing the completion time of the undelegation:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L161-L169
```

This message is expected to fail if:

* the delegation doesn't exist
* the validator doesn't exist
* the delegation has less shares than the ones worth of `Amount`
* existing `UnbondingDelegation` has maximum entries as defined by `params.MaxEntries`
* the `Amount` has a denomination different than one defined by `params.BondDenom`

When this message is processed the following actions occur:

* validator's `DelegatorShares` and the delegation's `Shares` are both reduced by the message `SharesAmount`
* calculate the token worth of the shares remove that amount tokens held within the validator
* with those removed tokens, if the validator is:
    * `Bonded` - add them to an entry in `UnbondingDelegation` (create `UnbondingDelegation` if it doesn't exist) with a completion time a full unbonding period from the current time. Update pool shares to reduce BondedTokens and increase NotBondedTokens by token worth of the shares.
    * `Unbonding` - add them to an entry in `UnbondingDelegation` (create `UnbondingDelegation` if it doesn't exist) with the same completion time as the validator (`UnbondingMinTime`).
    * `Unbonded` - then send the coins the message `DelegatorAddr`
* if there are no more `Shares` in the delegation, then the delegation object is removed from the store
    * under this situation if the delegation is the validator's self-delegation then also jail the validator.

![Unbond sequence](https://raw.githubusercontent.com/cosmos/cosmos-sdk/release/v0.46.x/docs/uml/svg/unbond_sequence.svg)

### MsgCancelUnbondingDelegation

The `MsgCancelUnbondingDelegation` message allows delegators to cancel the `unbondingDelegation` entry and delegate back to a previous validator. However, please note that this feature does not support canceling unbond delegations from jailed validators.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L38-L42
```

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L171-L185
```

This message is expected to fail if:

* the `unbondingDelegation` entry is already processed.
* the `cancel unbonding delegation` amount is greater than the `unbondingDelegation` entry balance.
* the `cancel unbonding delegation` height doesn't exist in the `unbondingDelegationQueue` of the delegator.
* the `unbondingDelegation` is from a jailed validator.

When this message is processed the following actions occur:

* if the `unbondingDelegation` Entry balance is zero
    * in this condition `unbondingDelegation` entry will be removed from `unbondingDelegationQueue`.
    * otherwise `unbondingDelegationQueue` will be updated with new `unbondingDelegation` entry balance and initial balance
* the validator's `DelegatorShares` and the delegation's `Shares` are both increased by the message `Amount`.

### MsgBeginRedelegate

The redelegation command allows delegators to instantly switch validators. Once
the unbonding period has passed, the redelegation is automatically completed in
the EndBlocker.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L30-L32
```

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L126-L139
```

This message returns a response containing the completion time of the redelegation:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L141-L145
```

This message is expected to fail if:

* the delegation doesn't exist
* the source or destination validators don't exist
* the delegation has less shares than the ones worth of `Amount`
* the source validator has a receiving redelegation which is not matured (aka. the redelegation may be transitive)
* existing `Redelegation` has maximum entries as defined by `params.MaxEntries`
* the `Amount` `Coin` has a denomination different than one defined by `params.BondDenom`

When this message is processed the following actions occur:

* the source validator's `DelegatorShares` and the delegations `Shares` are both reduced by the message `SharesAmount`
* calculate the token worth of the shares remove that amount tokens held within the source validator.
* if the source validator is:
    * `Bonded` - add an entry to the `Redelegation` (create `Redelegation` if it doesn't exist) with a completion time a full unbonding period from the current time. Update pool shares to reduce BondedTokens and increase NotBondedTokens by token worth of the shares (this may be effectively reversed in the next step however).
    * `Unbonding` - add an entry to the `Redelegation` (create `Redelegation` if it doesn't exist) with the same completion time as the validator (`UnbondingMinTime`).
    * `Unbonded` - no action required in this step
* Delegate the token worth to the destination validator, possibly moving tokens back to the bonded state.
* if there are no more `Shares` in the source delegation, then the source delegation object is removed from the store
    * under this situation if the delegation is the validator's self-delegation then also jail the validator.

![Begin redelegation sequence](https://raw.githubusercontent.com/cosmos/cosmos-sdk/release/v0.46.x/docs/uml/svg/begin_redelegation_sequence.svg)


### MsgUpdateParams

The `MsgUpdateParams` update the staking module parameters.
The params are updated through a governance proposal where the signer is the gov module account address.
When the `MinCommissionRate` is updated, all validators with a lower (max) commission rate than `MinCommissionRate` will be updated to `MinCommissionRate`.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L192-L203
```

The message handling can fail if:

* signer is not the authority defined in the staking keeper (usually the gov module account).

### MsgRotateConsPubKey

The `MsgRotateConsPubKey` updates the consensus pubkey of a validator
with a new pubkey, the validator must pay rotation fees (default fee 1000000stake) to rotate the consensus pubkey.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/staking/proto/cosmos/staking/v1beta1/tx.proto#L211-L222
```

The message handling can fail if:

* The new pubkey is not a `cryptotypes.PubKey`.
* The new pubkey is already associated with another validator.
* The new pubkey is already present in the cons pubkey rotation history.
* The validator address is not in validators list.
* The `max_cons_pubkey_rotations` limit reached within unbonding period.
* The validator doesn't have enough balance to pay for the rotation.


## End-Block

Each abci end block call, the operations to update queues and validator set
changes are specified to execute.

### Validator Set Changes

The staking validator set is updated during this process by state transitions
that run at the end of every block. As a part of this process any updated
validators are also returned back to CometBFT for inclusion in the CometBFT
validator set which is responsible for validating CometBFT messages at the
consensus layer. Operations are as following:

* the new validator set is taken as the top `params.MaxValidators` number of
  validators retrieved from the `ValidatorsByPower` index
* the previous validator set is compared with the new validator set:
    * missing validators begin unbonding and their `Tokens` are transferred from the
    `BondedPool` to the `NotBondedPool` `ModuleAccount`
    * new validators are instantly bonded and their `Tokens` are transferred from the
    `NotBondedPool` to the `BondedPool` `ModuleAccount`

In all cases, any validators leaving or entering the bonded validator set or
changing balances and staying within the bonded validator set incur an update
message reporting their new consensus power which is passed back to CometBFT.

The `LastTotalPower` and `LastValidatorsPower` hold the state of the total power
and validator power from the end of the last block, and are used to check for
changes that have occurred in `ValidatorsByPower` and the total new power, which
is calculated during `EndBlock`.

### Queues

Within staking, certain state-transitions are not instantaneous but take place
over a duration of time (typically the unbonding period). When these
transitions are mature certain operations must take place in order to complete
the state operation. This is achieved through the use of queues which are
checked/processed at the end of each block.

#### Unbonding Validators

When a validator is kicked out of the bonded validator set (either through
being jailed, or not having sufficient bonded tokens) it begins the unbonding
process along with all its delegations begin unbonding (while still being
delegated to this validator). At this point the validator is said to be an
"unbonding validator", whereby it will mature to become an "unbonded validator"
after the unbonding period has passed.

Each block the validator queue is to be checked for mature unbonding validators
(namely with a completion time <= current time and completion height <= current
block height). At this point any mature validators which do not have any
delegations remaining are deleted from state. For all other mature unbonding
validators that still have remaining delegations, the `validator.Status` is
switched from `types.Unbonding` to
`types.Unbonded`.

Unbonding operations can be put on hold by external modules via the `PutUnbondingOnHold(unbondingId)` method.
 As a result, an unbonding operation (e.g., an unbonding delegation) that is on hold, cannot complete
 even if it reaches maturity. For an unbonding operation with `unbondingId` to eventually complete
 (after it reaches maturity), every call to `PutUnbondingOnHold(unbondingId)` must be matched
 by a call to `UnbondingCanComplete(unbondingId)`.

#### Unbonding Delegations

Complete the unbonding of all mature `UnbondingDelegations.Entries` within the
`UnbondingDelegations` queue with the following procedure:

* transfer the balance coins to the delegator's wallet address
* remove the mature entry from `UnbondingDelegation.Entries`
* remove the `UnbondingDelegation` object from the store if there are no
  remaining entries.

#### Redelegations

Complete the unbonding of all mature `Redelegation.Entries` within the
`Redelegations` queue with the following procedure:

* remove the mature entry from `Redelegation.Entries`
* remove the `Redelegation` object from the store if there are no
  remaining entries.

#### ConsPubKeyRotations

After the completion of the unbonding period, matured rotations will be removed from the queues and indexes to unblock the validator for the next iterations.

* remove the mature entry from state of `ValidatorConsensusKeyRotationRecordQueueKey`
* remove the mature entry form state of 
`ValidatorConsensusKeyRotationRecordIndexKey`

## Hooks

Other modules may register operations to execute when a certain event has
occurred within staking.  These events can be registered to execute either
right `Before` or `After` the staking event (as per the hook name). The
following hooks can registered with staking:

* `AfterValidatorCreated(Context, ValAddress) error`
    * called when a validator is created
* `BeforeValidatorModified(Context, ValAddress) error`
    * called when a validator's state is changed
* `AfterValidatorRemoved(Context, ConsAddress, ValAddress) error`
    * called when a validator is deleted
* `AfterValidatorBonded(Context, ConsAddress, ValAddress) error`
    * called when a validator is bonded
* `AfterValidatorBeginUnbonding(Context, ConsAddress, ValAddress) error`
    * called when a validator begins unbonding
* `BeforeDelegationCreated(Context, AccAddress, ValAddress) error`
    * called when a delegation is created
* `BeforeDelegationSharesModified(Context, AccAddress, ValAddress) error`
    * called when a delegation's shares are modified
* `AfterDelegationModified(Context, AccAddress, ValAddress) error`
    * called when a delegation is created or modified
* `BeforeDelegationRemoved(Context, AccAddress, ValAddress) error`
    * called when a delegation is removed
* `AfterUnbondingInitiated(Context, UnbondingID)`
    * called when an unbonding operation (validator unbonding, unbonding delegation, redelegation) was initiated
* `AfterConsensusPubKeyUpdate(ctx Context, oldpubkey, newpubkey types.PubKey, fee sdk.Coin)`
    * called when a consensus pubkey rotation of a validator is initiated.


## Events

The staking module emits the following events:

### EndBlocker

| Type                  | Attribute Key         | Attribute Value           |
| --------------------- | --------------------- | ------------------------- |
| complete_unbonding    | amount                | {totalUnbondingAmount}    |
| complete_unbonding    | validator             | {validatorAddress}        |
| complete_unbonding    | delegator             | {delegatorAddress}        |
| complete_redelegation | amount                | {totalRedelegationAmount} |
| complete_redelegation | source_validator      | {srcValidatorAddress}     |
| complete_redelegation | destination_validator | {dstValidatorAddress}     |
| complete_redelegation | delegator             | {delegatorAddress}        |

## Msg's

### MsgCreateValidator

| Type             | Attribute Key | Attribute Value    |
| ---------------- | ------------- | ------------------ |
| create_validator | validator     | {validatorAddress} |
| create_validator | amount        | {delegationAmount} |
| message          | module        | staking            |
| message          | action        | create_validator   |
| message          | sender        | {senderAddress}    |

### MsgEditValidator

| Type           | Attribute Key       | Attribute Value     |
| -------------- | ------------------- | ------------------- |
| edit_validator | commission_rate     | {commissionRate}    |
| edit_validator | min_self_delegation | {minSelfDelegation} |
| message        | module              | staking             |
| message        | action              | edit_validator      |
| message        | sender              | {senderAddress}     |

### MsgDelegate

| Type     | Attribute Key | Attribute Value    |
| -------- | ------------- | ------------------ |
| delegate | validator     | {validatorAddress} |
| delegate | amount        | {delegationAmount} |
| message  | module        | staking            |
| message  | action        | delegate           |
| message  | sender        | {senderAddress}    |

### MsgUndelegate

| Type    | Attribute Key       | Attribute Value    |
| ------- | ------------------- | ------------------ |
| unbond  | validator           | {validatorAddress} |
| unbond  | amount              | {unbondAmount}     |
| unbond  | completion_time [0] | {completionTime}   |
| message | module              | staking            |
| message | action              | begin_unbonding    |
| message | sender              | {senderAddress}    |

* [0] Time is formatted in the RFC3339 standard

### MsgCancelUnbondingDelegation

| Type                          | Attribute Key       | Attribute Value                     |
| ----------------------------- | ------------------  | ------------------------------------|
| cancel_unbonding_delegation   | validator           | {validatorAddress}                  |
| cancel_unbonding_delegation   | delegator           | {delegatorAddress}                  |
| cancel_unbonding_delegation   | amount              | {cancelUnbondingDelegationAmount}   |
| cancel_unbonding_delegation   | creation_height     | {unbondingCreationHeight}           |
| message                       | module              | staking                             |
| message                       | action              | cancel_unbond                       |
| message                       | sender              | {senderAddress}                     |

### MsgBeginRedelegate

| Type       | Attribute Key         | Attribute Value       |
| ---------- | --------------------- | --------------------- |
| redelegate | source_validator      | {srcValidatorAddress} |
| redelegate | destination_validator | {dstValidatorAddress} |
| redelegate | amount                | {unbondAmount}        |
| redelegate | completion_time [0]   | {completionTime}      |
| message    | module                | staking               |
| message    | action                | begin_redelegate      |
| message    | sender                | {senderAddress}       |

* [0] Time is formatted in the RFC3339 standard

## Parameters

The staking module contains the following parameters:

| Key                    | Type             | Example                |
|-------------------     |------------------|------------------------|
| UnbondingTime          | string (time ns) | "259200000000000"      |
| MaxValidators          | uint16           | 100                    |
| KeyMaxEntries          | uint16           | 7                      |
| HistoricalEntries      | uint16           | 3                      |
| BondDenom              | string           | "stake"                |
| MinCommissionRate      | string           | "0.000000000000000000" |
| KeyRotationFee         | sdk.Coin         | "1000000stake"         |
| MaxConsPubkeyRotations | int              | 1                      |

:::warning
Manually updating the `MinCommissionRate` parameter will not affect the commission rate of the existing validators. It will only affect the commission rate of the new validators. Update the parameter with `MsgUpdateParams` to affect the commission rate of the existing validators as well.
:::

## Client

### CLI

A user can query and interact with the `staking` module using the CLI.

#### Query

The `query` commands allows users to query `staking` state.

```bash
simd query staking --help
```

##### delegation

The `delegation` command allows users to query delegations for an individual delegator on an individual validator.

Usage:

```bash
simd query staking delegation [delegator-addr] [validator-addr] [flags]
```

Example:

```bash
simd query staking delegation cosmos1gghjut3ccd8ay0zduzj64hwre2fxs9ld75ru9p cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj
```

Example Output:

```bash
balance:
  amount: "10000000000"
  denom: stake
delegation:
  delegator_address: cosmos1gghjut3ccd8ay0zduzj64hwre2fxs9ld75ru9p
  shares: "10000000000.000000000000000000"
  validator_address: cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj
```

##### delegations

The `delegations` command allows users to query delegations for an individual delegator on all validators.

Usage:

```bash
simd query staking delegations [delegator-addr] [flags]
```

Example:

```bash
simd query staking delegations cosmos1gghjut3ccd8ay0zduzj64hwre2fxs9ld75ru9p
```

Example Output:

```bash
delegation_responses:
- balance:
    amount: "10000000000"
    denom: stake
  delegation:
    delegator_address: cosmos1gghjut3ccd8ay0zduzj64hwre2fxs9ld75ru9p
    shares: "10000000000.000000000000000000"
    validator_address: cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj
- balance:
    amount: "10000000000"
    denom: stake
  delegation:
    delegator_address: cosmos1gghjut3ccd8ay0zduzj64hwre2fxs9ld75ru9p
    shares: "10000000000.000000000000000000"
    validator_address: cosmosvaloper1x20lytyf6zkcrv5edpkfkn8sz578qg5sqfyqnp
pagination:
  next_key: null
  total: "0"
```

##### delegations-to

The `delegations-to` command allows users to query delegations on an individual validator.

Usage:

```bash
simd query staking delegations-to [validator-addr] [flags]
```

Example:

```bash
simd query staking delegations-to cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj
```

Example Output:

```bash
- balance:
    amount: "504000000"
    denom: stake
  delegation:
    delegator_address: cosmos1q2qwwynhv8kh3lu5fkeex4awau9x8fwt45f5cp
    shares: "504000000.000000000000000000"
    validator_address: cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj
- balance:
    amount: "78125000000"
    denom: uixo
  delegation:
    delegator_address: cosmos1qvppl3479hw4clahe0kwdlfvf8uvjtcd99m2ca
    shares: "78125000000.000000000000000000"
    validator_address: cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj
pagination:
  next_key: null
  total: "0"
```

##### historical-info

The `historical-info` command allows users to query historical information at given height.

Usage:

```bash
simd query staking historical-info [height] [flags]
```

Example:

```bash
simd query staking historical-info 10
```

Example Output:

```bash
header:
  app_hash: Lbx8cXpI868wz8sgp4qPYVrlaKjevR5WP/IjUxwp3oo=
  chain_id: testnet
  consensus_hash: BICRvH3cKD93v7+R1zxE2ljD34qcvIZ0Bdi389qtoi8=
  data_hash: 47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=
  evidence_hash: 47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=
  height: "10"
  last_block_id:
    hash: RFbkpu6pWfSThXxKKl6EZVDnBSm16+U0l0xVjTX08Fk=
    part_set_header:
      hash: vpIvXD4rxD5GM4MXGz0Sad9I7//iVYLzZsEU4BVgWIU=
      total: 1
  last_commit_hash: Ne4uXyx4QtNp4Zx89kf9UK7oG9QVbdB6e7ZwZkhy8K0=
  last_results_hash: 47DEQpj8HBSa+/TImW+5JCeuQeRkm5NMpJWZG3hSuFU=
  next_validators_hash: nGBgKeWBjoxeKFti00CxHsnULORgKY4LiuQwBuUrhCs=
  proposer_address: mMEP2c2IRPLr99LedSRtBg9eONM=
  time: "2021-10-01T06:00:49.785790894Z"
  validators_hash: nGBgKeWBjoxeKFti00CxHsnULORgKY4LiuQwBuUrhCs=
  version:
    app: "0"
    block: "11"
valset:
- commission:
    commission_rates:
      max_change_rate: "0.010000000000000000"
      max_rate: "0.200000000000000000"
      rate: "0.100000000000000000"
    update_time: "2021-10-01T05:52:50.380144238Z"
  consensus_pubkey:
    '@type': /cosmos.crypto.ed25519.PubKey
    key: Auxs3865HpB/EfssYOzfqNhEJjzys2Fo6jD5B8tPgC8=
  delegator_shares: "10000000.000000000000000000"
  description:
    details: ""
    identity: ""
    moniker: myvalidator
    security_contact: ""
    website: ""
  jailed: false
  min_self_delegation: "1"
  operator_address: cosmosvaloper1rne8lgs98p0jqe82sgt0qr4rdn4hgvmgp9ggcc
  status: BOND_STATUS_BONDED
  tokens: "10000000"
  unbonding_height: "0"
  unbonding_time: "1970-01-01T00:00:00Z"
```

##### params

The `params` command allows users to query values set as staking parameters.

Usage:

```bash
simd query staking params [flags]
```

Example:

```bash
simd query staking params
```

Example Output:

```bash
bond_denom: stake
historical_entries: 10000
max_entries: 7
max_validators: 50
unbonding_time: 1814400s
```

##### pool

The `pool` command allows users to query values for amounts stored in the staking pool.

Usage:

```bash
simd q staking pool [flags]
```

Example:

```bash
simd q staking pool
```

Example Output:

```bash
bonded_tokens: "10000000"
not_bonded_tokens: "0"
```

##### redelegation

The `redelegation` command allows users to query a redelegation record based on delegator and a source and destination validator address.

Usage:

```bash
simd query staking redelegation [delegator-addr] [src-validator-addr] [dst-validator-addr] [flags]
```

Example:

```bash
simd query staking redelegation cosmos1gghjut3ccd8ay0zduzj64hwre2fxs9ld75ru9p cosmosvaloper1l2rsakp388kuv9k8qzq6lrm9taddae7fpx59wm cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj
```

Example Output:

```bash
pagination: null
redelegation_responses:
- entries:
  - balance: "50000000"
    redelegation_entry:
      completion_time: "2021-10-24T20:33:21.960084845Z"
      creation_height: 2.382847e+06
      initial_balance: "50000000"
      shares_dst: "50000000.000000000000000000"
  - balance: "5000000000"
    redelegation_entry:
      completion_time: "2021-10-25T21:33:54.446846862Z"
      creation_height: 2.397271e+06
      initial_balance: "5000000000"
      shares_dst: "5000000000.000000000000000000"
  redelegation:
    delegator_address: cosmos1gghjut3ccd8ay0zduzj64hwre2fxs9ld75ru9p
    entries: null
    validator_dst_address: cosmosvaloper1l2rsakp388kuv9k8qzq6lrm9taddae7fpx59wm
    validator_src_address: cosmosvaloper1l2rsakp388kuv9k8qzq6lrm9taddae7fpx59wm
```

##### redelegations

The `redelegations` command allows users to query all redelegation records for an individual delegator.

Usage:

```bash
simd query staking redelegations [delegator-addr] [flags]
```

Example:

```bash
simd query staking redelegation cosmos1gghjut3ccd8ay0zduzj64hwre2fxs9ld75ru9p
```

Example Output:

```bash
pagination:
  next_key: null
  total: "0"
redelegation_responses:
- entries:
  - balance: "50000000"
    redelegation_entry:
      completion_time: "2021-10-24T20:33:21.960084845Z"
      creation_height: 2.382847e+06
      initial_balance: "50000000"
      shares_dst: "50000000.000000000000000000"
  - balance: "5000000000"
    redelegation_entry:
      completion_time: "2021-10-25T21:33:54.446846862Z"
      creation_height: 2.397271e+06
      initial_balance: "5000000000"
      shares_dst: "5000000000.000000000000000000"
  redelegation:
    delegator_address: cosmos1gghjut3ccd8ay0zduzj64hwre2fxs9ld75ru9p
    entries: null
    validator_dst_address: cosmosvaloper1uccl5ugxrm7vqlzwqr04pjd320d2fz0z3hc6vm
    validator_src_address: cosmosvaloper1zppjyal5emta5cquje8ndkpz0rs046m7zqxrpp
- entries:
  - balance: "562770000000"
    redelegation_entry:
      completion_time: "2021-10-25T21:42:07.336911677Z"
      creation_height: 2.39735e+06
      initial_balance: "562770000000"
      shares_dst: "562770000000.000000000000000000"
  redelegation:
    delegator_address: cosmos1gghjut3ccd8ay0zduzj64hwre2fxs9ld75ru9p
    entries: null
    validator_dst_address: cosmosvaloper1uccl5ugxrm7vqlzwqr04pjd320d2fz0z3hc6vm
    validator_src_address: cosmosvaloper1zppjyal5emta5cquje8ndkpz0rs046m7zqxrpp
```

##### redelegations-from

The `redelegations-from` command allows users to query delegations that are redelegating _from_ a validator.

Usage:

```bash
simd query staking redelegations-from [validator-addr] [flags]
```

Example:

```bash
simd query staking redelegations-from cosmosvaloper1y4rzzrgl66eyhzt6gse2k7ej3zgwmngeleucjy
```

Example Output:

```bash
pagination:
  next_key: null
  total: "0"
redelegation_responses:
- entries:
  - balance: "50000000"
    redelegation_entry:
      completion_time: "2021-10-24T20:33:21.960084845Z"
      creation_height: 2.382847e+06
      initial_balance: "50000000"
      shares_dst: "50000000.000000000000000000"
  - balance: "5000000000"
    redelegation_entry:
      completion_time: "2021-10-25T21:33:54.446846862Z"
      creation_height: 2.397271e+06
      initial_balance: "5000000000"
      shares_dst: "5000000000.000000000000000000"
  redelegation:
    delegator_address: cosmos1pm6e78p4pgn0da365plzl4t56pxy8hwtqp2mph
    entries: null
    validator_dst_address: cosmosvaloper1uccl5ugxrm7vqlzwqr04pjd320d2fz0z3hc6vm
    validator_src_address: cosmosvaloper1y4rzzrgl66eyhzt6gse2k7ej3zgwmngeleucjy
- entries:
  - balance: "221000000"
    redelegation_entry:
      completion_time: "2021-10-05T21:05:45.669420544Z"
      creation_height: 2.120693e+06
      initial_balance: "221000000"
      shares_dst: "221000000.000000000000000000"
  redelegation:
    delegator_address: cosmos1zqv8qxy2zgn4c58fz8jt8jmhs3d0attcussrf6
    entries: null
    validator_dst_address: cosmosvaloper10mseqwnwtjaqfrwwp2nyrruwmjp6u5jhah4c3y
    validator_src_address: cosmosvaloper1y4rzzrgl66eyhzt6gse2k7ej3zgwmngeleucjy
```

##### unbonding-delegation

The `unbonding-delegation` command allows users to query unbonding delegations for an individual delegator on an individual validator.

Usage:

```bash
simd query staking unbonding-delegation [delegator-addr] [validator-addr] [flags]
```

Example:

```bash
simd query staking unbonding-delegation cosmos1gghjut3ccd8ay0zduzj64hwre2fxs9ld75ru9p cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj
```

Example Output:

```bash
delegator_address: cosmos1gghjut3ccd8ay0zduzj64hwre2fxs9ld75ru9p
entries:
- balance: "52000000"
  completion_time: "2021-11-02T11:35:55.391594709Z"
  creation_height: "55078"
  initial_balance: "52000000"
validator_address: cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj
```

##### unbonding-delegations

The `unbonding-delegations` command allows users to query all unbonding-delegations records for one delegator.

Usage:

```bash
simd query staking unbonding-delegations [delegator-addr] [flags]
```

Example:

```bash
simd query staking unbonding-delegations cosmos1gghjut3ccd8ay0zduzj64hwre2fxs9ld75ru9p
```

Example Output:

```bash
pagination:
  next_key: null
  total: "0"
unbonding_responses:
- delegator_address: cosmos1gghjut3ccd8ay0zduzj64hwre2fxs9ld75ru9p
  entries:
  - balance: "52000000"
    completion_time: "2021-11-02T11:35:55.391594709Z"
    creation_height: "55078"
    initial_balance: "52000000"
  validator_address: cosmosvaloper1t8ehvswxjfn3ejzkjtntcyrqwvmvuknzmvtaaa

```

##### unbonding-delegations-from

The `unbonding-delegations-from` command allows users to query delegations that are unbonding _from_ a validator.

Usage:

```bash
simd query staking unbonding-delegations-from [validator-addr] [flags]
```

Example:

```bash
simd query staking unbonding-delegations-from cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj
```

Example Output:

```bash
pagination:
  next_key: null
  total: "0"
unbonding_responses:
- delegator_address: cosmos1qqq9txnw4c77sdvzx0tkedsafl5s3vk7hn53fn
  entries:
  - balance: "150000000"
    completion_time: "2021-11-01T21:41:13.098141574Z"
    creation_height: "46823"
    initial_balance: "150000000"
  validator_address: cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj
- delegator_address: cosmos1peteje73eklqau66mr7h7rmewmt2vt99y24f5z
  entries:
  - balance: "24000000"
    completion_time: "2021-10-31T02:57:18.192280361Z"
    creation_height: "21516"
    initial_balance: "24000000"
  validator_address: cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj
```

##### validator

The `validator` command allows users to query details about an individual validator.

Usage:

```bash
simd query staking validator [validator-addr] [flags]
```

Example:

```bash
simd query staking validator cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj
```

Example Output:

```bash
commission:
  commission_rates:
    max_change_rate: "0.020000000000000000"
    max_rate: "0.200000000000000000"
    rate: "0.050000000000000000"
  update_time: "2021-10-01T19:24:52.663191049Z"
consensus_pubkey:
  '@type': /cosmos.crypto.ed25519.PubKey
  key: sIiexdJdYWn27+7iUHQJDnkp63gq/rzUq1Y+fxoGjXc=
delegator_shares: "32948270000.000000000000000000"
description:
  details: Witval is the validator arm from Vitwit. Vitwit is into software consulting
    and services business since 2015. We are working closely with Cosmos ecosystem
    since 2018. We are also building tools for the ecosystem, Aneka is our explorer
    for the cosmos ecosystem.
  identity: 51468B615127273A
  moniker: Witval
  security_contact: ""
  website: ""
jailed: false
min_self_delegation: "1"
operator_address: cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj
status: BOND_STATUS_BONDED
tokens: "32948270000"
unbonding_height: "0"
unbonding_time: "1970-01-01T00:00:00Z"
```

##### validators

The `validators` command allows users to query details about all validators on a network.

Usage:

```bash
simd query staking validators [flags]
```

Example:

```bash
simd query staking validators
```

Example Output:

```bash
pagination:
  next_key: FPTi7TKAjN63QqZh+BaXn6gBmD5/
  total: "0"
validators:
commission:
  commission_rates:
    max_change_rate: "0.020000000000000000"
    max_rate: "0.200000000000000000"
    rate: "0.050000000000000000"
  update_time: "2021-10-01T19:24:52.663191049Z"
consensus_pubkey:
  '@type': /cosmos.crypto.ed25519.PubKey
  key: sIiexdJdYWn27+7iUHQJDnkp63gq/rzUq1Y+fxoGjXc=
delegator_shares: "32948270000.000000000000000000"
description:
    details: Witval is the validator arm from Vitwit. Vitwit is into software consulting
      and services business since 2015. We are working closely with Cosmos ecosystem
      since 2018. We are also building tools for the ecosystem, Aneka is our explorer
      for the cosmos ecosystem.
    identity: 51468B615127273A
    moniker: Witval
    security_contact: ""
    website: ""
  jailed: false
  min_self_delegation: "1"
  operator_address: cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj
  status: BOND_STATUS_BONDED
  tokens: "32948270000"
  unbonding_height: "0"
  unbonding_time: "1970-01-01T00:00:00Z"
- commission:
    commission_rates:
      max_change_rate: "0.100000000000000000"
      max_rate: "0.200000000000000000"
      rate: "0.050000000000000000"
    update_time: "2021-10-04T18:02:21.446645619Z"
  consensus_pubkey:
    '@type': /cosmos.crypto.ed25519.PubKey
    key: GDNpuKDmCg9GnhnsiU4fCWktuGUemjNfvpCZiqoRIYA=
  delegator_shares: "559343421.000000000000000000"
  description:
    details: Noderunners is a professional validator in POS networks. We have a huge
      node running experience, reliable soft and hardware. Our commissions are always
      low, our support to delegators is always full. Stake with us and start receiving
      your Cosmos rewards now!
    identity: 812E82D12FEA3493
    moniker: Noderunners
    security_contact: info@noderunners.biz
    website: http://noderunners.biz
  jailed: false
  min_self_delegation: "1"
  operator_address: cosmosvaloper1q5ku90atkhktze83j9xjaks2p7uruag5zp6wt7
  status: BOND_STATUS_BONDED
  tokens: "559343421"
  unbonding_height: "0"
  unbonding_time: "1970-01-01T00:00:00Z"
```

#### Transactions

The `tx` commands allows users to interact with the `staking` module.

```bash
simd tx staking --help
```

##### create-validator

The command `create-validator` allows users to create new validator initialized with a self-delegation to it.

Usage:

```bash
simd tx staking create-validator [path/to/validator.json] [flags]
```

Example:

```bash
simd tx staking create-validator /path/to/validator.json \
  --chain-id="name_of_chain_id" \
  --gas="auto" \
  --gas-adjustment="1.2" \
  --gas-prices="0.025stake" \
  --from=mykey
```

where `validator.json` contains:

```json
{
  "pubkey": {"@type":"/cosmos.crypto.ed25519.PubKey","key":"BnbwFpeONLqvWqJb3qaUbL5aoIcW3fSuAp9nT3z5f20="},
  "amount": "1000000stake",
  "moniker": "my-moniker",
  "website": "https://myweb.site",
  "security": "security-contact@gmail.com",
  "details": "description of your validator",
  "commission-rate": "0.10",
  "commission-max-rate": "0.20",
  "commission-max-change-rate": "0.01",
  "min-self-delegation": "1"
}
```

and pubkey can be obtained by using `simd tendermint show-validator` command.

##### delegate

The command `delegate` allows users to delegate liquid tokens to a validator.

Usage:

```bash
simd tx staking delegate [validator-addr] [amount] [flags]
```

Example:

```bash
simd tx staking delegate cosmosvaloper1l2rsakp388kuv9k8qzq6lrm9taddae7fpx59wm 1000stake --from mykey
```

##### edit-validator

The command `edit-validator` allows users to edit an existing validator account.

Usage:

```bash
simd tx staking edit-validator [flags]
```

Example:

```bash
simd tx staking edit-validator --moniker "new_moniker_name" --website "new_website_url" --from mykey
```

##### redelegate

The command `redelegate` allows users to redelegate illiquid tokens from one validator to another.

Usage:

```bash
simd tx staking redelegate [src-validator-addr] [dst-validator-addr] [amount] [flags]
```

Example:

```bash
simd tx staking redelegate cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj cosmosvaloper1l2rsakp388kuv9k8qzq6lrm9taddae7fpx59wm 100stake --from mykey
```

##### unbond

The command `unbond` allows users to unbond shares from a validator.

Usage:

```bash
simd tx staking unbond [validator-addr] [amount] [flags]
```

Example:

```bash
simd tx staking unbond cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj 100stake --from mykey
```

##### cancel unbond

The command `cancel-unbond` allow users to cancel the unbonding delegation entry and delegate back to the original validator.

Usage:

```bash
simd tx staking cancel-unbond [validator-addr] [amount] [creation-height]
```

Example:

```bash
simd tx staking cancel-unbond cosmosvaloper1gghjut3ccd8ay0zduzj64hwre2fxs9ldmqhffj 100stake 123123 --from mykey
```

##### rotate cons pubkey

The command `rotate-cons-pubkey` allows validators to rotate the associated consensus pubkey to the new consensus pubkey.

Usage:

```bash
simd tx staking rotate-cons-pubkey [validator-address] [new-pubkey] [flags]
```

Example:

```bash
simd tx staking rotate-cons-pubkey myvalidator {"@type":"/cosmos.crypto.ed25519.PubKey","key":"oWg2ISpLF405Jcm2vXV+2v4fnjodh6aafuIdeoW+rUw="}
```

### gRPC

A user can query the `staking` module using gRPC endpoints.

#### Validators

The `Validators` endpoint queries all validators that match the given status.

```bash
cosmos.staking.v1beta1.Query/Validators
```

Example:

```bash
grpcurl -plaintext localhost:9090 cosmos.staking.v1beta1.Query/Validators
```

Example Output:

```bash
{
  "validators": [
    {
      "operatorAddress": "cosmosvaloper1rne8lgs98p0jqe82sgt0qr4rdn4hgvmgp9ggcc",
      "consensusPubkey": {"@type":"/cosmos.crypto.ed25519.PubKey","key":"Auxs3865HpB/EfssYOzfqNhEJjzys2Fo6jD5B8tPgC8="},
      "status": "BOND_STATUS_BONDED",
      "tokens": "10000000",
      "delegatorShares": "10000000000000000000000000",
      "description": {
        "moniker": "myvalidator"
      },
      "unbondingTime": "1970-01-01T00:00:00Z",
      "commission": {
        "commissionRates": {
          "rate": "100000000000000000",
          "maxRate": "200000000000000000",
          "maxChangeRate": "10000000000000000"
        },
        "updateTime": "2021-10-01T05:52:50.380144238Z"
      },
      "minSelfDelegation": "1"
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```

#### Validator

The `Validator` endpoint queries validator information for given validator address.

```bash
cosmos.staking.v1beta1.Query/Validator
```

Example:

```bash
grpcurl -plaintext -d '{"validator_addr":"cosmosvaloper1rne8lgs98p0jqe82sgt0qr4rdn4hgvmgp9ggcc"}' \
localhost:9090 cosmos.staking.v1beta1.Query/Validator
```

Example Output:

```bash
{
  "validator": {
    "operatorAddress": "cosmosvaloper1rne8lgs98p0jqe82sgt0qr4rdn4hgvmgp9ggcc",
    "consensusPubkey": {"@type":"/cosmos.crypto.ed25519.PubKey","key":"Auxs3865HpB/EfssYOzfqNhEJjzys2Fo6jD5B8tPgC8="},
    "status": "BOND_STATUS_BONDED",
    "tokens": "10000000",
    "delegatorShares": "10000000000000000000000000",
    "description": {
      "moniker": "myvalidator"
    },
    "unbondingTime": "1970-01-01T00:00:00Z",
    "commission": {
      "commissionRates": {
        "rate": "100000000000000000",
        "maxRate": "200000000000000000",
        "maxChangeRate": "10000000000000000"
      },
      "updateTime": "2021-10-01T05:52:50.380144238Z"
    },
    "minSelfDelegation": "1"
  }
}
```

#### ValidatorDelegations

The `ValidatorDelegations` endpoint queries delegate information for given validator.

```bash
cosmos.staking.v1beta1.Query/ValidatorDelegations
```

Example:

```bash
grpcurl -plaintext -d '{"validator_addr":"cosmosvaloper1rne8lgs98p0jqe82sgt0qr4rdn4hgvmgp9ggcc"}' \
localhost:9090 cosmos.staking.v1beta1.Query/ValidatorDelegations
```

Example Output:

```bash
{
  "delegationResponses": [
    {
      "delegation": {
        "delegatorAddress": "cosmos1rne8lgs98p0jqe82sgt0qr4rdn4hgvmgy3ua5t",
        "validatorAddress": "cosmosvaloper1rne8lgs98p0jqe82sgt0qr4rdn4hgvmgp9ggcc",
        "shares": "10000000000000000000000000"
      },
      "balance": {
        "denom": "stake",
        "amount": "10000000"
      }
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```

#### ValidatorUnbondingDelegations

The `ValidatorUnbondingDelegations` endpoint queries delegate information for given validator.

```bash
cosmos.staking.v1beta1.Query/ValidatorUnbondingDelegations
```

Example:

```bash
grpcurl -plaintext -d '{"validator_addr":"cosmosvaloper1rne8lgs98p0jqe82sgt0qr4rdn4hgvmgp9ggcc"}' \
localhost:9090 cosmos.staking.v1beta1.Query/ValidatorUnbondingDelegations
```

Example Output:

```bash
{
  "unbonding_responses": [
    {
      "delegator_address": "cosmos1z3pzzw84d6xn00pw9dy3yapqypfde7vg6965fy",
      "validator_address": "cosmosvaloper1rne8lgs98p0jqe82sgt0qr4rdn4hgvmgp9ggcc",
      "entries": [
        {
          "creation_height": "25325",
          "completion_time": "2021-10-31T09:24:36.797320636Z",
          "initial_balance": "20000000",
          "balance": "20000000"
        }
      ]
    },
    {
      "delegator_address": "cosmos1y8nyfvmqh50p6ldpzljk3yrglppdv3t8phju77",
      "validator_address": "cosmosvaloper1rne8lgs98p0jqe82sgt0qr4rdn4hgvmgp9ggcc",
      "entries": [
        {
          "creation_height": "13100",
          "completion_time": "2021-10-30T12:53:02.272266791Z",
          "initial_balance": "1000000",
          "balance": "1000000"
        }
      ]
    },
  ],
  "pagination": {
    "next_key": null,
    "total": "8"
  }
}
```

#### Delegation

The `Delegation` endpoint queries delegate information for given validator delegator pair.

```bash
cosmos.staking.v1beta1.Query/Delegation
```

Example:

```bash
grpcurl -plaintext \
-d '{"delegator_addr": "cosmos1y8nyfvmqh50p6ldpzljk3yrglppdv3t8phju77", validator_addr":"cosmosvaloper1rne8lgs98p0jqe82sgt0qr4rdn4hgvmgp9ggcc"}' \
localhost:9090 cosmos.staking.v1beta1.Query/Delegation
```

Example Output:

```bash
{
  "delegation_response":
  {
    "delegation":
      {
        "delegator_address":"cosmos1y8nyfvmqh50p6ldpzljk3yrglppdv3t8phju77",
        "validator_address":"cosmosvaloper1rne8lgs98p0jqe82sgt0qr4rdn4hgvmgp9ggcc",
        "shares":"25083119936.000000000000000000"
      },
    "balance":
      {
        "denom":"stake",
        "amount":"25083119936"
      }
  }
}
```

#### UnbondingDelegation

The `UnbondingDelegation` endpoint queries unbonding information for given validator delegator.

```bash
cosmos.staking.v1beta1.Query/UnbondingDelegation
```

Example:

```bash
grpcurl -plaintext \
-d '{"delegator_addr": "cosmos1y8nyfvmqh50p6ldpzljk3yrglppdv3t8phju77", validator_addr":"cosmosvaloper1rne8lgs98p0jqe82sgt0qr4rdn4hgvmgp9ggcc"}' \
localhost:9090 cosmos.staking.v1beta1.Query/UnbondingDelegation
```

Example Output:

```bash
{
  "unbond": {
    "delegator_address": "cosmos1y8nyfvmqh50p6ldpzljk3yrglppdv3t8phju77",
    "validator_address": "cosmosvaloper1rne8lgs98p0jqe82sgt0qr4rdn4hgvmgp9ggcc",
    "entries": [
      {
        "creation_height": "136984",
        "completion_time": "2021-11-08T05:38:47.505593891Z",
        "initial_balance": "400000000",
        "balance": "400000000"
      },
      {
        "creation_height": "137005",
        "completion_time": "2021-11-08T05:40:53.526196312Z",
        "initial_balance": "385000000",
        "balance": "385000000"
      }
    ]
  }
}
```

#### DelegatorDelegations

The `DelegatorDelegations` endpoint queries all delegations of a given delegator address.

```bash
cosmos.staking.v1beta1.Query/DelegatorDelegations
```

Example:

```bash
grpcurl -plaintext \
-d '{"delegator_addr": "cosmos1y8nyfvmqh50p6ldpzljk3yrglppdv3t8phju77"}' \
localhost:9090 cosmos.staking.v1beta1.Query/DelegatorDelegations
```

Example Output:

```bash
{
  "delegation_responses": [
    {"delegation":{"delegator_address":"cosmos1y8nyfvmqh50p6ldpzljk3yrglppdv3t8phju77","validator_address":"cosmosvaloper1eh5mwu044gd5ntkkc2xgfg8247mgc56fww3vc8","shares":"25083339023.000000000000000000"},"balance":{"denom":"stake","amount":"25083339023"}}
  ],
  "pagination": {
    "next_key": null,
    "total": "1"
  }
}
```

#### DelegatorUnbondingDelegations

The `DelegatorUnbondingDelegations` endpoint queries all unbonding delegations of a given delegator address.

```bash
cosmos.staking.v1beta1.Query/DelegatorUnbondingDelegations
```

Example:

```bash
grpcurl -plaintext \
-d '{"delegator_addr": "cosmos1y8nyfvmqh50p6ldpzljk3yrglppdv3t8phju77"}' \
localhost:9090 cosmos.staking.v1beta1.Query/DelegatorUnbondingDelegations
```

Example Output:

```bash
{
  "unbonding_responses": [
    {
      "delegator_address": "cosmos1y8nyfvmqh50p6ldpzljk3yrglppdv3t8phju77",
      "validator_address": "cosmosvaloper1sjllsnramtg3ewxqwwrwjxfgc4n4ef9uxyejze",
      "entries": [
        {
          "creation_height": "136984",
          "completion_time": "2021-11-08T05:38:47.505593891Z",
          "initial_balance": "400000000",
          "balance": "400000000"
        },
        {
          "creation_height": "137005",
          "completion_time": "2021-11-08T05:40:53.526196312Z",
          "initial_balance": "385000000",
          "balance": "385000000"
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

#### Redelegations

The `Redelegations` endpoint queries redelegations of given address.

```bash
cosmos.staking.v1beta1.Query/Redelegations
```

Example:

```bash
grpcurl -plaintext \
-d '{"delegator_addr": "cosmos1ld5p7hn43yuh8ht28gm9pfjgj2fctujp2tgwvf", "src_validator_addr" : "cosmosvaloper1j7euyj85fv2jugejrktj540emh9353ltgppc3g", "dst_validator_addr" : "cosmosvaloper1yy3tnegzmkdcm7czzcy3flw5z0zyr9vkkxrfse"}' \
localhost:9090 cosmos.staking.v1beta1.Query/Redelegations
```

Example Output:

```bash
{
  "redelegation_responses": [
    {
      "redelegation": {
        "delegator_address": "cosmos1ld5p7hn43yuh8ht28gm9pfjgj2fctujp2tgwvf",
        "validator_src_address": "cosmosvaloper1j7euyj85fv2jugejrktj540emh9353ltgppc3g",
        "validator_dst_address": "cosmosvaloper1yy3tnegzmkdcm7czzcy3flw5z0zyr9vkkxrfse",
        "entries": null
      },
      "entries": [
        {
          "redelegation_entry": {
            "creation_height": 135932,
            "completion_time": "2021-11-08T03:52:55.299147901Z",
            "initial_balance": "2900000",
            "shares_dst": "2900000.000000000000000000"
          },
          "balance": "2900000"
        }
      ]
    }
  ],
  "pagination": null
}
```

#### DelegatorValidators

The `DelegatorValidators` endpoint queries all validators information for given delegator.

```bash
cosmos.staking.v1beta1.Query/DelegatorValidators
```

Example:

```bash
grpcurl -plaintext \
-d '{"delegator_addr": "cosmos1ld5p7hn43yuh8ht28gm9pfjgj2fctujp2tgwvf"}' \
localhost:9090 cosmos.staking.v1beta1.Query/DelegatorValidators
```

Example Output:

```bash
{
  "validators": [
    {
      "operator_address": "cosmosvaloper1eh5mwu044gd5ntkkc2xgfg8247mgc56fww3vc8",
      "consensus_pubkey": {
        "@type": "/cosmos.crypto.ed25519.PubKey",
        "key": "UPwHWxH1zHJWGOa/m6JB3f5YjHMvPQPkVbDqqi+U7Uw="
      },
      "jailed": false,
      "status": "BOND_STATUS_BONDED",
      "tokens": "347260647559",
      "delegator_shares": "347260647559.000000000000000000",
      "description": {
        "moniker": "BouBouNode",
        "identity": "",
        "website": "https://boubounode.com",
        "security_contact": "",
        "details": "AI-based Validator. #1 AI Validator on Game of Stakes. Fairly priced. Don't trust (humans), verify. Made with BouBou love."
      },
      "unbonding_height": "0",
      "unbonding_time": "1970-01-01T00:00:00Z",
      "commission": {
        "commission_rates": {
          "rate": "0.061000000000000000",
          "max_rate": "0.300000000000000000",
          "max_change_rate": "0.150000000000000000"
        },
        "update_time": "2021-10-01T15:00:00Z"
      },
      "min_self_delegation": "1"
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "1"
  }
}
```

#### DelegatorValidator

The `DelegatorValidator` endpoint queries validator information for given delegator validator

```bash
cosmos.staking.v1beta1.Query/DelegatorValidator
```

Example:

```bash
grpcurl -plaintext \
-d '{"delegator_addr": "cosmos1eh5mwu044gd5ntkkc2xgfg8247mgc56f3n8rr7", "validator_addr": "cosmosvaloper1eh5mwu044gd5ntkkc2xgfg8247mgc56fww3vc8"}' \
localhost:9090 cosmos.staking.v1beta1.Query/DelegatorValidator
```

Example Output:

```bash
{
  "validator": {
    "operator_address": "cosmosvaloper1eh5mwu044gd5ntkkc2xgfg8247mgc56fww3vc8",
    "consensus_pubkey": {
      "@type": "/cosmos.crypto.ed25519.PubKey",
      "key": "UPwHWxH1zHJWGOa/m6JB3f5YjHMvPQPkVbDqqi+U7Uw="
    },
    "jailed": false,
    "status": "BOND_STATUS_BONDED",
    "tokens": "347262754841",
    "delegator_shares": "347262754841.000000000000000000",
    "description": {
      "moniker": "BouBouNode",
      "identity": "",
      "website": "https://boubounode.com",
      "security_contact": "",
      "details": "AI-based Validator. #1 AI Validator on Game of Stakes. Fairly priced. Don't trust (humans), verify. Made with BouBou love."
    },
    "unbonding_height": "0",
    "unbonding_time": "1970-01-01T00:00:00Z",
    "commission": {
      "commission_rates": {
        "rate": "0.061000000000000000",
        "max_rate": "0.300000000000000000",
        "max_change_rate": "0.150000000000000000"
      },
      "update_time": "2021-10-01T15:00:00Z"
    },
    "min_self_delegation": "1"
  }
}
```

#### Pool

The `Pool` endpoint queries the pool information.

```bash
cosmos.staking.v1beta1.Query/Pool
```

Example:

```bash
grpcurl -plaintext -d localhost:9090 cosmos.staking.v1beta1.Query/Pool
```

Example Output:

```bash
{
  "pool": {
    "not_bonded_tokens": "369054400189",
    "bonded_tokens": "15657192425623"
  }
}
```

#### Params

The `Params` endpoint queries the pool information.

```bash
cosmos.staking.v1beta1.Query/Params
```

Example:

```bash
grpcurl -plaintext localhost:9090 cosmos.staking.v1beta1.Query/Params
```

Example Output:

```bash
{
  "params": {
    "unbondingTime": "1814400s",
    "maxValidators": 100,
    "maxEntries": 7,
    "historicalEntries": 10000,
    "bondDenom": "stake"
  }
}
```

### REST

A user can query the `staking` module using REST endpoints.

#### DelegatorDelegations

The `DelegtaorDelegations` REST endpoint queries all delegations of a given delegator address.

```bash
/cosmos/staking/v1beta1/delegations/{delegatorAddr}
```

Example:

```bash
curl -X GET "http://localhost:1317/cosmos/staking/v1beta1/delegations/cosmos1vcs68xf2tnqes5tg0khr0vyevm40ff6zdxatp5" -H  "accept: application/json"
```

Example Output:

```bash
{
  "delegation_responses": [
    {
      "delegation": {
        "delegator_address": "cosmos1vcs68xf2tnqes5tg0khr0vyevm40ff6zdxatp5",
        "validator_address": "cosmosvaloper1quqxfrxkycr0uzt4yk0d57tcq3zk7srm7sm6r8",
        "shares": "256250000.000000000000000000"
      },
      "balance": {
        "denom": "stake",
        "amount": "256250000"
      }
    },
    {
      "delegation": {
        "delegator_address": "cosmos1vcs68xf2tnqes5tg0khr0vyevm40ff6zdxatp5",
        "validator_address": "cosmosvaloper194v8uwee2fvs2s8fa5k7j03ktwc87h5ym39jfv",
        "shares": "255150000.000000000000000000"
      },
      "balance": {
        "denom": "stake",
        "amount": "255150000"
      }
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "2"
  }
}
```

#### Redelegations

The `Redelegations` REST endpoint queries redelegations of given address.

```bash
/cosmos/staking/v1beta1/delegators/{delegatorAddr}/redelegations
```

Example:

```bash
curl -X GET \
"http://localhost:1317/cosmos/staking/v1beta1/delegators/cosmos1thfntksw0d35n2tkr0k8v54fr8wxtxwxl2c56e/redelegations?srcValidatorAddr=cosmosvaloper1lzhlnpahvznwfv4jmay2tgaha5kmz5qx4cuznf&dstValidatorAddr=cosmosvaloper1vq8tw77kp8lvxq9u3c8eeln9zymn68rng8pgt4" \
-H  "accept: application/json"
```

Example Output:

```bash
{
  "redelegation_responses": [
    {
      "redelegation": {
        "delegator_address": "cosmos1thfntksw0d35n2tkr0k8v54fr8wxtxwxl2c56e",
        "validator_src_address": "cosmosvaloper1lzhlnpahvznwfv4jmay2tgaha5kmz5qx4cuznf",
        "validator_dst_address": "cosmosvaloper1vq8tw77kp8lvxq9u3c8eeln9zymn68rng8pgt4",
        "entries": null
      },
      "entries": [
        {
          "redelegation_entry": {
            "creation_height": 151523,
            "completion_time": "2021-11-09T06:03:25.640682116Z",
            "initial_balance": "200000000",
            "shares_dst": "200000000.000000000000000000"
          },
          "balance": "200000000"
        }
      ]
    }
  ],
  "pagination": null
}
```

#### DelegatorUnbondingDelegations

The `DelegatorUnbondingDelegations` REST endpoint queries all unbonding delegations of a given delegator address.

```bash
/cosmos/staking/v1beta1/delegators/{delegatorAddr}/unbonding_delegations
```

Example:

```bash
curl -X GET \
"http://localhost:1317/cosmos/staking/v1beta1/delegators/cosmos1nxv42u3lv642q0fuzu2qmrku27zgut3n3z7lll/unbonding_delegations" \
-H  "accept: application/json"
```

Example Output:

```bash
{
  "unbonding_responses": [
    {
      "delegator_address": "cosmos1nxv42u3lv642q0fuzu2qmrku27zgut3n3z7lll",
      "validator_address": "cosmosvaloper1e7mvqlz50ch6gw4yjfemsc069wfre4qwmw53kq",
      "entries": [
        {
          "creation_height": "2442278",
          "completion_time": "2021-10-12T10:59:03.797335857Z",
          "initial_balance": "50000000000",
          "balance": "50000000000"
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

#### DelegatorValidators

The `DelegatorValidators` REST endpoint queries all validators information for given delegator address.

```bash
/cosmos/staking/v1beta1/delegators/{delegatorAddr}/validators
```

Example:

```bash
curl -X GET \
"http://localhost:1317/cosmos/staking/v1beta1/delegators/cosmos1xwazl8ftks4gn00y5x3c47auquc62ssune9ppv/validators" \
-H  "accept: application/json"
```

Example Output:

```bash
{
  "validators": [
    {
      "operator_address": "cosmosvaloper1xwazl8ftks4gn00y5x3c47auquc62ssuvynw64",
      "consensus_pubkey": {
        "@type": "/cosmos.crypto.ed25519.PubKey",
        "key": "5v4n3px3PkfNnKflSgepDnsMQR1hiNXnqOC11Y72/PQ="
      },
      "jailed": false,
      "status": "BOND_STATUS_BONDED",
      "tokens": "21592843799",
      "delegator_shares": "21592843799.000000000000000000",
      "description": {
        "moniker": "jabbey",
        "identity": "",
        "website": "https://twitter.com/JoeAbbey",
        "security_contact": "",
        "details": "just another dad in the cosmos"
      },
      "unbonding_height": "0",
      "unbonding_time": "1970-01-01T00:00:00Z",
      "commission": {
        "commission_rates": {
          "rate": "0.100000000000000000",
          "max_rate": "0.200000000000000000",
          "max_change_rate": "0.100000000000000000"
        },
        "update_time": "2021-10-09T19:03:54.984821705Z"
      },
      "min_self_delegation": "1"
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "1"
  }
}
```

#### DelegatorValidator

The `DelegatorValidator` REST endpoint queries validator information for given delegator validator pair.

```bash
/cosmos/staking/v1beta1/delegators/{delegatorAddr}/validators/{validatorAddr}
```

Example:

```bash
curl -X GET \
"http://localhost:1317/cosmos/staking/v1beta1/delegators/cosmos1xwazl8ftks4gn00y5x3c47auquc62ssune9ppv/validators/cosmosvaloper1xwazl8ftks4gn00y5x3c47auquc62ssuvynw64" \
-H  "accept: application/json"
```

Example Output:

```bash
{
  "validator": {
    "operator_address": "cosmosvaloper1xwazl8ftks4gn00y5x3c47auquc62ssuvynw64",
    "consensus_pubkey": {
      "@type": "/cosmos.crypto.ed25519.PubKey",
      "key": "5v4n3px3PkfNnKflSgepDnsMQR1hiNXnqOC11Y72/PQ="
    },
    "jailed": false,
    "status": "BOND_STATUS_BONDED",
    "tokens": "21592843799",
    "delegator_shares": "21592843799.000000000000000000",
    "description": {
      "moniker": "jabbey",
      "identity": "",
      "website": "https://twitter.com/JoeAbbey",
      "security_contact": "",
      "details": "just another dad in the cosmos"
    },
    "unbonding_height": "0",
    "unbonding_time": "1970-01-01T00:00:00Z",
    "commission": {
      "commission_rates": {
        "rate": "0.100000000000000000",
        "max_rate": "0.200000000000000000",
        "max_change_rate": "0.100000000000000000"
      },
      "update_time": "2021-10-09T19:03:54.984821705Z"
    },
    "min_self_delegation": "1"
  }
}
```

#### Parameters

The `Parameters` REST endpoint queries the staking parameters.

```bash
/cosmos/staking/v1beta1/params
```

Example:

```bash
curl -X GET "http://localhost:1317/cosmos/staking/v1beta1/params" -H  "accept: application/json"
```

Example Output:

```bash
{
  "params": {
    "unbonding_time": "2419200s",
    "max_validators": 100,
    "max_entries": 7,
    "historical_entries": 10000,
    "bond_denom": "stake"
  }
}
```

#### Pool

The `Pool` REST endpoint queries the pool information.

```bash
/cosmos/staking/v1beta1/pool
```

Example:

```bash
curl -X GET "http://localhost:1317/cosmos/staking/v1beta1/pool" -H  "accept: application/json"
```

Example Output:

```bash
{
  "pool": {
    "not_bonded_tokens": "432805737458",
    "bonded_tokens": "15783637712645"
  }
}
```

#### Validators

The `Validators` REST endpoint queries all validators that match the given status.

```bash
/cosmos/staking/v1beta1/validators
```

Example:

```bash
curl -X GET "http://localhost:1317/cosmos/staking/v1beta1/validators" -H  "accept: application/json"
```

Example Output:

```bash
{
  "validators": [
    {
      "operator_address": "cosmosvaloper1q3jsx9dpfhtyqqgetwpe5tmk8f0ms5qywje8tw",
      "consensus_pubkey": {
        "@type": "/cosmos.crypto.ed25519.PubKey",
        "key": "N7BPyek2aKuNZ0N/8YsrqSDhGZmgVaYUBuddY8pwKaE="
      },
      "jailed": false,
      "status": "BOND_STATUS_BONDED",
      "tokens": "383301887799",
      "delegator_shares": "383301887799.000000000000000000",
      "description": {
        "moniker": "SmartNodes",
        "identity": "D372724899D1EDC8",
        "website": "https://smartnodes.co",
        "security_contact": "",
        "details": "Earn Rewards with Crypto Staking & Node Deployment"
      },
      "unbonding_height": "0",
      "unbonding_time": "1970-01-01T00:00:00Z",
      "commission": {
        "commission_rates": {
          "rate": "0.050000000000000000",
          "max_rate": "0.200000000000000000",
          "max_change_rate": "0.100000000000000000"
        },
        "update_time": "2021-10-01T15:51:31.596618510Z"
      },
      "min_self_delegation": "1"
    },
    {
      "operator_address": "cosmosvaloper1q5ku90atkhktze83j9xjaks2p7uruag5zp6wt7",
      "consensus_pubkey": {
        "@type": "/cosmos.crypto.ed25519.PubKey",
        "key": "GDNpuKDmCg9GnhnsiU4fCWktuGUemjNfvpCZiqoRIYA="
      },
      "jailed": false,
      "status": "BOND_STATUS_UNBONDING",
      "tokens": "1017819654",
      "delegator_shares": "1017819654.000000000000000000",
      "description": {
        "moniker": "Noderunners",
        "identity": "812E82D12FEA3493",
        "website": "http://noderunners.biz",
        "security_contact": "info@noderunners.biz",
        "details": "Noderunners is a professional validator in POS networks. We have a huge node running experience, reliable soft and hardware. Our commissions are always low, our support to delegators is always full. Stake with us and start receiving your cosmos rewards now!"
      },
      "unbonding_height": "147302",
      "unbonding_time": "2021-11-08T22:58:53.718662452Z",
      "commission": {
        "commission_rates": {
          "rate": "0.050000000000000000",
          "max_rate": "0.200000000000000000",
          "max_change_rate": "0.100000000000000000"
        },
        "update_time": "2021-10-04T18:02:21.446645619Z"
      },
      "min_self_delegation": "1"
    }
  ],
  "pagination": {
    "next_key": "FONDBFkE4tEEf7yxWWKOD49jC2NK",
    "total": "2"
  }
}
```

#### Validator

The `Validator` REST endpoint queries validator information for given validator address.

```bash
/cosmos/staking/v1beta1/validators/{validatorAddr}
```

Example:

```bash
curl -X GET \
"http://localhost:1317/cosmos/staking/v1beta1/validators/cosmosvaloper16msryt3fqlxtvsy8u5ay7wv2p8mglfg9g70e3q" \
-H  "accept: application/json"
```

Example Output:

```bash
{
  "validator": {
    "operator_address": "cosmosvaloper16msryt3fqlxtvsy8u5ay7wv2p8mglfg9g70e3q",
    "consensus_pubkey": {
      "@type": "/cosmos.crypto.ed25519.PubKey",
      "key": "sIiexdJdYWn27+7iUHQJDnkp63gq/rzUq1Y+fxoGjXc="
    },
    "jailed": false,
    "status": "BOND_STATUS_BONDED",
    "tokens": "33027900000",
    "delegator_shares": "33027900000.000000000000000000",
    "description": {
      "moniker": "Witval",
      "identity": "51468B615127273A",
      "website": "",
      "security_contact": "",
      "details": "Witval is the validator arm from Vitwit. Vitwit is into software consulting and services business since 2015. We are working closely with Cosmos ecosystem since 2018. We are also building tools for the ecosystem, Aneka is our explorer for the cosmos ecosystem."
    },
    "unbonding_height": "0",
    "unbonding_time": "1970-01-01T00:00:00Z",
    "commission": {
      "commission_rates": {
        "rate": "0.050000000000000000",
        "max_rate": "0.200000000000000000",
        "max_change_rate": "0.020000000000000000"
      },
      "update_time": "2021-10-01T19:24:52.663191049Z"
    },
    "min_self_delegation": "1"
  }
}
```

#### ValidatorDelegations

The `ValidatorDelegations` REST endpoint queries delegate information for given validator.

```bash
/cosmos/staking/v1beta1/validators/{validatorAddr}/delegations
```

Example:

```bash
curl -X GET "http://localhost:1317/cosmos/staking/v1beta1/validators/cosmosvaloper16msryt3fqlxtvsy8u5ay7wv2p8mglfg9g70e3q/delegations" -H  "accept: application/json"
```

Example Output:

```bash
{
  "delegation_responses": [
    {
      "delegation": {
        "delegator_address": "cosmos190g5j8aszqhvtg7cprmev8xcxs6csra7xnk3n3",
        "validator_address": "cosmosvaloper16msryt3fqlxtvsy8u5ay7wv2p8mglfg9g70e3q",
        "shares": "31000000000.000000000000000000"
      },
      "balance": {
        "denom": "stake",
        "amount": "31000000000"
      }
    },
    {
      "delegation": {
        "delegator_address": "cosmos1ddle9tczl87gsvmeva3c48nenyng4n56qwq4ee",
        "validator_address": "cosmosvaloper16msryt3fqlxtvsy8u5ay7wv2p8mglfg9g70e3q",
        "shares": "628470000.000000000000000000"
      },
      "balance": {
        "denom": "stake",
        "amount": "628470000"
      }
    },
    {
      "delegation": {
        "delegator_address": "cosmos10fdvkczl76m040smd33lh9xn9j0cf26kk4s2nw",
        "validator_address": "cosmosvaloper16msryt3fqlxtvsy8u5ay7wv2p8mglfg9g70e3q",
        "shares": "838120000.000000000000000000"
      },
      "balance": {
        "denom": "stake",
        "amount": "838120000"
      }
    },
    {
      "delegation": {
        "delegator_address": "cosmos1n8f5fknsv2yt7a8u6nrx30zqy7lu9jfm0t5lq8",
        "validator_address": "cosmosvaloper16msryt3fqlxtvsy8u5ay7wv2p8mglfg9g70e3q",
        "shares": "500000000.000000000000000000"
      },
      "balance": {
        "denom": "stake",
        "amount": "500000000"
      }
    },
    {
      "delegation": {
        "delegator_address": "cosmos16msryt3fqlxtvsy8u5ay7wv2p8mglfg9hrek2e",
        "validator_address": "cosmosvaloper16msryt3fqlxtvsy8u5ay7wv2p8mglfg9g70e3q",
        "shares": "61310000.000000000000000000"
      },
      "balance": {
        "denom": "stake",
        "amount": "61310000"
      }
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "5"
  }
}
```

#### Delegation

The `Delegation` REST endpoint queries delegate information for given validator delegator pair.

```bash
/cosmos/staking/v1beta1/validators/{validatorAddr}/delegations/{delegatorAddr}
```

Example:

```bash
curl -X GET \
"http://localhost:1317/cosmos/staking/v1beta1/validators/cosmosvaloper16msryt3fqlxtvsy8u5ay7wv2p8mglfg9g70e3q/delegations/cosmos1n8f5fknsv2yt7a8u6nrx30zqy7lu9jfm0t5lq8" \
-H  "accept: application/json"
```

Example Output:

```bash
{
  "delegation_response": {
    "delegation": {
      "delegator_address": "cosmos1n8f5fknsv2yt7a8u6nrx30zqy7lu9jfm0t5lq8",
      "validator_address": "cosmosvaloper16msryt3fqlxtvsy8u5ay7wv2p8mglfg9g70e3q",
      "shares": "500000000.000000000000000000"
    },
    "balance": {
      "denom": "stake",
      "amount": "500000000"
    }
  }
}
```

#### UnbondingDelegation

The `UnbondingDelegation` REST endpoint queries unbonding information for given validator delegator pair.

```bash
/cosmos/staking/v1beta1/validators/{validatorAddr}/delegations/{delegatorAddr}/unbonding_delegation
```

Example:

```bash
curl -X GET \
"http://localhost:1317/cosmos/staking/v1beta1/validators/cosmosvaloper13v4spsah85ps4vtrw07vzea37gq5la5gktlkeu/delegations/cosmos1ze2ye5u5k3qdlexvt2e0nn0508p04094ya0qpm/unbonding_delegation" \
-H  "accept: application/json"
```

Example Output:

```bash
{
  "unbond": {
    "delegator_address": "cosmos1ze2ye5u5k3qdlexvt2e0nn0508p04094ya0qpm",
    "validator_address": "cosmosvaloper13v4spsah85ps4vtrw07vzea37gq5la5gktlkeu",
    "entries": [
      {
        "creation_height": "153687",
        "completion_time": "2021-11-09T09:41:18.352401903Z",
        "initial_balance": "525111",
        "balance": "525111"
      }
    ]
  }
}
```

#### ValidatorUnbondingDelegations

The `ValidatorUnbondingDelegations` REST endpoint queries unbonding delegations of a validator.

```bash
/cosmos/staking/v1beta1/validators/{validatorAddr}/unbonding_delegations
```

Example:

```bash
curl -X GET \
"http://localhost:1317/cosmos/staking/v1beta1/validators/cosmosvaloper13v4spsah85ps4vtrw07vzea37gq5la5gktlkeu/unbonding_delegations" \
-H  "accept: application/json"
```

Example Output:

```bash
{
  "unbonding_responses": [
    {
      "delegator_address": "cosmos1q9snn84jfrd9ge8t46kdcggpe58dua82vnj7uy",
      "validator_address": "cosmosvaloper13v4spsah85ps4vtrw07vzea37gq5la5gktlkeu",
      "entries": [
        {
          "creation_height": "90998",
          "completion_time": "2021-11-05T00:14:37.005841058Z",
          "initial_balance": "24000000",
          "balance": "24000000"
        }
      ]
    },
    {
      "delegator_address": "cosmos1qf36e6wmq9h4twhdvs6pyq9qcaeu7ye0s3dqq2",
      "validator_address": "cosmosvaloper13v4spsah85ps4vtrw07vzea37gq5la5gktlkeu",
      "entries": [
        {
          "creation_height": "47478",
          "completion_time": "2021-11-01T22:47:26.714116854Z",
          "initial_balance": "8000000",
          "balance": "8000000"
        }
      ]
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "2"
  }
}
```
