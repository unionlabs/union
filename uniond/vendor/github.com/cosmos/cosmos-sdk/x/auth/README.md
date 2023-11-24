---
sidebar_position: 1
---

# `x/auth`

## Abstract

This document specifies the auth module of the Cosmos SDK.

The auth module is responsible for specifying the base transaction and account types
for an application, since the SDK itself is agnostic to these particulars. It contains
the middlewares, where all basic transaction validity checks (signatures, nonces, auxiliary fields)
are performed, and exposes the account keeper, which allows other modules to read, write, and modify accounts.

This module is used in the Cosmos Hub.

## Contents

* [Concepts](#concepts)
    * [Gas & Fees](#gas--fees)
* [State](#state)
    * [Accounts](#accounts)
* [AnteHandlers](#antehandlers)
* [Keepers](#keepers)
    * [Account Keeper](#account-keeper)
* [Parameters](#parameters)
* [Client](#client)
    * [CLI](#cli)
    * [gRPC](#grpc)
    * [REST](#rest)

## Concepts

**Note:** The auth module is different from the [authz module](../modules/authz/).

The differences are:

* `auth` - authentication of accounts and transactions for Cosmos SDK applications and is responsible for specifying the base transaction and account types.
* `authz` - authorization for accounts to perform actions on behalf of other accounts and enables a granter to grant authorizations to a grantee that allows the grantee to execute messages on behalf of the granter.

### Gas & Fees

Fees serve two purposes for an operator of the network.

Fees limit the growth of the state stored by every full node and allow for
general purpose censorship of transactions of little economic value. Fees
are best suited as an anti-spam mechanism where validators are disinterested in
the use of the network and identities of users.

Fees are determined by the gas limits and gas prices transactions provide, where
`fees = ceil(gasLimit * gasPrices)`. Txs incur gas costs for all state reads/writes,
signature verification, as well as costs proportional to the tx size. Operators
should set minimum gas prices when starting their nodes. They must set the unit
costs of gas in each token denomination they wish to support:

`simd start ... --minimum-gas-prices=0.00001stake;0.05photinos`

When adding transactions to mempool or gossipping transactions, validators check
if the transaction's gas prices, which are determined by the provided fees, meet
any of the validator's minimum gas prices. In other words, a transaction must
provide a fee of at least one denomination that matches a validator's minimum
gas price.

CometBFT does not currently provide fee based mempool prioritization, and fee
based mempool filtering is local to node and not part of consensus. But with
minimum gas prices set, such a mechanism could be implemented by node operators.

Because the market value for tokens will fluctuate, validators are expected to
dynamically adjust their minimum gas prices to a level that would encourage the
use of the network.		

## State

### Accounts

Accounts contain authentication information for a uniquely identified external user of an SDK blockchain,
including public key, address, and account number / sequence number for replay protection. For efficiency,
since account balances must also be fetched to pay fees, account structs also store the balance of a user
as `sdk.Coins`.

Accounts are exposed externally as an interface, and stored internally as
either a base account or vesting account. Module clients wishing to add more
account types may do so.

* `0x01 | Address -> ProtocolBuffer(account)`

#### Account Interface

The account interface exposes methods to read and write standard account information.
Note that all of these methods operate on an account struct conforming to the
interface - in order to write the account to the store, the account keeper will
need to be used.

```go
// AccountI is an interface used to store coins at a given address within state.
// It presumes a notion of sequence numbers for replay protection,
// a notion of account numbers for replay protection for previously pruned accounts,
// and a pubkey for authentication purposes.
//
// Many complex conditions can be used in the concrete struct which implements AccountI.
type AccountI interface {
	proto.Message

	GetAddress() sdk.AccAddress
	SetAddress(sdk.AccAddress) error // errors if already set.

	GetPubKey() crypto.PubKey // can return nil.
	SetPubKey(crypto.PubKey) error

	GetAccountNumber() uint64
	SetAccountNumber(uint64) error

	GetSequence() uint64
	SetSequence(uint64) error

	// Ensure that account implements stringer
	String() string
}
```

##### Base Account

A base account is the simplest and most common account type, which just stores all requisite
fields directly in a struct.

```protobuf
// BaseAccount defines a base account type. It contains all the necessary fields
// for basic account functionality. Any custom account type should extend this
// type for additional functionality (e.g. vesting).
message BaseAccount {
  string address = 1;
  google.protobuf.Any pub_key = 2;
  uint64 account_number = 3;
  uint64 sequence       = 4;
}
```

### Vesting Account

See [Vesting](https://docs.cosmos.network/main/modules/auth/vesting/).

## AnteHandlers

The `x/auth` module presently has no transaction handlers of its own, but does expose the special `AnteHandler`, used for performing basic validity checks on a transaction, such that it could be thrown out of the mempool.
The `AnteHandler` can be seen as a set of decorators that check transactions within the current context, per [ADR 010](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-010-modular-antehandler.md).

Note that the `AnteHandler` is called on both `CheckTx` and `DeliverTx`, as CometBFT proposers presently have the ability to include in their proposed block transactions which fail `CheckTx`.

### Decorators

The auth module provides `AnteDecorator`s that are recursively chained together into a single `AnteHandler` in the following order:

* `SetUpContextDecorator`: Sets the `GasMeter` in the `Context` and wraps the next `AnteHandler` with a defer clause to recover from any downstream `OutOfGas` panics in the `AnteHandler` chain to return an error with information on gas provided and gas used.

* `RejectExtensionOptionsDecorator`: Rejects all extension options which can optionally be included in protobuf transactions.

* `MempoolFeeDecorator`: Checks if the `tx` fee is above local mempool `minFee` parameter during `CheckTx`.

* `ValidateBasicDecorator`: Calls `tx.ValidateBasic` and returns any non-nil error.

* `TxTimeoutHeightDecorator`: Check for a `tx` height timeout.

* `ValidateMemoDecorator`: Validates `tx` memo with application parameters and returns any non-nil error.

* `ConsumeGasTxSizeDecorator`: Consumes gas proportional to the `tx` size based on application parameters.

* `DeductFeeDecorator`: Deducts the `FeeAmount` from first signer of the `tx`. If the `x/feegrant` module is enabled and a fee granter is set, it deducts fees from the fee granter account.

* `SetPubKeyDecorator`: Sets the pubkey from a `tx`'s signers that does not already have its corresponding pubkey saved in the state machine and in the current context.

* `ValidateSigCountDecorator`: Validates the number of signatures in `tx` based on app-parameters.

* `SigGasConsumeDecorator`: Consumes parameter-defined amount of gas for each signature. This requires pubkeys to be set in context for all signers as part of `SetPubKeyDecorator`.

* `SigVerificationDecorator`: Verifies all signatures are valid. This requires pubkeys to be set in context for all signers as part of `SetPubKeyDecorator`.

* `IncrementSequenceDecorator`: Increments the account sequence for each signer to prevent replay attacks.

## Keepers

The auth module only exposes one keeper, the account keeper, which can be used to read and write accounts.

### Account Keeper

Presently only one fully-permissioned account keeper is exposed, which has the ability to both read and write
all fields of all accounts, and to iterate over all stored accounts.

```go
// AccountKeeperI is the interface contract that x/auth's keeper implements.
type AccountKeeperI interface {
	// Return a new account with the next account number and the specified address. Does not save the new account to the store.
	NewAccountWithAddress(sdk.Context, sdk.AccAddress) types.AccountI

	// Return a new account with the next account number. Does not save the new account to the store.
	NewAccount(sdk.Context, types.AccountI) types.AccountI

	// Check if an account exists in the store.
	HasAccount(sdk.Context, sdk.AccAddress) bool

	// Retrieve an account from the store.
	GetAccount(sdk.Context, sdk.AccAddress) types.AccountI

	// Set an account in the store.
	SetAccount(sdk.Context, types.AccountI)

	// Remove an account from the store.
	RemoveAccount(sdk.Context, types.AccountI)

	// Iterate over all accounts, calling the provided function. Stop iteration when it returns true.
	IterateAccounts(sdk.Context, func(types.AccountI) bool)

	// Fetch the public key of an account at a specified address
	GetPubKey(sdk.Context, sdk.AccAddress) (crypto.PubKey, error)

	// Fetch the sequence of an account at a specified address.
	GetSequence(sdk.Context, sdk.AccAddress) (uint64, error)

	// Fetch the next account number, and increment the internal counter.
	NextAccountNumber(sdk.Context) uint64
}
```

## Parameters

The auth module contains the following parameters:

| Key                    | Type            | Example |
| ---------------------- | --------------- | ------- |
| MaxMemoCharacters      |      uint64     | 256     |
| TxSigLimit             |      uint64     | 7       |
| TxSizeCostPerByte      |      uint64     | 10      |
| SigVerifyCostED25519   |      uint64     | 590     |
| SigVerifyCostSecp256k1 |      uint64     | 1000    |

## Client

### CLI

A user can query and interact with the `auth` module using the CLI.

### Query

The `query` commands allow users to query `auth` state.

```bash
simd query auth --help
```

#### account

The `account` command allow users to query for an account by it's address.

```bash
simd query auth account [address] [flags]
```

Example:

```bash
simd query auth account cosmos1...
```

Example Output:

```bash
'@type': /cosmos.auth.v1beta1.BaseAccount
account_number: "0"
address: cosmos1zwg6tpl8aw4rawv8sgag9086lpw5hv33u5ctr2
pub_key:
  '@type': /cosmos.crypto.secp256k1.PubKey
  key: ApDrE38zZdd7wLmFS9YmqO684y5DG6fjZ4rVeihF/AQD
sequence: "1"
```

#### accounts

The `accounts` command allow users to query all the available accounts.

```bash
simd query auth accounts [flags]
```

Example:

```bash
simd query auth accounts
```

Example Output:

```bash
accounts:
- '@type': /cosmos.auth.v1beta1.BaseAccount
  account_number: "0"
  address: cosmos1zwg6tpl8aw4rawv8sgag9086lpw5hv33u5ctr2
  pub_key:
    '@type': /cosmos.crypto.secp256k1.PubKey
    key: ApDrE38zZdd7wLmFS9YmqO684y5DG6fjZ4rVeihF/AQD
  sequence: "1"
- '@type': /cosmos.auth.v1beta1.ModuleAccount
  base_account:
    account_number: "8"
    address: cosmos1yl6hdjhmkf37639730gffanpzndzdpmhwlkfhr
    pub_key: null
    sequence: "0"
  name: transfer
  permissions:
  - minter
  - burner
- '@type': /cosmos.auth.v1beta1.ModuleAccount
  base_account:
    account_number: "4"
    address: cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh
    pub_key: null
    sequence: "0"
  name: bonded_tokens_pool
  permissions:
  - burner
  - staking
- '@type': /cosmos.auth.v1beta1.ModuleAccount
  base_account:
    account_number: "5"
    address: cosmos1tygms3xhhs3yv487phx3dw4a95jn7t7lpm470r
    pub_key: null
    sequence: "0"
  name: not_bonded_tokens_pool
  permissions:
  - burner
  - staking
- '@type': /cosmos.auth.v1beta1.ModuleAccount
  base_account:
    account_number: "6"
    address: cosmos10d07y265gmmuvt4z0w9aw880jnsr700j6zn9kn
    pub_key: null
    sequence: "0"
  name: gov
  permissions:
  - burner
- '@type': /cosmos.auth.v1beta1.ModuleAccount
  base_account:
    account_number: "3"
    address: cosmos1jv65s3grqf6v6jl3dp4t6c9t9rk99cd88lyufl
    pub_key: null
    sequence: "0"
  name: distribution
  permissions: []
- '@type': /cosmos.auth.v1beta1.BaseAccount
  account_number: "1"
  address: cosmos147k3r7v2tvwqhcmaxcfql7j8rmkrlsemxshd3j
  pub_key: null
  sequence: "0"
- '@type': /cosmos.auth.v1beta1.ModuleAccount
  base_account:
    account_number: "7"
    address: cosmos1m3h30wlvsf8llruxtpukdvsy0km2kum8g38c8q
    pub_key: null
    sequence: "0"
  name: mint
  permissions:
  - minter
- '@type': /cosmos.auth.v1beta1.ModuleAccount
  base_account:
    account_number: "2"
    address: cosmos17xpfvakm2amg962yls6f84z3kell8c5lserqta
    pub_key: null
    sequence: "0"
  name: fee_collector
  permissions: []
pagination:
  next_key: null
  total: "0"
```

#### params

The `params` command allow users to query the current auth parameters.

```bash
simd query auth params [flags]
```

Example:

```bash
simd query auth params
```

Example Output:

```bash
max_memo_characters: "256"
sig_verify_cost_ed25519: "590"
sig_verify_cost_secp256k1: "1000"
tx_sig_limit: "7"
tx_size_cost_per_byte: "10"
```

### Transactions

The `auth` module supports transactions commands to help you with signing and more. Compared to other modules you can access directly the `auth` module transactions commands using the only `tx` command.

Use directly the `--help` flag to get more information about the `tx` command.

```bash
simd tx --help
```

#### `sign`

The `sign` command allows users to sign transactions that was generated offline.

```bash
simd tx sign tx.json --from $ALICE > tx.signed.json
```

The result is a signed transaction that can be broadcasted to the network thanks to the broadcast command.

More information about the `sign` command can be found running `simd tx sign --help`.

#### `sign-batch`

The `sign-batch` command allows users to sign multiples offline generated transactions.
The transactions can be in one file, with one tx per line, or in multiple files.

```bash
simd tx sign txs.json --from $ALICE > tx.signed.json
```

or

```bash 
simd tx sign tx1.json tx2.json tx3.json --from $ALICE > tx.signed.json
```

The result is multiples signed transactions. For combining the signed transactions into one transactions, use the `--append` flag.

More information about the `sign-batch` command can be found running `simd tx sign-batch --help`.

#### `multi-sign`

The `multi-sign` command allows users to sign transactions that was generated offline by a multisig account.

```bash
simd tx multisign transaction.json k1k2k3 k1sig.json k2sig.json k3sig.json
```

Where `k1k2k3` is the multisig account address, `k1sig.json` is the signature of the first signer, `k2sig.json` is the signature of the second signer, and `k3sig.json` is the signature of the third signer.

More information about the `multi-sign` command can be found running `simd tx multi-sign --help`.

#### `multisign-batch`

The `multisign-batch` works the same way as `sign-batch`, but for multisig accounts.
With the difference that the `multisign-batch` command requires all transactions to be in one file, and the `--append` flag does not exist.

More information about the `multisign-batch` command can be found running `simd tx multisign-batch --help`.

#### `validate-signatures`

The `validate-signatures` command allows users to validate the signatures of a signed transaction.

```bash
$ simd tx validate-signatures tx.signed.json
Signers:
  0: cosmos1l6vsqhh7rnwsyr2kyz3jjg3qduaz8gwgyl8275

Signatures:
  0: cosmos1l6vsqhh7rnwsyr2kyz3jjg3qduaz8gwgyl8275                      [OK]
```

More information about the `validate-signatures` command can be found running `simd tx validate-signatures --help`.

#### `broadcast`

The `broadcast` command allows users to broadcast a signed transaction to the network.

```bash
simd tx broadcast tx.signed.json
```

More information about the `broadcast` command can be found running `simd tx broadcast --help`.


### gRPC

A user can query the `auth` module using gRPC endpoints.

#### Account

The `account` endpoint allow users to query for an account by it's address.

```bash
cosmos.auth.v1beta1.Query/Account
```

Example:

```bash
grpcurl -plaintext \
    -d '{"address":"cosmos1.."}' \
    localhost:9090 \
    cosmos.auth.v1beta1.Query/Account
```

Example Output:

```bash
{
  "account":{
    "@type":"/cosmos.auth.v1beta1.BaseAccount",
    "address":"cosmos1zwg6tpl8aw4rawv8sgag9086lpw5hv33u5ctr2",
    "pubKey":{
      "@type":"/cosmos.crypto.secp256k1.PubKey",
      "key":"ApDrE38zZdd7wLmFS9YmqO684y5DG6fjZ4rVeihF/AQD"
    },
    "sequence":"1"
  }
}
```

#### Accounts

The `accounts` endpoint allow users to query all the available accounts.

```bash
cosmos.auth.v1beta1.Query/Accounts
```

Example:

```bash
grpcurl -plaintext \
    localhost:9090 \
    cosmos.auth.v1beta1.Query/Accounts
```

Example Output:

```bash
{
   "accounts":[
      {
         "@type":"/cosmos.auth.v1beta1.BaseAccount",
         "address":"cosmos1zwg6tpl8aw4rawv8sgag9086lpw5hv33u5ctr2",
         "pubKey":{
            "@type":"/cosmos.crypto.secp256k1.PubKey",
            "key":"ApDrE38zZdd7wLmFS9YmqO684y5DG6fjZ4rVeihF/AQD"
         },
         "sequence":"1"
      },
      {
         "@type":"/cosmos.auth.v1beta1.ModuleAccount",
         "baseAccount":{
            "address":"cosmos1yl6hdjhmkf37639730gffanpzndzdpmhwlkfhr",
            "accountNumber":"8"
         },
         "name":"transfer",
         "permissions":[
            "minter",
            "burner"
         ]
      },
      {
         "@type":"/cosmos.auth.v1beta1.ModuleAccount",
         "baseAccount":{
            "address":"cosmos1fl48vsnmsdzcv85q5d2q4z5ajdha8yu34mf0eh",
            "accountNumber":"4"
         },
         "name":"bonded_tokens_pool",
         "permissions":[
            "burner",
            "staking"
         ]
      },
      {
         "@type":"/cosmos.auth.v1beta1.ModuleAccount",
         "baseAccount":{
            "address":"cosmos1tygms3xhhs3yv487phx3dw4a95jn7t7lpm470r",
            "accountNumber":"5"
         },
         "name":"not_bonded_tokens_pool",
         "permissions":[
            "burner",
            "staking"
         ]
      },
      {
         "@type":"/cosmos.auth.v1beta1.ModuleAccount",
         "baseAccount":{
            "address":"cosmos10d07y265gmmuvt4z0w9aw880jnsr700j6zn9kn",
            "accountNumber":"6"
         },
         "name":"gov",
         "permissions":[
            "burner"
         ]
      },
      {
         "@type":"/cosmos.auth.v1beta1.ModuleAccount",
         "baseAccount":{
            "address":"cosmos1jv65s3grqf6v6jl3dp4t6c9t9rk99cd88lyufl",
            "accountNumber":"3"
         },
         "name":"distribution"
      },
      {
         "@type":"/cosmos.auth.v1beta1.BaseAccount",
         "accountNumber":"1",
         "address":"cosmos147k3r7v2tvwqhcmaxcfql7j8rmkrlsemxshd3j"
      },
      {
         "@type":"/cosmos.auth.v1beta1.ModuleAccount",
         "baseAccount":{
            "address":"cosmos1m3h30wlvsf8llruxtpukdvsy0km2kum8g38c8q",
            "accountNumber":"7"
         },
         "name":"mint",
         "permissions":[
            "minter"
         ]
      },
      {
         "@type":"/cosmos.auth.v1beta1.ModuleAccount",
         "baseAccount":{
            "address":"cosmos17xpfvakm2amg962yls6f84z3kell8c5lserqta",
            "accountNumber":"2"
         },
         "name":"fee_collector"
      }
   ],
   "pagination":{
      "total":"9"
   }
}
```

#### Params

The `params` endpoint allow users to query the current auth parameters.

```bash
cosmos.auth.v1beta1.Query/Params
```

Example:

```bash
grpcurl -plaintext \
    localhost:9090 \
    cosmos.auth.v1beta1.Query/Params
```

Example Output:

```bash
{
  "params": {
    "maxMemoCharacters": "256",
    "txSigLimit": "7",
    "txSizeCostPerByte": "10",
    "sigVerifyCostEd25519": "590",
    "sigVerifyCostSecp256k1": "1000"
  }
}
```

### REST

A user can query the `auth` module using REST endpoints.

#### Account

The `account` endpoint allow users to query for an account by it's address.

```bash
/cosmos/auth/v1beta1/account?address={address}
```

#### Accounts

The `accounts` endpoint allow users to query all the available accounts.

```bash
/cosmos/auth/v1beta1/accounts
```

#### Params

The `params` endpoint allow users to query the current auth parameters.

```bash
/cosmos/auth/v1beta1/params
```
