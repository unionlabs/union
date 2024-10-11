# ADR 033: pubsub 2.0

Author: Anton Kaliaev (@melekes)

## Changelog

02-10-2018: Initial draft

16-01-2019: Second version based on our conversation with Jae

17-01-2019: Third version explaining how new design solves current issues

25-01-2019: Fourth version to treat buffered and unbuffered channels differently

## Context

Since the initial version of the pubsub, there's been a number of issues
raised: [#951], [#1879], [#1880]. Some of them are high-level issues questioning the
core design choices made. Others are minor and mostly about the interface of
`Subscribe()` / `Publish()` functions.

### Sync vs Async

Now, when publishing a message to subscribers, we can do it in a goroutine:

_using channels for data transmission_
```go
for each subscriber {
    out := subscriber.outc
    go func() {
        out <- msg
    }
}
```

_by invoking callback functions_
```go
for each subscriber {
    go subscriber.callbackFn()
}
```

This gives us greater performance and allows us to avoid "slow client problem"
(when other subscribers have to wait for a slow subscriber). A pool of
goroutines can be used to avoid uncontrolled memory growth.

In certain cases, this is what you want. But in our case, because we need
strict ordering of events (if event A was published before B, the guaranteed
delivery order will be A -> B), we can't publish msg in a new goroutine every time.

We can also have a goroutine per subscriber, although we'd need to be careful
with the number of subscribers. It's more difficult to implement as well +
unclear if we'll benefit from it (cause we'd be forced to create N additional
channels to distribute msg to these goroutines).

### Non-blocking send

There is also a question whenever we should have a non-blocking send.
Currently, sends are blocking, so publishing to one client can block on
publishing to another. This means a slow or unresponsive client can halt the
system. Instead, we can use a non-blocking send:

```go
for each subscriber {
    out := subscriber.outc
    select {
        case out <- msg:
        default:
            log("subscriber %v buffer is full, skipping...")
    }
}
```

This fixes the "slow client problem", but there is no way for a slow client to
know if it had missed a message. We could return a second channel and close it
to indicate subscription termination. On the other hand, if we're going to
stick with blocking send, **devs must always ensure subscriber's handling code
does not block**, which is a hard task to put on their shoulders.

The interim option is to run goroutines pool for a single message, wait for all
goroutines to finish. This will solve "slow client problem", but we'd still
have to wait `max(goroutine_X_time)` before we can publish the next message.

### Channels vs Callbacks

Yet another question is whether we should use channels for message transmission or
call subscriber-defined callback functions. Callback functions give subscribers
more flexibility - you can use mutexes in there, channels, spawn goroutines,
anything you really want. But they also carry local scope, which can result in
memory leaks and/or memory usage increase.

Go channels are de-facto standard for carrying data between goroutines.

### Why `Subscribe()` accepts an `out` channel?

Because in our tests, we create buffered channels (cap: 1). Alternatively, we
can make capacity an argument and return a channel.

## Decision

### MsgAndTags

Use a `MsgAndTags` struct on the subscription channel to indicate what tags the
msg matched.

```go
type MsgAndTags struct {
    Msg interface{}
    Tags TagMap
}
```

### Subscription Struct


Change `Subscribe()` function to return a `Subscription` struct:

```go
type Subscription struct {
  // private fields
}

func (s *Subscription) Out() <-chan MsgAndTags
func (s *Subscription) Canceled() <-chan struct{}
func (s *Subscription) Err() error
```

`Out()` returns a channel onto which messages and tags are published.
`Unsubscribe`/`UnsubscribeAll` does not close the channel to avoid clients from
receiving a nil message.

`Canceled()` returns a channel that's closed when the subscription is terminated
and supposed to be used in a select statement.

If the channel returned by `Canceled()` is not closed yet, `Err()` returns nil.
If the channel is closed, `Err()` returns a non-nil error explaining why:
`ErrUnsubscribed` if the subscriber choose to unsubscribe,
`ErrOutOfCapacity` if the subscriber is not pulling messages fast enough and the channel returned by `Out()` became full.
After `Err()` returns a non-nil error, successive calls to `Err() return the same error.

```go
subscription, err := pubsub.Subscribe(...)
if err != nil {
  // ...
}
for {
select {
  case msgAndTags <- subscription.Out():
    // ...
  case <-subscription.Canceled():
    return subscription.Err()
}
```

### Capacity and Subscriptions

Make the `Out()` channel buffered (with capacity 1) by default. In most cases, we want to
terminate the slow subscriber. Only in rare cases, we want to block the pubsub
(e.g. when debugging consensus). This should lower the chances of the pubsub
being frozen.

```go
// outCap can be used to set capacity of Out channel
// (1 by default, must be greater than 0).
Subscribe(ctx context.Context, clientID string, query Query, outCap... int) (Subscription, error) {
```

Use a different function for an unbuffered channel:

```go
// Subscription uses an unbuffered channel. Publishing will block.
SubscribeUnbuffered(ctx context.Context, clientID string, query Query) (Subscription, error) {
```

SubscribeUnbuffered should not be exposed to users.

### Blocking/Nonblocking

The publisher should treat these kinds of channels separately.
It should block on unbuffered channels (for use with internal consensus events
in the consensus tests) and not block on the buffered ones. If a client is too
slow to keep up with it's messages, it's subscription is terminated:

for each subscription {
    out := subscription.outChan
    if cap(out) == 0 {
        // block on unbuffered channel
        out <- msg
    } else {
        // don't block on buffered channels
        select {
            case out <- msg:
            default:
                // set the error, notify on the cancel chan
                subscription.err = fmt.Errorf("client is too slow for msg)
                close(subscription.cancelChan)

                // ... unsubscribe and close out
        }
    }
}

### How this new design solves the current issues?

[#951] ([#1880]):

Because of non-blocking send, situation where we'll deadlock is not possible
anymore. If the client stops reading messages, it will be removed.

[#1879]:

MsgAndTags is used now instead of a plain message.

### Future problems and their possible solutions

[#2826]

One question I am still pondering about: how to prevent pubsub from slowing
down consensus. We can increase the pubsub queue size (which is 0 now). Also,
it's probably a good idea to limit the total number of subscribers.

This can be made automatically. Say we set queue size to 1000 and, when it's >=
80% full, refuse new subscriptions.

## Status

Implemented

## Consequences

### Positive

- more idiomatic interface
- subscribers know what tags msg was published with
- subscribers aware of the reason their subscription was canceled

### Negative

- (since v1) no concurrency when it comes to publishing messages

### Neutral


[#951]: https://github.com/tendermint/tendermint/issues/951
[#1879]: https://github.com/tendermint/tendermint/issues/1879
[#1880]: https://github.com/tendermint/tendermint/issues/1880
[#2826]: https://github.com/tendermint/tendermint/issues/2826
