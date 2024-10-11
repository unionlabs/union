# RFC 006: Handlers

## Changelog

* January 26, 2024: Initialized

## Background

The Cosmos SDK has a very powerful and flexible module system that has been tested
and proven to be very good in production. The design of how messages are handled
is built around Protobuf services and gRPC. This design was proposed and implemented
during a time when we migrated from Amino to Protocol Buffers. This design has
fulfilled the needs of users today. While this design is useful it has caused a
elevated learning curve to be adopted by users. Today, these services are the
only way to write a module. This RFC proposes a new design that simplifies the
design and enables new use cases we are seeing today.

Taking a step back, we have seen the emergence of rollups and proving technologies.
These technologies enable new use cases and new methods of achieving various goals.
When we look at things like proving we look to [TinyGo](https://TinyGo.org/). When
we have attempted to use TinyGo with existing modules we have run into a hiccup,
the use of [gRPC](https://github.com/TinyGo-org/TinyGo/issues/2814) within modules.
This has led us to look at a design which would allow the usage of TinyGo and
other technologies.

We looked at TinyGo for our first target in order to compile down to a 32 bit environment which could be used with
things like [Risc-0](https://www.risczero.com/), [Fluent](https://fluentlabs.xyz/) and other technologies. When speaking with the teams behind these technologies
we found that they were interested in using the Cosmos SDK but were unable to due to being unable to use TinyGo or the
Cosmos SDK go code in a 32 bit environment.

The Cosmos SDK team has been hard at work over the last few months designing and implementing a modular core layer, with
the idea that proving can be enabled later on. This design allows us to push the design of what can be done with the
Cosmos SDK to the next level. In the future when we have proving tools and technologies integrated parts of the new core
layer will be able to be used in conjunction with proving technologies without the need to rewrite the stack.


## Proposal

This proposal is around enabling modules to be compiled to an environment in which they can be used with TinyGo and/or
different proving technologies.

> Note the usage of handlers in modules is optional, modules can still use the existing design. This design is meant to
> be a new way to write modules, with proving in mind, and is not meant to replace the existing design.

This proposal is for server/v2. Baseapp will continue to work in the same way as it does today.

### Pre and Post Message Handlers

In the Cosmos SDK, there exists hooks on messages and execution of function calls. Separating the two we will focus on
message hooks. When a message is implemented it can be unknown if others will use the module and if a message will need
hooks. When hooks are needed before or after a message, users are required to fork the module. This is not ideal as it
leads to a lot of forks of modules and a lot of code duplication.

Pre and Post message handlers solve this issue. Where we allow modules to register listeners for messages in order to
execute something before and/or after the message. Although hooks can be bypassed by doing keeper function calls, we can
assume that as we shift the communication design of the SDK to use messages instead of keeper calls it will be safe to
assume that the bypass surface of hooks will reduce to zero.

If an application developer would like to check the sender of funds before the message is executed they can
register a pre message handler. If the message is called by a user the pre message handler will be called with the custom
logic. If the sender is not allowed to send funds the pre-message handler can return an error and the message will not be
executed.

> Note: This is different from the ante-handler and post-handler we have today. These will still exist in the same form.

A module can register handlers for any or all message(s), this allows for modules to be extended without the need to fork.

A module will implement the below for a pre-message hook:

```golang
package core_appmodule

import (
	"context"

	"google.golang.org/protobuf/runtime/protoiface"
)

type PreMsgHandlerRouter interface {
	// RegisterGlobalPreMsgHandler will register a pre msg handler that hooks before any message executes.
	// Handler will be called before ANY message executes.
	RegisterGlobalPreMsgHandler(handler func(ctx context.Context, msg transaction.Msg) error)
	// RegisterPreMsgHandler will register a pre msg handler that hooks before the provided message
	// with the given message name executes. Handler will be called before the message is executed
	// by the module.
	RegisterPreMsgHandler(msgName string, handler func(ctx context.Context, msg transaction.Msg) error)
}

type HasPreMsgHandler interface {
	RegisterPreMsgHandler(router PreMsgHandlerRouter)
}
```

A module will implement the below for a postmessage hook:

```go
package core_appmodule

import (
	"context"

	"google.golang.org/protobuf/runtime/protoiface"
)

type PostMsgHandlerRouter interface {
	// RegisterGlobalPostMsgHandler will register a post msg handler that hooks after any message executes.
	// Handler will be called after ANY message executes, alongside the response.
	RegisterGlobalPostMsgHandler(handler func(ctx context.Context, msg, msgResp transaction.Msg) error)
	// RegisterPostMsgHandler will register a pre msg handler that hooks after the provided message
	// with the given message name executes. Handler will be called after the message is executed
	// by the module, alongside the response returned by the module.
	RegisterPostMsgHandler(msgName string, handler func(ctx context.Context, msg, msgResp transaction.Msg) error)
}

type HasPostMsgHandler interface {
	RegisterPostMsgHandler(router PostMsgHandlerRouter)
}
```

We note the following behaviors:

* A pre msg handler returning an error will yield to a state transition revert.
* A post msg handler returning an error will yield to a state transition revert.
* A post msg handler will not be called if the execution handler (message handler) fails.
* A post msg handler will be called only if the execution handler succeeds alongside the response provided by the execution handler.

### Message and Query Handlers

Similar to the above design, message handlers will allow the application developer to replace existing gRPC based services
with handlers. This enables the module to be compiled down to TinyGo, and abandon the gRPC dependency. As mentioned
upgrading the modules immediately is not mandatory, module developers can do so in a gradual way. Application developers have the option to use the existing gRPC services or the new handlers.

For message handlers we propose the introduction of the following core/appmodule interfaces and functions:

```go
package core_appmodule

import (
	"context"

	"google.golang.org/protobuf/runtime/protoiface"
)

type MsgHandlerRouter interface {
	RegisterMsgHandler(msgName string, handler func(ctx context.Context, msg transaction.Msg) (msgResp transaction.Msg, err error))
}

type HasMsgHandler interface {
	RegisterMsgHandlers(router MsgHandlerRouter)
}

// RegisterMsgHandler is a helper function to retain type safety when creating handlers, so we do not need to cast messages.
func RegisterMsgHandler[Req, Resp transaction.Msg](router MsgHandlerRouter, handler func(ctx context.Context, req Req) (resp Resp, err error)) {
	// impl detail
}
```

Example

```go
package bank

func (b BankKeeper) Send(ctx context.Context, msg bank.MsgSend) (bank.MsgSendResponse, error) {
	// logic
}

func (b BankModule) RegisterMsgHandlers(router core_appmodule.MsgHandlerRouter) {
	// the RegisterMsgHandler function takes care of doing type casting and conversions, ensuring we retain type safety
	core_appmodule.RegisterMsgHandler(router, b.Send)
}

```

This change is fully state machine compatible as, even if we were using gRPC, messages were
routed using the message type name and not the gRCP method name.

We apply the same principles of MsgHandlers to QueryHandlers, by introducing a new core/appmodule interface:

```go
package core_appmodule

import (
	"context"

	"google.golang.org/protobuf/runtime/protoiface"
)

type QueryHandlerRouter interface {
	RegisterQueryHandler(msgName string, handler func(ctx context.Context, req transaction.Msg) (resp transaction.Msg, err error))
}

type HasQueryHandler interface {
	RegisterQueryHandlers(router QueryHandlerRouter)
}

// RegisterQueryHandler is a helper function to retain type safety when creating handlers, so we do not need to cast messages.
func RegisterQueryHandler[Req, Resp transaction.Msg](router QueryHandlerRouter, handler func(ctx context.Context, req Req) (resp Resp, err error)) {
	// impl detail
}

```

The difference between gRPC handlers and query handlers is that we expect query handlers to be deterministic and usable
in consensus by other modules. Non consensus queries should be registered outside of the state machine itself, and we will
provide guidelines on how to do so with serverv2.

As a consequence queries would be now mapped by their message name.

We can provide JSON exposure of the Query APIs following this rest API format:

```
method: POST
path: /msg_name
ReqBody: protojson.Marshal(msg)
----
RespBody: protojson.Marshal(msgResp)
```

### Consensus Messages

Similar to the above design, consensus messages will allow the underlying consensus engine to speak to the modules. Today we get consensus related information from `sdk.Context`. In server/v2 we are unable to continue with this design due to the forced dependency leakage of comet throughout the repo. Secondly, while we already have `cometInfo` if we were to put this on the new execution client we would be tying CometBFT to the application manager and STF.

In the case of CometBFT, consensus would register handlers for consensus messages for evidence, voteinfo and consensus params. This would allow the consensus engine to speak to the modules.


```go
package consensus

func (b ConsensusKeeper) ConsensusParams(ctx context.Context, msg bank.MsgConsensusParams) (bank.MsgConsensusParamsResponse, error) {
	// logic
}

func (b CircuitModule) RegisterConsensusHandlers(router core_appmodule.MsgHandlerRouter) {
	// the RegisterConsensusHandler function takes care of doing type casting and conversions, ensuring we retain type safety
	core_appmodule.RegisterConsensusHandler(router, b.Send)
}

```


## Consequences

* REST endpoints for message and queries change due to lack of services and gRPC gateway annotations.
* When using gRPC directly, one must query a schema endpoint in order to see all possible messages and queries.

### Backwards Compatibility

The way to interact with modules changes, REST and gRPC will still be available.

### Positive

* Allows modules to be compiled to TinyGo.
* Reduces the cosmos-sdk's learning curve, since understanding gRPC semantics is not a must anymore.
* Allows other modules to extend existing modules behaviour using pre and post msg handlers, without forking.
* The system becomes overall more simple as gRPC is not anymore a hard dependency and requirement for the state machine.
* Reduces the need on sdk.Context
* Concurrently safe
* Reduces public interface of modules

### Negative

* Pre, Post and Consensus msg handlers are a new concept that module developers need to learn (although not immediately).

### Neutral

> {neutral consequences}

### References

> Links to external materials needed to follow the discussion may be added here.
>
> In addition, if the discussion in a request for comments leads to any design
> decisions, it may be helpful to add links to the ADR documents here after the
> discussion has settled.

## Discussion

> This section contains the core of the discussion.
>
> There is no fixed format for this section, but ideally changes to this
> section should be updated before merging to reflect any discussion that took
> place on the PR that made those changes.
