# P2P Config

Here we describe configuration options around the Peer Exchange.
These can be set using flags or via the `$CMTHOME/config/config.toml` file.

## Seed Mode

`--p2p.seed_mode`

The node operates in seed mode. In seed mode, a node continuously crawls the network for peers,
and upon incoming connection shares some peers and disconnects.

## Seeds

`--p2p.seeds “id100000000000000000000000000000000@1.2.3.4:26656,id200000000000000000000000000000000@2.3.4.5:4444”`

Dials these seeds when we need more peers. They should return a list of peers and then disconnect.
If we already have enough peers in the address book, we may never need to dial them.

## Persistent Peers

`--p2p.persistent_peers “id100000000000000000000000000000000@1.2.3.4:26656,id200000000000000000000000000000000@2.3.4.5:26656”`

Dial these peers and auto-redial them if the connection fails.
These are intended to be trusted persistent peers that can help
anchor us in the p2p network. The auto-redial uses exponential
backoff and will give up after a day of trying to connect.

But If `persistent_peers_max_dial_period` is set greater than zero,
pause between each dial to each persistent peer will not exceed `persistent_peers_max_dial_period`
during exponential backoff and we keep trying again without giving up

**Note:** If `seeds` and `persistent_peers` intersect,
the user will be warned that seeds may auto-close connections
and that the node may not be able to keep the connection persistent.

## Private Peers

`--p2p.private_peer_ids “id100000000000000000000000000000000,id200000000000000000000000000000000”`

These are IDs of the peers that we do not add to the address book or gossip to
other peers. They stay private to us.

## Unconditional Peers

`--p2p.unconditional_peer_ids “id100000000000000000000000000000000,id200000000000000000000000000000000”`

These are IDs of the peers which are allowed to be connected by both inbound or outbound regardless of
`max_num_inbound_peers` or `max_num_outbound_peers` of user's node reached or not.
