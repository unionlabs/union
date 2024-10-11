# Peer Discovery

A CometBFT P2P network has different kinds of nodes with different requirements for connectivity to one another.
This document describes what kind of nodes CometBFT should enable and how they should work.

## Seeds

Seeds are the first point of contact for a new node.
They return a list of known active peers and then disconnect.

Seeds should operate full nodes with the PEX reactor in a "crawler" mode
that continuously explores to validate the availability of peers.

Seeds should only respond with some top percentile of the best peers it knows about.

## New Full Node

A new node needs a few things to connect to the network:

- a list of seeds, which can be provided to CometBFT via config file or flags,
  or hardcoded into the software by in-process apps
- a `ChainID`, also called `Network` at the p2p layer
- a recent block height, H, and hash, HASH for the blockchain.

The values `H` and `HASH` must be received and corroborated by means external to CometBFT, and specific to the user - ie. via the user's trusted social consensus.
This requirement to validate `H` and `HASH` out-of-band and via social consensus
is the essential difference in security models between Proof-of-Work and Proof-of-Stake blockchains.

With the above, the node then queries some seeds for peers for its chain,
dials those peers, and runs the CometBFT protocols with those it successfully connects to.

When the peer catches up to height H, it ensures the block hash matches HASH.
If not, CometBFT will exit, and the user must try again - either they are connected
to bad peers or their social consensus is invalid.

## Restarted Full Node

A node checks its address book on startup and attempts to connect to peers from there.
If it can't connect to any peers after some time, it falls back to the seeds to find more.

Restarted full nodes can run the `blockchain` or `consensus` reactor protocols to sync up
to the latest state of the blockchain from wherever they were last.
In a Proof-of-Stake context, if they are sufficiently far behind (greater than the length
of the unbonding period), they will need to validate a recent `H` and `HASH` out-of-band again
so they know they have synced the correct chain.

## Validator Node

A validator node is a node that interfaces with a validator signing key.
These nodes require the highest security, and should not accept incoming connections.
They should maintain outgoing connections to a controlled set of "Sentry Nodes" that serve
as their proxy shield to the rest of the network.

Validators that know and trust each other can accept incoming connections from one another and maintain direct private connectivity via VPN.

## Sentry Node

Sentry nodes are guardians of a validator node and provide it access to the rest of the network.
They should be well connected to other full nodes on the network.
Sentry nodes may be dynamic, but should maintain persistent connections to some evolving random subset of each other.
They should always expect to have direct incoming connections from the validator node and its backup(s).
They do not report the validator node's address in the PEX and
they may be more strict about the quality of peers they keep.

Sentry nodes belonging to validators that trust each other may wish to maintain persistent connections via VPN with one another, but only report each other sparingly in the PEX.
