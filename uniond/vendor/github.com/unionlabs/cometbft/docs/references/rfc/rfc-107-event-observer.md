# RFC 107: Internal Signalling Using Event Observers

## Changelog

- 2023-07-24: First draft (@thanethomson)

## Abstract

The overall problem follows from that discussed in [\#1055] and [RFC 104]:
CometBFT is difficult to reason about and change in part due to the complexity
of the internal interaction between different reactors/services.

RFC 104 explored and ruled out the possibility of employing a loosely coupled
model like the [Actor] model. This RFC explores the possibility of a simpler,
more tightly coupled model with a range of benefits compared to the actor model
and what is currently implemented. It is possible that this would also help
simplify testing, since the tests in CometBFT are the biggest users (by volume)
of the current event bus subsystem.

## Background

Various design patterns are potentially useful in addressing the problem of
coupling of concurrent components in a system like CometBFT. The [Observer
Pattern], for instance, is implicitly implemented in the [`EventBus`] in
CometBFT, but in a very loosely coupled manner analogous to how it is
implemented in the Actor model. Such loosely coupled approaches are generally
better suited for cases where coupling between components needs to adapt at
runtime, but this is not the case for CometBFT - all impactful coupling happens
at compile time. This points to the possibility that this pattern is
inappropriately applied, except in the case of the WebSocket-based event
subscriptions.

Another alternative is possible within CometBFT if one wants to access
information from other reactors/services: the [`Switch`] allows developers to
look up reactors, at runtime, and access methods directly on those reactors.
This is again an inappropriate pattern because all lookups are hard-coded, and
reactors/services are not dynamically created/destroyed at runtime.

This suggests that a different approach is necessary for cross-component
interaction - ideally one which provides more robust compile-time guarantees
than the current ones.

## Discussion

A more type-safe, understandable Observer pattern is proposed here than what
currently exists in CometBFT. For example, in the consensus state there are many
places where events are published via the event bus, e.g.:

- <https://github.com/cometbft/cometbft/blob/091a1f312e5f2f4b183fab1d57d729a6c478ff1f/consensus/state.go#L743>
- <https://github.com/cometbft/cometbft/blob/091a1f312e5f2f4b183fab1d57d729a6c478ff1f/consensus/state.go#L964>
- <https://github.com/cometbft/cometbft/blob/091a1f312e5f2f4b183fab1d57d729a6c478ff1f/consensus/state.go#L971>
- etc.

All of these event publishing methods, not only for consensus state but for
other types of event publishers, are defined on this central `EventBus` type,
which seems to signal that its functionality should be decentralized.

### Strongly Typed Event Observer

A simple alternative pattern here would be to define an **event observer**
interface for every major component of the system capable of producing events.
For consensus state, this may end up looking something like:

```golang
package consensus

// StateObserver is specific to the consensus.State struct, and all of its
// methods are called from within consensus.State instead of using an "event
// bus". This allows for greater compile-time guarantees through doing away with
// the generic pub/sub mechanism in the event bus.
//
// Note how all methods are infallible (i.e. they do not return any errors).
// This is functionally equivalent to the fire-and-forget pattern implemented by
// the event bus.
//
// Also note how method names are prefixed by the name of the relevant producer
// of events (in this case "ConsensusState", corresponding to the
// consensus.State struct). This is intentional to allow composition of
// observers of multiple different components without function names clashing.
//
// Finally, given that this is just straightforward Go, it is up to either the
// caller or the callee to decide how to handle the concurrency of certain
// events. The event bus approach, by contrast, is always concurrent and relies
// on Go channels, which could end up filling up and causing back-pressure into
// the caller (already observed in slow WebSocket subscribers).
type StateObserver interface {
    ConsensusStateNewRoundStep(ev EventDataRoundState)
    ConsensusStateTimeoutPropose(ev EventDataRoundState)
    ConsensusStateTimeoutWait(ev EventDataRoundState)
    // ...
}
```

And on `consensus.State` one could easily either supply such an observer in the
constructor, or define a new method that allows one to set the observer, e.g.:

```golang
package consensus

func (cs *State) SetObserver(obs StateObserver) {
    cs.observer = obs
}
```

Then, instead of publishing events via the event bus, one would simply do the
following:

```diff
-        if err := cs.eventBus.PublishEventNewRoundStep(rs); err != nil {
-            cs.Logger.Error("failed publishing new round step", "err", err)
-        }
+        // Notify the observer
+        cs.observer.ConsensusStateNewRoundStep(rs)
```

### Comparing "Subscription" Interfaces

The `EventBus` offers two slightly different subscription mechanisms for events,
both of which rely on Go channels under the hood for their implementation.

- [`Subscribe`]
- [`SubscribeUnbuffered`]

The observer pattern proposed in this RFC can facilitate both patterns and more,
given that one has the option of intervening synchronously in a blocking way
when the "publisher" calls the observer event handler. Using the proposed
observer pattern, it is up to either the publisher or the observer to implement
their own concurrency.

### Fanout Observers

How then does one simulate the same sort of pub/sub functionality that the
`EventBus` provides using this approach? Where this sort of behaviour is
absolutely necessary, it is trivial to build a "fanout" observer that implements
this interface, e.g.:

```golang
type StateFanoutObserver struct {
    observers []StateObserver
}

func NewStateFanoutObserver(observers ...StateObserver) *StateFanoutObserver {
    return &StateFanoutObserver{
        observers: observers,
    }
}

func (o *StateFanoutObserver) ConsensusStateNewRoundStep(ev EventDataRoundStep) {
    for _, obs := range o.observers {
        obs.ConsensusStateNewRoundStep(ev)
    }
}

// ...
```

### Testing

Many tests in the CometBFT codebase rely heavily on subscribing to specific
events directly via the event bus. This can easily be accomplished using the
approach described in the [Fanout Observers](#fanout-observers) section:

```golang
state.SetObserver(
    NewStateFanoutObserver(
        // An observer specifically for use during testing.
        newTestStateObserver(),
        // ... other observers here that would be used in production
    ),
)
```

One could easily also define an ergonomic observer type that would allow inline
definition and overriding of only specific event handlers:

```golang
type testObserver struct {
    newRoundStep   func(EventDataRoundState)
    timeoutPropose func(EventDataRoundState)
    // ...
}

func (o *testObserver) ConsensusStateNewRoundStep(ev EventDataRoundState) {
    if o.newRoundStep != nil {
        o.newRoundStep(ev)
    }
}

// ...

func TestCustomObserver(t *testing.T) {
    testObs := &testObserver{}
    testObs.newRoundStep = func(ev EventDataRoundState) {
        // Custom code here called upon new round step
    }
    // ...
}
```

### Event Subscription

The current WebSocket-based event subscription mechanism is envisaged to go away
at some point in future, and there is no other mechanism by which external
observers can subscribe to events.

[ADR 101], however, provides a more general alternative through which
integrators can gain access to event data from outside of the node. Once ADR 101
has been implemented, the whole WebSocket-based interface could be removed.

### Pros and Cons

The benefits of the proposed approach include:

- Greater compile-time correctness guarantees
- Code becomes easier to reason about, since one can easily follow the call
  chain for certain events using one's IDE instead of needing to search the
  codebase for subscriptions to certain events
- Easier to test
- Does away with needing to access internal/private `eventBus` variables within
  reactors/state from tests ([example][test-eventbus-access])
- Splits event generation and handling out into a per-package responsibility,
  more cleanly separating and modularizing the codebase

The drawbacks of the proposed approach include:

- Potentially involves writing more code (volume-wise) than what is currently
  present, although the new code would be simpler
- Concurrency concerns need to be reasoned about carefully, as back-pressure is
  still possible depending on how observers are implemented

[\#1055]: https://github.com/cometbft/cometbft/issues/1055
[RFC 104]: rfc-104-actor-model.md
[Actor]: https://en.wikipedia.org/wiki/Actor_model
[Observer Pattern]: https://en.wikipedia.org/wiki/Observer_pattern
[`EventBus`]: https://github.com/cometbft/cometbft/blob/b23ef56f8e6d8a7015a7f816a61f2e53b0b07b0d/types/event_bus.go#L33
[`Switch`]: https://github.com/cometbft/cometbft/blob/b23ef56f8e6d8a7015a7f816a61f2e53b0b07b0d/p2p/switch.go#L70
[test-eventbus-access]: https://github.com/cometbft/cometbft/blob/091a1f312e5f2f4b183fab1d57d729a6c478ff1f/consensus/mempool_test.go#L40
[ADR 101]: https://github.com/cometbft/cometbft/issues/574
[`Subscribe`]: https://github.com/cometbft/cometbft/blob/a9deb305e51278c25ad92b249caa092d24c5fc29/types/event_bus.go#L75
[`SubscribeUnbuffered`]: https://github.com/cometbft/cometbft/blob/a9deb305e51278c25ad92b249caa092d24c5fc29/types/event_bus.go#L86
