# ADR 012: PeerTransport

## Context

One of the more apparent problems with the current architecture in the p2p
package is that there is no clear separation of concerns between different
components. Most notably the `Switch` is currently doing physical connection
handling. An artifact is the dependency of the Switch on
`[config.P2PConfig`](https://github.com/tendermint/tendermint/blob/05a76fb517f50da27b4bfcdc7b4cf185fc61eff6/config/config.go#L272-L339).

Addresses:

- [#2046](https://github.com/tendermint/tendermint/issues/2046)
- [#2047](https://github.com/tendermint/tendermint/issues/2047)

First iteration in [#2067](https://github.com/tendermint/tendermint/issues/2067)

## Decision

Transport concerns will be handled by a new component (`PeerTransport`) which
will provide Peers at its boundary to the caller. In turn `Switch` will use
this new component accept new `Peer`s and dial them based on `NetAddress`.

### PeerTransport

Responsible for emitting and connecting to Peers. The implementation of `Peer`
is left to the transport, which implies that the chosen transport dictates the
characteristics of the implementation handed back to the `Switch`. Each
transport implementation is responsible to filter establishing peers specific
to its domain, for the default multiplexed implementation the following will
apply:

- connections from our own node
- handshake fails
- upgrade to secret connection fails
- prevent duplicate ip
- prevent duplicate id
- nodeinfo incompatibility

```go
// PeerTransport proxies incoming and outgoing peer connections.
type PeerTransport interface {
	// Accept returns a newly connected Peer.
	Accept() (Peer, error)

	// Dial connects to a Peer.
	Dial(NetAddress) (Peer, error)
}

// EXAMPLE OF DEFAULT IMPLEMENTATION

// multiplexTransport accepts tcp connections and upgrades to multiplexted
// peers.
type multiplexTransport struct {
	listener net.Listener

	acceptc chan accept
	closec  <-chan struct{}
	listenc <-chan struct{}

	dialTimeout      time.Duration
	handshakeTimeout time.Duration
	nodeAddr         NetAddress
	nodeInfo         NodeInfo
	nodeKey          NodeKey

	// TODO(xla): Remove when MConnection is refactored into mPeer.
	mConfig conn.MConnConfig
}

var _ PeerTransport = (*multiplexTransport)(nil)

// NewMTransport returns network connected multiplexed peers.
func NewMTransport(
	nodeAddr NetAddress,
	nodeInfo NodeInfo,
	nodeKey NodeKey,
) *multiplexTransport
```

### Switch

From now the Switch will depend on a fully setup `PeerTransport` to
retrieve/reach out to its peers. As the more low-level concerns are pushed to
the transport, we can omit passing the `config.P2PConfig` to the Switch.

```go
func NewSwitch(transport PeerTransport, opts ...SwitchOption) *Switch
```

## Status

In Review.

## Consequences

### Positive

- free Switch from transport concerns - simpler implementation
- pluggable transport implementation - simpler test setup
- remove Switch dependency on P2PConfig - easier to test

### Negative

- more setup for tests which depend on Switches

### Neutral

- multiplexed will be the default implementation

[0] These guards could be potentially extended to be pluggable much like
middlewares to express different concerns required by differentally configured
environments.
