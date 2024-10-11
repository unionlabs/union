# RFC 104: Internal Messaging Using the Actor Model

## Changelog

- 2023-07-13: Emphasize/clarify conclusions (@thanethomson)
- 2023-06-29: First draft (@thanethomson)

## Abstract

At present, CometBFT is a collection of components that run concurrently within
the same process and interact with each other via several different mechanisms:

1. The [`EventBus`][event-bus]
2. The [`Switch`][switch], where reactors can look up other reactors by name and
   interact with them directly by calling their methods
3. Go channels held by reactors

This results in a number of problems:

- Difficulty and complexity in testing individual reactors in isolation, which
  results in brittle and unnecessarily complex tests.
- Difficulty reasoning about overall system operation, which slows onboarding of
  new developers and debugging.
- Non-deterministic tests which are of dubious value; these also slow down
  development efforts.

This RFC explores the possibility and potential implications of making use of
the [Actor] model internally to replace the reactor/service model, all while
coalescing the three existing concurrent communication approaches into a single
one.

## Background

The [Actor] model as an idea has been around since the 1970s and is most widely
known to be employed in [Erlang][erlang]. It imposes an architecture on a system
whereby all concurrent subprocesses/tasks are modelled as "actors", which are
the basic unit of concurrency in the system. Each actor is responsible for the
management of its own internal state, and the only way actor state can be
mutated is through messaging.

### Messaging patterns

The most trivial interface for an actor is one that has a single message
handler.

```go
type Actor interface {
    // Receive handles every incoming message sent to an actor. It is the sole
    // method responsible for mutating the state of the actor.
    Receive(ctx Context)
}

type Context interface {
    // System provides a reference to the primary actor system in which the
    // actor is running. This allows the actor to spawn and kill other actors,
    // send messages to other actors, etc.
    System() ActorSystem

    // Sender provides a reference to the sender of a message, which allows the
    // receiving actor to send a response if required.
    Sender() ActorRef

    // Message sent by the sender. Notice the "any" type - a common pattern in
    // Go-based actor frameworks, which effectively results in type erasure.
    //
    // It is, of course, possible to implement some degree of compile-time
    // guarantees using Go generics, but this is still somewhat limited compared
    // to directly calling methods on a struct.
    Message() any
}
```

Actors can be built to facilitate different messaging patterns of varying
degrees of complexity to one or more other actors:

- One-shot/fire-and-forget messaging
- Request/reply semantics
- Publish/subscribe mechanisms, where one actor can be created with the sole
  purpose of being the pub/sub router actor, whose sole job is to manage topics,
  subscribers and publishing messages on certain topics to specific subscribers

### Existing actor frameworks

As the actor model has grown in popularity, various actor frameworks have become
available in a variety of programming languages. Examples of frameworks in
different programming languages include:

- [Proto Actor] (Go/C\#/Java/Kotlin)
- [Akka] (Java/Scala/C\#)
- [Quasar] (Java/Kotlin)
- [CAF] (C++)
- [ractor] (Rust)
- [Actix] (Rust)

Most existing frameworks are incredibly complex, owing to the fact that they aim
to provide the foundation for complex, automatically scalable cloud-based
services. Issues such as distribution of actor-based computation across multiple
networked processes, as well as automatic persistence of actor state to
facilitate fault recovery, are catered for by the underlying framework. Much of
this functionality would not be useful or easily integrated into CometBFT at the
time of this writing.

Actor frameworks normally also provide a form of supervision hierarchy that
allows for internal component failure recovery (depending on recovery policies
that can be adjusted on a per-actor basis). It also allows "supervisor" actors
greater control over the lifecycle of their "children" actors.

## Discussion

### Pros and cons of the actor model

The benefits of employing the actor model in CometBFT are:

1. Greater ease of reasoning about how different components work if they follow
   well-defined patterns of interaction.
2. Simplification of tests for concurrent components, since they can be fully
   exercised simply by way of messaging. This opens up the possibility of
   employing model-based testing for a greater number of components.

The drawbacks of introducing the actor model in CometBFT are:

1. It would require refactoring the entire codebase, which would be a
   non-trivial and expensive exercise.
2. When compared to calling component methods, sending messages to a component
   usually results in much looser type/interface restrictions. This effectively
   results in type erasure, which makes it more difficult to give compile-time
   guarantees as to the correctness of the code and increases the testing burden
   needed to ensure correctness.

### Employing the actor model in CometBFT

If CometBFT were to employ an actor model, every concurrent process would need
to be modelled as an actor. This includes every reactor/service in the system,
but also every major coroutine spawned by those reactors/services (if this is
not done, the system ends up again using a mix of concurrency approaches with
their concomitant complexity).

### Using an existing framework

The major benefit of using an existing framework is that one does not need to
reinvent the wheel, and reduces the quantity of code for which the CometBFT team
is responsible.

The major drawbacks of using an existing framework are:

1. **Additional dependency-related risk.** Using a framework introduces an
   additional dependency, which could introduce new risks into the project. This
   not only includes security risks, but also long-term project risk if the
   dependency ends up being unmaintained.

2. **Greater complexity.** Much of the complexity of such frameworks derives
   from the idea that one may want to automatically scale an actor-based
   application out horizontally and have its concurrent operations automatically
   parallelized across multiple processes with all networking taken care of, as
   if by "magic", by the underlying framework.

   While potentially useful in a future version of CometBFT, this particular
   feature set is not considered for implementation or use in this ADR. As such,
   using frameworks such as [Proto Actor] would most likely be overkill for the
   needs of the CometBFT project.

3. **Non-standard semantics/coding conventions.** Frameworks generally implement
   their own semantics that diverge from standard semantics and conventions of
   the programming language in which they are implemented. This is true of the
   [Proto Actor] framework - one of the most popular actor frameworks written in
   Go.

4. **Loss of type information.** When interacting with actors by way of
   references, one normally sends `any`-typed messages to those actors as
   opposed to calling strongly typed functions. Erasing type information assists
   in more loosely coupling system components, but provides poorer compile-time
   correctness guarantees and increases the range of tests needed.

### Conclusion

It seems as though neither of adopting an actor model-based framework, nor a
strict actor model, is desirable for CometBFT due to the drawbacks covered in
this RFC.

The primary lesson to be taken from the actor model, however, is that, if there
were a way to ensure that all reactor/service state is fully self-contained
within each reactor/service, and interacting with that reactor/service followed
a disciplined and consistent pattern, the system would be dramatically easier to
both understand and test. For instance, if all state for a particular protocol
were self-contained, single-threaded and only ever mutated directly by a single
active entity (i.e. reactor/service), the evolution of the state machine of that
protocol could be more fully exercised by testing approaches like fuzzing or
MBT.

Implementing such changes would still involve substantial refactoring, but much
less so than introducing a full-blown actor system. It would also allow for
progressive refactoring of reactors/services over time.

[event-bus]: https://github.com/cometbft/cometbft/blob/b23ef56f8e6d8a7015a7f816a61f2e53b0b07b0d/types/event_bus.go#L33
[switch]: https://github.com/cometbft/cometbft/blob/b23ef56f8e6d8a7015a7f816a61f2e53b0b07b0d/p2p/switch.go#L70
[Actor]: https://en.wikipedia.org/wiki/Actor_model
[Erlang]: https://www.erlang.org/
[Akka]: https://akka.io/
[Proto Actor]: https://proto.actor/
[Quasar]: http://docs.paralleluniverse.co/quasar/
[CAF]: https://www.actor-framework.org/
[Actix]: https://actix.rs/docs/actix/actor
[ractor]: https://github.com/slawlor/ractor
