---
sidebar_position: 1
---

# `x/distribution`

## Overview

This _simple_ distribution mechanism describes a functional way to passively
distribute rewards between validators and delegators. Note that this mechanism does
not distribute funds in as precisely as active reward distribution mechanisms and
will therefore be upgraded in the future.

The mechanism operates as follows. Collected rewards are pooled globally and
divided out passively to validators and delegators. Each validator has the
opportunity to charge commission to the delegators on the rewards collected on
behalf of the delegators. Fees are collected directly into a global reward pool
and validator proposer-reward pool. Due to the nature of passive accounting,
whenever changes to parameters which affect the rate of reward distribution
occurs, withdrawal of rewards must also occur.

* Whenever withdrawing, one must withdraw the maximum amount they are entitled
   to, leaving nothing in the pool.
* Whenever bonding, unbonding, or re-delegating tokens to an existing account, a
   full withdrawal of the rewards must occur (as the rules for lazy accounting
   change).
* Whenever a validator chooses to change the commission on rewards, all accumulated
   commission rewards must be simultaneously withdrawn.

The above scenarios are covered in `hooks.md`.

The distribution mechanism outlined herein is used to lazily distribute the
following rewards between validators and associated delegators:

* multi-token fees to be socially distributed
* inflated staked asset provisions
* validator commission on all rewards earned by their delegators stake

Fees are pooled within a global pool. The mechanisms used allow for validators
and delegators to independently and lazily withdraw their rewards.

## Shortcomings

As a part of the lazy computations, each delegator holds an accumulation term
specific to each validator which is used to estimate what their approximate
fair portion of tokens held in the global fee pool is owed to them.

```text
entitlement = delegator-accumulation / all-delegators-accumulation
```

Under the circumstance that there was constant and equal flow of incoming
reward tokens every block, this distribution mechanism would be equal to the
active distribution (distribute individually to all delegators each block).
However, this is unrealistic so deviations from the active distribution will
occur based on fluctuations of incoming reward tokens as well as timing of
reward withdrawal by other delegators.

