---
sidebar_position: 1
---

# `x/feegrant`

## Abstract

This document specifies the fee grant module. For the full ADR, please see [Fee Grant ADR-029](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-029-fee-grant-module.md).

This module allows accounts to grant fee allowances and to use fees from their accounts. Grantees can execute any transaction without the need to maintain sufficient fees.

## Contents

* [Concepts](#concepts)
* [State](#state)
    * [FeeAllowance](#feeallowance)
    * [FeeAllowanceQueue](#feeallowancequeue)
* [Messages](#messages)
    * [Msg/GrantAllowance](#msggrantallowance)
    * [Msg/RevokeAllowance](#msgrevokeallowance)
* [Events](#events)
* [Msg Server](#msg-server)
    * [MsgGrantAllowance](#msggrantallowance-1)
    * [MsgRevokeAllowance](#msgrevokeallowance-1)
    * [Exec fee allowance](#exec-fee-allowance)
* [Client](#client)
    * [CLI](#cli)
    * [gRPC](#grpc)

## Concepts

### Grant

`Grant` is stored in the KVStore to record a grant with full context. Every grant will contain `granter`, `grantee` and what kind of `allowance` is granted. `granter` is an account address who is giving permission to `grantee` (the beneficiary account address) to pay for some or all of `grantee`'s transaction fees. `allowance` defines what kind of fee allowance (`BasicAllowance` or `PeriodicAllowance`, see below) is granted to `grantee`. `allowance` accepts an interface which implements `FeeAllowanceI`, encoded as `Any` type. There can be only one existing fee grant allowed for a `grantee` and `granter`, self grants are not allowed.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/feegrant/v1beta1/feegrant.proto#L83-L93
```

`FeeAllowanceI` looks like:

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/feegrant/fees.go#L9-L32
```

### Fee Allowance types

There are two types of fee allowances present at the moment:

* `BasicAllowance`
* `PeriodicAllowance`
* `AllowedMsgAllowance`

### BasicAllowance

`BasicAllowance` is permission for `grantee` to use fee from a `granter`'s account. If any of the `spend_limit` or `expiration` reaches its limit, the grant will be removed from the state.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/feegrant/v1beta1/feegrant.proto#L15-L28
```

* `spend_limit` is the limit of coins that are allowed to be used from the `granter` account. If it is empty, it assumes there's no spend limit, `grantee` can use any number of available coins from `granter` account address before the expiration.

* `expiration` specifies an optional time when this allowance expires. If the value is left empty, there is no expiry for the grant.

* When a grant is created with empty values for `spend_limit` and `expiration`, it is still a valid grant. It won't restrict the `grantee` to use any number of coins from `granter` and it won't have any expiration. The only way to restrict the `grantee` is by revoking the grant.

### PeriodicAllowance

`PeriodicAllowance` is a repeating fee allowance for the mentioned period, we can mention when the grant can expire as well as when a period can reset. We can also define the maximum number of coins that can be used in a mentioned period of time.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/feegrant/v1beta1/feegrant.proto#L34-L68
```

* `basic` is the instance of `BasicAllowance` which is optional for periodic fee allowance. If empty, the grant will have no `expiration` and no `spend_limit`.

* `period` is the specific period of time, after each period passes, `period_can_spend` will be reset.

* `period_spend_limit` specifies the maximum number of coins that can be spent in the period.

* `period_can_spend` is the number of coins left to be spent before the period_reset time.

* `period_reset` keeps track of when a next period reset should happen.

### AllowedMsgAllowance

`AllowedMsgAllowance` is a fee allowance, it can be any of `BasicFeeAllowance`, `PeriodicAllowance` but restricted only to the allowed messages mentioned by the granter.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/feegrant/v1beta1/feegrant.proto#L70-L81
```

* `allowance` is either `BasicAllowance` or `PeriodicAllowance`.

* `allowed_messages` is array of messages allowed to execute the given allowance.

### FeeGranter flag

`feegrant` module introduces a `FeeGranter` flag for CLI for the sake of executing transactions with fee granter. When this flag is set, `clientCtx` will append the granter account address for transactions generated through CLI.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/client/cmd.go#L249-L260
```

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/client/tx/tx.go#L109-L109
```

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/auth/tx/builder.go#L275-L284
```

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/tx/v1beta1/tx.proto#L203-L224
```

Example cmd:

```go
./simd tx gov submit-proposal --title="Test Proposal" --description="My awesome proposal" --type="Text" --from validator-key --fee-granter=cosmos1xh44hxt7spr67hqaa7nyx5gnutrz5fraw6grxn --chain-id=testnet --fees="10stake"
```

### Granted Fee Deductions

Fees are deducted from grants in the `x/auth` ante handler. To learn more about how ante handlers work, read the [Auth Module AnteHandlers Guide](../auth/README.md#antehandlers).

### Gas

In order to prevent DoS attacks, using a filtered `x/feegrant` incurs gas. The SDK must assure that the `grantee`'s transactions all conform to the filter set by the `granter`. The SDK does this by iterating over the allowed messages in the filter and charging 10 gas per filtered message. The SDK will then iterate over the messages being sent by the `grantee` to ensure the messages adhere to the filter, also charging 10 gas per message. The SDK will stop iterating and fail the transaction if it finds a message that does not conform to the filter.

**WARNING**: The gas is charged against the granted allowance. Ensure your messages conform to the filter, if any, before sending transactions using your allowance.

### Pruning

A queue in the state maintained with the prefix of expiration of the grants and checks them on EndBlock with the current block time for every block to prune.

## State

### FeeAllowance

Fee Allowances are identified by combining `Grantee` (the account address of fee allowance grantee) with the `Granter` (the account address of fee allowance granter).

Fee allowance grants are stored in the state as follows:

* Grant: `0x00 | grantee_addr_len (1 byte) | grantee_addr_bytes |  granter_addr_len (1 byte) | granter_addr_bytes -> ProtocolBuffer(Grant)`

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/x/feegrant/feegrant.pb.go#L222-L230
```

### FeeAllowanceQueue

Fee Allowances queue items are identified by combining the `FeeAllowancePrefixQueue` (i.e., 0x01), `expiration`, `grantee` (the account address of fee allowance grantee), `granter` (the account address of fee allowance granter). Endblocker checks `FeeAllowanceQueue` state for the expired grants and prunes them from  `FeeAllowance` if there are any found.

Fee allowance queue keys are stored in the state as follows:

* Grant: `0x01 | expiration_bytes | grantee_addr_len (1 byte) | grantee_addr_bytes |  granter_addr_len (1 byte) | granter_addr_bytes -> EmptyBytes`

## Messages

### Msg/GrantAllowance

A fee allowance grant will be created with the `MsgGrantAllowance` message.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/feegrant/v1beta1/tx.proto#L25-L39
```

### Msg/RevokeAllowance

An allowed grant fee allowance can be removed with the `MsgRevokeAllowance` message.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/feegrant/v1beta1/tx.proto#L41-L54
```

## Events

The feegrant module emits the following events:

## Msg Server

### MsgGrantAllowance

| Type    | Attribute Key | Attribute Value  |
| ------- | ------------- | ---------------- |
| message | action        | set_feegrant     |
| message | granter       | {granterAddress} |
| message | grantee       | {granteeAddress} |

### MsgRevokeAllowance

| Type    | Attribute Key | Attribute Value  |
| ------- | ------------- | ---------------- |
| message | action        | revoke_feegrant  |
| message | granter       | {granterAddress} |
| message | grantee       | {granteeAddress} |

### Exec fee allowance

| Type    | Attribute Key | Attribute Value  |
| ------- | ------------- | ---------------- |
| message | action        | use_feegrant     |
| message | granter       | {granterAddress} |
| message | grantee       | {granteeAddress} |

### Prune fee allowances

| Type    | Attribute Key | Attribute Value  |
| ------- | ------------- | ---------------- |
| message | action        |  prune_feegrant  |
| message | pruner        | {prunerAddress}  |


## Client

### CLI

A user can query and interact with the `feegrant` module using the CLI.

#### Query

The `query` commands allow users to query `feegrant` state.

```shell
simd query feegrant --help
```

##### grant

The `grant` command allows users to query a grant for a given granter-grantee pair.

```shell
simd query feegrant grant [granter] [grantee] [flags]
```

Example:

```shell
simd query feegrant grant cosmos1.. cosmos1..
```

Example Output:

```yml
allowance:
  '@type': /cosmos.feegrant.v1beta1.BasicAllowance
  expiration: null
  spend_limit:
  - amount: "100"
    denom: stake
grantee: cosmos1..
granter: cosmos1..
```

##### grants

The `grants` command allows users to query all grants for a given grantee.

```shell
simd query feegrant grants [grantee] [flags]
```

Example:

```shell
simd query feegrant grants cosmos1..
```

Example Output:

```yml
allowances:
- allowance:
    '@type': /cosmos.feegrant.v1beta1.BasicAllowance
    expiration: null
    spend_limit:
    - amount: "100"
      denom: stake
  grantee: cosmos1..
  granter: cosmos1..
pagination:
  next_key: null
  total: "0"
```

#### Transactions

The `tx` commands allow users to interact with the `feegrant` module.

```shell
simd tx feegrant --help
```

##### grant

The `grant` command allows users to grant fee allowances to another account. The fee allowance can have an expiration date, a total spend limit, and/or a periodic spend limit.

```shell
simd tx feegrant grant [granter] [grantee] [flags]
```

Example (one-time spend limit):

```shell
simd tx feegrant grant cosmos1.. cosmos1.. --spend-limit 100stake
```

Example (periodic spend limit):

```shell
simd tx feegrant grant cosmos1.. cosmos1.. --period 3600 --period-limit 10stake
```

##### revoke

The `revoke` command allows users to revoke a granted fee allowance.

```shell
simd tx feegrant revoke [granter] [grantee] [flags]
```

Example:

```shell
simd tx feegrant revoke cosmos1.. cosmos1..
```

### gRPC

A user can query the `feegrant` module using gRPC endpoints.

#### Allowance

The `Allowance` endpoint allows users to query a granted fee allowance.

```shell
cosmos.feegrant.v1beta1.Query/Allowance
```

Example:

```shell
grpcurl -plaintext \
    -d '{"grantee":"cosmos1..","granter":"cosmos1.."}' \
    localhost:9090 \
    cosmos.feegrant.v1beta1.Query/Allowance
```

Example Output:

```json
{
  "allowance": {
    "granter": "cosmos1..",
    "grantee": "cosmos1..",
    "allowance": {"@type":"/cosmos.feegrant.v1beta1.BasicAllowance","spendLimit":[{"denom":"stake","amount":"100"}]}
  }
}
```

#### Allowances

The `Allowances` endpoint allows users to query all granted fee allowances for a given grantee.

```shell
cosmos.feegrant.v1beta1.Query/Allowances
```

Example:

```shell
grpcurl -plaintext \
    -d '{"address":"cosmos1.."}' \
    localhost:9090 \
    cosmos.feegrant.v1beta1.Query/Allowances
```

Example Output:

```json
{
  "allowances": [
    {
      "granter": "cosmos1..",
      "grantee": "cosmos1..",
      "allowance": {"@type":"/cosmos.feegrant.v1beta1.BasicAllowance","spendLimit":[{"denom":"stake","amount":"100"}]}
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```
