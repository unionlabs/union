---
sidebar_position: 1
---

# `x/crisis`

## Overview

The crisis module halts the blockchain under the circumstance that a blockchain
invariant is broken. Invariants can be registered with the application during the
application initialization process.

## Contents

* [State](#state)
* [Messages](#messages)
* [Events](#events)
* [Parameters](#parameters)
* [Client](#client)
    * [CLI](#cli)

## State

### ConstantFee

Due to the anticipated large gas cost requirement to verify an invariant (and
potential to exceed the maximum allowable block gas limit) a constant fee is
used instead of the standard gas consumption method. The constant fee is
intended to be larger than the anticipated gas cost of running the invariant
with the standard gas consumption method.

The ConstantFee param is stored in the module params state with the prefix of `0x01`,
it can be updated with governance or the address with authority.

* Params: `mint/params -> legacy_amino(sdk.Coin)`

## Messages

In this section we describe the processing of the crisis messages and the
corresponding updates to the state.

### MsgVerifyInvariant

Blockchain invariants can be checked using the `MsgVerifyInvariant` message.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.47.0-rc1/proto/cosmos/crisis/v1beta1/tx.proto#L26-L42
```

This message is expected to fail if:

* the sender does not have enough coins for the constant fee
* the invariant route is not registered

This message checks the invariant provided, and if the invariant is broken it
panics, halting the blockchain. If the invariant is broken, the constant fee is
never deducted as the transaction is never committed to a block (equivalent to
being refunded). However, if the invariant is not broken, the constant fee will
not be refunded.

## Events

The crisis module emits the following events:

### Handlers

#### MsgVerifyInvariance

| Type      | Attribute Key | Attribute Value  |
|-----------|---------------|------------------|
| invariant | route         | {invariantRoute} |
| message   | module        | crisis           |
| message   | action        | verify_invariant |
| message   | sender        | {senderAddress}  |

## Parameters

The crisis module contains the following parameters:

| Key         | Type          | Example                           |
|-------------|---------------|-----------------------------------|
| ConstantFee | object (coin) | {"denom":"uatom","amount":"1000"} |

## Client

### CLI

A user can query and interact with the `crisis` module using the CLI.

#### Transactions

The `tx` commands allow users to interact with the `crisis` module.

```bash
simd tx crisis --help
```

##### invariant-broken

The `invariant-broken` command submits proof when an invariant was broken to halt the chain

```bash
simd tx crisis invariant-broken [module-name] [invariant-route] [flags]
```

Example:

```bash
simd tx crisis invariant-broken bank total-supply --from=[keyname or address]
```
