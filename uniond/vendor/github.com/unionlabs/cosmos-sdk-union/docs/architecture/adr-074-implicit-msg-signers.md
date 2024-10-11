# ADR 074: Messages with Implicit Signers

## Changelog

* 2024-06-10: Initial draft

## Status

PROPOSED Not Implemented

## Abstract

This ADR introduces a new `MsgV2` standard where the signer of the message is implied by the
credentials of the party sending it, and unlike the current design not part of the message body.
This can be used for both simple inter-module message passing and simpler messages in transactions.

## Context

Historically operations in the SDK have been modelled with the `sdk.Msg` interface and
the account signing the message has to be explicitly extracted from the body of `Msg`s.
Originally this was via a `GetSigners` method on the `sdk.Msg` interface which returned
instances of `sdk.AccAddress` which itself relied on a global variable for decoding
the addresses from bech32 strings. This was a messy situation. In addition, the implementation
for `GetSigners` was different for each `Msg` type and clients would need to do a custom
implementation for each `Msg` type. These were improved somewhat with the introduction of
the `cosmos.msg.v1.signer` protobuf option which allowed for a more standardised way of
defining who the signer of a message was and its implementation in the `x/tx` module which
extracts signers dynamically and allowed removing the dependency on the global bech32
configuration.

Still this design introduces a fair amount of complexity. For instance, inter-module message
passing ([ADR 033](./adr-033-protobuf-inter-module-comm.md)) has been in discussion for years
without much progress and one of the main blockers is figuring out how to properly authenticate
messages in a performant and consistent way. With embedded message signers there will always need
to be a step of extracting the signer and then checking with the module sending is actually
authorized to perform the operation. With dynamic signer extraction, although the system is
more consistent, more performance overhead is introduced. In any case why should an inter-module
message passing system need to do so much conversion, parsing, etc. just to check if a message
is authenticated? In addition, we have the complexity where modules can actually have many valid
addresses. How are we to accommodate this? Should there be a lookup into `x/auth` to check if an
address belongs to a module or not? All of these thorny questions are delaying the delivery of
inter-module message passing because we do not want an implementation that is overly complex.
There are many use cases for inter-module message passing which are still relevant, the most
immediate of which is a more robust denom management system in `x/bank` `v2` which is being explored
in [ADR 071](https://github.com/cosmos/cosmos-sdk/pull/20316).

## Alternatives

Alternatives that have been considered are extending the current `x/tx` signer extraction system
to inter-module message passing as defined in [ADR 033](./adr-033-protobuf-inter-module-comm.md).

## Decision

We have decided to introduce a new `MsgV2` standard whereby the signer of the message is implied
by the credentials of the party sending it. These messages will be distinct from the existing messages
and define new semantics with the understanding that signers are implicit.

In the case of messages passed internally by a module or `x/account` instance, the signer of a message
will simply be the main root address of the module or account sending the message. An interface for
safely passing such messages to the message router will need to be defined.

In the case of messages passed externally in transactions, `MsgV2` instances will need to be wrapped
in a `MsgV2` envelope:
```protobuf
message MsgV2 {
  string signer = 1;
  google.protobuf.Any msg = 2;  
}
```

Because the `cosmos.msg.v1.signer` annotation is required currently, `MsgV2` types should set the message option
`cosmos.msg.v2.is_msg` to `true` instead.

Here is an example comparing a v1 an v2 message:
```protobuf
// v1
message MsgSendV1 {
  option (cosmos.msg.v1.signer) = "from_address";
  string from_address = 1 ;
  string to_address = 2;
  repeated Coin amount = 3;
}

// v2
message MsgSendV2 {
  option (cosmos.msg.v2.is_msg) = true;
  // from address is implied by the signer
  string to_address = 1;
  repeated Coin amount = 2;
}
```

Modules defining handlers for `MsgV2` instances will need to extract the sender from the `context.Context` that is
passed in. An interface in `core` which will be present on the `appmodule.Environment` will be defined for this purpose:
```go
type GetSenderService interface {
  GetSender(ctx context.Context) []byte
}
```

Sender addresses that are returned by the service will be simple `[]byte` slices and any bech32 conversion will be
done by the framework.

## Consequences

### Backwards Compatibility

This design does not depreciate the existing method of embedded signers in `Msg`s and is totally compatible with it.

### Positive

* Allows for a simple inter-module communication design which can be used soon for the `bank` `v2` redesign.
* Allows for simpler client implementations for messages in the future.

### Negative

* There will be two message designs and developers will need to pick between them.

### Neutral

## Further Discussions

Two possible directions that have been proposed are:
1. allowing for the omission of the `cosmos.msg.v2.is_msg` option and assuming any `Msg`s registered that do not include `cosmos.msg.v1.signer` are `MsgV2` instances. The pitfall is that this could be incorrect if `Msg` v1 behavior is actually decided but the user forgot the `cosmos.msg.v1.signer` option.
2. allow `Msg` v1 instances to be wrapped in a `MsgV2` envelope as well to simplify things client-side. In this scenario we would need to either a) check that the signer in the envelope and the signer in the message are the same or b) allow the signer in the message to be empty and then set it inside the state machine before it reaches the module. While this may be easier for some clients, it may introduce unexpected behavior with Ledger signing via Amino JSON or SIGN_MODE_TEXTUAL. 

Both of these are seem as quality of life improvements for some users, but not strictly necessary and could have some pitfalls so further discussion is needed.

## References

* [ADR 033](./adr-033-protobuf-inter-module-comm.md)