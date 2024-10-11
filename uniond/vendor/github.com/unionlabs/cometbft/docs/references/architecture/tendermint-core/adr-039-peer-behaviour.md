# ADR 039: Peer Behaviour Interface

## Changelog
* 07-03-2019: Initial draft
* 14-03-2019: Updates from feedback

## Context

The responsibility for signaling and acting upon peer behavior lacks a single 
owning component and is heavily coupled with the network stack[<sup>1</sup>](#references). Reactors
maintain a reference to the `p2p.Switch` which they use to call 
`switch.StopPeerForError(...)` when a peer misbehaves and 
`switch.MarkAsGood(...)` when a peer contributes in some meaningful way. 
While the switch handles `StopPeerForError` internally, the `MarkAsGood` 
method delegates to another component, `p2p.AddrBook`. This scheme of delegation 
across Switch obscures the responsibility for handling peer behavior
and ties up the reactors in a larger dependency graph when testing.

## Decision

Introduce a `PeerBehaviour` interface and concrete implementations which
provide methods for reactors to signal peer behavior without direct
coupling `p2p.Switch`.  Introduce a ErrorBehaviourPeer to provide
concrete reasons for stopping peers. Introduce GoodBehaviourPeer to provide
concrete ways in which a peer contributes.

### Implementation Changes

PeerBehaviour then becomes an interface for signaling peer errors as well
as for marking peers as `good`.

```go
type PeerBehaviour interface {
    Behaved(peer Peer, reason GoodBehaviourPeer)
    Errored(peer Peer, reason ErrorBehaviourPeer)
}
```

Instead of signaling peers to stop with arbitrary reasons:
`reason interface{}` 

We introduce a concrete error type ErrorBehaviourPeer:
```go
type ErrorBehaviourPeer int

const (
    ErrorBehaviourUnknown = iota
    ErrorBehaviourBadMessage
    ErrorBehaviourMessageOutofOrder
    ...
)
```

To provide additional information on the ways a peer contributed, we introduce
the GoodBehaviourPeer type.

```go
type GoodBehaviourPeer int

const (
    GoodBehaviourVote = iota
    GoodBehaviourBlockPart
    ...
)
```

As a first iteration we provide a concrete implementation which wraps
the switch:
```go
type SwitchedPeerBehaviour struct {
    sw *Switch
}

func (spb *SwitchedPeerBehaviour) Errored(peer Peer, reason ErrorBehaviourPeer) {
    spb.sw.StopPeerForError(peer, reason)
}

func (spb *SwitchedPeerBehaviour) Behaved(peer Peer, reason GoodBehaviourPeer) {
    spb.sw.MarkPeerAsGood(peer)
}

func NewSwitchedPeerBehaviour(sw *Switch) *SwitchedPeerBehaviour {
    return &SwitchedPeerBehaviour{
        sw: sw,
    }
}
```

Reactors, which are often difficult to unit test[<sup>2</sup>](#references) could use an implementation which exposes the signals produced by the reactor in
manufactured scenarios:

```go
type ErrorBehaviours map[Peer][]ErrorBehaviourPeer
type GoodBehaviours map[Peer][]GoodBehaviourPeer

type StorePeerBehaviour struct {
    eb ErrorBehaviours
    gb GoodBehaviours
}

func NewStorePeerBehaviour() *StorePeerBehaviour{
    return &StorePeerBehaviour{
        eb: make(ErrorBehaviours),
        gb: make(GoodBehaviours),
    }
}

func (spb StorePeerBehaviour) Errored(peer Peer, reason ErrorBehaviourPeer) {
    if _, ok := spb.eb[peer]; !ok {
        spb.eb[peer] = []ErrorBehaviours{reason}
    } else {
        spb.eb[peer] = append(spb.eb[peer], reason)
    }
}

func (mpb *StorePeerBehaviour) GetErrored() ErrorBehaviours {
    return mpb.eb
}


func (spb StorePeerBehaviour) Behaved(peer Peer, reason GoodBehaviourPeer) {
    if _, ok := spb.gb[peer]; !ok {
        spb.gb[peer] = []GoodBehaviourPeer{reason}
    } else {
        spb.gb[peer] = append(spb.gb[peer], reason)
    }
}

func (spb *StorePeerBehaviour) GetBehaved() GoodBehaviours {
    return spb.gb
}
```

## Status

Accepted

## Consequences

### Positive

    * De-couple signaling from acting upon peer behavior.
    * Reduce the coupling of reactors and the Switch and the network
      stack
    * The responsibility of managing peer behavior can be migrated to
      a single component instead of split between the switch and the
      address book.

### Negative

    * The first iteration will simply wrap the Switch and introduce a
      level of indirection.

### Neutral

## References

1. Issue [#2067](https://github.com/tendermint/tendermint/issues/2067): P2P Refactor
2. PR: [#3506](https://github.com/tendermint/tendermint/pull/3506): ADR 036: Blockchain Reactor Refactor
