---
sidebar_position: 1
---

# `x/protocolpool`

## Concepts

Protopool is a module that handle functionality around community pool funds. This provides a separate module account for community pool making it easier to track the pool assets. We no longer track community pool assets in distribution module, but instead in this protocolpool module. Funds are migrated from the distribution module's community pool to protocolpool's module account.

The module is also designed with a lazy "claim-based" system, which means that users are required to actively claim allocated funds from allocated budget if any budget proposal has been submitted and passed. The module does not automatically distribute funds to recipients. This design choice allows for more flexibility and control over fund distribution.

## State Transitions

### FundCommunityPool

FundCommunityPool can be called by any valid account to send funds to the protocolpool module account.

```protobuf
  // FundCommunityPool defines a method to allow an account to directly
  // fund the community pool.
  rpc FundCommunityPool(MsgFundCommunityPool) returns (MsgFundCommunityPoolResponse);
```

### CommunityPoolSpend

CommunityPoolSpend can be called by the module authority (default governance module account) or any account with authorization to spend funds from the protocolpool module account to a receiver address.

```protobuf
  // CommunityPoolSpend defines a governance  operation for sending tokens from
  // the community pool in the x/protocolpool module to another account, which
  // could be the governance module itself. The authority is defined in the
  // keeper.
  rpc CommunityPoolSpend(MsgCommunityPoolSpend) returns (MsgCommunityPoolSpendResponse);
```

### SubmitBudgetProposal

SubmitBudgetProposal is a message used to propose a budget allocation for a specific recipient. The proposed funds will be distributed periodically over a specified time frame.

It's the responsibility of users to actively claim their allocated funds based on the terms of the approved budget proposals.

```protobuf
  // SubmitBudgetProposal defines a method to set a budget proposal.
  rpc SubmitBudgetProposal(MsgSubmitBudgetProposal) returns (MsgSubmitBudgetProposalResponse);
```

### ClaimBudget

ClaimBudget is a message used to claim funds from a previously submitted budget proposal. When a budget proposal is approved and funds are allocated, recipients can use this message to claim their share of the budget. Funds are distributed in tranches over specific periods, and users can claim their share of budget at the appropriate time.

```protobuf
  // ClaimBudget defines a method to claim the distributed budget.
  rpc ClaimBudget(MsgClaimBudget) returns (MsgClaimBudgetResponse);

```

### CreateContinuousFund

CreateContinuousFund is a message used to initiate a continuous fund for a specific recipient. The proposed percentage of funds will be distributed only on withdraw request for the recipient. The fund distribution continues until expiry time is reached or continuous fund request is canceled.
NOTE:  This feature is designed to work with the SDK's default bond denom. 

```protobuf
  // CreateContinuousFund defines a method to add funds continuously.
  rpc CreateContinuousFund(MsgCreateContinuousFund) returns (MsgCreateContinuousFundResponse);
```

### CancelContinuousFund

CancelContinuousFund is a message used to cancel an existing continuous fund proposal for a specific recipient. Cancelling a continuous fund stops further distribution of funds, and the state object is removed from storage.

```protobuf
  // CancelContinuousFund defines a method for cancelling continuous fund.
  rpc CancelContinuousFund(MsgCancelContinuousFund) returns (MsgCancelContinuousFundResponse);
```

## Messages

### MsgFundCommunityPool

This message sends coins directly from the sender to the community pool.

:::tip
If you know the protocolpool module account address, you can directly use bank `send` transaction instead.
:::

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/protocolpool/proto/cosmos/protocolpool/v1/tx.proto#L43-L53
```

* The msg will fail if the amount cannot be transferred from the sender to the protocolpool module account.

```go
func (k Keeper) FundCommunityPool(ctx context.Context, amount sdk.Coins, sender sdk.AccAddress) error {
	return k.bankKeeper.SendCoinsFromAccountToModule(ctx, sender, types.ModuleName, amount)
}
```

### MsgCommunityPoolSpend

This message distributes funds from the protocolpool module account to the recipient using `DistributeFromCommunityPool` keeper method.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/protocolpool/proto/cosmos/protocolpool/v1/tx.proto#L58-L69
```

The message will fail under the following conditions:

* The amount cannot be transferred to the recipient from the protocolpool module account.
* The `recipient` address is restricted

```go
func (k Keeper) DistributeFromCommunityPool(ctx context.Context, amount sdk.Coins, receiveAddr sdk.AccAddress) error {
	return k.bankKeeper.SendCoinsFromModuleToAccount(ctx, types.ModuleName, receiveAddr, amount)
}
```

### MsgSubmitBudgetProposal

This message is used to submit a budget proposal to allocate funds for a specific recipient. The proposed funds will be distributed periodically over a specified time frame.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/protocolpool/proto/cosmos/protocolpool/v1/tx.proto#L75-L94
```

The message will fail under the following conditions:

* The budget per tranche is zero.
* The recipient address is empty or restricted.
* The start time is less than current block time.
* The number of tranches is not a positive integer.
* The period length is not a positive integer.

:::warning
If two budgets to the same address are created, the budget would be updated with the new budget.
:::

```go reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/protocolpool/keeper/msg_server.go#L37-L59
```

### MsgClaimBudget

This message is used to claim funds from a previously submitted budget proposal. When a budget proposal is passed and funds are allocated, recipients can use this message to claim their share of the budget.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/protocolpool/proto/cosmos/protocolpool/v1/tx.proto#L100-L104
```

The message will fail under the following conditions:

- The recipient address is empty or restricted.
- The budget proposal for the recipient does not exist.
- The budget proposal has not reached its distribution start time.
- The budget proposal's distribution period has not passed yet.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/protocolpool/keeper/msg_server.go#L28-L35
```

### MsgCreateContinuousFund

This message is used to create a continuous fund for a specific recipient. The proposed percentage of funds will be distributed only on withdraw request for the recipient. This fund distribution continues until expiry time is reached or continuous fund request is canceled.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/protocolpool/proto/cosmos/protocolpool/v1/tx.proto#L114-L130
```

The message will fail under the following conditions:

- The recipient address is empty or restricted.
- The percentage is zero/negative/greater than one.
- The Expiry time is less than the current block time.

:::warning
If two continuous fund proposals to the same address are created, the previous ContinuousFund would be updated with the new ContinuousFund.
:::

```go reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/protocolpool/keeper/msg_server.go#L103-L166
```

### MsgCancelContinuousFund

This message is used to cancel an existing continuous fund proposal for a specific recipient. Once canceled, the continuous fund will no longer distribute funds at each end block, and the state object will be removed. Users should be cautious when canceling continuous funds, as it may affect the planned distribution for the recipient.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/protocolpool/proto/cosmos/protocolpool/v1/tx.proto#L136-L161
```

The message will fail under the following conditions:

- The recipient address is empty or restricted.
- The ContinuousFund for the recipient does not exist.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/protocolpool/keeper/msg_server.go#L188-L226
```

## Client

It takes the advantage of `AutoCLI`

```go reference
https://github.com/cosmos/cosmos-sdk/blob/release/v0.52.x/x/protocolpool/autocli.go
```
