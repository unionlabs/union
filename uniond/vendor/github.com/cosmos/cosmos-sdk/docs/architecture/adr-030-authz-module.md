# ADR 030: Authorization Module

## Changelog

* 2019-11-06: Initial Draft
* 2020-10-12: Updated Draft
* 2020-11-13: Accepted
* 2020-05-06: proto API updates, use `sdk.Msg` instead of `sdk.ServiceMsg` (the latter concept was removed from Cosmos SDK)
* 2022-04-20: Updated the `SendAuthorization` proto docs to clarify the `SpendLimit` is a required field. (Generic authorization can be used with bank msg type url to create limit less bank authorization)

## Status

Accepted

## Abstract

This ADR defines the `x/authz` module which allows accounts to grant authorizations to perform actions
on behalf of that account to other accounts.

## Context

The concrete use cases which motivated this module include:

* the desire to delegate the ability to vote on proposals to other accounts besides the account which one has
delegated stake
* "sub-keys" functionality, as originally proposed in [\#4480](https://github.com/cosmos/cosmos-sdk/issues/4480) which
is a term used to describe the functionality provided by this module together with
the `fee_grant` module from [ADR 029](./adr-029-fee-grant-module.md) and the [group module](https://github.com/cosmos/cosmos-sdk/tree/main/x/group).

The "sub-keys" functionality roughly refers to the ability for one account to grant some subset of its capabilities to
other accounts with possibly less robust, but easier to use security measures. For instance, a master account representing
an organization could grant the ability to spend small amounts of the organization's funds to individual employee accounts.
Or an individual (or group) with a multisig wallet could grant the ability to vote on proposals to any one of the member
keys.

The current implementation is based on work done by the [Gaian's team at Hackatom Berlin 2019](https://github.com/cosmos-gaians/cosmos-sdk/tree/hackatom/x/delegation).

## Decision

We will create a module named `authz` which provides functionality for
granting arbitrary privileges from one account (the _granter_) to another account (the _grantee_). Authorizations
must be granted for a particular `Msg` service methods one by one using an implementation
of `Authorization` interface.

### Types

Authorizations determine exactly what privileges are granted. They are extensible
and can be defined for any `Msg` service method even outside of the module where
the `Msg` method is defined. `Authorization`s reference `Msg`s using their TypeURL.

#### Authorization

```go
type Authorization interface {
	proto.Message

	// MsgTypeURL returns the fully-qualified Msg TypeURL (as described in ADR 020),
	// which will process and accept or reject a request.
	MsgTypeURL() string

	// Accept determines whether this grant permits the provided sdk.Msg to be performed, and if
	// so provides an upgraded authorization instance.
	Accept(ctx sdk.Context, msg sdk.Msg) (AcceptResponse, error)

	// ValidateBasic does a simple validation check that
	// doesn't require access to any other information.
	ValidateBasic() error
}

// AcceptResponse instruments the controller of an authz message if the request is accepted
// and if it should be updated or deleted.
type AcceptResponse struct {
	// If Accept=true, the controller can accept and authorization and handle the update.
	Accept bool
	// If Delete=true, the controller must delete the authorization object and release
	// storage resources.
	Delete bool
	// Controller, who is calling Authorization.Accept must check if `Updated != nil`. If yes,
	// it must use the updated version and handle the update on the storage level.
	Updated Authorization
}
```

For example a `SendAuthorization` like this is defined for `MsgSend` that takes
a `SpendLimit` and updates it down to zero:

```go
type SendAuthorization struct {
	// SpendLimit specifies the maximum amount of tokens that can be spent
	// by this authorization and will be updated as tokens are spent. This field is required. (Generic authorization 
	// can be used with bank msg type url to create limit less bank authorization).
	SpendLimit sdk.Coins
}

func (a SendAuthorization) MsgTypeURL() string {
	return sdk.MsgTypeURL(&MsgSend{})
}

func (a SendAuthorization) Accept(ctx sdk.Context, msg sdk.Msg) (authz.AcceptResponse, error) {
	mSend, ok := msg.(*MsgSend)
	if !ok {
		return authz.AcceptResponse{}, sdkerrors.ErrInvalidType.Wrap("type mismatch")
	}
	limitLeft, isNegative := a.SpendLimit.SafeSub(mSend.Amount)
	if isNegative {
		return authz.AcceptResponse{}, sdkerrors.ErrInsufficientFunds.Wrapf("requested amount is more than spend limit")
	}
	if limitLeft.IsZero() {
		return authz.AcceptResponse{Accept: true, Delete: true}, nil
	}

	return authz.AcceptResponse{Accept: true, Delete: false, Updated: &SendAuthorization{SpendLimit: limitLeft}}, nil
}
```

A different type of capability for `MsgSend` could be implemented
using the `Authorization` interface with no need to change the underlying
`bank` module.

##### Small notes on `AcceptResponse`

* The `AcceptResponse.Accept` field will be set to `true` if the authorization is accepted.
However, if it is rejected, the function `Accept` will raise an error (without setting `AcceptResponse.Accept` to `false`).

* The `AcceptResponse.Updated` field will be set to a non-nil value only if there is a real change to the authorization.
If authorization remains the same (as is, for instance, always the case for a [`GenericAuthorization`](#genericauthorization)),
the field will be `nil`.

### `Msg` Service

```protobuf
service Msg {
  // Grant grants the provided authorization to the grantee on the granter's
  // account with the provided expiration time.
  rpc Grant(MsgGrant) returns (MsgGrantResponse);

  // Exec attempts to execute the provided messages using
  // authorizations granted to the grantee. Each message should have only
  // one signer corresponding to the granter of the authorization.
  rpc Exec(MsgExec) returns (MsgExecResponse);

  // Revoke revokes any authorization corresponding to the provided method name on the
  // granter's account that has been granted to the grantee.
  rpc Revoke(MsgRevoke) returns (MsgRevokeResponse);
}

// Grant gives permissions to execute
// the provided method with expiration time.
message Grant {
  google.protobuf.Any       authorization = 1 [(cosmos_proto.accepts_interface) = "cosmos.authz.v1beta1.Authorization"];
  google.protobuf.Timestamp expiration    = 2 [(gogoproto.stdtime) = true, (gogoproto.nullable) = false];
}

message MsgGrant {
  string granter = 1;
  string grantee = 2;

  Grant grant = 3 [(gogoproto.nullable) = false];
}

message MsgExecResponse {
  cosmos.base.abci.v1beta1.Result result = 1;
}

message MsgExec {
  string   grantee                  = 1;
  // Authorization Msg requests to execute. Each msg must implement Authorization interface
  repeated google.protobuf.Any msgs = 2 [(cosmos_proto.accepts_interface) = "cosmos.base.v1beta1.Msg"];;
}
```

### Router Middleware

The `authz` `Keeper` will expose a `DispatchActions` method which allows other modules to send `Msg`s
to the router based on `Authorization` grants:

```go
type Keeper interface {
	// DispatchActions routes the provided msgs to their respective handlers if the grantee was granted an authorization
	// to send those messages by the first (and only) signer of each msg.
    DispatchActions(ctx sdk.Context, grantee sdk.AccAddress, msgs []sdk.Msg) sdk.Result`
}
```

### CLI

#### `tx exec` Method

When a CLI user wants to run a transaction on behalf of another account using `MsgExec`, they
can use the `exec` method. For instance `gaiacli tx gov vote 1 yes --from <grantee> --generate-only | gaiacli tx authz exec --send-as <granter> --from <grantee>`
would send a transaction like this:

```go
MsgExec {
  Grantee: mykey,
  Msgs: []sdk.Msg{
    MsgVote {
      ProposalID: 1,
      Voter: cosmos3thsdgh983egh823
      Option: Yes
    }
  }
}
```

#### `tx grant <grantee> <authorization> --from <granter>`

This CLI command will send a `MsgGrant` transaction. `authorization` should be encoded as
JSON on the CLI.

#### `tx revoke <grantee> <method-name> --from <granter>`

This CLI command will send a `MsgRevoke` transaction.

### Built-in Authorizations

#### `SendAuthorization`

```protobuf
// SendAuthorization allows the grantee to spend up to spend_limit coins from
// the granter's account.
message SendAuthorization {
  repeated cosmos.base.v1beta1.Coin spend_limit = 1;
}
```

#### `GenericAuthorization`

```protobuf
// GenericAuthorization gives the grantee unrestricted permissions to execute
// the provided method on behalf of the granter's account.
message GenericAuthorization {
  option (cosmos_proto.implements_interface) = "Authorization";

  // Msg, identified by it's type URL, to grant unrestricted permissions to execute
  string msg = 1;
}
```

## Consequences

### Positive

* Users will be able to authorize arbitrary actions on behalf of their accounts to other
users, improving key management for many use cases
* The solution is more generic than previously considered approaches and the
`Authorization` interface approach can be extended to cover other use cases by
SDK users

### Negative

### Neutral

## References

* Initial Hackatom implementation: https://github.com/cosmos-gaians/cosmos-sdk/tree/hackatom/x/delegation
* Post-Hackatom spec: https://gist.github.com/aaronc/b60628017352df5983791cad30babe56#delegation-module
* B-Harvest subkeys spec: https://github.com/cosmos/cosmos-sdk/issues/4480
