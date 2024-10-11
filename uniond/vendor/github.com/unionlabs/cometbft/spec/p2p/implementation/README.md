# Implementation of the p2p layer

This section documents the implementation of the peer-to-peer (p2p)
communication layer in CometBFT.

The documentation was [produced](https://github.com/tendermint/tendermint/pull/9348)
using the `v0.34.*` releases
and the branch [`v0.34.x`](https://github.com/cometbft/cometbft/tree/v0.34.x)
of this repository as reference.
As there were no substantial changes in the p2p implementation, the
documentation also applies to the releases `v0.37.*` and `v0.38.*` [^v35].

[^v35]: The releases `v0.35.*` and `v0.36.*`, which included a major
  refactoring of the p2p layer implementation, were [discontinued][v35postmorten].

[v35postmorten]: https://interchain-io.medium.com/discontinuing-tendermint-v0-35-a-postmortem-on-the-new-networking-layer-3696c811dabc

## Contents

The documentation follows the organization of the
[`p2p` package](https://github.com/cometbft/cometbft/tree/v0.34.x/p2p),
which implements the following abstractions:

- [Transport](./transport.md): establishes secure and authenticated
   connections with peers;
- [Switch](./switch.md): responsible for dialing peers and accepting
   connections from peers, for managing established connections, and for
   routing messages between the reactors and peers,
   that is, between local and remote instances of the CometBFT protocols;
- [PEX Reactor](./pex.md): due to the several roles of this component, the
  documentation is split in several parts:
    - [Peer Exchange protocol](./pex-protocol.md): enables nodes to exchange peer addresses, thus implementing a peer discovery service;
    - [Address Book](./addressbook.md): stores discovered peer addresses and
  quality metrics associated to peers with which the node has interacted;
    - [Peer Manager](./peer_manager.md): defines when and to which peers a node
  should dial, in order to establish outbound connections;
- [Types](./types.md) and [Configuration](./configuration.md) provide a list of
  existing types and configuration parameters used by the p2p package.
