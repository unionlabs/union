---
sidebar_position: 1
---

# `x/authz`

## Abstract

`x/authz` is an implementation of a Cosmos SDK module, per [ADR 30](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-030-authz-module.md), that allows
granting arbitrary privileges from one account (the granter) to another account (the grantee). Authorizations must be granted for a particular Msg service method one by one using an implementation of the `Authorization` interface.

## Contents

* [Concepts](#concepts)
    * [Authorization and Grant](#authorization-and-grant)
    * [Built-in Authorizations](#built-in-authorizations)
    * [Gas](#gas)
* [State](#state)
    * [Grant](#grant)
    * [GrantQueue](#grantqueue)
* [Messages](#messages)
    * [MsgGrant](#msggrant)
    * [MsgRevoke](#msgrevoke)
    * [MsgRevokeAll](#msgrevokeall)
    * [MsgExec](#msgexec)
    * [MsgPruneExpiredGrants](#msgpruneexpiredgrants)
* [Events](#events)
* [Client](#client)
    * [CLI](#cli)
    * [gRPC](#grpc)
    * [REST](#rest)

## Concepts

### Authorization and Grant

The `x/authz` module defines interfaces and messages grant authorizations to perform actions
on behalf of one account to other accounts. The design is defined in the [ADR 030](https://github.com/cosmos/cosmos-sdk/blob/main/docs/architecture/adr-030-authz-module.md).

A *grant* is an allowance to execute a Msg by the grantee on behalf of the granter.
Authorization is an interface that must be implemented by a concrete authorization logic to validate and execute grants. Authorizations are extensible and can be defined for any Msg service method, even if the Msg method is defined outside of the module. See the `SendAuthorization` example in the next section for more details.

**Note:** The authz module is different from the [auth (authentication)](../modules/auth/) module, which is responsible for specifying the base transaction and account types.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/authz/authorizations.go#L14-L28
```

### Built-in Authorizations

The Cosmos SDK `x/authz` module comes with following authorization types:

#### GenericAuthorization

`GenericAuthorization` implements the `Authorization` interface that gives unrestricted permission to execute the provided Msg on behalf of granter's account.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/authz/proto/cosmos/authz/v1beta1/authz.proto#L14-L22
```

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/authz/generic_authorization.go#L18-L34
```

* `msg` stores Msg type URL.

#### SendAuthorization

`SendAuthorization` implements the `Authorization` interface for the `cosmos.bank.v1beta1.MsgSend` Msg.

* It takes a (positive) `SpendLimit` that specifies the maximum amount of tokens the grantee can spend. The `SpendLimit` is updated as the tokens are spent.
* It takes an (optional) `AllowList` that specifies to which addresses a grantee can send token.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/bank/proto/cosmos/bank/v1beta1/authz.proto#L11-L29
```

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/bank/types/send_authorization.go#L33-L73
```

* `spend_limit` keeps track of how many coins are left in the authorization.
* `allow_list` specifies an optional list of addresses to whom the grantee can send tokens on behalf of the granter.

#### StakeAuthorization

`StakeAuthorization` implements the `Authorization` interface for messages in the [staking module](https://docs.cosmos.network/main/build/modules/staking). It takes an `AuthorizationType` to specify whether you want to authorize delegation, undelegation, redelegation or cancel unbonding delegation, each of which must be authorized separately. It also takes an optional `MaxTokens` that keeps track of a limit to the amount of tokens that can be delegated/undelegated/redelegated. If left empty, the amount is unlimited. Additionally, this Msg takes an `AllowList` or a `DenyList`, enabling you to specify which validators the grantee can or cannot stake with.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/staking/proto/cosmos/staking/v1beta1/authz.proto#L11-L34
```

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/staking/types/authz.go#L78-L166
```

### Gas

To prevent DoS attacks, granting `StakeAuthorization`s with `x/authz` incurs gas. `StakeAuthorization` allows you to authorize another account to delegate, undelegate, or redelegate tokens to validators. The granter can define a list of validators for which they allow or deny delegations. The Cosmos SDK then iterates over these lists and charge 10 gas for each validator included in both lists.

Since the state maintains a list of granter-grantee pairs with same expiration, we iterate over this list to remove the grant from the list (in case of any revoke of particular `msgType`), charging 20 gas for each iteration.

## State

### Grant

Grants are identified by combining granter address (the address bytes of the granter), grantee address (the address bytes of the grantee) and Authorization type (its type URL). Hence we only allow one grant for the (granter, grantee, Authorization) triple.

* Grant: `0x01 | granter_address_len (1 byte) | granter_address_bytes | grantee_address_len (1 byte) | grantee_address_bytes |  msgType_bytes -> ProtocolBuffer(AuthorizationGrant)`

The grant object encapsulates an `Authorization` type and an expiration timestamp:

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/authz/proto/cosmos/authz/v1beta1/authz.proto#L24-L32
```

### GrantQueue

We are maintaining a queue for authz pruning. Whenever a grant is created, an item will be added to `GrantQueue` with a key of expiration, granter, grantee.

In `EndBlock` (which runs for every block) we continuously check and prune the expired grants by forming a prefix key with current blocktime that passed the stored expiration in `GrantQueue`, we iterate through all the matched records from `GrantQueue` and delete maximum of 200 grants from the `GrantQueue` & `Grant`s store for each run.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/authz/keeper/keeper.go#L479-L520
```

* GrantQueue: `0x02 | expiration_bytes | granter_address_len (1 byte) | granter_address_bytes | grantee_address_len (1 byte) | grantee_address_bytes -> ProtocolBuffer(GrantQueueItem)`

The `expiration_bytes` are the expiration date in UTC with the format `"2006-01-02T15:04:05.000000000"`.

```go reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/authz/keeper/keys.go#L84-L100
```

The `GrantQueueItem` object contains the list of type urls between granter and grantee that expire at the time indicated in the key.

## Messages

In this section we describe the processing of messages for the authz module.

### MsgGrant

An authorization grant is created using the `MsgGrant` message.
If there is already a grant for the `(granter, grantee, Authorization)` triple, then the new grant overwrites the previous one. To update or extend an existing grant, a new grant with the same `(granter, grantee, Authorization)` triple should be created.

An authorization grant for authz `MsgGrant` is not allowed and will return an error. This is for preventing user from accidentally authorizing their entire account to a different account.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/authz/proto/cosmos/authz/v1beta1/tx.proto#L45-L55
```

The message handling should fail if:

* both granter and grantee have the same address.
* provided `Expiration` time is less than current unix timestamp (but a grant will be created if no `expiration` time is provided since `expiration` is optional).
* provided `Grant.Authorization` is not implemented.
* `Authorization.MsgTypeURL()` is not defined in the router (there is no defined handler in the app router to handle that Msg types).

### MsgRevoke

A grant can be removed with the `MsgRevoke` message.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/authz/proto/cosmos/authz/v1beta1/tx.proto#L79-L88
```

The message handling should fail if:

* both granter and grantee have the same address.
* provided `MsgTypeUrl` is empty.

NOTE: The `MsgExec` message removes a grant if the grant has expired.

### MsgRevokeAll

The `MsgRevokeAll` message revokes all grants issued by the specified granter. This is useful for quickly removing all authorizations granted by a single granter without specifying individual message types or grantees.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/authz/proto/cosmos/authz/v1beta1/tx.proto#L93-L100
```

The message handling should fail if:

* the `granter` address is not provided or invalid.
* the `granter` does not have any active grants.

### MsgExec

When a grantee wants to execute a transaction on behalf of a granter, they must send `MsgExec`.

```protobuf reference
https://github.com/cosmos/cosmos-sdk/blob/v0.52.0-beta.1/x/authz/proto/cosmos/authz/v1beta1/tx.proto#L60-L72
```

The message handling should fail if:

* provided `Authorization` is not implemented.
* grantee doesn't have permission to run the transaction.
* if granted authorization is expired.

### MsgPruneExpiredGrants

Message that clean up 75 expired grants. A user has no benefit sending this transaction, it is only used by the chain to clean up expired grants.

## Events

The authz module emits proto events defined in [the Protobuf reference](https://buf.build/cosmos/cosmos-sdk/docs/main/cosmos.authz.v1beta1#cosmos.authz.v1beta1.EventGrant).

## Client

### CLI

A user can query and interact with the `authz` module using the CLI.

#### Query

The `query` commands allow users to query `authz` state.

```bash
simd query authz --help
```

##### grants

The `grants` command allows users to query grants for a granter-grantee pair. If the message type URL is set, it selects grants only for that message type.

```bash
simd query authz grants [granter-addr] [grantee-addr] [msg-type-url]? [flags]
```

Example:

```bash
simd query authz grants cosmos1.. cosmos1.. /cosmos.bank.v1beta1.MsgSend
```

Example Output:

```bash
grants:
- authorization:
    '@type': /cosmos.bank.v1beta1.SendAuthorization
    spend_limit:
    - amount: "100"
      denom: stake
  expiration: "2022-01-01T00:00:00Z"
pagination: null
```

#### Transactions

The `tx` commands allow users to interact with the `authz` module.

```bash
simd tx authz --help
```

##### exec

The `exec` command allows a grantee to execute a transaction on behalf of granter.

```bash
  simd tx authz exec [tx-json-file] --from [grantee] [flags]
```

Example:

```bash
simd tx authz exec tx.json --from=cosmos1..
```

##### grant

The `grant` command allows a granter to grant an authorization to a grantee.

```bash
simd tx authz grant <grantee> <authorization_type="send"|"generic"|"delegate"|"unbond"|"redelegate"> --from <granter> [flags]
```
-  The `send` authorization_type refers to the built-in `SendAuthorization` type. The custom flags available are `spend-limit` (required) and `allow-list` (optional) , documented [here](#SendAuthorization)

Example:

```bash
    simd tx authz grant cosmos1.. send --spend-limit=100stake --allow-list=cosmos1...,cosmos2... --from=cosmos1..
```
- The `generic` authorization_type refers to the built-in `GenericAuthorization` type. The custom flag available is `msg-type` ( required) documented [here](#GenericAuthorization).

> Note: `msg-type` is any valid Cosmos SDK `Msg` type url.

Example:
```bash
    simd tx authz grant cosmos1.. generic --msg-type=/cosmos.bank.v1beta1.MsgSend --from=cosmos1..
```
- The `delegate`,`unbond`,`redelegate` authorization_types refer to the built-in `StakeAuthorization` type. The custom flags available are `spend-limit` (optional), `allowed-validators` (optional) and `deny-validators` (optional) documented  [here](#StakeAuthorization).
> Note: `allowed-validators` and `deny-validators` cannot both be empty. `spend-limit` represents the `MaxTokens`

Example:

```bash
simd tx authz grant cosmos1.. delegate --spend-limit=100stake --allowed-validators=cosmos...,cosmos... --deny-validators=cosmos... --from=cosmos1..
```

##### revoke

The `revoke` command allows a granter to revoke an authorization from a grantee.

```bash
simd tx authz revoke [grantee] [msg-type-url] --from=[granter] [flags]
```

Example:

```bash
simd tx authz revoke cosmos1.. /cosmos.bank.v1beta1.MsgSend --from=cosmos1..
```

##### revoke-all

The `revoke-all` command allows a granter to revoke all authorizations created by the granter.

```bash
simd tx authz revoke-all --from=[granter] [flags]
```

Example:

```bash
simd tx authz revoke-all --from=cosmos1..
```

### gRPC

A user can query the `authz` module using gRPC endpoints.

#### Grants

The `Grants` endpoint allows users to query grants for a granter-grantee pair. If the message type URL is set, it selects grants only for that message type.

```bash
cosmos.authz.v1beta1.Query/Grants
```

Example:

```bash
grpcurl -plaintext \
    -d '{"granter":"cosmos1..","grantee":"cosmos1..","msg_type_url":"/cosmos.bank.v1beta1.MsgSend"}' \
    localhost:9090 \
    cosmos.authz.v1beta1.Query/Grants
```

### REST

A user can query the `authz` module using REST endpoints.

```bash
/cosmos/authz/v1beta1/grants
```

Example:

```bash
curl "localhost:1317/cosmos/authz/v1beta1/grants?granter=cosmos1..&grantee=cosmos1..&msg_type_url=/cosmos.bank.v1beta1.MsgSend"
```