If you happen to know that incoming rewards are about to significantly increase,
you are incentivized to not withdraw until after this event, increasing the
worth of your existing _accum_. See [#2764](https://github.com/cosmos/cosmos-sdk/issues/2764)
for further details.

## Effect on Staking

Charging commission on Atom provisions while also allowing for Atom-provisions
to be auto-bonded (distributed directly to the validators bonded stake) is
problematic within BPoS. Fundamentally, these two mechanisms are mutually
exclusive. If both commission and auto-bonding mechanisms are simultaneously
applied to the staking-token then the distribution of staking-tokens between
any validator and its delegators will change with each block. This then
necessitates a calculation for each delegation records for each block -
which is considered computationally expensive.

In conclusion, we can only have Atom commission and unbonded atoms
provisions or bonded atom provisions with no Atom commission, and we elect to
implement the former. Stakeholders wishing to rebond their provisions may elect
to set up a script to periodically withdraw and rebond rewards.

## Contents

* [Concepts](#concepts)
* [State](#state)
    * [FeePool](#feepool)
    * [Validator Distribution](#validator-distribution)
    * [Delegation Distribution](#delegation-distribution)
    * [Params](#params)
* [Begin Block](#begin-block)
* [Messages](#messages)
* [Hooks](#hooks)
* [Events](#events)
* [Parameters](#parameters)
* [Client](#client)
    * [CLI](#cli)
    * [gRPC](#grpc)

## Concepts

In Proof of Stake (PoS) blockchains, rewards gained from transaction fees are paid to validators. The fee distribution module fairly distributes the rewards to the validators' constituent delegators.

Rewards are calculated per period. The period is updated each time a validator's delegation changes, for example, when the validator receives a new delegation.
The rewards for a single validator can then be calculated by taking the total rewards for the period before the delegation started, minus the current total rewards.
To learn more, see the [F1 Fee Distribution paper](https://github.com/cosmos/cosmos-sdk/tree/main/docs/spec/fee_distribution/f1_fee_distr.pdf).

The commission to the validator is paid when the validator is removed or when the validator requests a withdrawal.
The commission is calculated and incremented at every `BeginBlock` operation to update accumulated fee amounts.

The rewards to a delegator are distributed when the delegation is changed or removed, or a withdrawal is requested.
Before rewards are distributed, all slashes to the validator that occurred during the current delegation are applied.

### Reference Counting in F1 Fee Distribution

In F1 fee distribution, the rewards a delegator receives are calculated when their delegation is withdrawn. This calculation must read the terms of the summation of rewards divided by the share of tokens from the period which they ended when they delegated, and the final period that was created for the withdrawal.

Additionally, as slashes change the amount of tokens a delegation will have (but we calculate this lazily,
only when a delegator un-delegates), we must calculate rewards in separate periods before / after any slashes
which occurred in between when a delegator delegated and when they withdrew their rewards. Thus slashes, like
delegations, reference the period which was ended by the slash event.

All stored historical rewards records for periods which are no longer referenced by any delegations
or any slashes can thus be safely removed, as they will never be read (future delegations and future
slashes will always reference future periods). This is implemented by tracking a `ReferenceCount`
along with each historical reward storage entry. Each time a new object (delegation or slash)
is created which might need to reference the historical record, the reference count is incremented.
Each time one object which previously needed to reference the historical record is deleted, the reference
count is decremented. If the reference count hits zero, the historical record is deleted.

## State

### FeePool

All globally tracked parameters for distribution are stored within
`FeePool`. Rewards are collected and added to the reward pool and
distributed to validators/delegators from here.

Note that the reward pool holds decimal coins (`DecCoins`) to allow
for fractions of coins to be received from operations like inflation.
When coins are distributed from the pool they are truncated back to
`sdk.Coins` which are non-decimal.

* FeePool: `0x00 -> ProtocolBuffer(FeePool)`

```go
// coins with decimal
type DecCoins []DecCoin

type DecCoin struct {
    Amount math.LegacyDec
    Denom  string
}
```

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/distribution/v1beta1/distribution.proto#L116-L123
```

### Validator Distribution

Validator distribution information for the relevant validator is updated each time:

1. delegation amount to a validator is updated,
2. any delegator withdraws from a validator, or
3. the validator withdraws its commission.

* ValidatorDistInfo: `0x02 | ValOperatorAddrLen (1 byte) | ValOperatorAddr -> ProtocolBuffer(validatorDistribution)`

```go
type ValidatorDistInfo struct {
    OperatorAddress     sdk.AccAddress
    SelfBondRewards     sdkmath.DecCoins
    ValidatorCommission types.ValidatorAccumulatedCommission
}
```

### Delegation Distribution

Each delegation distribution only needs to record the height at which it last
withdrew fees. Because a delegation must withdraw fees each time it's
properties change (aka bonded tokens etc.) its properties will remain constant
and the delegator's _accumulation_ factor can be calculated passively knowing
only the height of the last withdrawal and its current properties.

* DelegationDistInfo: `0x02 | DelegatorAddrLen (1 byte) | DelegatorAddr | ValOperatorAddrLen (1 byte) | ValOperatorAddr -> ProtocolBuffer(delegatorDist)`

```go
type DelegationDistInfo struct {
    WithdrawalHeight int64    // last time this delegation withdrew rewards
}
```

### Params

The distribution module stores it's params in state with the prefix of `0x09`,
it can be updated with governance or the address with authority.

* Params: `0x09 | ProtocolBuffer(Params)`

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/distribution/v1beta1/distribution.proto#L12-L42
```

## Begin Block

At each `BeginBlock`, all fees received in the previous block are transferred to
the distribution `ModuleAccount` account. When a delegator or validator
withdraws their rewards, they are taken out of the `ModuleAccount`. During begin
block, the different claims on the fees collected are updated as follows:

* The reserve community tax is charged.
* The remainder is distributed proportionally by voting power to all bonded validators

### The Distribution Scheme

See [params](#params) for description of parameters.

Let `fees` be the total fees collected in the previous block, including
inflationary rewards to the stake. All fees are collected in a specific module
account during the block. During `BeginBlock`, they are sent to the
`"distribution"` `ModuleAccount`. No other sending of tokens occurs. Instead, the
rewards each account is entitled to are stored, and withdrawals can be triggered
through the messages `FundCommunityPool`, `WithdrawValidatorCommission` and
`WithdrawDelegatorReward`.

#### Reward to the Community Pool

The community pool gets `community_tax * fees`, plus any remaining dust after
validators get their rewards that are always rounded down to the nearest
integer value.

#### Reward To the Validators

The proposer receives no extra rewards. All fees are distributed among all the
bonded validators, including the proposer, in proportion to their consensus power.

```text
powFrac = validator power / total bonded validator power
voteMul = 1 - community_tax
```

All validators receive `fees * voteMul * powFrac`.

#### Rewards to Delegators

Each validator's rewards are distributed to its delegators. The validator also
has a self-delegation that is treated like a regular delegation in
distribution calculations.

The validator sets a commission rate. The commission rate is flexible, but each
validator sets a maximum rate and a maximum daily increase. These maximums cannot be exceeded and protect delegators from sudden increases of validator commission rates to prevent validators from taking all of the rewards.

The outstanding rewards that the operator is entitled to are stored in
`ValidatorAccumulatedCommission`, while the rewards the delegators are entitled
to are stored in `ValidatorCurrentRewards`. The [F1 fee distribution scheme](#concepts) is used to calculate the rewards per delegator as they
withdraw or update their delegation, and is thus not handled in `BeginBlock`.

#### Example Distribution

For this example distribution, the underlying consensus engine selects block proposers in
proportion to their power relative to the entire bonded power.

All validators are equally performant at including pre-commits in their proposed
blocks. Then hold `(pre_commits included) / (total bonded validator power)`
constant so that the amortized block reward for the validator is `( validator power / total bonded power) * (1 - community tax rate)` of
the total rewards. Consequently, the reward for a single delegator is:

```text
(delegator proportion of the validator power / validator power) * (validator power / total bonded power)
  * (1 - community tax rate) * (1 - validator commission rate)
= (delegator proportion of the validator power / total bonded power) * (1 -
community tax rate) * (1 - validator commission rate)
```

## Messages

### MsgSetWithdrawAddress

By default, the withdraw address is the delegator address. To change its withdraw address, a delegator must send a `MsgSetWithdrawAddress` message.
Changing the withdraw address is possible only if the parameter `WithdrawAddrEnabled` is set to `true`.

The withdraw address cannot be any of the module accounts. These accounts are blocked from being withdraw addresses by being added to the distribution keeper's `blockedAddrs` array at initialization.

Response:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/distribution/v1beta1/tx.proto#L49-L60
```

```go
func (k Keeper) SetWithdrawAddr(ctx context.Context, delegatorAddr sdk.AccAddress, withdrawAddr sdk.AccAddress) error
	if k.blockedAddrs[withdrawAddr.String()] {
		fail with "`{withdrawAddr}` is not allowed to receive external funds"
	}

	if !k.GetWithdrawAddrEnabled(ctx) {
		fail with `ErrSetWithdrawAddrDisabled`
	}

	k.SetDelegatorWithdrawAddr(ctx, delegatorAddr, withdrawAddr)
```

### MsgWithdrawDelegatorReward

A delegator can withdraw its rewards.
Internally in the distribution module, this transaction simultaneously removes the previous delegation with associated rewards, the same as if the delegator simply started a new delegation of the same value.
The rewards are sent immediately from the distribution `ModuleAccount` to the withdraw address.
Any remainder (truncated decimals) are sent to the community pool.
The starting height of the delegation is set to the current validator period, and the reference count for the previous period is decremented.
The amount withdrawn is deducted from the `ValidatorOutstandingRewards` variable for the validator.

In the F1 distribution, the total rewards are calculated per validator period, and a delegator receives a piece of those rewards in proportion to their stake in the validator.
In basic F1, the total rewards that all the delegators are entitled to between to periods is calculated the following way.
Let `R(X)` be the total accumulated rewards up to period `X` divided by the tokens staked at that time. The delegator allocation is `R(X) * delegator_stake`.
Then the rewards for all the delegators for staking between periods `A` and `B` are `(R(B) - R(A)) * total stake`.
However, these calculated rewards don't account for slashing.

Taking the slashes into account requires iteration.
Let `F(X)` be the fraction a validator is to be slashed for a slashing event that happened at period `X`.
If the validator was slashed at periods `P1, ..., PN`, where `A < P1`, `PN < B`, the distribution module calculates the individual delegator's rewards, `T(A, B)`, as follows:

```go
stake := initial stake
rewards := 0
previous := A
for P in P1, ..., PN`:
    rewards = (R(P) - previous) * stake
    stake = stake * F(P)
    previous = P
rewards = rewards + (R(B) - R(PN)) * stake
```

The historical rewards are calculated retroactively by playing back all the slashes and then attenuating the delegator's stake at each step.
The final calculated stake is equivalent to the actual staked coins in the delegation with a margin of error due to rounding errors.

Response:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/distribution/v1beta1/tx.proto#L66-L77
```

### WithdrawValidatorCommission

The validator can send the WithdrawValidatorCommission message to withdraw their accumulated commission.
The commission is calculated in every block during `BeginBlock`, so no iteration is required to withdraw.
The amount withdrawn is deducted from the `ValidatorOutstandingRewards` variable for the validator.
Only integer amounts can be sent. If the accumulated awards have decimals, the amount is truncated before the withdrawal is sent, and the remainder is left to be withdrawn later.

### FundCommunityPool

This message sends coins directly from the sender to the community pool.

The transaction fails if the amount cannot be transferred from the sender to the distribution module account.

```go
func (k Keeper) FundCommunityPool(ctx context.Context, amount sdk.Coins, sender sdk.AccAddress) error {
  if err := k.bankKeeper.SendCoinsFromAccountToModule(ctx, sender, types.ModuleName, amount); err != nil {
    return err
  }

  feePool, err := k.FeePool.Get(ctx)
  if err != nil {
    return err
  }

  feePool.CommunityPool = feePool.CommunityPool.Add(sdk.NewDecCoinsFromCoins(amount...)...)
	
  if err := k.FeePool.Set(ctx, feePool); err != nil {
    return err
  }

  return nil
}
```

### Common distribution operations

These operations take place during many different messages.

#### Initialize delegation

Each time a delegation is changed, the rewards are withdrawn and the delegation is reinitialized.
Initializing a delegation increments the validator period and keeps track of the starting period of the delegation.

```go
// initialize starting info for a new delegation
func (k Keeper) initializeDelegation(ctx context.Context, val sdk.ValAddress, del sdk.AccAddress) {
    // period has already been incremented - we want to store the period ended by this delegation action
    previousPeriod := k.GetValidatorCurrentRewards(ctx, val).Period - 1

	// increment reference count for the period we're going to track
	k.incrementReferenceCount(ctx, val, previousPeriod)

	validator := k.stakingKeeper.Validator(ctx, val)
	delegation := k.stakingKeeper.Delegation(ctx, del, val)

	// calculate delegation stake in tokens
	// we don't store directly, so multiply delegation shares * (tokens per share)
	// note: necessary to truncate so we don't allow withdrawing more rewards than owed
	stake := validator.TokensFromSharesTruncated(delegation.GetShares())
	k.SetDelegatorStartingInfo(ctx, val, del, types.NewDelegatorStartingInfo(previousPeriod, stake, uint64(ctx.BlockHeight())))
}
```

### MsgUpdateParams

Distribution module params can be updated through `MsgUpdateParams`, which can be done using governance proposal and the signer will always be gov module account address.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/distribution/v1beta1/tx.proto#L133-L147
```

The message handling can fail if:

* signer is not the gov module account address.

## Hooks

Available hooks that can be called by and from this module.

### Create or modify delegation distribution

* triggered-by: `staking.MsgDelegate`, `staking.MsgBeginRedelegate`, `staking.MsgUndelegate`

#### Before

* The delegation rewards are withdrawn to the withdraw address of the delegator.
  The rewards include the current period and exclude the starting period.
* The validator period is incremented.
  The validator period is incremented because the validator's power and share distribution might have changed.
* The reference count for the delegator's starting period is decremented.

#### After

The starting height of the delegation is set to the previous period.
Because of the `Before`-hook, this period is the last period for which the delegator was rewarded.

### Validator created

* triggered-by: `staking.MsgCreateValidator`

When a validator is created, the following validator variables are initialized:

* Historical rewards
* Current accumulated rewards
* Accumulated commission
* Total outstanding rewards
* Period

By default, all values are set to a `0`, except period, which is set to `1`.

### Validator removed

* triggered-by: `staking.RemoveValidator`

Outstanding commission is sent to the validator's self-delegation withdrawal address.
Remaining delegator rewards get sent to the community fee pool.

Note: The validator gets removed only when it has no remaining delegations.
At that time, all outstanding delegator rewards will have been withdrawn.
Any remaining rewards are dust amounts.

### Validator is slashed

* triggered-by: `staking.Slash`
* The current validator period reference count is incremented.
  The reference count is incremented because the slash event has created a reference to it.
* The validator period is incremented.
* The slash event is stored for later use.
  The slash event will be referenced when calculating delegator rewards.

## Events

The distribution module emits the following events:

### BeginBlocker

| Type            | Attribute Key | Attribute Value    |
|-----------------|---------------|--------------------|
| proposer_reward | validator     | {validatorAddress} |
| proposer_reward | reward        | {proposerReward}   |
| commission      | amount        | {commissionAmount} |
| commission      | validator     | {validatorAddress} |
| rewards         | amount        | {rewardAmount}     |
| rewards         | validator     | {validatorAddress} |

### Handlers

#### MsgSetWithdrawAddress

| Type                 | Attribute Key    | Attribute Value      |
|----------------------|------------------|----------------------|
| set_withdraw_address | withdraw_address | {withdrawAddress}    |
| message              | module           | distribution         |
| message              | action           | set_withdraw_address |
| message              | sender           | {senderAddress}      |

#### MsgWithdrawDelegatorReward

| Type    | Attribute Key | Attribute Value           |
|---------|---------------|---------------------------|
| withdraw_rewards | amount        | {rewardAmount}            |
| withdraw_rewards | validator     | {validatorAddress}        |
| message          | module        | distribution              |
| message          | action        | withdraw_delegator_reward |
| message          | sender        | {senderAddress}           |

#### MsgWithdrawValidatorCommission

| Type       | Attribute Key | Attribute Value               |
|------------|---------------|-------------------------------|
| withdraw_commission | amount        | {commissionAmount}            |
| message    | module        | distribution                  |
| message    | action        | withdraw_validator_commission |
| message    | sender        | {senderAddress}               |

## Parameters

The distribution module contains the following parameters:

| Key                 | Type         | Example                    |
| ------------------- | ------------ | -------------------------- |
| communitytax        | string (dec) | "0.020000000000000000" [0] |
| withdrawaddrenabled | bool         | true                       |

* [0] `communitytax` must be positive and cannot exceed 1.00.
* `baseproposerreward` and `bonusproposerreward` were parameters that are deprecated in v0.47 and are not used.

:::note
The reserve pool is the pool of collected funds for use by governance taken via the `CommunityTax`.
Currently with the Cosmos SDK, tokens collected by the CommunityTax are accounted for but unspendable.
:::

## Client

## CLI

A user can query and interact with the `distribution` module using the CLI.

#### Query

The `query` commands allow users to query `distribution` state.

```shell
simd query distribution --help
```

##### commission

The `commission` command allows users to query validator commission rewards by address.

```shell
simd query distribution commission [address] [flags]
```

Example:

```shell
simd query distribution commission cosmosvaloper1...
```

Example Output:

```yml
commission:
- amount: "1000000.000000000000000000"
  denom: stake
```

##### community-pool

The `community-pool` command allows users to query all coin balances within the community pool.

```shell
simd query distribution community-pool [flags]
```

Example:

```shell
simd query distribution community-pool
```

Example Output:

```yml
pool:
- amount: "1000000.000000000000000000"
  denom: stake
```

##### params

The `params` command allows users to query the parameters of the `distribution` module.

```shell
simd query distribution params [flags]
```

Example:

```shell
simd query distribution params
```

Example Output:

```yml
base_proposer_reward: "0.000000000000000000"
bonus_proposer_reward: "0.000000000000000000"
community_tax: "0.020000000000000000"
withdraw_addr_enabled: true
```

##### rewards

The `rewards` command allows users to query delegator rewards. Users can optionally include the validator address to query rewards earned from a specific validator.

```shell
simd query distribution rewards [delegator-addr] [validator-addr] [flags]
```

Example:

```shell
simd query distribution rewards cosmos1...
```

Example Output:

```yml
rewards:
- reward:
  - amount: "1000000.000000000000000000"
    denom: stake
  validator_address: cosmosvaloper1..
total:
- amount: "1000000.000000000000000000"
  denom: stake
```

##### slashes

The `slashes` command allows users to query all slashes for a given block range.

```shell
simd query distribution slashes [validator] [start-height] [end-height] [flags]
```

Example:

```shell
simd query distribution slashes cosmosvaloper1... 1 1000
```

Example Output:

```yml
pagination:
  next_key: null
  total: "0"
slashes:
- validator_period: 20,
  fraction: "0.009999999999999999"
```

##### validator-outstanding-rewards

The `validator-outstanding-rewards` command allows users to query all outstanding (un-withdrawn) rewards for a validator and all their delegations.

```shell
simd query distribution validator-outstanding-rewards [validator] [flags]
```

Example:

```shell
simd query distribution validator-outstanding-rewards cosmosvaloper1...
```

Example Output:

```yml
rewards:
- amount: "1000000.000000000000000000"
  denom: stake
```

##### validator-distribution-info

The `validator-distribution-info` command allows users to query validator commission and self-delegation rewards for validator.

````shell
simd query distribution validator-distribution-info cosmosvaloper1...
```

Example Output:

```yml
commission:
- amount: "100000.000000000000000000"
  denom: stake
operator_address: cosmosvaloper1...
self_bond_rewards:
- amount: "100000.000000000000000000"
  denom: stake
```

#### Transactions

The `tx` commands allow users to interact with the `distribution` module.

```shell
simd tx distribution --help
```

##### fund-community-pool

The `fund-community-pool` command allows users to send funds to the community pool.

```shell
simd tx distribution fund-community-pool [amount] [flags]
```

Example:

```shell
simd tx distribution fund-community-pool 100stake --from cosmos1...
```

##### set-withdraw-addr

The `set-withdraw-addr` command allows users to set the withdraw address for rewards associated with a delegator address.

```shell
simd tx distribution set-withdraw-addr [withdraw-addr] [flags]
```

Example:

```shell
simd tx distribution set-withdraw-addr cosmos1... --from cosmos1...
```

##### withdraw-all-rewards

The `withdraw-all-rewards` command allows users to withdraw all rewards for a delegator.

```shell
simd tx distribution withdraw-all-rewards [flags]
```

Example:

```shell
simd tx distribution withdraw-all-rewards --from cosmos1...
```

##### withdraw-rewards

The `withdraw-rewards` command allows users to withdraw all rewards from a given delegation address,
and optionally withdraw validator commission if the delegation address given is a validator operator and the user proves the `--commission` flag.

```shell
simd tx distribution withdraw-rewards [validator-addr] [flags]
```

Example:

```shell
simd tx distribution withdraw-rewards cosmosvaloper1... --from cosmos1... --commission
```

### gRPC

A user can query the `distribution` module using gRPC endpoints.

#### Params

The `Params` endpoint allows users to query parameters of the `distribution` module.

Example:

```shell
grpcurl -plaintext \
    localhost:9090 \
    cosmos.distribution.v1beta1.Query/Params
```

Example Output:

```json
{
  "params": {
    "communityTax": "20000000000000000",
    "baseProposerReward": "00000000000000000",
    "bonusProposerReward": "00000000000000000",
    "withdrawAddrEnabled": true
  }
}
```

#### ValidatorDistributionInfo

The `ValidatorDistributionInfo` queries validator commission and self-delegation rewards for validator.

Example:

```shell
grpcurl -plaintext \
    -d '{"validator_address":"cosmosvalop1..."}' \
    localhost:9090 \
    cosmos.distribution.v1beta1.Query/ValidatorDistributionInfo
```

Example Output:

```json
{
  "commission": {
    "commission": [
      {
        "denom": "stake",
        "amount": "1000000000000000"
      }
    ]
  },
  "self_bond_rewards": [
    {
      "denom": "stake",
      "amount": "1000000000000000"
    }
  ],
  "validator_address": "cosmosvalop1..."
}
```

#### ValidatorOutstandingRewards

The `ValidatorOutstandingRewards` endpoint allows users to query rewards of a validator address.

Example:

```shell
grpcurl -plaintext \
    -d '{"validator_address":"cosmosvalop1.."}' \
    localhost:9090 \
    cosmos.distribution.v1beta1.Query/ValidatorOutstandingRewards
```

Example Output:

```json
{
  "rewards": {
    "rewards": [
      {
        "denom": "stake",
        "amount": "1000000000000000"
      }
    ]
  }
}
```

#### ValidatorCommission

The `ValidatorCommission` endpoint allows users to query accumulated commission for a validator.

Example:

```shell
grpcurl -plaintext \
    -d '{"validator_address":"cosmosvalop1.."}' \
    localhost:9090 \
    cosmos.distribution.v1beta1.Query/ValidatorCommission
```

Example Output:

```json
{
  "commission": {
    "commission": [
      {
        "denom": "stake",
        "amount": "1000000000000000"
      }
    ]
  }
}
```

#### ValidatorSlashes

The `ValidatorSlashes` endpoint allows users to query slash events of a validator.

Example:

```shell
grpcurl -plaintext \
    -d '{"validator_address":"cosmosvalop1.."}' \
    localhost:9090 \
    cosmos.distribution.v1beta1.Query/ValidatorSlashes
```

Example Output:

```json
{
  "slashes": [
    {
      "validator_period": "20",
      "fraction": "0.009999999999999999"
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```

#### DelegationRewards

The `DelegationRewards` endpoint allows users to query the total rewards accrued by a delegation.

Example:

```shell
grpcurl -plaintext \
    -d '{"delegator_address":"cosmos1...","validator_address":"cosmosvalop1..."}' \
    localhost:9090 \
    cosmos.distribution.v1beta1.Query/DelegationRewards
```

Example Output:

```json
{
  "rewards": [
    {
      "denom": "stake",
      "amount": "1000000000000000"
    }
  ]
}
```

#### DelegationTotalRewards

The `DelegationTotalRewards` endpoint allows users to query the total rewards accrued by each validator.

Example:

```shell
grpcurl -plaintext \
    -d '{"delegator_address":"cosmos1..."}' \
    localhost:9090 \
    cosmos.distribution.v1beta1.Query/DelegationTotalRewards
```

Example Output:

```json
{
  "rewards": [
    {
      "validatorAddress": "cosmosvaloper1...",
      "reward": [
        {
          "denom": "stake",
          "amount": "1000000000000000"
        }
      ]
    }
  ],
  "total": [
    {
      "denom": "stake",
      "amount": "1000000000000000"
    }
  ]
}
```

#### DelegatorValidators

The `DelegatorValidators` endpoint allows users to query all validators for given delegator.

Example:

```shell
grpcurl -plaintext \
    -d '{"delegator_address":"cosmos1..."}' \
    localhost:9090 \
    cosmos.distribution.v1beta1.Query/DelegatorValidators
```

Example Output:

```json
{
  "validators": ["cosmosvaloper1..."]
}
```

#### DelegatorWithdrawAddress

The `DelegatorWithdrawAddress` endpoint allows users to query the withdraw address of a delegator.

Example:

```shell
grpcurl -plaintext \
    -d '{"delegator_address":"cosmos1..."}' \
    localhost:9090 \
    cosmos.distribution.v1beta1.Query/DelegatorWithdrawAddress
```

Example Output:

```json
{
  "withdrawAddress": "cosmos1..."
}
```

#### CommunityPool

The `CommunityPool` endpoint allows users to query the community pool coins.

Example:

```shell
grpcurl -plaintext \
    localhost:9090 \
    cosmos.distribution.v1beta1.Query/CommunityPool
```

Example Output:

```json
{
  "pool": [
    {
      "denom": "stake",
      "amount": "1000000000000000000"
    }
  ]
}
```
