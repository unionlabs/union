# Reactor API

A component has to implement the [`p2p.Reactor` interface][reactor-interface]
in order to use communication services provided by the p2p layer.
This interface is currently the main source of documentation for a reactor.

The goal of this document is to specify the behaviour of the p2p communication
layer when interacting with a reactor.
So while the [`Reactor interface`][reactor-interface] declares the methods
invoked and determines what the p2p layer expects from a reactor,
this documentation focuses on the **temporal behaviour** that a reactor implementation
should expect from the p2p layer. (That is, in which orders the functions may be called)

This specification is accompanied by the [`reactor.qnt`](./reactor.qnt) file,
a more comprehensive model of the reactor's operation written in
[Quint][quint-repo], an executable specification language.
The methods declared in the [`Reactor`][reactor-interface] interface are
modeled in Quint, in the form of `pure def` methods, providing some examples of
how they should be implemented.
The behaviour of the p2p layer when interacting with a reactor, by invoking the
interface methods, is modeled in the form of state transitions, or `action`s in
the Quint nomenclature.

## Overview

The following _grammar_ is a simplified representation of the expected sequence of calls
from the p2p layer to a reactor.
Note that the grammar represents events referring to a _single reactor_, while
the p2p layer supports the execution of multiple reactors.
For a more detailed representation of the sequence of calls from the p2p layer
to reactors, please refer to the companion Quint model.

While useful to provide an overview of the operation of a reactor,
grammars have some limitations in terms of the behaviour they can express.
For instance, the following grammar only represents the management of _a single peer_,
namely of a peer with a given ID which can connect, disconnect, and reconnect
multiple times to the node.
The p2p layer and every reactor should be able to handle multiple distinct peers in parallel.
This means that multiple occurrences of non-terminal `peer-management` of the
grammar below can "run" independently and in parallel, each one referring and
producing events associated to a different peer:

```abnf
start           = registration on-start *peer-management on-stop
registration    = get-channels set-switch

; Refers to a single peer, a reactor must support multiple concurrent peers
peer-management = init-peer start-peer stop-peer
start-peer      = [*receive] (connected-peer / start-error)
connected-peer  = add-peer *receive
stop-peer       = [peer-error] remove-peer

; Service interface
on-start        = %s"OnStart()"
on-stop         = %s"OnStop()"
; Reactor interface
get-channels    = %s"GetChannels()"
set-switch      = %s"SetSwitch(*Switch)"
init-peer       = %s"InitPeer(Peer)"
add-peer        = %s"AddPeer(Peer)"
remove-peer     = %s"RemovePeer(Peer, reason)"
receive         = %s"Receive(Envelope)"

; Errors, for reference
start-error     = %s"log(Error starting peer)"
peer-error      = %s"log(Stopping peer for error)"
```

