---
order: 1
parent:
  title: P2P
  order: 6
---

# Peer-to-Peer Communication

A CometBFT network is composed of multiple CometBFT instances, hereafter called
`nodes`, that interact by exchanging messages.

The CometBFT protocols are designed under the assumption of a partially-connected network model.
This means that a node is not assumed to be directly connected to every other
node in the network.
Instead, each node is directly connected to only a subset of other nodes,
hereafter called its `peers`.

The peer-to-peer (p2p) communication layer is then the component of CometBFT that:

1. establishes connections between nodes in a CometBFT network
2. manages the communication between a node and the connected peers
3. intermediates the exchange of messages between peers in CometBFT protocols

The specification the p2p layer is a work in progress,
tracked by [issue #19](https://github.com/cometbft/cometbft/issues/19).
The current content is organized as follows:

- [`implementation`](./implementation/README.md): documents the current state
  of the implementation of the p2p layer, covering the main components of the
  `p2p` package. The documentation covers, in a fairly comprehensive way,
   the items 1. and 2. from the list above.
- [`reactor-api`](./reactor-api/README.md): specifies the API offered by the
  p2p layer to the protocol layer, through the `Reactor` abstraction.
  This is a high-level specification (i.e., it should not be implementation-specific)
  of the p2p layer API, covering item 3. from the list above.
- [`legacy-docs`](./legacy-docs/): We keep older documentation in 
  the `legacy-docs` directory, as overall, it contains useful information. 
  However, part of this content is redundant,
  being more comprehensively covered in more recent documents,
  and some implementation details might be outdated
  (see [issue #981](https://github.com/cometbft/cometbft/issues/981)).

In addition to this content, some unfinished, work in progress, and auxiliary
material can be found in the
[knowledge-base](https://github.com/cometbft/knowledge-base/tree/main/p2p) repository.
