---
sidebar_position: 1
---

# `x/bank`

## Abstract

This document specifies the bank module of the Cosmos SDK.

The bank module is responsible for handling multi-asset coin transfers between
accounts and tracking special-case pseudo-transfers which must work differently
with particular kinds of accounts (notably delegating/undelegating for vesting
accounts). It exposes several interfaces with varying capabilities for secure
interaction with other modules which must alter user balances.

In addition, the bank module tracks and provides query support for the total
supply of all assets used in the application.

This module is used in the Cosmos Hub.

## Contents

* [Supply](#supply)
    * [Total Supply](#total-supply)
* [Module Accounts](#module-accounts)
    * [Permissions](#permissions)
* [State](#state)
* [Params](#params)
* [Keepers](#keepers)
* [Messages](#messages)
* [Events](#events)
    * [Message Events](#message-events)
    * [Keeper Events](#keeper-events)
* [Parameters](#parameters)
    * [SendEnabled](#sendenabled)
    * [DefaultSendEnabled](#defaultsendenabled)
* [Client](#client)
    * [CLI](#cli)
    * [Query](#query)
    * [Transactions](#transactions)
* [gRPC](#grpc)

## Supply

The `supply` functionality:

* passively tracks the total supply of coins within a chain,
* provides a pattern for modules to hold/interact with `Coins`, and
* introduces the invariant check to verify a chain's total supply.

### Total Supply

The total `Supply` of the network is equal to the sum of all coins from the
account. The total supply is updated every time a `Coin` is minted (eg: as part
of the inflation mechanism) or burned (eg: due to slashing or if a governance
proposal is vetoed).

## Module Accounts

The supply functionality introduces a new type of `auth.Account` which can be used by
modules to allocate tokens and in special cases mint or burn tokens. At a base
level these module accounts are capable of sending/receiving tokens to and from
`auth.Account`s and other module accounts. This design replaces previous
alternative designs where, to hold tokens, modules would burn the incoming
tokens from the sender account, and then track those tokens internally. Later,
in order to send tokens, the module would need to effectively mint tokens
within a destination account. The new design removes duplicate logic between
modules to perform this accounting.

The `ModuleAccount` interface is defined as follows:

```go
type ModuleAccount interface {
  auth.Account               // same methods as the Account interface

  GetName() string           // name of the module; used to obtain the address
  GetPermissions() []string  // permissions of module account
  HasPermission(string) bool
}
```

> **WARNING!**
> Any module or message handler that allows either direct or indirect sending of funds must explicitly guarantee those funds cannot be sent to module accounts (unless allowed).

The supply `Keeper` also introduces new wrapper functions for the auth `Keeper`
and the bank `Keeper` that are related to `ModuleAccount`s in order to be able
to:

* Get and set `ModuleAccount`s by providing the `Name`.
* Send coins from and to other `ModuleAccount`s or standard `Account`s
  (`BaseAccount` or `VestingAccount`) by passing only the `Name`.
* `Mint` or `Burn` coins for a `ModuleAccount` (restricted to its permissions).

### Permissions

Each `ModuleAccount` has a different set of permissions that provide different
object capabilities to perform certain actions. Permissions need to be
registered upon the creation of the supply `Keeper` so that every time a
`ModuleAccount` calls the allowed functions, the `Keeper` can lookup the
permissions to that specific account and perform or not perform the action.

The available permissions are:

* `Minter`: allows for a module to mint a specific amount of coins.
* `Burner`: allows for a module to burn a specific amount of coins.
* `Staking`: allows for a module to delegate and undelegate a specific amount of coins.

## State

The `x/bank` module keeps state of the following primary objects:

1. Account balances
2. Denomination metadata
3. The total supply of all balances
4. Information on which denominations are allowed to be sent.

In addition, the `x/bank` module keeps the following indexes to manage the
aforementioned state:

* Supply Index: `0x0 | byte(denom) -> byte(amount)`
* Denom Metadata Index: `0x1 | byte(denom) -> ProtocolBuffer(Metadata)`
* Balances Index: `0x2 | byte(address length) | []byte(address) | []byte(balance.Denom) -> ProtocolBuffer(balance)`
* Reverse Denomination to Address Index: `0x03 | byte(denom) | 0x00 | []byte(address) -> 0`

## Params

The bank module stores it's params in state with the prefix of `0x05`,
it can be updated with governance or the address with authority.

* Params: `0x05 | ProtocolBuffer(Params)`

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/bank/v1beta1/bank.proto#L12-L23
```

## Keepers

The bank module provides these exported keeper interfaces that can be
passed to other modules that read or update account balances. Modules
should use the least-permissive interface that provides the functionality they
require.

Best practices dictate careful review of `bank` module code to ensure that
permissions are limited in the way that you expect.

### Denied Addresses

The `x/bank` module accepts a map of addresses that are considered blocklisted
from directly and explicitly receiving funds through means such as `MsgSend` and
`MsgMultiSend` and direct API calls like `SendCoinsFromModuleToAccount`.

Typically, these addresses are module accounts. If these addresses receive funds
outside the expected rules of the state machine, invariants are likely to be
broken and could result in a halted network.

By providing the `x/bank` module with a blocklisted set of addresses, an error occurs for the operation if a user or client attempts to directly or indirectly send funds to a blocklisted account, for example, by using [IBC](https://ibc.cosmos.network).

### Common Types

#### Input

An input of a multiparty transfer

```protobuf
// Input models transaction input.
message Input {
  string   address                        = 1;
  repeated cosmos.base.v1beta1.Coin coins = 2;
}
```

#### Output

An output of a multiparty transfer.

```protobuf
// Output models transaction outputs.
message Output {
  string   address                        = 1;
  repeated cosmos.base.v1beta1.Coin coins = 2;
}
```

### BaseKeeper

The base keeper provides full-permission access: the ability to arbitrary modify any account's balance and mint or burn coins.

Restricted permission to mint per module could be achieved by using baseKeeper with `WithMintCoinsRestriction` to give specific restrictions to mint (e.g. only minting certain denom).

```go
// Keeper defines a module interface that facilitates the transfer of coins
// between accounts.
type Keeper interface {
    SendKeeper
    WithMintCoinsRestriction(MintingRestrictionFn) BaseKeeper

    InitGenesis(context.Context, *types.GenesisState)
    ExportGenesis(context.Context) *types.GenesisState

    GetSupply(ctx context.Context, denom string) sdk.Coin
    HasSupply(ctx context.Context, denom string) bool
    GetPaginatedTotalSupply(ctx context.Context, pagination *query.PageRequest) (sdk.Coins, *query.PageResponse, error)
    IterateTotalSupply(ctx context.Context, cb func(sdk.Coin) bool)
    GetDenomMetaData(ctx context.Context, denom string) (types.Metadata, bool)
    HasDenomMetaData(ctx context.Context, denom string) bool
    SetDenomMetaData(ctx context.Context, denomMetaData types.Metadata)
    IterateAllDenomMetaData(ctx context.Context, cb func(types.Metadata) bool)

    SendCoinsFromModuleToAccount(ctx context.Context, senderModule string, recipientAddr sdk.AccAddress, amt sdk.Coins) error
    SendCoinsFromModuleToModule(ctx context.Context, senderModule, recipientModule string, amt sdk.Coins) error
    SendCoinsFromAccountToModule(ctx context.Context, senderAddr sdk.AccAddress, recipientModule string, amt sdk.Coins) error
    DelegateCoinsFromAccountToModule(ctx context.Context, senderAddr sdk.AccAddress, recipientModule string, amt sdk.Coins) error
    UndelegateCoinsFromModuleToAccount(ctx context.Context, senderModule string, recipientAddr sdk.AccAddress, amt sdk.Coins) error
    MintCoins(ctx context.Context, moduleName string, amt sdk.Coins) error
    BurnCoins(ctx context.Context, moduleName string, amt sdk.Coins) error

    DelegateCoins(ctx context.Context, delegatorAddr, moduleAccAddr sdk.AccAddress, amt sdk.Coins) error
    UndelegateCoins(ctx context.Context, moduleAccAddr, delegatorAddr sdk.AccAddress, amt sdk.Coins) error

    // GetAuthority gets the address capable of executing governance proposal messages. Usually the gov module account.
    GetAuthority() string

    types.QueryServer
}
```

### SendKeeper

The send keeper provides access to account balances and the ability to transfer coins between
accounts. The send keeper does not alter the total supply (mint or burn coins).

```go
// SendKeeper defines a module interface that facilitates the transfer of coins
// between accounts without the possibility of creating coins.
type SendKeeper interface {
    ViewKeeper

    AppendSendRestriction(restriction SendRestrictionFn)
    PrependSendRestriction(restriction SendRestrictionFn)
    ClearSendRestriction()

    InputOutputCoins(ctx context.Context, input types.Input, outputs []types.Output) error
    SendCoins(ctx context.Context, fromAddr, toAddr sdk.AccAddress, amt sdk.Coins) error

    GetParams(ctx context.Context) types.Params
    SetParams(ctx context.Context, params types.Params) error

    IsSendEnabledDenom(ctx context.Context, denom string) bool
    SetSendEnabled(ctx context.Context, denom string, value bool)
    SetAllSendEnabled(ctx context.Context, sendEnableds []*types.SendEnabled)
    DeleteSendEnabled(ctx context.Context, denom string)
    IterateSendEnabledEntries(ctx context.Context, cb func(denom string, sendEnabled bool) (stop bool))
    GetAllSendEnabledEntries(ctx context.Context) []types.SendEnabled

    IsSendEnabledCoin(ctx context.Context, coin sdk.Coin) bool
    IsSendEnabledCoins(ctx context.Context, coins ...sdk.Coin) error

    BlockedAddr(addr sdk.AccAddress) bool
}
```

#### Send Restrictions

The `SendKeeper` applies a `SendRestrictionFn` before each transfer of funds.

```golang
// A SendRestrictionFn can restrict sends and/or provide a new receiver address.
type SendRestrictionFn func(ctx context.Context, fromAddr, toAddr sdk.AccAddress, amt sdk.Coins) (newToAddr sdk.AccAddress, err error)
```

After the `SendKeeper` (or `BaseKeeper`) has been created, send restrictions can be added to it using the `AppendSendRestriction` or `PrependSendRestriction` functions.
Both functions compose the provided restriction with any previously provided restrictions.
`AppendSendRestriction` adds the provided restriction to be run after any previously provided send restrictions.
`PrependSendRestriction` adds the restriction to be run before any previously provided send restrictions.
The composition will short-circuit when an error is encountered. I.e. if the first one returns an error, the second is not run.

During `SendCoins`, the send restriction is applied after coins are removed from the from address, but before adding them to the to address.
During `InputOutputCoins`, the send restriction is applied after the input coins are removed and once for each output before the funds are added.

A send restriction function should make use of a custom value in the context to allow bypassing that specific restriction.

For example, in your module's keeper package, you'd define the send restriction function:

```golang
var _ banktypes.SendRestrictionFn = Keeper{}.SendRestrictionFn

func (k Keeper) SendRestrictionFn(ctx context.Context, fromAddr, toAddr sdk.AccAddress, amt sdk.Coins) (sdk.AccAddress, error) {
	// Bypass if the context says to.
	if mymodule.HasBypass(ctx) {
		return toAddr, nil
	}

	// Your custom send restriction logic goes here.
	return nil, errors.New("not implemented")
}
```

The bank keeper should be provided to your keeper's constructor so the send restriction can be added to it:

```golang
func NewKeeper(cdc codec.BinaryCodec, storeKey storetypes.StoreKey, bankKeeper mymodule.BankKeeper) Keeper {
	rv := Keeper{/*...*/}
	bankKeeper.AppendSendRestriction(rv.SendRestrictionFn)
	return rv
}
```

Then, in the `mymodule` package, define the context helpers:

```golang
const bypassKey = "bypass-mymodule-restriction"

// WithBypass returns a new context that will cause the mymodule bank send restriction to be skipped.
func WithBypass(ctx context.Context) context.Context {
	return sdk.UnwrapSDKContext(ctx).WithValue(bypassKey, true)
}

// WithoutBypass returns a new context that will cause the mymodule bank send restriction to not be skipped.
func WithoutBypass(ctx context.Context) context.Context {
	return sdk.UnwrapSDKContext(ctx).WithValue(bypassKey, false)
}

// HasBypass checks the context to see if the mymodule bank send restriction should be skipped.
func HasBypass(ctx context.Context) bool {
	bypassValue := ctx.Value(bypassKey)
	if bypassValue == nil {
		return false
	}
	bypass, isBool := bypassValue.(bool)
	return isBool && bypass
}
```

Now, anywhere where you want to use `SendCoins` or `InputOutputCoins`, but you don't want your send restriction applied:

```golang
func (k Keeper) DoThing(ctx context.Context, fromAddr, toAddr sdk.AccAddress, amt sdk.Coins) error {
	return k.bankKeeper.SendCoins(mymodule.WithBypass(ctx), fromAddr, toAddr, amt)
}
```

### ViewKeeper

The view keeper provides read-only access to account balances. The view keeper does not have balance alteration functionality. All balance lookups are `O(1)`.

```go
// ViewKeeper defines a module interface that facilitates read only access to
// account balances.
type ViewKeeper interface {
    ValidateBalance(ctx context.Context, addr sdk.AccAddress) error
    HasBalance(ctx context.Context, addr sdk.AccAddress, amt sdk.Coin) bool

    GetAllBalances(ctx context.Context, addr sdk.AccAddress) sdk.Coins
    GetAccountsBalances(ctx context.Context) []types.Balance
    GetBalance(ctx context.Context, addr sdk.AccAddress, denom string) sdk.Coin
    LockedCoins(ctx context.Context, addr sdk.AccAddress) sdk.Coins
    SpendableCoins(ctx context.Context, addr sdk.AccAddress) sdk.Coins
    SpendableCoin(ctx context.Context, addr sdk.AccAddress, denom string) sdk.Coin

    IterateAccountBalances(ctx context.Context, addr sdk.AccAddress, cb func(coin sdk.Coin) (stop bool))
    IterateAllBalances(ctx context.Context, cb func(address sdk.AccAddress, coin sdk.Coin) (stop bool))
}
```

## Messages

### MsgSend

Send coins from one address to another.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/bank/v1beta1/tx.proto#L38-L53
```

The message will fail under the following conditions:

* The coins do not have sending enabled
* The `to` address is restricted

### MsgMultiSend

Send coins from one sender and to a series of different address. If any of the receiving addresses do not correspond to an existing account, a new account is created.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/bank/v1beta1/tx.proto#L58-L69
```

The message will fail under the following conditions:

* Any of the coins do not have sending enabled
* Any of the `to` addresses are restricted
* Any of the coins are locked
* The inputs and outputs do not correctly correspond to one another

### MsgUpdateParams

The `bank` module params can be updated through `MsgUpdateParams`, which can be done using governance proposal. The signer will always be the `gov` module account address. 

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/bank/v1beta1/tx.proto#L74-L88
```

The message handling can fail if:

* signer is not the gov module account address.

### MsgSetSendEnabled

Used with the x/gov module to set create/edit SendEnabled entries.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/bank/v1beta1/tx.proto#L96-L117
```

The message will fail under the following conditions:

* The authority is not a bech32 address.
* The authority is not x/gov module's address.
* There are multiple SendEnabled entries with the same Denom.
* One or more SendEnabled entries has an invalid Denom.

## Events

The bank module emits the following events:

### Message Events

#### MsgSend

| Type     | Attribute Key | Attribute Value    |
| -------- | ------------- | ------------------ |
| transfer | recipient     | {recipientAddress} |
| transfer | amount        | {amount}           |
| message  | module        | bank               |
| message  | action        | send               |
| message  | sender        | {senderAddress}    |

#### MsgMultiSend

| Type     | Attribute Key | Attribute Value    |
| -------- | ------------- | ------------------ |
| transfer | recipient     | {recipientAddress} |
| transfer | amount        | {amount}           |
| message  | module        | bank               |
| message  | action        | multisend          |
| message  | sender        | {senderAddress}    |

### Keeper Events

In addition to message events, the bank keeper will produce events when the following methods are called (or any method which ends up calling them)

#### MintCoins

```json
{
  "type": "coinbase",
  "attributes": [
    {
      "key": "minter",
      "value": "{{sdk.AccAddress of the module minting coins}}",
      "index": true
    },
    {
      "key": "amount",
      "value": "{{sdk.Coins being minted}}",
      "index": true
    }
  ]
}
```

```json
{
  "type": "coin_received",
  "attributes": [
    {
      "key": "receiver",
      "value": "{{sdk.AccAddress of the module minting coins}}",
      "index": true
    },
    {
      "key": "amount",
      "value": "{{sdk.Coins being received}}",
      "index": true
    }
  ]
}
```

#### BurnCoins

```json
{
  "type": "burn",
  "attributes": [
    {
      "key": "burner",
      "value": "{{sdk.AccAddress of the module burning coins}}",
      "index": true
    },
    {
      "key": "amount",
      "value": "{{sdk.Coins being burned}}",
      "index": true
    }
  ]
}
```

```json
{
  "type": "coin_spent",
  "attributes": [
    {
      "key": "spender",
      "value": "{{sdk.AccAddress of the module burning coins}}",
      "index": true
    },
    {
      "key": "amount",
      "value": "{{sdk.Coins being burned}}",
      "index": true
    }
  ]
}
```

#### addCoins

```json
{
  "type": "coin_received",
  "attributes": [
    {
      "key": "receiver",
      "value": "{{sdk.AccAddress of the address beneficiary of the coins}}",
      "index": true
    },
    {
      "key": "amount",
      "value": "{{sdk.Coins being received}}",
      "index": true
    }
  ]
}
```

#### subUnlockedCoins/DelegateCoins

```json
{
  "type": "coin_spent",
  "attributes": [
    {
      "key": "spender",
      "value": "{{sdk.AccAddress of the address which is spending coins}}",
      "index": true
    },
    {
      "key": "amount",
      "value": "{{sdk.Coins being spent}}",
      "index": true
    }
  ]
}
```

## Parameters

The bank module contains the following parameters

### SendEnabled

The SendEnabled parameter is now deprecated and not to be use. It is replaced
with state store records.


### DefaultSendEnabled

The default send enabled value controls send transfer capability for all
coin denominations unless specifically included in the array of `SendEnabled`
parameters.

## Client

### CLI

A user can query and interact with the `bank` module using the CLI.

#### Query

The `query` commands allow users to query `bank` state.

```shell
simd query bank --help
```

##### balances

The `balances` command allows users to query account balances by address.

```shell
simd query bank balances [address] [flags]
```

Example:

```shell
simd query bank balances cosmos1..
```

Example Output:

```yml
balances:
- amount: "1000000000"
  denom: stake
pagination:
  next_key: null
  total: "0"
```

##### denom-metadata

The `denom-metadata` command allows users to query metadata for coin denominations. A user can query metadata for a single denomination using the `--denom` flag or all denominations without it.

```shell
simd query bank denom-metadata [flags]
```

Example:

```shell
simd query bank denom-metadata --denom stake
```

Example Output:

```yml
metadata:
  base: stake
  denom_units:
  - aliases:
    - STAKE
    denom: stake
  description: native staking token of simulation app
  display: stake
  name: SimApp Token
  symbol: STK
```

##### total

The `total` command allows users to query the total supply of coins. A user can query the total supply for a single coin using the `--denom` flag or all coins without it.

```shell
simd query bank total [flags]
```

Example:

```shell
simd query bank total --denom stake
```

Example Output:

```yml
amount: "10000000000"
denom: stake
```

##### send-enabled

The `send-enabled` command allows users to query for all or some SendEnabled entries.

```shell
simd query bank send-enabled [denom1 ...] [flags]
```

Example:

```shell
simd query bank send-enabled
```

Example output:

```yml
send_enabled:
- denom: foocoin
  enabled: true
- denom: barcoin
pagination:
  next-key: null
  total: 2 
```

#### Transactions

The `tx` commands allow users to interact with the `bank` module.

```shell
simd tx bank --help
```

##### send

The `send` command allows users to send funds from one account to another.

```shell
simd tx bank send [from_key_or_address] [to_address] [amount] [flags]
```

Example:

```shell
simd tx bank send cosmos1.. cosmos1.. 100stake
```

## gRPC

A user can query the `bank` module using gRPC endpoints.

### Balance

The `Balance` endpoint allows users to query account balance by address for a given denomination.

```shell
cosmos.bank.v1beta1.Query/Balance
```

Example:

```shell
grpcurl -plaintext \
    -d '{"address":"cosmos1..","denom":"stake"}' \
    localhost:9090 \
    cosmos.bank.v1beta1.Query/Balance
```

Example Output:

```json
{
  "balance": {
    "denom": "stake",
    "amount": "1000000000"
  }
}
```

### AllBalances

The `AllBalances` endpoint allows users to query account balance by address for all denominations.

```shell
cosmos.bank.v1beta1.Query/AllBalances
```

Example:

```shell
grpcurl -plaintext \
    -d '{"address":"cosmos1.."}' \
    localhost:9090 \
    cosmos.bank.v1beta1.Query/AllBalances
```

Example Output:

```json
{
  "balances": [
    {
      "denom": "stake",
      "amount": "1000000000"
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```

### DenomMetadata

The `DenomMetadata` endpoint allows users to query metadata for a single coin denomination.

```shell
cosmos.bank.v1beta1.Query/DenomMetadata
```

Example:

```shell
grpcurl -plaintext \
    -d '{"denom":"stake"}' \
    localhost:9090 \
    cosmos.bank.v1beta1.Query/DenomMetadata
```

Example Output:

```json
{
  "metadata": {
    "description": "native staking token of simulation app",
    "denomUnits": [
      {
        "denom": "stake",
        "aliases": [
          "STAKE"
        ]
      }
    ],
    "base": "stake",
    "display": "stake",
    "name": "SimApp Token",
    "symbol": "STK"
  }
}
```

### DenomsMetadata

The `DenomsMetadata` endpoint allows users to query metadata for all coin denominations.

```shell
cosmos.bank.v1beta1.Query/DenomsMetadata
```

Example:

```shell
grpcurl -plaintext \
    localhost:9090 \
    cosmos.bank.v1beta1.Query/DenomsMetadata
```

Example Output:

```json
{
  "metadatas": [
    {
      "description": "native staking token of simulation app",
      "denomUnits": [
        {
          "denom": "stake",
          "aliases": [
            "STAKE"
          ]
        }
      ],
      "base": "stake",
      "display": "stake",
      "name": "SimApp Token",
      "symbol": "STK"
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```

### DenomOwners

The `DenomOwners` endpoint allows users to query metadata for a single coin denomination.

```shell
cosmos.bank.v1beta1.Query/DenomOwners
```

Example:

```shell
grpcurl -plaintext \
    -d '{"denom":"stake"}' \
    localhost:9090 \
    cosmos.bank.v1beta1.Query/DenomOwners
```

Example Output:

```json
{
  "denomOwners": [
    {
      "address": "cosmos1..",
      "balance": {
        "denom": "stake",
        "amount": "5000000000"
      }
    },
    {
      "address": "cosmos1..",
      "balance": {
        "denom": "stake",
        "amount": "5000000000"
      }
    },
  ],
  "pagination": {
    "total": "2"
  }
}
```

### TotalSupply

The `TotalSupply` endpoint allows users to query the total supply of all coins.

```shell
cosmos.bank.v1beta1.Query/TotalSupply
```

Example:

```shell
grpcurl -plaintext \
    localhost:9090 \
    cosmos.bank.v1beta1.Query/TotalSupply
```

Example Output:

```json
{
  "supply": [
    {
      "denom": "stake",
      "amount": "10000000000"
    }
  ],
  "pagination": {
    "total": "1"
  }
}
```

### SupplyOf

The `SupplyOf` endpoint allows users to query the total supply of a single coin.

```shell
cosmos.bank.v1beta1.Query/SupplyOf
```

Example:

```shell
grpcurl -plaintext \
    -d '{"denom":"stake"}' \
    localhost:9090 \
    cosmos.bank.v1beta1.Query/SupplyOf
```

Example Output:

```json
{
  "amount": {
    "denom": "stake",
    "amount": "10000000000"
  }
}
```

### Params

The `Params` endpoint allows users to query the parameters of the `bank` module.

```shell
cosmos.bank.v1beta1.Query/Params
```

Example:

```shell
grpcurl -plaintext \
    localhost:9090 \
    cosmos.bank.v1beta1.Query/Params
```

Example Output:

```json
{
  "params": {
    "defaultSendEnabled": true
  }
}
```

### SendEnabled

The `SendEnabled` enpoints allows users to query the SendEnabled entries of the `bank` module.

Any denominations NOT returned, use the `Params.DefaultSendEnabled` value.

```shell
cosmos.bank.v1beta1.Query/SendEnabled
```

Example:

```shell
grpcurl -plaintext \
    localhost:9090 \
    cosmos.bank.v1beta1.Query/SendEnabled
```

Example Output:

```json
{
  "send_enabled": [
    {
      "denom": "foocoin",
      "enabled": true
    },
    {
      "denom": "barcoin"
    }
  ],
  "pagination": {
    "next-key": null,
    "total": 2
  }
}
```
