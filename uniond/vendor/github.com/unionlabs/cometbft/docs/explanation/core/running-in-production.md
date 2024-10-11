---
order: 4
---

# Running in production

## Database

By default, CometBFT uses the `syndtr/goleveldb` package for its in-process
key-value database. See the [install
instructions](../introduction/install.md) for details.

CometBFT keeps multiple distinct databases in the `$CMTHOME/data`:

- `blockstore.db`: Keeps the entire blockchain - stores blocks,
  block commits, and block metadata, each indexed by height. Used to sync new
  peers.
- `evidence.db`: Stores all verified evidence of misbehavior.
- `state.db`: Stores the current blockchain state (i.e. height, validators,
  consensus params). Only grows if consensus params or validators change. Also
  used to temporarily store intermediate results during block processing.
- `tx_index.db`: Indexes transactions and by tx hash and height. The tx results are indexed if they are added to the `FinalizeBlock` response in the application.

> By default, CometBFT will only index transactions by their hash and height, if you want the result events to be indexed, see [indexing transactions](../../guides/app-dev/indexing-transactions.md#adding-events) 
for details.

Applications can expose block pruning strategies to the node operator.
Please read the documentation of your application to find out more details.

Applications can use [state sync](state-sync.md) to help nodes bootstrap quickly.

## Logging

Default logging level (`log_level = "main:info,state:info,statesync:info,*:error"`) should suffice for
normal operation mode. It will log info messages from the `main`, `state` and
`statesync` modules and error messages from all other modules.

The format of the logging level is:

```
<module1>:<level>,<module2>:<level>,...,<moduleN>:<level>
```

Where `<moduleN>` is the module that generated the log message, `<level>` is
one of the log levels: `info`, `error`, `debug` or `none`. Some of
the modules can be found [here](how-to-read-logs.md#list-of-modules). Others
could be observed by running CometBFT. `none` log level could be used
to suppress messages from a particular module or all modules (`log_level =
"state:info,*:none"` will only log info messages from the `state` module).

If you're trying to debug CometBFT or asked to provide logs with debug logging
level, you can do so by running CometBFT with `--log_level="*:debug"`.

## Write Ahead Logs (WAL)

CometBFT uses write ahead logs for the consensus (`cs.wal`) and the mempool
(`mempool.wal`). Both WALs have a max size of 1GB and are automatically rotated.

### Consensus WAL

The `consensus.wal` is used to ensure we can recover from a crash at any point
in the consensus state machine.
It writes all consensus messages (timeouts, proposals, block part, or vote)
to a single file, flushing to disk before processing messages from its own
validator. Since CometBFT validators are expected to never sign a conflicting vote, the
WAL ensures we can always recover deterministically to the latest state of the consensus without
using the network or re-signing any consensus messages.

If your `consensus.wal` is corrupted, see [below](#wal-corruption).

### Mempool WAL

The `mempool.wal` logs all incoming transactions before running CheckTx, but is
otherwise not used in any programmatic way. It's just a kind of manual
safe guard. Note the mempool provides no durability guarantees - a tx sent to one or many nodes
may never make it into the blockchain if those nodes crash before being able to
propose it. Clients must monitor their transactions by subscribing over websockets,
polling for them, or using `/broadcast_tx_commit`. In the worst case, transactions can be
resent from the mempool WAL manually.

For the above reasons, the `mempool.wal` is disabled by default. To enable, set
`mempool.wal_dir` to where you want the WAL to be located (e.g.
`data/mempool.wal`).

## DoS Exposure and Mitigation

Validators are supposed to setup [Sentry Node Architecture](validators.md)
to prevent Denial-of-Service attacks.

### P2P

The core of the CometBFT peer-to-peer system is `MConnection`. Each
connection has `MaxPacketMsgPayloadSize`, which is the maximum packet
size and bounded send & receive queues. One can impose restrictions on
send & receive rate per connection (`SendRate`, `RecvRate`).

The number of open P2P connections can become quite large, and hit the operating system's open
file limit (since TCP connections are considered files on UNIX-based systems). Nodes should be
given a sizable open file limit, e.g. 8192, via `ulimit -n 8192` or other deployment-specific
mechanisms.

### RPC

#### Attack Exposure and Mitigation

**It is generally not recommended for RPC endpoints to be exposed publicly, and
especially so if the node in question is a validator**, as the CometBFT RPC does
not currently provide advanced security features. Public exposure of RPC
endpoints without appropriate protection can make the associated node vulnerable
to a variety of attacks.

It is entirely up to operators to ensure, if nodes' RPC endpoints have to be
exposed publicly, that appropriate measures have been taken to mitigate against
attacks. Some examples of mitigation measures include, but are not limited to:

- Never publicly exposing the RPC endpoints of validators (i.e. if the RPC
  endpoints absolutely have to be exposed, ensure you do so only on full nodes
  and with appropriate protection)
- Correct usage of rate-limiting, authentication and caching (e.g. as provided
  by reverse proxies like [nginx](https://nginx.org/) and/or DDoS protection
  services like [Cloudflare](https://www.cloudflare.com))
- Only exposing the specific endpoints absolutely necessary for the relevant use
  cases (configurable via nginx/Cloudflare/etc.)

If no expertise is available to the operator to assist with securing nodes' RPC
endpoints, it is strongly recommended to never expose those endpoints publicly.

**Under no condition should any of the [unsafe RPC endpoints](../rpc/#/Unsafe)
ever be exposed publicly.**

#### Endpoints Returning Multiple Entries

Endpoints returning multiple entries are limited by default to return 30
elements (100 max). See the [RPC Documentation](https://docs.cometbft.com/main/rpc/)
for more information.

## Debugging CometBFT

If you ever have to debug CometBFT, the first thing you should probably do is
check out the logs. See [How to read logs](how-to-read-logs.md), where we
explain what certain log statements mean.

If, after skimming through the logs, things are not clear still, the next thing
to try is querying the `/status` RPC endpoint. It provides the necessary info:
whenever the node is syncing or not, what height it is on, etc.

```bash
curl http(s)://{ip}:{rpcPort}/status
```

`/dump_consensus_state` will give you a detailed overview of the consensus
state (proposer, latest validators, peers states). From it, you should be able
to figure out why, for example, the network had halted.

```bash
curl http(s)://{ip}:{rpcPort}/dump_consensus_state
```

There is a reduced version of this endpoint - `/consensus_state`, which returns
just the votes seen at the current height.

If, after consulting with the logs and above endpoints, you still have no idea
what's happening, consider using `cometbft debug kill` sub-command. This
command will scrap all the available info and kill the process. See
[Debugging](../../guides/tools/debugging.md) for the exact format.

You can inspect the resulting archive yourself or create an issue on
[Github](https://github.com/cometbft/cometbft). Before opening an issue
however, be sure to check if there's [no existing
issue](https://github.com/cometbft/cometbft/issues) already.

## Monitoring CometBFT

Each CometBFT instance has a standard `/health` RPC endpoint, which responds
with 200 (OK) if everything is fine and 500 (or no response) - if something is
wrong.

Other useful endpoints include mentioned earlier `/status`, `/net_info` and
`/validators`.

CometBFT also can report and serve Prometheus metrics. See
[Metrics](metrics.md).

`cometbft debug dump` sub-command can be used to periodically dump useful
information into an archive. See [Debugging](../../guides/tools/debugging.md) for more
information.

## What happens when my app dies

You are supposed to run CometBFT under a [process
supervisor](https://en.wikipedia.org/wiki/Process_supervision) (like
systemd or runit). It will ensure CometBFT is always running (despite
possible errors).

Getting back to the original question, if your application dies,
CometBFT will panic. After a process supervisor restarts your
application, CometBFT should be able to reconnect successfully. The
order of restart does not matter for it.

## Signal handling

We catch SIGINT and SIGTERM and try to clean up nicely. For other
signals we use the default behavior in Go:
[Default behavior of signals in Go programs](https://golang.org/pkg/os/signal/#hdr-Default_behavior_of_signals_in_Go_programs).

## Corruption

**NOTE:** Make sure you have a backup of the CometBFT data directory.

### Possible causes

Remember that most corruption is caused by hardware issues:

- RAID controllers with faulty / worn out battery backup, and an unexpected power loss
- Hard disk drives with write-back cache enabled, and an unexpected power loss
- Cheap SSDs with insufficient power-loss protection, and an unexpected power-loss
- Defective RAM
- Defective or overheating CPU(s)

Other causes can be:

- Database systems configured with fsync=off and an OS crash or power loss
- Filesystems configured to use write barriers plus a storage layer that ignores write barriers. LVM is a particular culprit.
- CometBFT bugs
- Operating system bugs
- Admin error (e.g., directly modifying CometBFT data-directory contents)

(Source: <https://wiki.postgresql.org/wiki/Corruption>)

### WAL Corruption

If consensus WAL is corrupted at the latest height and you are trying to start
CometBFT, replay will fail with panic.

Recovering from data corruption can be hard and time-consuming. Here are two approaches you can take:

1. Delete the WAL file and restart CometBFT. It will attempt to sync with other peers.
2. Try to repair the WAL file manually:

1) Create a backup of the corrupted WAL file:

    ```sh
    cp "$CMTHOME/data/cs.wal/wal" > /tmp/corrupted_wal_backup
    ```

2) Use `./scripts/wal2json` to create a human-readable version:

    ```sh
    ./scripts/wal2json/wal2json "$CMTHOME/data/cs.wal/wal" > /tmp/corrupted_wal
    ```

3) Search for a "CORRUPTED MESSAGE" line.
4) By looking at the previous message and the message after the corrupted one
   and looking at the logs, try to rebuild the message. If the consequent
   messages are marked as corrupted too (this may happen if length header
   got corrupted or some writes did not make it to the WAL ~ truncation),
   then remove all the lines starting from the corrupted one and restart
   CometBFT.

    ```sh
    $EDITOR /tmp/corrupted_wal
    ```

5) After editing, convert this file back into binary form by running:

    ```sh
    ./scripts/json2wal/json2wal /tmp/corrupted_wal  $CMTHOME/data/cs.wal/wal
    ```

## Hardware

### Processor and Memory

While actual specs vary depending on the load and validators count, minimal
requirements are:

- 1GB RAM
- 25GB of disk space
- 1.4 GHz CPU

SSD disks are preferable for applications with high transaction throughput.

Recommended:

- 2GB RAM
- 100GB SSD
- x64 2.0 GHz 2v CPU

While for now, CometBFT stores all the history and it may require significant
disk space over time, we are planning to implement state syncing (See [this
issue](https://github.com/tendermint/tendermint/issues/828)). So, storing all
the past blocks will not be necessary.

### Validator signing on 32 bit architectures (or ARM)

Both our `ed25519` and `secp256k1` implementations require constant time
`uint64` multiplication. Non-constant time crypto can (and has) leaked
private keys on both `ed25519` and `secp256k1`. This doesn't exist in hardware
on 32 bit x86 platforms ([source](https://bearssl.org/ctmul.html)), and it
depends on the compiler to enforce that it is constant time. It's unclear at
this point whether the Golang compiler does this correctly for all
implementations.

**We do not support nor recommend running a validator on 32 bit architectures OR
the "VIA Nano 2000 Series", and the architectures in the ARM section rated
"S-".**

### Operating Systems

CometBFT can be compiled for a wide range of operating systems thanks to Go
language (the list of \$OS/\$ARCH pairs can be found
[here](https://golang.org/doc/install/source#environment)).

While we do not favor any operation system, more secure and stable Linux server
distributions (like CentOS) should be preferred over desktop operation systems
(like Mac OS).

### Miscellaneous

NOTE: if you are going to use CometBFT in a public domain, make sure
you read [hardware recommendations](https://cosmos.network/validators) for a validator in the
Cosmos network.

## Configuration parameters

- `p2p.flush_throttle_timeout`
- `p2p.max_packet_msg_payload_size`
- `p2p.send_rate`
- `p2p.recv_rate`

If you are going to use CometBFT in a private domain and you have a
private high-speed network among your peers, it makes sense to lower
flush throttle timeout and increase other params.

```toml
[p2p]

send_rate=20000000 # 2MB/s
recv_rate=20000000 # 2MB/s
flush_throttle_timeout=10
max_packet_msg_payload_size=10240 # 10KB
```

- `mempool.recheck`

After every block, CometBFT rechecks every transaction left in the
mempool to see if transactions committed in that block affected the
application state, so some of the transactions left may become invalid.
If that does not apply to your application, you can disable it by
setting `mempool.recheck=false`.

- `mempool.broadcast`

Setting this to false will stop the mempool from relaying transactions
to other peers until they are included in a block. It means only the
peer you send the tx to will see it until it is included in a block.

- `consensus.peer_gossip_sleep_duration`

You can try to reduce the time your node sleeps before checking if
there's something to send its peers.

- `consensus.timeout_commit`

We want `timeout_commit` to be greater than zero when there is economics on the line
because proposers should wait to hear for more votes. But if you don't
care about that and want the fastest consensus, you can skip it. It will
be kept `1s` by default for public deployments (e.g. [Cosmos
Hub](https://hub.cosmos.network/)) while for enterprise
applications, setting it to `0` is not a problem.

You can try lowering it though.

- `p2p.addr_book_strict`

By default, CometBFT checks whenever a peer's address is routable before
saving it to the address book. The address is considered as routable if the IP
is [valid and within allowed ranges](https://github.com/cometbft/cometbft/blob/main/p2p/netaddress.go#L258).

This may not be the case for private or local networks, where your IP range is usually
strictly limited and private. If that case, you need to set `addr_book_strict`
to `false` (turn it off).

- `rpc.max_open_connections`

By default, the number of simultaneous connections is limited because most OS
give you limited number of file descriptors.

If you want to accept greater number of connections, you will need to increase
these limits.

[Sysctls to tune the system to be able to open more connections](https://github.com/satori-com/tcpkali/blob/master/doc/tcpkali.man.md#sysctls-to-tune-the-system-to-be-able-to-open-more-connections)

The process file limits must also be increased, e.g. via `ulimit -n 8192`.

...for N connections, such as 50k:

```md
kern.maxfiles=10000+2*N         # BSD
kern.maxfilesperproc=100+2*N    # BSD
kern.ipc.maxsockets=10000+2*N   # BSD
fs.file-max=10000+2*N           # Linux
net.ipv4.tcp_max_orphans=N      # Linux

# For load-generating clients.
net.ipv4.ip_local_port_range="10000  65535"  # Linux.
net.inet.ip.portrange.first=10000  # BSD/Mac.
net.inet.ip.portrange.last=65535   # (Enough for N < 55535)
net.ipv4.tcp_tw_reuse=1         # Linux
net.inet.tcp.maxtcptw=2*N       # BSD

# If using netfilter on Linux:
net.netfilter.nf_conntrack_max=N
echo $((N/8)) > /sys/module/nf_conntrack/parameters/hashsize
```

The similar option exists for limiting the number of gRPC connections -
`rpc.grpc_max_open_connections`.
