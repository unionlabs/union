---
sidebar_position: 1
---

# `x/slashing`

## Abstract

This section specifies the slashing module of the Cosmos SDK, which implements functionality
first outlined in the [Cosmos Whitepaper](https://cosmos.network/about/whitepaper) in June 2016.

The slashing module enables Cosmos SDK-based blockchains to disincentivize any attributable action
by a protocol-recognized actor with value at stake by penalizing them ("slashing").

Penalties may include, but are not limited to:

* Burning some amount of their stake
* Removing their ability to vote on future blocks for a period of time.

This module will be used by the Cosmos Hub, the first hub in the Cosmos ecosystem.

## Contents

* [Concepts](#concepts)
    * [States](#states)
    * [Tombstone Caps](#tombstone-caps)
    * [Infraction Timelines](#infraction-timelines)
* [State](#state)
    * [Signing Info (Liveness)](#signing-info-liveness)
    * [Params](#params)
* [Messages](#messages)
    * [Unjail](#unjail)
* [BeginBlock](#beginblock)
    * [Liveness Tracking](#liveness-tracking)
* [Hooks](#hooks)
* [Events](#events)
* [Staking Tombstone](#staking-tombstone)
* [Parameters](#parameters)
* [CLI](#cli)
    * [Query](#query)
    * [Transactions](#transactions)
    * [gRPC](#grpc)
    * [REST](#rest)

## Concepts

### States

At any given time, there are any number of validators registered in the state
machine. Each block, the top `MaxValidators` (defined by `x/staking`) validators
who are not jailed become _bonded_, meaning that they may propose and vote on
blocks. Validators who are _bonded_ are _at stake_, meaning that part or all of
their stake and their delegators' stake is at risk if they commit a protocol fault.

For each of these validators we keep a `ValidatorSigningInfo` record that contains
information partaining to validator's liveness and other infraction related
attributes.

### Tombstone Caps

In order to mitigate the impact of initially likely categories of non-malicious
protocol faults, the Cosmos Hub implements for each validator
a _tombstone_ cap, which only allows a validator to be slashed once for a double
sign fault. For example, if you misconfigure your HSM and double-sign a bunch of
old blocks, you'll only be punished for the first double-sign (and then immediately tombstombed). This will still be quite expensive and desirable to avoid, but tombstone caps
somewhat blunt the economic impact of unintentional misconfiguration.

Liveness faults do not have caps, as they can't stack upon each other. Liveness bugs are "detected" as soon as the infraction occurs, and the validators are immediately put in jail, so it is not possible for them to commit multiple liveness faults without unjailing in between.

### Infraction Timelines

To illustrate how the `x/slashing` module handles submitted evidence through
CometBFT consensus, consider the following examples:

**Definitions**:

_[_ : timeline start  
_]_ : timeline end  
_C<sub>n</sub>_ : infraction `n` committed  
_D<sub>n</sub>_ : infraction `n` discovered  
_V<sub>b</sub>_ : validator bonded  
_V<sub>u</sub>_ : validator unbonded

#### Single Double Sign Infraction

\[----------C<sub>1</sub>----D<sub>1</sub>,V<sub>u</sub>-----\]

A single infraction is committed then later discovered, at which point the
validator is unbonded and slashed at the full amount for the infraction.

#### Multiple Double Sign Infractions

\[----------C<sub>1</sub>--C<sub>2</sub>---C<sub>3</sub>---D<sub>1</sub>,D<sub>2</sub>,D<sub>3</sub>V<sub>u</sub>-----\]

Multiple infractions are committed and then later discovered, at which point the
validator is jailed and slashed for only one infraction. Because the validator
is also tombstoned, they can not rejoin the validator set.

## State

### Signing Info (Liveness)

Every block includes a set of precommits by the validators for the previous block,
known as the `LastCommitInfo` provided by CometBFT. A `LastCommitInfo` is valid so
long as it contains precommits from +2/3 of total voting power.

Proposers are incentivized to include precommits from all validators in the CometBFT `LastCommitInfo`
by receiving additional fees proportional to the difference between the voting
power included in the `LastCommitInfo` and +2/3 (see [fee distribution](../distribution/README.md#begin-block)).

```go
type LastCommitInfo struct {
	Round int32
	Votes []VoteInfo
}
```

Validators are penalized for failing to be included in the `LastCommitInfo` for some
number of blocks by being automatically jailed, potentially slashed, and unbonded.

Information about validator's liveness activity is tracked through `ValidatorSigningInfo`.
It is indexed in the store as follows:

* ValidatorSigningInfo: `0x01 | ConsAddrLen (1 byte) | ConsAddress -> ProtocolBuffer(ValSigningInfo)`
* MissedBlocksBitArray: `0x02 | ConsAddrLen (1 byte) | ConsAddress | LittleEndianUint64(signArrayIndex) -> VarInt(didMiss)` (varint is a number encoding format)

The first mapping allows us to easily lookup the recent signing info for a
validator based on the validator's consensus address.

The second mapping (`MissedBlocksBitArray`) acts
as a bit-array of size `SignedBlocksWindow` that tells us if the validator missed
the block for a given index in the bit-array. The index in the bit-array is given
as little endian uint64.
The result is a `varint` that takes on `0` or `1`, where `0` indicates the
validator did not miss (did sign) the corresponding block, and `1` indicates
they missed the block (did not sign).

Note that the `MissedBlocksBitArray` is not explicitly initialized up-front. Keys
are added as we progress through the first `SignedBlocksWindow` blocks for a newly
bonded validator. The `SignedBlocksWindow` parameter defines the size
(number of blocks) of the sliding window used to track validator liveness.

The information stored for tracking validator liveness is as follows:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/slashing/v1beta1/slashing.proto#L13-L35
```

### Params

The slashing module stores it's params in state with the prefix of `0x00`,
it can be updated with governance or the address with authority.

* Params: `0x00 | ProtocolBuffer(Params)`

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/slashing/v1beta1/slashing.proto#L37-L59
```

## Messages

In this section we describe the processing of messages for the `slashing` module.

### Unjail

If a validator was automatically unbonded due to downtime and wishes to come back online &
possibly rejoin the bonded set, it must send `MsgUnjail`:

```protobuf
// MsgUnjail is an sdk.Msg used for unjailing a jailed validator, thus returning
// them into the bonded validator set, so they can begin receiving provisions
// and rewards again.
message MsgUnjail {
  string validator_addr = 1;
}
```

Below is a pseudocode of the `MsgSrv/Unjail` RPC:

```go
unjail(tx MsgUnjail)
    validator = getValidator(tx.ValidatorAddr)
    if validator == nil
      fail with "No validator found"

    if getSelfDelegation(validator) == 0
      fail with "validator must self delegate before unjailing"

    if !validator.Jailed
      fail with "Validator not jailed, cannot unjail"

    info = GetValidatorSigningInfo(operator)
    if info.Tombstoned
      fail with "Tombstoned validator cannot be unjailed"
    if block time < info.JailedUntil
      fail with "Validator still jailed, cannot unjail until period has expired"

    validator.Jailed = false
    setValidator(validator)

    return
```

If the validator has enough stake to be in the top `n = MaximumBondedValidators`, it will be automatically rebonded,
and all delegators still delegated to the validator will be rebonded and begin to again collect
provisions and rewards.

## BeginBlock

### Liveness Tracking

At the beginning of each block, we update the `ValidatorSigningInfo` for each
validator and check if they've crossed below the liveness threshold over a
sliding window. This sliding window is defined by `SignedBlocksWindow` and the
index in this window is determined by `IndexOffset` found in the validator's
`ValidatorSigningInfo`. For each block processed, the `IndexOffset` is incremented
regardless if the validator signed or not. Once the index is determined, the
`MissedBlocksBitArray` and `MissedBlocksCounter` are updated accordingly.

Finally, in order to determine if a validator crosses below the liveness threshold,
we fetch the maximum number of blocks missed, `maxMissed`, which is
`SignedBlocksWindow - (MinSignedPerWindow * SignedBlocksWindow)` and the minimum
height at which we can determine liveness, `minHeight`. If the current block is
greater than `minHeight` and the validator's `MissedBlocksCounter` is greater than
`maxMissed`, they will be slashed by `SlashFractionDowntime`, will be jailed
for `DowntimeJailDuration`, and have the following values reset:
`MissedBlocksBitArray`, `MissedBlocksCounter`, and `IndexOffset`.

**Note**: Liveness slashes do **NOT** lead to a tombstombing.

```go
height := block.Height

for vote in block.LastCommitInfo.Votes {
  signInfo := GetValidatorSigningInfo(vote.Validator.Address)

  // This is a relative index, so we counts blocks the validator SHOULD have
  // signed. We use the 0-value default signing info if not present, except for
  // start height.
  index := signInfo.IndexOffset % SignedBlocksWindow()
  signInfo.IndexOffset++

  // Update MissedBlocksBitArray and MissedBlocksCounter. The MissedBlocksCounter
  // just tracks the sum of MissedBlocksBitArray. That way we avoid needing to
  // read/write the whole array each time.
  missedPrevious := GetValidatorMissedBlockBitArray(vote.Validator.Address, index)
  missed := !signed

  switch {
  case !missedPrevious && missed:
    // array index has changed from not missed to missed, increment counter
    SetValidatorMissedBlockBitArray(vote.Validator.Address, index, true)
    signInfo.MissedBlocksCounter++

  case missedPrevious && !missed:
    // array index has changed from missed to not missed, decrement counter
    SetValidatorMissedBlockBitArray(vote.Validator.Address, index, false)
    signInfo.MissedBlocksCounter--

  default:
    // array index at this index has not changed; no need to update counter
  }

  if missed {
    // emit events...
  }

  minHeight := signInfo.StartHeight + SignedBlocksWindow()
  maxMissed := SignedBlocksWindow() - MinSignedPerWindow()

  // If we are past the minimum height and the validator has missed too many
  // jail and slash them.
  if height > minHeight && signInfo.MissedBlocksCounter > maxMissed {
    validator := ValidatorByConsAddr(vote.Validator.Address)

    // emit events...

    // We need to retrieve the stake distribution which signed the block, so we
    // subtract ValidatorUpdateDelay from the block height, and subtract an
    // additional 1 since this is the LastCommit.
    //
    // Note, that this CAN result in a negative "distributionHeight" up to
    // -ValidatorUpdateDelay-1, i.e. at the end of the pre-genesis block (none) = at the beginning of the genesis block.
    // That's fine since this is just used to filter unbonding delegations & redelegations.
    distributionHeight := height - sdk.ValidatorUpdateDelay - 1

    SlashWithInfractionReason(vote.Validator.Address, distributionHeight, vote.Validator.Power, SlashFractionDowntime(), stakingtypes.Downtime)
    Jail(vote.Validator.Address)

    signInfo.JailedUntil = block.Time.Add(DowntimeJailDuration())

    // We need to reset the counter & array so that the validator won't be
    // immediately slashed for downtime upon rebonding.
    signInfo.MissedBlocksCounter = 0
    signInfo.IndexOffset = 0
    ClearValidatorMissedBlockBitArray(vote.Validator.Address)
  }

  SetValidatorSigningInfo(vote.Validator.Address, signInfo)
}
```

## Hooks

This section contains a description of the module's `hooks`. Hooks are operations that are executed automatically when events are raised.

### Staking hooks

The slashing module implements the `StakingHooks` defined in `x/staking` and are used as record-keeping of validators information. During the app initialization, these hooks should be registered in the staking module struct.

The following hooks impact the slashing state:

* `AfterValidatorBonded` creates a `ValidatorSigningInfo` instance as described in the following section.
* `AfterValidatorCreated` stores a validator's consensus key.
* `AfterValidatorRemoved` removes a validator's consensus key.

### Validator Bonded

Upon successful first-time bonding of a new validator, we create a new `ValidatorSigningInfo` structure for the
now-bonded validator, which `StartHeight` of the current block.

If the validator was out of the validator set and gets bonded again, its new bonded height is set.

```go
onValidatorBonded(address sdk.ValAddress)

  signingInfo, found = GetValidatorSigningInfo(address)
  if !found {
    signingInfo = ValidatorSigningInfo {
      StartHeight         : CurrentHeight,
      IndexOffset         : 0,
      JailedUntil         : time.Unix(0, 0),
      Tombstone           : false,
      MissedBloskCounter  : 0
    } else {
      signingInfo.StartHeight = CurrentHeight
    }

    setValidatorSigningInfo(signingInfo)
  }

  return
```

## Events

The slashing module emits the following events:

### MsgServer

#### MsgUnjail

| Type    | Attribute Key | Attribute Value    |
| ------- | ------------- | ------------------ |
| message | module        | slashing           |
| message | sender        | {validatorAddress} |

### Keeper

### BeginBlocker: HandleValidatorSignature

| Type  | Attribute Key | Attribute Value             |
| ----- | ------------- | --------------------------- |
| slash | address       | {validatorConsensusAddress} |
| slash | power         | {validatorPower}            |
| slash | reason        | {slashReason}               |
| slash | jailed [0]    | {validatorConsensusAddress} |
| slash | burned coins  | {math.Int}                   |

* [0] Only included if the validator is jailed.

| Type     | Attribute Key | Attribute Value             |
| -------- | ------------- | --------------------------- |
| liveness | address       | {validatorConsensusAddress} |
| liveness | missed_blocks | {missedBlocksCounter}       |
| liveness | height        | {blockHeight}               |

#### Slash

* same as `"slash"` event from `HandleValidatorSignature`, but without the `jailed` attribute.

#### Jail

| Type  | Attribute Key | Attribute Value    |
| ----- | ------------- | ------------------ |
| slash | jailed        | {validatorAddress} |

## Staking Tombstone

### Abstract

In the current implementation of the `slashing` module, when the consensus engine
informs the state machine of a validator's consensus fault, the validator is
partially slashed, and put into a "jail period", a period of time in which they
are not allowed to rejoin the validator set. However, because of the nature of
consensus faults and ABCI, there can be a delay between an infraction occurring,
and evidence of the infraction reaching the state machine (this is one of the
primary reasons for the existence of the unbonding period).

> Note: The tombstone concept, only applies to faults that have a delay between
> the infraction occurring and evidence reaching the state machine. For example,
> evidence of a validator double signing may take a while to reach the state machine
> due to unpredictable evidence gossip layer delays and the ability of validators to
> selectively reveal double-signatures (e.g. to infrequently-online light clients).
> Liveness slashing, on the other hand, is detected immediately as soon as the
> infraction occurs, and therefore no slashing period is needed. A validator is
> immediately put into jail period, and they cannot commit another liveness fault
> until they unjail. In the future, there may be other types of byzantine faults
> that have delays (for example, submitting evidence of an invalid proposal as a transaction).
> When implemented, it will have to be decided whether these future types of
> byzantine faults will result in a tombstoning (and if not, the slash amounts
> will not be capped by a slashing period).

In the current system design, once a validator is put in the jail for a consensus
fault, after the `JailPeriod` they are allowed to send a transaction to `unjail`
themselves, and thus rejoin the validator set.

One of the "design desires" of the `slashing` module is that if multiple
infractions occur before evidence is executed (and a validator is put in jail),
they should only be punished for single worst infraction, but not cumulatively.
For example, if the sequence of events is:

1. Validator A commits Infraction 1 (worth 30% slash)
2. Validator A commits Infraction 2 (worth 40% slash)
3. Validator A commits Infraction 3 (worth 35% slash)
4. Evidence for Infraction 1 reaches state machine (and validator is put in jail)
5. Evidence for Infraction 2 reaches state machine
6. Evidence for Infraction 3 reaches state machine

Only Infraction 2 should have its slash take effect, as it is the highest. This
is done, so that in the case of the compromise of a validator's consensus key,
they will only be punished once, even if the hacker double-signs many blocks.
Because, the unjailing has to be done with the validator's operator key, they
have a chance to re-secure their consensus key, and then signal that they are
ready using their operator key. We call this period during which we track only
the max infraction, the "slashing period".

Once, a validator rejoins by unjailing themselves, we begin a new slashing period;
if they commit a new infraction after unjailing, it gets slashed cumulatively on
top of the worst infraction from the previous slashing period.

However, while infractions are grouped based off of the slashing periods, because
evidence can be submitted up to an `unbondingPeriod` after the infraction, we
still have to allow for evidence to be submitted for previous slashing periods.
For example, if the sequence of events is:

1. Validator A commits Infraction 1 (worth 30% slash)
2. Validator A commits Infraction 2 (worth 40% slash)
3. Evidence for Infraction 1 reaches state machine (and Validator A is put in jail)
4. Validator A unjails

We are now in a new slashing period, however we still have to keep the door open
for the previous infraction, as the evidence for Infraction 2 may still come in.
As the number of slashing periods increase, it creates more complexity as we have
to keep track of the highest infraction amount for every single slashing period.

> Note: Currently, according to the `slashing` module spec, a new slashing period
> is created every time a validator is unbonded then rebonded. This should probably
> be changed to jailed/unjailed. See issue [#3205](https://github.com/cosmos/cosmos-sdk/issues/3205)
> for further details. For the remainder of this, I will assume that we only start
> a new slashing period when a validator gets unjailed.

The maximum number of slashing periods is the `len(UnbondingPeriod) / len(JailPeriod)`.
The current defaults in Gaia for the `UnbondingPeriod` and `JailPeriod` are 3 weeks
and 2 days, respectively. This means there could potentially be up to 11 slashing
periods concurrently being tracked per validator. If we set the `JailPeriod >= UnbondingPeriod`,
we only have to track 1 slashing period (i.e not have to track slashing periods).

Currently, in the jail period implementation, once a validator unjails, all of
their delegators who are delegated to them (haven't unbonded / redelegated away),
stay with them. Given that consensus safety faults are so egregious
(way more so than liveness faults), it is probably prudent to have delegators not
"auto-rebond" to the validator.

#### Proposal: infinite jail

We propose setting the "jail time" for a
validator who commits a consensus safety fault, to `infinite` (i.e. a tombstone state).
This essentially kicks the validator out of the validator set and does not allow
them to re-enter the validator set. All of their delegators (including the operator themselves)
have to either unbond or redelegate away. The validator operator can create a new
validator if they would like, with a new operator key and consensus key, but they
have to "re-earn" their delegations back.

Implementing the tombstone system and getting rid of the slashing period tracking
will make the `slashing` module way simpler, especially because we can remove all
of the hooks defined in the `slashing` module consumed by the `staking` module
(the `slashing` module still consumes hooks defined in `staking`).

#### Single slashing amount

Another optimization that can be made is that if we assume that all ABCI faults
for CometBFT consensus are slashed at the same level, we don't have to keep
track of "max slash". Once an ABCI fault happens, we don't have to worry about
comparing potential future ones to find the max.

Currently the only CometBFT ABCI fault is:

* Unjustified precommits (double signs)

It is currently planned to include the following fault in the near future:

* Signing a precommit when you're in unbonding phase (needed to make light client bisection safe)

Given that these faults are both attributable byzantine faults, we will likely
want to slash them equally, and thus we can enact the above change.

> Note: This change may make sense for current CometBFT consensus, but maybe
> not for a different consensus algorithm or future versions of CometBFT that
> may want to punish at different levels (for example, partial slashing).

## Parameters

The slashing module contains the following parameters:

| Key                     | Type           | Example                |
| ----------------------- | -------------- | ---------------------- |
| SignedBlocksWindow      | string (int64) | "100"                  |
| MinSignedPerWindow      | string (dec)   | "0.500000000000000000" |
| DowntimeJailDuration    | string (ns)    | "600000000000"         |
| SlashFractionDoubleSign | string (dec)   | "0.050000000000000000" |
| SlashFractionDowntime   | string (dec)   | "0.010000000000000000" |

## CLI

A user can query and interact with the `slashing` module using the CLI.

### Query

The `query` commands allow users to query `slashing` state.

```shell
simd query slashing --help
```

#### params

The `params` command allows users to query genesis parameters for the slashing module.

```shell
simd query slashing params [flags]
```

Example:

```shell
simd query slashing params
```

Example Output:

```yml
downtime_jail_duration: 600s
min_signed_per_window: "0.500000000000000000"
signed_blocks_window: "100"
slash_fraction_double_sign: "0.050000000000000000"
slash_fraction_downtime: "0.010000000000000000"
```

#### signing-info

The `signing-info` command allows users to query signing-info of the validator using consensus public key.

```shell
simd query slashing signing-infos [flags]
```

Example:

```shell
simd query slashing signing-info '{"@type":"/cosmos.crypto.ed25519.PubKey","key":"Auxs3865HpB/EfssYOzfqNhEJjzys6jD5B6tPgC8="}'

```

Example Output:

```yml
address: cosmosvalcons1nrqsld3aw6lh6t082frdqc84uwxn0t958c
index_offset: "2068"
jailed_until: "1970-01-01T00:00:00Z"
missed_blocks_counter: "0"
start_height: "0"
tombstoned: false
```

#### signing-infos

The `signing-infos` command allows users to query signing infos of all validators.

```shell
simd query slashing signing-infos [flags]
```

Example:

```shell
simd query slashing signing-infos
```

Example Output:

```yml
info:
- address: cosmosvalcons1nrqsld3aw6lh6t082frdqc84uwxn0t958c
  index_offset: "2075"
  jailed_until: "1970-01-01T00:00:00Z"
  missed_blocks_counter: "0"
  start_height: "0"
  tombstoned: false
pagination:
  next_key: null
  total: "0"
```

### Transactions

The `tx` commands allow users to interact with the `slashing` module.

```bash
simd tx slashing --help
```

#### unjail

The `unjail` command allows users to unjail a validator previously jailed for downtime.

```bash
simd tx slashing unjail --from mykey [flags]
```

Example:

```bash
simd tx slashing unjail --from mykey
```

### gRPC

A user can query the `slashing` module using gRPC endpoints.

#### Params

The `Params` endpoint allows users to query the parameters of slashing module.

```shell
cosmos.slashing.v1beta1.Query/Params
```

Example:

```shell
grpcurl -plaintext localhost:9090 cosmos.slashing.v1beta1.Query/Params
```

Example Output:

```json
{
  "params": {
    "signedBlocksWindow": "100",
    "minSignedPerWindow": "NTAwMDAwMDAwMDAwMDAwMDAw",
    "downtimeJailDuration": "600s",
    "slashFractionDoubleSign": "NTAwMDAwMDAwMDAwMDAwMDA=",
    "slashFractionDowntime": "MTAwMDAwMDAwMDAwMDAwMDA="
  }
}
```

#### SigningInfo

The SigningInfo queries the signing info of given cons address.

```shell
cosmos.slashing.v1beta1.Query/SigningInfo
```

Example:

```shell
grpcurl -plaintext -d '{"cons_address":"cosmosvalcons1nrqsld3aw6lh6t082frdqc84uwxn0t958c"}' localhost:9090 cosmos.slashing.v1beta1.Query/SigningInfo
```

Example Output:

```json
{
  "valSigningInfo": {
    "address": "cosmosvalcons1nrqsld3aw6lh6t082frdqc84uwxn0t958c",
    "indexOffset": "3493",
    "jailedUntil": "1970-01-01T00:00:00Z"
  }
}
```

#### SigningInfos

The SigningInfos queries signing info of all validators.

```shell
cosmos.slashing.v1beta1.Query/SigningInfos
```

Example:

```shell
grpcurl -plaintext localhost:9090 cosmos.slashing.v1beta1.Query/SigningInfos
```

Example Output:

```json
{
  "info": [
    {
      "address": "cosmosvalcons1nrqslkwd3pz096lh6t082frdqc84uwxn0t958c",
      "indexOffset": "2467",
      "jailedUntil": "1970-01-01T00:00:00Z"
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```

### REST

A user can query the `slashing` module using REST endpoints.

#### Params

```shell
/cosmos/slashing/v1beta1/params
```

Example:

```shell
curl "localhost:1317/cosmos/slashing/v1beta1/params"
```

Example Output:

```json
{
  "params": {
    "signed_blocks_window": "100",
    "min_signed_per_window": "0.500000000000000000",
    "downtime_jail_duration": "600s",
    "slash_fraction_double_sign": "0.050000000000000000",
    "slash_fraction_downtime": "0.010000000000000000"
}
```

#### signing_info

```shell
/cosmos/slashing/v1beta1/signing_infos/%s
```

Example:

```shell
curl "localhost:1317/cosmos/slashing/v1beta1/signing_infos/cosmosvalcons1nrqslkwd3pz096lh6t082frdqc84uwxn0t958c"
```

Example Output:

```json
{
  "val_signing_info": {
    "address": "cosmosvalcons1nrqslkwd3pz096lh6t082frdqc84uwxn0t958c",
    "start_height": "0",
    "index_offset": "4184",
    "jailed_until": "1970-01-01T00:00:00Z",
    "tombstoned": false,
    "missed_blocks_counter": "0"
  }
}
```

#### signing_infos

```shell
/cosmos/slashing/v1beta1/signing_infos
```

Example:

```shell
curl "localhost:1317/cosmos/slashing/v1beta1/signing_infos
```

Example Output:

```json
{
  "info": [
    {
      "address": "cosmosvalcons1nrqslkwd3pz096lh6t082frdqc84uwxn0t958c",
      "start_height": "0",
      "index_offset": "4169",
      "jailed_until": "1970-01-01T00:00:00Z",
      "tombstoned": false,
      "missed_blocks_counter": "0"
    }
  ],
  "pagination": {
    "next_key": null,
    "total": "1"
  }
}
```
