# Peers

This document explains how CometBFT Peers are identified and how they connect to one another.

## Peer Identity

CometBFT peers are expected to maintain long-term persistent identities in the form of a public key.
Each peer has an ID defined as `peer.ID == peer.PubKey.Address()`, where `Address` uses the scheme defined in `crypto` package.

A single peer ID can have multiple IP addresses associated with it, but a node
will only ever connect to one at a time.

When attempting to connect to a peer, we use the PeerURL: `<ID>@<IP>:<PORT>`.
We will attempt to connect to the peer at IP:PORT, and verify,
via authenticated encryption, that it is in possession of the private key
corresponding to `<ID>`. This prevents man-in-the-middle attacks on the peer layer.

## Connections

All p2p connections use TCP.
Upon establishing a successful TCP connection with a peer,
two handshakes are performed: one for authenticated encryption, and one for CometBFT versioning.
Both handshakes have configurable timeouts (they should complete quickly).

### Authenticated Encryption Handshake

CometBFT implements the Station-to-Station protocol
using X25519 keys for Diffie-Helman key-exchange and chacha20poly1305 for encryption.

Previous versions of this protocol (0.32 and below) suffered from malleability attacks whereas an active man
in the middle attacker could compromise confidentiality as described in [Prime, Order Please!
Revisiting Small Subgroup and Invalid Curve Attacks on
Protocols using Diffie-Hellman](https://eprint.iacr.org/2019/526.pdf).

We have added dependency on the Merlin a keccak based transcript hashing protocol to ensure non-malleability.

It goes as follows:

- generate an ephemeral X25519 keypair
- send the ephemeral public key to the peer
- wait to receive the peer's ephemeral public key
- create a new Merlin Transcript with the string "TENDERMINT_SECRET_CONNECTION_TRANSCRIPT_HASH"
- Sort the ephemeral keys and add the high labeled "EPHEMERAL_UPPER_PUBLIC_KEY" and the low keys labeled "EPHEMERAL_LOWER_PUBLIC_KEY" to the Merlin transcript.
- compute the Diffie-Hellman shared secret using the peers ephemeral public key and our ephemeral private key
- add the DH secret to the transcript labeled DH_SECRET.
- generate two keys to use for encryption (sending and receiving) and a challenge for authentication as follows:
    - create a hkdf-sha256 instance with the key being the diffie hellman shared secret, and info parameter as
    `TENDERMINT_SECRET_CONNECTION_KEY_AND_CHALLENGE_GEN`
    - get 64 bytes of output from hkdf-sha256
    - if we had the smaller ephemeral pubkey, use the first 32 bytes for the key for receiving, the second 32 bytes for sending; else the opposite.
- use a separate nonce for receiving and sending. Both nonces start at 0, and should support the full 96 bit nonce range
- all communications from now on are encrypted in 1400 byte frames (plus encoding overhead),
  using the respective secret and nonce. Each nonce is incremented by one after each use.
- we now have an encrypted channel, but still need to authenticate
- extract a 32 bytes challenge from merlin transcript with the label "SECRET_CONNECTION_MAC"
- sign the common challenge obtained from the hkdf with our persistent private key
- send the amino encoded persistent pubkey and signature to the peer
- wait to receive the persistent public key and signature from the peer
- verify the signature on the challenge using the peer's persistent public key

If this is an outgoing connection (we dialed the peer) and we used a peer ID,
then finally verify that the peer's persistent public key corresponds to the peer ID we dialed,
ie. `peer.PubKey.Address() == <ID>`.

The connection has now been authenticated. All traffic is encrypted.

Note: only the dialer can authenticate the identity of the peer,
but this is what we care about since when we join the network we wish to
ensure we have reached the intended peer (and are not being MITMd).

### Peer Filter

Before continuing, we check if the new peer has the same ID as ourselves or
an existing peer. If so, we disconnect.

We also check the peer's address and public key against
an optional whitelist which can be managed through the ABCI app -
if the whitelist is enabled and the peer does not qualify, the connection is
terminated.

### CometBFT Version Handshake

The CometBFT Version Handshake allows the peers to exchange their NodeInfo:

```golang
type NodeInfo struct {
  Version    p2p.Version
  ID         p2p.ID
  ListenAddr string

  Network    string
  SoftwareVersion    string
  Channels   []int8

  Moniker    string
  Other      NodeInfoOther
}

type Version struct {
 P2P uint64
 Block uint64
 App uint64
}

type NodeInfoOther struct {
 TxIndex          string
 RPCAddress       string
}
```

The connection is disconnected if:

- `peer.NodeInfo.ID` is not equal `peerConn.ID`
- `peer.NodeInfo.Version.Block` does not match ours
- `peer.NodeInfo.Network` is not the same as ours
- `peer.Channels` does not intersect with our known Channels.
- `peer.NodeInfo.ListenAddr` is malformed or is a DNS host that cannot be
  resolved

At this point, if we have not disconnected, the peer is valid.
It is added to the switch and hence all reactors via the `AddPeer` method.
Note that each reactor may handle multiple channels.

## Connection Activity

Once a peer is added, incoming messages for a given reactor are handled through
that reactor's `Receive` method, and output messages are sent directly by the Reactors
on each peer. A typical reactor maintains per-peer go-routine(s) that handle this.