The grammar is written in case-sensitive Augmented Backus–Naur form (ABNF,
specified in [IETF RFC 7405](https://datatracker.ietf.org/doc/html/rfc7405)).
It is inspired on the grammar produced to specify the interaction of CometBFT
with an ABCI application, available [here](../../abci/abci%2B%2B_comet_expected_behavior.md).

## Registration

To become a reactor, a component has first to implement the
[`Reactor`][reactor-interface] interface,
then to register the implementation with the p2p layer, using the
`Switch.AddReactor(name string, reactor Reactor)` method,
with a global unique `name` for the reactor.

The registration must happen before the node, in general, and the p2p layer,
in particular, are started.
In other words, there is no support for registering a reactor on a running node:
reactors must be registered as part of the setup of a node.

```abnf
registration    = get-channels set-switch
```

The p2p layer retrieves from the reactor a list of channels the reactor is
responsible for, using the `GetChannels()` method.
The reactor implementation should thereafter expect the delivery of every
message received by the p2p layer in the informed channels.

The second method `SetSwitch(Switch)` concludes the handshake between the
reactor and the p2p layer.
The `Switch` is the main component of the p2p layer, being responsible for
establishing connections with peers and routing messages.
The `Switch` instance provides a number of methods for all registered reactors,
documented in the companion [API for Reactors](./p2p-api.md#switch-api) document.

## Service interface

A reactor must implement the [`Service`](../../../libs/service/service.go) interface,
in particular, a startup `OnStart()` and a shutdown `OnStop()` methods:

```abnf
start           = registration on-start *peer-management on-stop
```

As part of the startup of a node, all registered reactors are started by the p2p layer.
And when the node is shut down, all registered reactors are stopped by the p2p layer.
Observe that the `Service` interface specification establishes that a service
can be started and stopped only once.
So before being started or once stopped by the p2p layer, the reactor should
not expect any interaction.

## Peer management

The core of a reactor's operation is the interaction with peers or, more
precisely, with companion reactors operating on the same channels in peers connected to the node.
The grammar extract below represents the interaction of the reactor with a
single peer:

```abnf
; Refers to a single peer, a reactor must support multiple concurrent peers
peer-management = init-peer start-peer stop-peer
```

The p2p layer informs all registered reactors when it establishes a connection
with a `Peer`, using the `InitPeer(Peer)` method.
When this method is invoked, the `Peer` has not yet been started, namely the
routines for sending messages to and receiving messages from the peer are not running.
This method should be used to initialize state or data related to the new
peer, but not to interact with it.

The next step is to start the communication routines with the new `Peer`.
As detailed in the following, this procedure may or may not succeed.
In any case, the peer is eventually stopped, which concludes the management of
that `Peer` instance.

## Start peer

Once `InitPeer(Peer)` is invoked for every registered reactor, the p2p layer starts the peer's
communication routines and adds the `Peer` to the set of connected peers.
If both steps are concluded without errors, the reactor's `AddPeer(Peer)` is invoked:

```abnf
start-peer      = [*receive] (connected-peer / start-error)
connected-peer  = add-peer *receive
```

In case of errors, a message is logged informing that the p2p layer failed to start the peer.
This is not a common scenario and it is only expected to happen when
interacting with a misbehaving or slow peer. A practical example is reported on this
[issue](https://github.com/tendermint/tendermint/pull/9500).

It is up to the reactor to define how to process the `AddPeer(Peer)` event.
The typical behavior is to start routines that, given some conditions or events,
send messages to the added peer, using the provided `Peer` instance.
The companion [API for Reactors](./p2p-api.md#peer-api) documents the methods
provided by `Peer` instances, available from when they are added to the reactors.

## Stop Peer

The p2p layer informs all registered reactors when it disconnects from a `Peer`,
using the `RemovePeer(Peer, reason)` method:

```abnf
stop-peer       = [peer-error] remove-peer
```

This method is invoked after the p2p layer has stopped peer's send and receive routines.
Depending of the `reason` for which the peer was stopped, different log
messages can be produced.
After removing a peer from all reactors, the `Peer` instance is also removed from
the set of connected peers.
This enables the same peer to reconnect and `InitPeer(Peer)` to be invoked for
the new connection.

From the removal of a `Peer` , the reactor should not receive any further message
from the peer and must not try sending messages to the removed peer.
This usually means stopping the routines that were started by the companion
`Add(Peer)` method.

## Receive messages

The main duty of a reactor is to handle incoming messages on the channels it
has registered with the p2p layer.

The _pre-condition_ for receiving a message from a `Peer` is that the p2p layer
has previously invoked `InitPeer(Peer)`.
This means that the reactor must be able to receive a message from a `Peer`
_before_ `AddPeer(Peer)` is invoked.
This happens because the peer's send and receive routines are started before,
and should be already running when the p2p layer adds the peer to every
registered reactor.

```abnf
start-peer      = [*receive] (connected-peer / start-error)
connected-peer  = add-peer *receive
```

The most common scenario, however, is to start receiving messages from a peer
after `AddPeer(Peer)` is invoked.
An arbitrary number of messages can be received, until the peer is stopped and
`RemovePeer(Peer)` is invoked.

When a message is received from a connected peer on any of the channels
registered by the reactor, the p2p layer will deliver the message to the
reactor via the `Receive(Envelope)` method.
The message is packed into an `Envelope` that contains:

- `ChannelID`: the channel the message belongs to
- `Src`: the source `Peer` handler, from which the message was received
- `Message`: the actual message's payload, unmarshalled using protocol buffers

Two important observations regarding the implementation of the `Receive` method:

1. Concurrency: the implementation should consider concurrent invocations of
   the `Receive` method carrying messages from different peers, as the
   interaction with different peers is independent and messages can be received in parallel.
1. Non-blocking: the implementation of the `Receive` method is expected not to block,
   as it is invoked directly by the receive routines.
   In other words, while `Receive` does not return, other messages from the
   same sender are not delivered to any reactor.

[reactor-interface]: ../../../p2p/base_reactor.go
[quint-repo]: https://github.com/informalsystems/quint
