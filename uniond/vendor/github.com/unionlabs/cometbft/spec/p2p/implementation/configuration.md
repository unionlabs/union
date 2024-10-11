# CometBFT p2p configuration

This document contains configurable parameters a node operator can use to tune the p2p behaviour.

| Parameter                                              | Default               | Description                                                                                                                                                                                     |
| ---                                                    | ---                   | ---                                                                                                                                                                                             |
| ListenAddress                                          | "tcp://0.0.0.0:26656" | Address to listen for incoming connections (0.0.0.0:0 means any interface, any port)                                                                                                            |
| ExternalAddress                                        | ""                    | Address to advertise to peers for them to dial                                                                                                                                                  |
| [Seeds](./pex-protocol.md#seed-nodes)                  | empty                 | Comma separated list of seed nodes to connect to (ID@host:port )                                                                                                                                |
| [Persistent peers](./peer_manager.md#persistent-peers) | empty                 | Comma separated list of nodes to keep persistent connections to (ID@host:port )                                                                                                                 |
| [AddrBook](./addressbook.md)                           | defaultAddrBookPath   | Path do address book                                                                                                                                                                            |
| AddrBookStrict                                         | true                  | Set true for strict address routability rules and false for private or local networks                                                                                                           |
| [MaxNumInboundPeers](./switch.md#accepting-peers)      | 40                    | Maximum number of inbound peers                                                                                                                                                                 |
| [MaxNumOutboundPeers](./peer_manager.md#ensure-peers)  | 10                    | Maximum number of outbound peers to connect to, excluding persistent peers                                                                                                                      |
| [UnconditionalPeers](./switch.md#accepting-peers)      | empty                 | These are IDs of the peers which are allowed to be (re)connected as both inbound or outbound regardless of whether the node reached `max_num_inbound_peers` or `max_num_outbound_peers` or not. |
| PersistentPeersMaxDialPeriod                           | 0 * time.Second       | Maximum pause when redialing a persistent peer (if zero, exponential backoff is used)                                                                                                           |
| FlushThrottleTimeout                                   | 10 * time.Millisecond | Time to wait before flushing messages out on the connection                                                                                                                                     |
| MaxPacketMsgPayloadSize                                | 1024                  | Maximum size of a message packet payload, in bytes                                                                                                                                              |
| SendRate                                               | 5120000 (5 mB/s)      | Rate at which packets can be sent, in bytes/second                                                                                                                                              |
| RecvRate                                               | 5120000 (5 mB/s)      | Rate at which packets can be received, in bytes/second                                                                                                                                          |
| [PexReactor](./pex.md)                                 | true                  | Set true to enable the peer-exchange reactor                                                                                                                                                    |
| SeedMode                                               | false                 | Seed mode, in which node constantly crawls the network and looks for. Does not work if the peer-exchange reactor is disabled.                                                                   |
| PrivatePeerIDs                                         | empty                 | Comma separated list of peer IDsthat we do not add to the address book or gossip to other peers. They stay private to us.                                                                       |
| AllowDuplicateIP                                       | false                 | Toggle to disable guard against peers connecting from the same ip.                                                                                                                              |
| [HandshakeTimeout](./transport.md#connection-upgrade)  | 20 * time.Second      | Timeout for handshake completion between peers                                                                                                                                                  |
| [DialTimeout](./switch.md#dialing-peers)               | 3 * time.Second       | Timeout for dialing a peer                                                                                                                                                                      |


These parameters can be set using the `$CMTHOME/config/config.toml` file. A subset of them can also be changed via command line using the following command line flags:

| Parameter           | Flag                         | Example                                                                                                                        |
| ---                 | ---                          | ---                                                                                                                            |
| Listen address      | `p2p.laddr`                  | "tcp://0.0.0.0:26656"                                                                                                          |
| Seed nodes          | `p2p.seeds`                  | `--p2p.seeds “id100000000000000000000000000000000@1.2.3.4:26656,id200000000000000000000000000000000@2.3.4.5:4444”`             |
| Persistent peers    | `p2p.persistent_peers`       | `--p2p.persistent_peers “id100000000000000000000000000000000@1.2.3.4:26656,id200000000000000000000000000000000@2.3.4.5:26656”` |
| Unconditional peers | `p2p.unconditional_peer_ids` | `--p2p.unconditional_peer_ids “id100000000000000000000000000000000,id200000000000000000000000000000000”`                       |
| PexReactor          | `p2p.pex`                    | `--p2p.pex`                                                                                                                    |
| Seed mode           | `p2p.seed_mode`              | `--p2p.seed_mode`                                                                                                              |
| Private peer ids    | `p2p.private_peer_ids`       | `--p2p.private_peer_ids “id100000000000000000000000000000000,id200000000000000000000000000000000”`                             |

 **Note on persistent peers**

 If `persistent_peers_max_dial_period` is set greater than zero, the
pause between each dial to each persistent peer will not exceed `persistent_peers_max_dial_period`
during exponential backoff and we keep trying again without giving up.

If `seeds` and `persistent_peers` intersect,
the user will be warned that seeds may auto-close connections
and that the node may not be able to keep the connection persistent.
