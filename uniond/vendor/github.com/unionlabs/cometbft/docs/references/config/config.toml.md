---
order: 1
parent:
  title: config.toml
  description: CometBFT general configuration
  order: 3
---
<!---
The current CometBFT documentation template is not perfect for reference manuals.
These style suggestions make it more readable.
--->
<style>
.PageContent table {margin: 0;}           /* tables are left-aligned */
</style>

<!--- Entry template: Use this as a template to add more parameter descriptions.
### tablename.property_key <- full path to property in the file
This is the two-sentence summary of the parameter. It does all kinds of stuff.
```toml
tablename.property_key = "default value"
```

| Value type          | string/bool/int/etc                | <- can be further clarified (duration, hex, base64-encoded)
|:--------------------|:-----------------------------------|
| **Possible values** | `"default value or exact example"` |
|                     | `generic description`              |

Lorem ipsum and everything else that you can think about this value and stuff.
This is the enhanced description and additional notes that is worth talking about.
Description about the possible values too.
--->

# config.toml
The `config.toml` file is a standard [TOML](https://toml.io/en/v1.0.0) file that configures the basic functionality
of CometBFT, including the configuration of the reactors.

The default configuration file created by running the command `cometbft init`. `The config.toml` is created with
all the parameters set with their default values.

All relative paths in the configuration are relative to `$CMTHOME`.
(See [the HOME folder](./README.md#the-home-folder) for more details.)

## Base configuration
The root table defines generic node settings. It is implemented in a struct called `BaseConfig`, hence the name.

### version
The version of the CometBFT binary that created or last modified the config file.
```toml
version = "1.0.0"
```

| Value type          | string                  |
|:--------------------|:------------------------|
| **Possible values** | semantic version string |
|                     | `""`                    |

This string validates the configuration file for the binary. The string has to be either a
[valid semver](https://semver.org) string or an empty string. In any other case, the binary halts with an
`ERROR: error in config file: invalid version string` error.

In the future, the code might make restrictions on what version of the file is compatible with what version of the
binary. There is no such check in place right now. Configuration and binary versions are interchangeable.

### proxy_app
The TCP or UNIX socket of the ABCI application or the name of an example ABCI application compiled in with the CometBFT
library.
```toml
proxy_app = "tcp://127.0.0.1:26658"
```

| Value type          | string                                                  |
|:--------------------|:--------------------------------------------------------|
| **Possible values** | TCP Stream socket (e.g. `"tcp://127.0.0.1:26658"`)      |
|                     | Unix domain socket (e.g. `"unix:///var/run/abci.sock"`) |
|                     | `"kvstore"`                                             |
|                     | `"persistent_kvstore"`                                  |
|                     | `"noop"`                                                |

When the ABCI application is written in a different language than Golang, (for example the
[Nomic binary](https://github.com/nomic-io/nomic) is written in Rust) the application can open a TCP port or create a
UNIX domain socket to communicate with CometBFT, while CometBFT runs as a separate process.

IP addresses other than `localhost` (IPv4: `127.0.0.1`, IPv6: `::1`) are strongly discouraged. It has not been tested, and it has strong performance and security implications.
The [abci](#abci) parameter is used in conjunction with this parameter to define the protocol used for communication.

In other cases (for example in the [Gaia binary](https://github.com/cosmos/gaia)), CometBFT is imported as a library
and the configuration entry is unused.

For development and testing, the [built-in ABCI application](../../guides/app-dev/abci-cli.md) can be used without additional processes running.

### moniker
A custom human-readable name for this node.
```toml
moniker = "my.host.name"
```

| Value type          | string                                                   |
|:--------------------|:---------------------------------------------------------|
| **Possible values** | any human-readable string                                |

The main use of this entry is to keep track of the different nodes in a local environment. For example, the `/status` RPC
endpoint will return the node moniker in the `.result.moniker` key.

Monikers do not need to be unique. They are for local administrator use and troubleshooting.

Nodes on the peer-to-peer network are identified by `nodeID@host:port` as discussed in the
[node_key.json](node_key.json.md) section.

### db_backend
The chosen database backend for the node.
```toml
db_backend = "goleveldb"
```

| Value type          | string        | dependencies  | GitHub                                           |
|:--------------------|:--------------|:--------------|:-------------------------------------------------|
| **Possible values** | `"goleveldb"` | pure Golang   | [goleveldb](https://github.com/syndtr/goleveldb) |
|                     | `"cleveldb"`  | requires gcc  | [leveldb](https://github.com/google/leveldb)     |
|                     | `"boltdb"`    | pure Golang   | [bbolt](https://github.com/etcd-io/bbolt)        |
|                     | `"rocksdb"`   | requires gcc  | [grocksdb](https://github.com/linxGnu/grocksdb)  |
|                     | `"badgerdb"`  | pure Golang   | [badger](https://github.com/dgraph-io/badger)    |
|                     | `"pebbledb"`  | pure Golang   | [pebble](https://github.com/cockroachdb/pebble)  |

During the build process, by default, only the `goleveldb` library is built into the binary.
To add support for alternative databases, you need to add them in the build tags.
For example: `go build -tags cleveldb,rocksdb`.

The RocksDB fork has API changes from the upstream RocksDB implementation. All other databases claim a stable API.

The CometBFT team tests rely on the GoLevelDB implementation. All other implementations are considered experimental from
a CometBFT perspective. The supported databases are part of the [cometbft-db](https://github.com/cometbft/cometbft-db) library
that CometBFT uses as a common database interface to various databases.

**NOTE**: `boltdb` and `cleveldb` are deprecated and will be removed in a future release.

### db_dir
The directory path where the database is stored.
```toml
db_dir = "data"
```

| Value type          | string                                           |
|:--------------------|:-------------------------------------------------|
| **Possible values** | relative directory path, appended to `$CMTHOME` |
|                     | absolute directory path                         |

The default relative path translates to `$CMTHOME/data`. In case `$CMTHOME` is unset, it defaults to
`$HOME/.cometbft/data`.

### log_level
A comma-separated list of `module:level` pairs that describe the log level of each module. Alternatively, a single word
can be set which will apply that log level to all modules.
```toml
log_level = "info"
```

| Value type     | string          |                                        |
|:---------------|:----------------|----------------------------------------|
| **Modules**    | `"main"`        | CometBFT main application logs         |
|                | `"consensus"`   | consensus reactor logs                 |
|                | `"p2p"`         | p2p reactor logs                       |
|                | `"pex"`         | Peer Exchange logs                     |
|                | `"proxy"`       | ABCI proxy service (MultiAppConn) logs |
|                | `"abci-client"` | ABCI client service logs               |
|                | `"rpc-server"`  | RPC server logs                        |
|                | `"txindex"`     | Indexer service logs                   |
|                | `"events"`      | Events service logs                    |
|                | `"pubsub"`      | PubSub service logs                    |
|                | `"evidence"`    | Evidence reactor logs                  |
|                | `"statesync"`   | StateSync reactor logs                 |
|                | `"mempool"`     | Mempool reactor logs                   |
|                | `"blocksync"`   | BlockSync reactor logs                 |
|                | `"state"`       | Pruner service logs                    |
|                | `"*"`           | All modules                            |
| **Log levels** | `"debug"`       |                                        |
|                | `"info"`        |                                        |
|                | `"error"`       |                                        |
|                | `"none"`        |                                        |

At the end of a `module:level` list, a default log level can be set for modules with no level set. Use `*` instead of a
module name to set a default log level. The default is `*:info`.

Examples:

Set the consensus reactor to `debug` log level and the `p2p` reactor to `none`. Everything else should be set to `error`:
```toml
log_level = "consensus:debug,p2p:none,*:error"
```
Set RPC server logs to `debug` and leave everything else at `info`:
```toml
log_level = "rpc-server:debug"
```

### log_format
Define the output format of the logs.
```toml
log_format = "plain"
```

| Value type          | string    |
|:--------------------|:----------|
| **Possible values** | `"plain"` |
|                     | `"json"`  |

`plain` provides ANSI color-coded plain-text logs.

`json` provides JSON objects (one per line, not prettified) using the following (incomplete) schema:
```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "$id": "https://cometbft.com/log.schema.json",
  "title": "JSON log",
  "description": "A log entry in JSON object format",
  "type": "object",
  "properties": {
    "level": {
      "description": "log level",
      "type": "string"
    },
    "ts": {
      "description": "timestamp in RFC3339Nano format; the trailing zeroes are removed from the seconds field",
      "type": "string"
    },
    "_msg": {
      "description": "core log message",
      "type": "string"
    },
    "module": {
      "description": "module name that emitted the log",
      "type": "string"
    },
    "impl": {
      "description": "some modules point out specific areas or tasks in log entries",
      "type": "string"
    },
    "msg": {
      "description": "some entries have more granular messages than just the core _msg",
      "type": "string"
    },
    "height": {
      "description": "some entries happen at a specific height",
      "type": "integer",
      "exclusiveMinimum": 0
    },
    "app_hash": {
      "description": "some entries happen at a specific app_hash",
      "type": "string"
    }
  },
  "required": [ "level", "ts", "_msg", "module" ]
}
```
> Note: The list of properties is not exhaustive. When implementing log parsing, check your logs and update the schema.

<!--- Todo: Probably we should create separate schemas for the different log levels or modules. --->

### genesis_file
Path to the JSON file containing the initial conditions for a CometBFT blockchain and the initial state of the application (more details [here](./genesis.json.md)).
```toml
genesis_file = "config/genesis.json"
```

| Value type          | string                                          |
|:--------------------|:------------------------------------------------|
| **Possible values** | relative directory path, appended to `$CMTHOME` |
|                     | absolute directory path                         |

The default relative path translates to `$CMTHOME/config/genesis.json`. In case `$CMTHOME` is unset, it defaults to
`$HOME/.cometbft/config/genesis.json`.

### priv_validator_key_file
Path to the JSON file containing the private key to use as a validator in the consensus protocol (more details [here](./priv_validator_key.json.md)).
```toml
priv_validator_key_file = "config/priv_validator_key.json"
```

| Value type          | string                                          |
|:--------------------|:------------------------------------------------|
| **Possible values** | relative directory path, appended to `$CMTHOME` |
|                     | absolute directory path                         |

The default relative path translates to `$CMTHOME/config/priv_validator_key.json`. In case `$CMTHOME` is unset, it
defaults to `$HOME/.cometbft/config/priv_validator_key.json`.


### priv_validator_state_file
Path to the JSON file containing the last sign state of a validator (more details [here](./priv_validator_state.json.md)).
```toml
priv_validator_state_file = "data/priv_validator_state.json"
```

| Value type          | string                                          |
|:--------------------|:------------------------------------------------|
| **Possible values** | relative directory path, appended to `$CMTHOME` |
|                     | absolute directory path                         |

The default relative path translates to `$CMTHOME/data/priv_validator_state.json`. In case `$CMTHOME` is unset, it
defaults to `$HOME/.cometbft/data/priv_validator_state.json`.

### priv_validator_laddr
TCP or UNIX socket listen address for CometBFT that allows external consensus signing processes to connect.
```toml
priv_validator_laddr = ""
```

| Value type          | string                                                     |
|:--------------------|:-----------------------------------------------------------|
| **Possible values** | TCP Stream socket (e.g. `"tcp://127.0.0.1:26665"`)         |
|                     | Unix domain socket (e.g. `"unix:///var/run/privval.sock"`) |

When consensus signing is outsourced from CometBFT (typically to a Hardware Security Module, like a
[YubiHSM](https://www.yubico.com/product/yubihsm-2) device), this address is opened by CometBFT for incoming connections
from the signing service.

Make sure the port is available on the host machine and firewalls allow the signing service to connect to it.

More information on a supported signing service can be found in the [TMKMS](https://github.com/iqlusioninc/tmkms)
documentation.

### node_key_file
Path to the JSON file containing the private key to use for node authentication in the p2p protocol (more details [here](./node_key.json.md)).
```toml
node_key_file = "config/node_key.json"
```

| Value type          | string                                          |
|:--------------------|:------------------------------------------------|
| **Possible values** | relative directory path, appended to `$CMTHOME` |
|                     | absolute directory path                         |

The default relative path translates to `$CMTHOME/config/node_key.json`. In case `$CMTHOME` is unset, it defaults to
`$HOME/.cometbft/config/node_key.json`.

### abci
The mechanism used to connect to the ABCI application.
````toml
abci = "socket"
````

| Value type          | string     |
|:--------------------|:-----------|
| **Possible values** | `"socket"` |
|                     | `"grpc"`   |
|                     | `""    `   |

This mechanism is used when connecting to the ABCI application over the [proxy_app](#proxy_app) socket.

### filter_peers
When connecting to a new peer, filter the connection through an ABCI query to decide, if the connection should be kept.
```toml
filter_peers = false
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `false` |
|                     | `true`  |

When this setting is `true`, the ABCI application has to implement a query that will allow
the connection to be kept of or dropped.

This feature will likely be deprecated.

## RPC Server
These configuration options change the behaviour of the built-in RPC server.

The RPC server is exposed without any kind of security control or authentication. Do NOT expose this server
on the public Internet without appropriate precautions. Make sure it is secured, load-balanced, etc.

### rpc.laddr
TCP or UNIX socket address for the RPC server to listen on.
```toml
laddr = "tcp://127.0.0.1:26657"
```

| Value type          | string                                            |
|:--------------------|:--------------------------------------------------|
| **Possible values** | TCP Stream socket (e.g. `"tcp://127.0.0.1:26657"`)     |
|                     | Unix domain socket (e.g. `"unix:///var/run/rpc.sock"`) |

The RPC server endpoints have OpenAPI specification definitions through [Swagger UI](../../rpc).
<!---
NOTE: The OpenAPI reference (../../rpc) is injected into the documentation during
the CometBFT docs build process. See https://github.com/cometbft/cometbft-docs/
for details.
--->

Please refer to the [RPC documentation](https://docs.cometbft.com/v1.0/rpc/) for more information.

### rpc.cors_allowed_origins
A list of origins a cross-domain request can be executed from.
```toml
cors_allowed_origins = []
```

| Value type          | array of string                            |                      |
|:--------------------|:-------------------------------------------|----------------------|
| **Possible values** | `[]`                                       | disable CORS support |
|                     | `["*"]`                                    | allow any origin     |
|                     | array of strings containing domain origins |                      |

Domain origins are fully qualified domain names with protocol prefixed, for example `"https://cometbft.com"` or
they can contain exactly one wildcard to extend to multiple subdomains, for example: `"https://*.myapis.com"`.

Example:

Allow only some subdomains for CORS requests:
```toml
cors_allowed_origins = ["https://www.cometbft.com", "https://*.apis.cometbft.com"]
```

### rpc.cors_allowed_methods
A list of methods the client is allowed to use with cross-domain requests.
```toml
cors_allowed_methods = ["HEAD", "GET", "POST", ]
```

| Value type                              | array of string |
|:----------------------------------------|:----------------|
| **Possible string values in the array** | `"HEAD"`        |
|                                         | `"GET"`         |
|                                         | `"POST"`        |

You can read more about the methods in the
[Mozilla CORS documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS).

### rpc.cors_allowed_headers
A list of headers the client is allowed to use with cross-domain requests.
```toml
cors_allowed_headers = ["Origin", "Accept", "Content-Type", "X-Requested-With", "X-Server-Time", ]
```

| Value type                              | array of string      |
|:----------------------------------------|:---------------------|
| **Possible string values in the array** | `"Accept"`           |
|                                         | `"Accept-Language"`  |
|                                         | `"Content-Language"` |
|                                         | `"Content-Type"`     |
|                                         | `"Range"`            |

The list of possible values are from the [Fetch spec](https://fetch.spec.whatwg.org/#cors-safelisted-request-header)
which defines `Origin` as a forbidden value. Read the
[Mozilla CORS documentation](https://developer.mozilla.org/en-US/docs/Web/HTTP/CORS) and do your own tests, if you want
to use this parameter.

<!--- Possibly, we should clarify the allowed values better. --->

### rpc.unsafe
Activate unsafe RPC endpoints.
```toml
unsafe = false
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `false` |
|                     | `true`  |

| Unsafe RPC endpoints    | Description                                                                           |
|:------------------------|---------------------------------------------------------------------------------------|
| `/dial_seeds`           | dials the given seeds (comma-separated id@IP:port)                                    |
| `/dial_peers`           | dials the given peers (comma-separated id@IP:port), optionally making them persistent |
| `/unsafe_flush_mempool` | removes all transactions from the mempool                                             |

Keep this `false` on production systems.

### rpc.max_open_connections
Maximum number of simultaneous open connections. This includes WebSocket connections.
```toml
max_open_connections = 900
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt; 0  |

If you want to accept a larger number of connections than the default 900, make sure that you increase the maximum
number of open connections in the operating system. Usually, the `ulimit` command can help with that.

This value can be estimated by the following calculation:
```
$(ulimit -Sn) - {p2p.max_num_inbound_peers} - {p2p.max_num_outbound_peers} - {number of WAL, DB and other open files}
```

Estimating the number of WAL, DB and other files at `50`, and using the default soft limit of Debian Linux (`1024`):
```
1024 - 40 - 10 - 50 = 924 (~900)
```

Note, that macOS has a default soft limit of `256`. Make sure you calculate this value for the operating system CometBFT
runs on.

### rpc.max_subscription_clients
Maximum number of unique clientIDs that can subscribe to events at the `/subscribe` RPC endpoint.
```toml
max_subscription_clients = 100
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

### rpc.max_subscriptions_per_client
Maximum number of unique queries a given client can subscribe to at the `/subscribe` RPC endpoint.
```toml
max_subscriptions_per_client = 5
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

### rpc.experimental_subscription_buffer_size
> EXPERIMENTAL parameter!

Experimental parameter to specify the maximum number of events a node will buffer, per subscription, before returning
an error and closing the subscription.
```toml
experimental_subscription_buffer_size = 200
```

| Value type          | integer   |
|:--------------------|:----------|
| **Possible values** | &gt;= 100 |

Higher values will accommodate higher event throughput rates (and will use more memory).

### rpc.experimental_websocket_write_buffer_size
> EXPERIMENTAL parameter!

Experimental parameter to specify the maximum number of events that can be buffered per WebSocket client.
```toml
experimental_websocket_write_buffer_size = 200
```

| Value type          | integer                                         |
|:--------------------|:------------------------------------------------|
| **Possible values** | &gt;= rpc.experimental_subscription_buffer_size |

If clients cannot read from the WebSocket endpoint fast enough, they will be disconnected, so increasing this parameter
may reduce the chances of them being disconnected (but will cause the node to use more memory).

If set lower than `rpc.experimental_subscription_buffer_size`, connections could be dropped unnecessarily. This value
should ideally be somewhat higher to accommodate non-subscription-related RPC responses.

### rpc.experimental_close_on_slow_client
> EXPERIMENTAL parameter!

Close the WebSocket client in case it cannot read events fast enough. Allows greater predictability in subscription
behaviour.
```toml
experimental_close_on_slow_client = false
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `false` |
|                     | `true`  |

The default behaviour for WebSocket clients is to silently drop events, if they cannot read them fast enough. This does
not cause an error and creates unpredictability.
Enabling this setting creates a predictable outcome by closing the WebSocket connection in case it cannot read events
fast enough.

### rpc.timeout_broadcast_tx_commit
Timeout waiting for a transaction to be committed when using the `/broadcast_tx_commit` RPC endpoint.
```toml
timeout_broadcast_tx_commit = "10s"
```

| Value type          | string (duration)          |
|:--------------------|:---------------------------|
| **Possible values** | &gt; `"0s"`; &lt;= `"10s"` |

Using a value larger than `"10s"` will result in increasing the global HTTP write timeout, which applies to all connections
and endpoints. There is an old developer discussion about this [here](https://github.com/tendermint/tendermint/issues/3435).

> Note: It is generally recommended *not* to use the `broadcast_tx_commit` method in production, and instead prefer `/broadcast_tx_sync`.

### rpc.max_request_batch_size
Maximum number of requests that can be sent in a JSON-RPC batch request.
```toml
max_request_batch_size = 10
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

If the number of requests sent in a JSON-RPC batch exceed the maximum batch size configured, an error will be returned.

The default value is set to `10`, which will limit the number of requests to 10 requests per a JSON-RPC batch request.

If you don't want to enforce a maximum number of requests for a batch request set this value to `0`.

Reference: https://www.jsonrpc.org/specification#batch

### rpc.max_body_bytes
Maximum size of request body, in bytes.
```toml
max_body_bytes = 1000000
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

### rpc.max_header_bytes
Maximum size of request header, in bytes.
```toml
max_header_bytes = 1048576
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

### rpc.tls_cert_file
TLS certificates file path for HTTPS server use.
```toml
tls_cert_file = ""
```

| Value type          | string                                                 |
|:--------------------|:-------------------------------------------------------|
| **Possible values** | relative directory path, appended to `$CMTHOME/config` |
|                     | absolute directory path                                |
|                     |  `""`                                                  |

The default relative path translates to `$CMTHOME/config`. In case `$CMTHOME` is unset, it defaults to
`$HOME/.cometbft/config`.

If the certificate is signed by a certificate authority, the certificate file should be the concatenation of the
server certificate, any intermediate certificates, and the Certificate Authority certificate.

The [rpc.tls_key_file](#rpctls_key_file) property also has to be set with the matching private key.

If this property is not set, the HTTP protocol will be used by the default server

### rpc.tls_key_file
TLS private key file path for HTTPS server use.
```toml
tls_key_file = ""
```

| Value type          | string                                                 |
|:--------------------|:-------------------------------------------------------|
| **Possible values** | relative directory path, appended to `$CMTHOME/config` |
|                     | absolute directory path                                |
|                     | `""`                                                   |

The default relative path translates to `$CMTHOME/config`. In case `$CMTHOME` is unset, it defaults to
`$HOME/.cometbft/config`.

The [rpc.tls_cert_file](#rpctls_cert_file) property also has to be set with the matching server certificate.

If this property is not set, the HTTP protocol will be used by the default server

### rpc.pprof_laddr
Profiling data listen address and port. Without protocol prefix.
```toml
pprof_laddr = ""
```

| Value type          | string                       |
|:--------------------|:-----------------------------|
| **Possible values** | IP:port (`"127.0.0.1:6060"`) |
|                     | :port (`":6060"`)            |
|                     | `""`                         |

HTTP is always assumed as the protocol.

See the Golang [profiling](https://golang.org/pkg/net/http/pprof) documentation for more information.

## gRPC Server
These configuration options change the behaviour of the built-in gRPC server.

Each gRPC service can be turned on/off, and in some cases configured, individually.
If the gRPC server is not enabled, all individual services' configurations are ignored.

The gRPC server is exposed without any kind of security control or authentication. Do NOT expose this server
on the public Internet without appropriate precautions. Make sure it is secured, authenticated, load-balanced, etc.

### grpc.laddr
TCP or UNIX socket address for the gRPC server to listen on.
```toml
laddr = ""
```

| Value type          | string                                                  |
|:--------------------|:--------------------------------------------------------|
| **Possible values** | TCP Stream socket (e.g. `"tcp://127.0.0.1:26661"`)      |
|                     | Unix domain socket (e.g. `"unix:///var/run/abci.sock"`) |
|                     | `""`                                                    |

If not specified, the gRPC server will be disabled.

### grpc.version_service.enabled
The gRPC version service provides version information about the node and the protocols it uses.
```toml
enabled = true
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `true`  |
|                     | `false` |

If [`grpc.laddr`](#grpcladdr) is empty, this setting is ignored and the service is not enabled.

### grpc.block_service.enabled
The gRPC block service returns block information.
```toml
enabled = true
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `true`  |
|                     | `false` |

If [`grpc.laddr`](#grpcladdr) is empty, this setting is ignored and the service is not enabled.

### grpc.block_results_service.enabled
The gRPC block results service returns block results for a given height. If no height is given, it will return the block
results from the latest height.
```toml
enabled = true
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `true`  |
|                     | `false` |

If [`grpc.laddr`](#grpcladdr) is empty, this setting is ignored and the service is not enabled.

### grpc.privileged.laddr
Configuration for privileged gRPC endpoints, which should **never** be exposed to the public internet.
```toml
laddr = ""
```

| Value type          | string                                                  |
|:--------------------|:--------------------------------------------------------|
| **Possible values** | TCP Stream socket (e.g. `"tcp://127.0.0.1:26662"`)      |
|                     | Unix domain socket (e.g. `"unix:///var/run/abci.sock"`) |
|                     | `""`                                                    |

If not specified, the gRPC privileged endpoints will be disabled.

### grpc.privileged.pruning_service
Configuration specifically for the gRPC pruning service, which is considered a privileged service.
```toml
enabled = false
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `false` |
|                     | `true`  |

Only controls whether the pruning service is accessible via the gRPC API - not whether a previously set pruning service
retain height is honored by the node. See the [storage.pruning](#storagepruninginterval) section for control over pruning.

If [`grpc.laddr`](#grpcladdr) is empty, this setting is ignored and the service is not enabled.

## Peer-to-peer

These configuration options change the behaviour of the peer-to-peer protocol.

### p2p.laddr

TCP socket address for the P2P service to listen on and accept connections.
```toml
laddr = "tcp://0.0.0.0:26656"
```

| Value type          | string                                            |
|:--------------------|:--------------------------------------------------|
| **Possible values** | TCP Stream socket (e.g. `"tcp://0.0.0.0:26657"`)     |

### p2p.external_address

TCP address that peers should use in order to connect to the node.
This is the address that the node advertises to peers.
If not set, the [`p2p.laddr`](#p2pladdr) is advertised.

Useful when the node is running on a non-routable address or when the
node does not have the capabilities to figure out its IP public address.
For example, this is useful when running from a cloud service (e.g, AWS, Digital Ocean).
In these scenarios, the public or external address of the node should be set to
`p2p.external_address`, while INADDR_ANY (i.e., `0.0.0.0`) should be used as
the listen address ([`p2p.laddr`](#p2pladdr)).

```toml
external_address = ""
```

| Value type          | string                      |
|:--------------------|:----------------------------|
| **Possible values** | IP:port (`"1.2.3.4:26656"`) |
|                     | `""`                        |

The port has to point to the node's P2P port.

Example with a node on a NATed non-routable network:
- Node has local or private IP address `10.10.10.10` and uses port `10000` for
  P2P communication: set this address as the [listen address](#p2pladdr) (`p2p.laddr`).
- The network gateway has the public IP `1.2.3.4` and we want to use publicly
  open port `26656` on the IP address. In this case, a redirection has to be
  set up from `1.2.3.4:26656` to `10.10.10.10:1000` in the gateway implementing NAT;
- Or the node has an associated public or external IP `1.2.3.4`
  that is mapped to its local or private IP.
- Set `p2p.external_address` to `1.2.3.4:26656`.

### p2p.seeds

Comma-separated list of seed nodes.

```toml
seeds = ""
```

| Value type                        | string (comma-separated list)           |
|:----------------------------------|:----------------------------------------|
| **Possible values within commas** | nodeID@IP:port (`"abcd@1.2.3.4:26656"`) |
|                                   | `""`                                    |

The node will try to connect to any of the configured seed nodes when it needs
addresses of potential peers to connect.

Example:
```toml
seeds = "abcd@1.2.3.4:26656,deadbeef@5.6.7.8:10000"
```

### p2p.persistent_peers

Comma-separated list of nodes to keep persistent connections to.

```toml
persistent_peers = ""
```

| Value type                        | string (comma-separated list)           |
|:----------------------------------|:----------------------------------------|
| **Possible values within commas** | nodeID@IP:port (`"abcd@1.2.3.4:26656"`) |
|                                   | `""`                                    |

The node will attempt to establish connections to all configured persistent peers.
This in particular means that persistent peers do not count towards
the configured [`p2p.max_num_outbound_peers`](#p2pmax_num_outbound_peers)
(refer to [issue 1304](https://github.com/cometbft/cometbft/issues/1304) for more details).
Moreover, if a connection to a persistent peer is lost, the node will attempt
reconnecting to that peer.

Once connected to a persistent peer, the node will request addresses of
potential peers.
This means that when persistent peers are configured the node may not need to
rely on potential peers provided by [seed nodes](#p2pseeds).

Example:
```toml
persistent_peers = "fedcba@11.22.33.44:26656,beefdead@55.66.77.88:20000"
```

### p2p.addr_book_file

Path to the address book file.

```toml
addr_book_file = "config/addrbook.json"
```

| Value type          | string                                          |
|:--------------------|:------------------------------------------------|
| **Possible values** | relative directory path, appended to `$CMTHOME` |
|                     | absolute directory path                         |

The default relative path translates to `$CMTHOME/config/addrbook.json`. In case `$CMTHOME` is unset, it defaults to
`$HOME/.cometbft/config/addrbook.json`.

The node periodically persists the content of its address book (addresses of
potential peers and information regarding connected peers) to the address book file.
If the node is started with a non-empty address book file, it may not need to
rely on potential peers provided by [seed nodes](#p2pseeds).

### p2p.addr_book_strict

Strict address routability rules disallow non-routable IP addresses in the address book. When `false`, private network
IP addresses are enabled to be stored in the address book and dialed.

```toml
addr_book_strict = true
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `true`  |
|                     | `false` |

Set it to `false` for testing on private network. Most production nodes can keep it at `true`.

### p2p.max_num_inbound_peers

Maximum number of inbound peers,
that is, peers from which the node accepts connections.

```toml
max_num_inbound_peers = 40
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

The [`p2p.max_num_inbound_peers`](#p2pmax_num_inbound_peers) and
[`p2p.max_num_outbound_peers`](#p2pmax_num_outbound_peers) values
work together to define how many P2P connections the node will
maintain at maximum capacity.

Nodes configured as [unconditional peers](#p2punconditional_peer_ids) do not count towards the
configured `p2p.max_num_inbound_peers` limit.

The connections are bidirectional, so any connection can send or receive messages, blocks, and other data. The separation into
inbound and outbound setting only distinguishes the initial setup of the connection: outbound connections are initiated
by the node while inbound connections are initiated by a remote party.

Nodes on non-routable networks have to set their gateway to port-forward the P2P port for inbound connections to reach
the node. Inbound connections can be accepted as long as the node has an address accessible from the Internet (using NAT or other methods).
Refer to the [p2p.external_address](#p2pexternal_address) configuration for details.

### p2p.max_num_outbound_peers

Maximum number of outbound peers,
that is, peers to which the node dials and establishes connections.

```toml
max_num_outbound_peers = 10
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

The [`p2p.max_num_inbound_peers`](#p2pmax_num_inbound_peers) and
[`p2p.max_num_outbound_peers`](#p2pmax_num_outbound_peers) values
work together to define how many P2P connections the node will
maintain at maximum capacity.

The `p2p.max_num_outbound_peers` configuration should be seen as the target
number of outbound connections that a node is expected to establish.
While the maximum configured number of outbound connections is not reached,
the node will attempt to establish connections to potential peers.

This configuration only has effect if the [PEX reactor](#p2ppex) is enabled.
Nodes configured as [persistent peers](#p2ppersistent_peers) do not count towards the
configured `p2p.max_num_outbound_peers` limit
(refer to [issue 1304](https://github.com/cometbft/cometbft/issues/1304) for more details).

The connections are bidirectional, so any connection can send or receive messages, blocks, and other data. The separation into
inbound and outbound setting only distinguishes the initial setup of the connection: outbound connections are initiated
by the node while inbound connections are initiated by a remote party.

Nodes on non-routable networks have to set their gateway to port-forward the P2P port for inbound connections to reach
the node. Outbound connections can only be initiated to peers that have addresses accessible from the Internet (using NAT or other methods).
Refer to the [p2p.external_address](#p2pexternal_address) configuration for details.

### p2p.unconditional_peer_ids

List of node IDs that are allowed to connect to the node even when connection limits are exceeded.

```toml
unconditional_peer_ids = ""
```

| Value type          | string (comma-separated)         |
|:--------------------|:---------------------------------|
| **Possible values** | comma-separated list of node IDs |
|                     | `""`                             |

If a peer listed in this property establishes a connection to the node, it will be accepted even if the
configured [`p2p.max_num_inbound_peers`](#p2pmax_num_inbound_peers) limit was reached.
Peers on this list also do not count towards the
configured [`p2p.max_num_outbound_peers`](#p2pmax_num_outbound_peers) limit.

Contrary to other settings, only the node ID has to be defined here, not the IP:port of the remote node.

### p2p.persistent_peers_max_dial_period

Maximum pause between successive attempts when dialing a persistent peer.

```toml
persistent_peers_max_dial_period = "0s"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0s"`      |

When set to `"0s"`, an exponential backoff is applied when re-dialing the persistent peer over and over.

### p2p.flush_throttle_timeout

Time to wait before flushing messages out on a connection.

```toml
flush_throttle_timeout = "10ms"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0ms"`     |

The flush operation writes any buffered data to the connection. The flush is throttled, so if multiple triggers come in within the
configured timeout, only one flush is executed.

Setting the value to `0ms` makes flushing messages out on a connection immediate.
While this might reduce latency, it may degrade throughput as batching
outstanding messages is essentially disabled.

### p2p.max_packet_msg_payload_size

Maximum size of a packet payload, in bytes.

```toml
max_packet_msg_payload_size = 1024
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt; 0  |

Messages exchanged via P2P connections are split into packets.
Packets contain some metadata and message data (payload).
The value configures the maximum size in bytes of the payload
included in a packet.

### p2p.send_rate

Rate at which packets can be sent, in bytes/second.

```toml
send_rate = 5120000
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt; 0  |

The value represents the amount of packet bytes that can be sent per second
by each P2P connection.

### p2p.recv_rate

Rate at which packets can be received, in bytes/second.

```toml
recv_rate = 5120000
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt; 0  |

The value represents the amount of packet bytes that can be received per second
by each P2P connection.

### p2p.pex

```toml
pex = true
```

Enable peer exchange (PEX) reactor.

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `true`  |
|                     | `false` |

The peer exchange reactor is responsible for exchanging addresses of potential
peers among nodes.
If the PEX reactor is disabled, the node can only connect to
addresses configured as [persistent peers](#p2ppersistent_peers).

In the [Sentry Node Architecture](https://forum.cosmos.network/t/sentry-node-architecture-overview/454) on the Cosmos Hub,
validator nodes should have the PEX reactor disabled,
as their connections are manually configured via [persistent peers](#p2ppersistent_peers).
Public nodes, such as sentry nodes, should have the PEX reactor enabled,
as this allows them to discover and connect to public peers in the network.

### p2p.seed_mode

In seed mode, the node crawls the network and looks for peers.

```toml
seed_mode = false
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `false` |
|                     | `true`  |

In seed mode, the node becomes an online address book. Any incoming connections
can receive a sample of the gathered addresses but no other information (for
example blocks or consensus data) is provided. The node simply disconnects
from the peer after sending the addresses.

Nodes operating in seed mode should be configured as [seeds](#p2pseeds) for other
nodes in the network.

The [`p2p.pex`](#p2ppex) option has to be set to `true` for the seed mode to work.

### p2p.private_peer_ids

Comma separated list of peer IDs to keep private, they will not be gossiped to other peers.

```toml
private_peer_ids = ""
```

| Value type                        | string (comma-separated list)     |
|:----------------------------------|:----------------------------------|
| **Possible values within commas** | nodeID (`"abcdef0123456789abcd"`) |
|                                   | `""`                              |

The addresses with the listed node IDs will not be sent to other peers when the PEX reactor
([`p2p.pex`](#p2ppex)) is enabled. This allows a more granular setting instead of completely disabling the peer exchange
reactor.

For example, sentry nodes in the
[Sentry Node Architecture](https://forum.cosmos.network/t/sentry-node-architecture-overview/454) on the Cosmos Hub can
use this setting to make sure they do not gossip the node ID of the validator node, while they can still accept node
addresses from the Internet.

### p2p.allow_duplicate_ip

Toggle to disable guard against peers connecting from the same IP.

```toml
allow_duplicate_ip = false
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `false` |
|                     | `true`  |

When this setting is set to `true`, multiple connections are allowed from the same IP address (for example, on different
ports).

### p2p.handshake_timeout

Timeout duration for protocol handshake (or secret connection negotiation).

```toml
handshake_timeout = "20s"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0s"`      |

This high-level timeout value is applied when the TCP connection has been
established with a peer, and the node and peer are negotiating its upgrade into
a secret authenticated connection.

The value `"0s"` is undefined, and it can lead to unexpected behaviour.

### p2p.dial_timeout

Timeout duration for the low-level dialer that connects to the remote address on the TCP network.

```toml
dial_timeout = "3s"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0s"`      |

This parameter is the timeout value for dialing on TCP networks. If a hostname is used instead of an IP address and the
hostname resolves to multiple IP addresses, the timeout is spread over each consecutive dial, such that each is given an
appropriate fraction of the time to connect.

Setting the value to `"0s"` disables the timeout.

## Mempool
Mempool allows gathering and broadcasting uncommitted transactions among nodes.

The **mempool** is storage for uncommitted transactions; the **mempool cache** is internal storage within the
mempool for seen transactions. The mempool cache provides a list of transactions already received to filter out
incoming duplicate transactions and prevent duplicate full transaction validations.

### mempool.type
The type of mempool this node will use.
```toml
type = "flood"
```

| Value type          | string    |
|:--------------------|:----------|
| **Possible values** | `"flood"` |
|                     | `"nop"`   |

`"flood"` is the original mempool implemented for CometBFT. It is a concurrent linked list with flooding gossip
protocol.

`"nop"` is a "no operation" or disabled mempool, where the ABCI application is responsible for storing, disseminating and
proposing transactions. Note, that it requires empty blocks to be created:
[`consensus.create_empty_blocks = true`](#consensuscreate_empty_blocks) has to be set.

### mempool.recheck
Validity check of transactions already in the mempool when a block is finalized.
```toml
recheck = true
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `true`  |
|                     | `false` |

Committing a block affects the application state, hence the remaining transactions in the mempool after a block commit
might become invalid. Setting `recheck = true` will go through the remaining transactions and remove invalid ones.

If your application may remove transactions passed by CometBFT to your `PrepareProposal` handler,
you probably want to set this configuration to `true` to avoid possible leaks in your mempool
(transactions staying in the mempool until the node is next restarted).

### mempool.recheck_timeout
Time to wait for the application to return CheckTx responses after all recheck requests have been
sent. Responses that arrive after the timeout expires are discarded.
```toml
recheck_timeout = "1000ms"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"1000ms"`   |

This setting only applies to non-local ABCI clients and when `recheck` is enabled.

The ideal value will strongly depend on the application. It could roughly be estimated as the
average size of the mempool multiplied by the average time it takes the application to validate one
transaction. We consider that the ABCI application runs in the same location as the CometBFT binary
(see [`proxy_app`](#proxy_app)) so that the recheck duration is not affected by network delays when
making requests and receiving responses.

### mempool.broadcast
Broadcast the mempool content (uncommitted transactions) to other nodes.
```toml
broadcast = true
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `true`  |
|                     | `false` |

This ensures that uncommitted transactions have a chance to reach multiple validators and get committed by one of them.

Setting this to `false` will stop the mempool from relaying transactions to other peers.
Validators behind sentry nodes typically set this to `false`,
as their sentry nodes take care of disseminating transactions to the rest of the network.

### mempool.wal_dir
Mempool write-ahead log folder path.
```toml
wal_dir = ""
```

| Value type          | string                                          |
|:--------------------|:------------------------------------------------|
| **Possible values** | relative directory path, appended to `$CMTHOME` |
|                     | absolute directory path                         |
|                     | `""`                                            |

In case `$CMTHOME` is unset, it defaults to `$HOME/.cometbft`.

This value is unused by CometBFT. It was not hooked up to the mempool reactor.

The mempool implementation does not persist any transaction data to disk (unlike evidence).

### mempool.size
Maximum number of transactions in the mempool.
```toml
size = 5000
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

If the mempool is full, incoming transactions are dropped.

The value `0` is undefined.

### mempool.max_tx_bytes
Maximum size in bytes of a single transaction accepted into the mempool.
```toml
max_tx_bytes = 1048576
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

Transactions bigger than the maximum configured size are rejected by mempool,
this applies to both transactions submitted by clients via RPC endpoints, and
transactions receveing from peers on the mempool protocol.

### mempool.max_txs_bytes
The maximum size in bytes of all transactions stored in the mempool.
```toml
max_txs_bytes = 67108864
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

This is the raw, total size in bytes of all transactions in the mempool. For example, given 1MB
transactions and a 5MB maximum mempool byte size, the mempool will
only accept five transactions.

The maximum mempool byte size should be a factor of the network's maximum block size
(which is a [consensus parameter](https://docs.cometbft.com/v1.0/spec/abci/abci++_app_requirements#blockparamsmaxbytes)).
The rationale is to consider how many blocks have to be produced in order to
drain all transactions stored in a full mempool.

When the mempool is full, incoming transactions are dropped.

The default value is 64 Mibibyte (2^26 bytes).
This is roughly equivalent to 16 blocks of 4 MiB.

### mempool.cache_size
Mempool internal cache size for already seen transactions.
```toml
cache_size = 10000
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

The mempool cache is an internal store for transactions that the local node has already seen. Storing these transactions help in filtering incoming duplicate
transactions: we can compare incoming transactions to already seen transactions and filter them out without going
through the process of validating the incoming transaction.

### mempool.keep-invalid-txs-in-cache
Invalid transactions might become valid in the future, hence they are not added to the mempool cache by default.
Turning this setting on will add an incoming transaction to the cache even if it is deemed invalid by the application (via `CheckTx`).
```toml
keep-invalid-txs-in-cache = false
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `false` |
|                     | `true`  |

If this setting is set to `true`, the mempool cache will add incoming transactions even if they are invalid. It is useful in cases when
invalid transactions can never become valid again.

This setting can be used by operators to lower the impact of some spam transactions: when a large number of duplicate
spam transactions are noted on the network, temporarily turning this setting to `true` will filter out the duplicates
quicker than validating each transaction one-by-one. It will also filter out transactions that are supposed to become
valid at a later date.

### mempool.experimental_max_gossip_connections_to_persistent_peers
> EXPERIMENTAL parameter!

Limit the number of persistent peer nodes that get mempool transaction broadcasts.
```toml
experimental_max_gossip_connections_to_persistent_peers = 0
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

When set to `0`, the mempool is broadcasting to all the nodes listed in the
[`p2p.persistent_peers`](#p2ppersistent_peers) list. If the number is above `0`, the number of nodes that get broadcasts
will be limited to this setting.

Unconditional peers and peers not listed in the [`p2p.persistent_peers`](#p2ppersistent_peers) list are not affected by
this parameter.

See
[`mempool.experimental_max_gossip_connections_to_non_persistent_peers`](#mempoolexperimental_max_gossip_connections_to_persistent_peers)
to limit mempool broadcasts that are not in the list of [`p2p.persistent_peers`](#p2ppersistent_peers).

### mempool.experimental_max_gossip_connections_to_non_persistent_peers
> EXPERIMENTAL parameter!

Limit the number of peer nodes that get mempool transaction broadcasts. This parameter does not limit nodes that are
in the [`p2p.persistent_peers`](#p2ppersistent_peers) list.
```toml
experimental_max_gossip_connections_to_non_persistent_peers = 0
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

When set to `0`, the mempool is broadcasting to all the nodes. If the number is above `0`, the number of nodes that get
broadcasts will be limited to this setting.

Unconditional peers and peers listed in the [`p2p.persistent_peers`](#p2ppersistent_peers) list are not affected by
this parameter.

See
[`mempool.experimental_max_gossip_connections_to_persistent_peers`](#mempoolexperimental_max_gossip_connections_to_persistent_peers)
to limit broadcasts to persistent peer nodes.

For non-persistent peers, if enabled, a value of 10 is recommended based on experimental performance results using the
default P2P configuration.

## State synchronization
State sync rapidly bootstraps a new node by discovering, fetching, and restoring a state machine snapshot from peers
instead of fetching and replaying historical blocks. It requires some peers in the network to take and serve state
machine snapshots. State sync is not attempted if the starting node has any local state (i.e., it is recovering).

The node will have a truncated block history, starting from the height of the snapshot.

### statesync.enable
Enable state synchronization.
```toml
enable = false
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `false` |
|                     | `true`  |

Enable state synchronization on first start.

### statesync.rpc_servers
Comma-separated list of RPC servers for light client verification of the synced state machine,
and retrieval of state data for node bootstrapping.
```toml
rpc_servers = ""
```

| Value type                        | string (comma-separated list)      |
|:----------------------------------|:-----------------------------------|
| **Possible values within commas** | nodeID@IP:port (`"1.2.3.4:26657"`) |
|                                   | `""`                               |

At least two RPC servers have to be defined for state synchronization to work.

### statesync.trust_height
The height of the trusted header hash.
```toml
trust_height = 0
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

`0` is only allowed when state synchronization is disabled.

### statesync.trust_hash
Header hash obtained from a trusted source.
```toml
trust_hash = ""
```

| Value type          | string             |
|:--------------------|:-------------------|
| **Possible values** | hex-encoded number |
|                     | ""                 |

`""` is only allowed when state synchronization is disabled.

This is the header hash value obtained from the trusted source at height
[statesync.trust_height](#statesynctrust_height).

### statesync.trust_period
The period during which validators can be trusted.
```toml
trust_period = "168h0m0s"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0s"`      |

For Cosmos SDK-based chains, `statesync.trust_period` should usually be about 2/3rd of the unbonding period
(about 2 weeks) during which they can be financially punished (slashed) for misbehavior.

### statesync.discovery_time
Time to spend discovering snapshots before initiating a restore.
```toml
discovery_time = "15s"
```

If `discovery_time` is &gt; 0 and  &lt; 5 seconds, its value will be overridden to 5 seconds.

If `discovery_time` is zero, the node will not wait for replies once it has broadcast the "snapshot request" message to its peers. If no snapshot data is received, state sync will fail without retrying.

If `discovery_time` is &gt;= 5 seconds, the node will broadcast the "snapshot request" message to its peers and then wait for `discovery_time`. If no snapshot data has been received after that period, the node will retry: it will broadcast the "snapshot request" message again and wait for `discovery_time`, and so on.

### statesync.temp_dir
Temporary directory for state sync snapshot chunks.
```toml
temp_dir = ""
```

| Value type          | string                  |
|:--------------------|:------------------------|
| **Possible values** | undefined               |

This value is unused by CometBFT. It was not hooked up to the state sync reactor.

The codebase will always revert to `/tmp/<random_name>` for state snapshot chunks. Make sure you have enough space on
your drive that holds `/tmp`.

### statesync.chunk_request_timeout
The timeout duration before re-requesting a chunk, possibly from a different peer.
```toml
chunk_request_timeout = "10s"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"5s"`      |

If a smaller duration is set when state syncing is enabled, an error message is raised.

### statesync.chunk_fetchers
The number of concurrent chunk fetchers to run.

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

`0` is only allowed when state synchronization is disabled.

## Block synchronization
Block synchronization configuration is limited to defining a version of block synchronization to use.

### blocksync.version
Block Sync version to use.
```toml
version = "v0"
```

| Value type          | string  |
|:--------------------|:--------|
| **Possible values** | `"v0"`  |

All other versions are deprecated. Further versions may be added in future releases.

## Consensus

Consensus parameters define how the consensus protocol should behave.

### consensus.wal_file

Location of the consensus Write-Ahead Log (WAL) file.

```toml
wal_file = "data/cs.wal/wal"
```

| Value type          | string                                          |
|:--------------------|:------------------------------------------------|
| **Possible values** | relative directory path, appended to `$CMTHOME` |
|                     | absolute directory path                         |

The default relative path translates to `$CMTHOME/data/cs.wal/wal`. In case `$CMTHOME` is unset, it defaults to
`$HOME/.cometbft/data/cs.wal/wal`.

The consensus WAL stores all consensus messages received and broadcast by a
node, as well as some important consensus events (e.g., new height and new round step).
The goal of this log is to enable a node that crashes and later recovers
to re-join consensus with the same state it has before crashing.
Recovering nodes that "forget" the actions taken before crashing are faulty
nodes that are likely to present Byzantine behavior (e.g., double signing).

## Consensus timeouts

In this section we describe the consensus timeout parameters. For a more detailed explanation
of these timeout parameters please refer to the [Consensus timeouts explained](#consensus-timeouts-explained)
section below.

### consensus.timeout_propose

How long a node waits for the proposal block before prevoting nil.

```toml
timeout_propose = "3s"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0s"`      |

The proposal block of a round of consensus is broadcast by the proposer of that round.
The `timeout_propose` should be large enough to encompass the common-case
propagation delay of a `Proposal` and one or more `BlockPart` (depending on the
proposed block size) messages from any validator to the node.

If the proposed block is not received within `timeout_propose`, validators
issue a prevote for nil, indicating that they have not received, and
therefore are unable to vote for, the block proposed in that round.

Setting `timeout_propose` to `0s` means that the validator does not wait at all
for the proposal block and always prevotes nil.
This has obvious liveness implications since this validator will never prevote
for proposed blocks.

### consensus.timeout_propose_delta

How much `timeout_propose` increases with each round.

```toml
timeout_propose_delta = "500ms"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0ms"`     |

Consensus timeouts are adaptive.
This means that when a round of consensus fails to commit a block, the next
round of consensus will adopt increased timeout durations.
Timeouts increase linearly over rounds, so that the `timeout_propose` adopted
in round `r` is `timeout_propose + r * timeout_propose_delta`.

### consensus.timeout_prevote

How long a node waits, after receiving +2/3 conflicting prevotes, before pre-committing nil.

```toml
timeout_prevote = "1s"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0s"`      |

A validator that receives +2/3 prevotes for a block, precommits that block.
If it receives +2/3 prevotes for nil, it precommits nil.
But if prevotes are received from +2/3 validators, but the prevotes do not
match (e.g., they are for different blocks or for blocks and nil), the
validator waits for `timeout_prevote` time before precommiting nil.
This gives the validator a chance to wait for additional prevotes and to
possibly observe +2/3 prevotes for a block.

Setting `timeout_prevote` to `0s` means that the validator will not wait
for additional prevotes (other than the mandatory +2/3) before precommitting nil.
This has important liveness implications and should be avoided.

### consensus.timeout_prevote_delta

How much the `timeout_prevote` increases with each round.

```toml
timeout_prevote_delta = "500ms"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0ms"`     |

Consensus timeouts are adaptive.
This means that when a round of consensus fails to commit a block, the next
round of consensus will adopt increased timeout durations.
Timeouts increase linearly over rounds, so that the `timeout_prevote` adopted
in round `r` is `timeout_prevote + r * timeout_prevote_delta`.

### consensus.timeout_precommit

How long a node waits, after receiving +2/3 conflicting precommits, before moving to the next round.

```toml
timeout_precommit = "1s"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0s"`      |

A node that receives +2/3 precommits for a block commits that block.
This is a successful consensus round.
If no block gathers +2/3 precommits, the node cannot commit.
This is an unsuccessful consensus round and the node will start an additional
round of consensus.
Before starting the next round, the node waits for `timeout_precommit` time.
This gives the node a chance to wait for additional precommits and to possibly
observe +2/3 precommits for a block, which would allow the node to commit that
block in the current round.

Setting `timeout_precommit` to `0s` means that the validator will not wait
for additional precommits (other than the mandatory +2/3) before moving to the
next round.
This has important liveness implications and should be avoided.

### consensus.timeout_precommit_delta
How much the timeout_precommit increases with each round.
```toml
timeout_precommit_delta = "500ms"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0ms"`     |

Consensus timeouts are adaptive.
This means that when a round of consensus fails to commit a block, the next
round of consensus will adopt increased timeout durations.
Timeouts increase linearly over rounds, so that the `timeout_precommit` adopted
in round `r` is `timeout_precommit + r * timeout_precommit_delta`.

### consensus.timeout_commit

How long a node waits after committing a block, before starting on the next height.

```toml
timeout_commit = "1s"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0s"`      |

The `timeout_commit` represents the minimum interval between the commit of a
block until the start of the next height of consensus.
It gives the node a chance to gather additional precommits for the committed
block, more than the mandatory +2/3 precommits required to commit a block.
The more precommits are gathered for a block, the greater are the safety
guarantees and the easier is to detect misbehaving validators.

The `timeout_commit` is not a required component of the consensus algorithm,
meaning that there are no liveness implications if it is set to `0s`.
But it may have implications in the way the application rewards validators.

Setting `timeout_commit` to `0s` means that the node will start the next height
as soon as it gathers all the mandatory +2/3 precommits for a block.

### consensus.skip_timeout_commit

Start the next height as soon as the node gathers all the mandatory +2/3 precommits for a block.

```toml
skip_timeout_commit = false
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `false` |
|                     | `true`  |

Deprecated: use `timeout_commit = "0s"` instead.

### consensus.double_sign_check_height

How many blocks to look back to check the existence of the node's consensus votes before joining consensus.

```toml
double_sign_check_height = 0
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

When non-zero, the validator will panic upon restart if the validator's current
consensus key was used to sign any precommit message for the last
`double_sign_check_height` blocks.
If this happens, the validators should stop the state machine, wait for some
blocks, and then restart the state machine again.

### consensus.create_empty_blocks

Propose empty blocks if the validator's mempool does not have any transaction.

```toml
create_empty_blocks = true
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `true`  |
|                     | `false` |

When set to `true`, empty blocks are produced and proposed to indicate that the
chain is still operative.


When set to `false`, blocks are not produced or proposed while there are no
transactions in the validator's mempool.

Notice that empty blocks are still proposed whenever the application hash
(`app_hash`) has been updated.

In this setting, blocks are created when transactions are received.

Note after the block H, CometBFT creates something we call a "proof block"
(only if the application hash changed) H+1. The reason for this is to support
proofs. If you have a transaction in block H that changes the state to X, the
new application hash will only be included in block H+1. If after your
transaction is committed, you want to get a light-client proof for the new state
(X), you need the new block to be committed in order to do that because the new
block has the new application hash for the state X. That's why we make a new
(empty) block if the application hash changes. Otherwise, you won't be able to
make a proof for the new state.

Plus, if you set `create_empty_blocks_interval` to something other than the
default (`0`), CometBFT will be creating empty blocks even in the absence of
transactions every `create_empty_blocks_interval`. For instance, with
`create_empty_blocks = false` and `create_empty_blocks_interval = "30s"`,
CometBFT will only create blocks if there are transactions, or after waiting
30 seconds without receiving any transactions.

Setting it to false is more relevant for networks with a low volume number of transactions.

### consensus.create_empty_blocks_interval

How long a validator should wait before proposing an empty block.

```toml
create_empty_blocks_interval = "0s"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0s"`      |

If there are no transactions in the validator's mempool, the validator
waits for `create_empty_blocks_interval` before producing and proposing an
empty block (with no transactions).

If [`create_empty_blocks`](#createemptyblocks) is set to `false` and
`create_empty_blocks_interval` is set to `0s`, the validator will wait
indefinitely until a transaction is available in its mempool,
to then produce and propose a block.

Notice that empty blocks are still proposed without waiting for `create_empty_blocks_interval`
whenever the application hash
(`app_hash`) has been updated.

### consensus.peer_gossip_sleep_duration

Consensus reactor internal sleep duration when there is no message to send to a peer.

```toml
peer_gossip_sleep_duration = "100ms"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0ms"`     |

The consensus reactor gossips consensus messages, by sending or forwarding them
to peers.
When there are no messages to be sent to a peer, each reactor routine waits for
`peer_gossip_sleep_duration` time before checking if there are new messages to
be sent to that peer, or if the peer state has been meanwhile updated.

This generic sleep duration allows other reactor routines to run when a reactor
routine has no work to do.

### consensus.peer_gossip_intraloop_sleep_duration

Consensus reactor upper bound for a random sleep duration.

```toml
peer_gossip_intraloop_sleep_duration = "0s"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0s"`     |

The consensus reactor gossips consensus messages, by sending or forwarding them
to peers.

If `peer_gossip_intraloop_sleep_duration` is set to a non-zero value, random
sleeps are inserted in the reactor routines when the node is waiting
for `HasProposalBlockPart` messages or `HasVote` messages.
The goal is to reduce the amount of `BlockPart` and `Vote` messages sent.
The value of this parameter is the upper bound for the random duration that is
used by the sleep commands inserted in each loop of the reactor routines.

### consensus.peer_query_maj23_sleep_duration

Consensus reactor interval between querying peers for +2/3 vote majorities.

```toml
peer_query_maj23_sleep_duration = "2s"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0s"`      |

The consensus reactor gossips consensus messages, by sending or forwarding them
to peers.

The `VoteSetMaj23` message is used by the consensus reactor to query peers
regarding vote messages (prevotes or precommits) they have for a specific
block.
These queries are only triggered when +2/3 votes are observed.

The value of `peer_query_maj23_sleep_duration` is the interval between sending
those queries to a peer.

## Storage
In production environments, configuring storage parameters accurately is essential as it can greatly impact the amount
of disk space utilized.

CometBFT supports storage pruning to delete data indicated as not needed by the application or the data companion.
Other than the pruning interval and compaction options, the configuration parameters in this section refer to the data
companion. The applications pruning configuration is communicated to CometBFT via ABCI.

Note that for some databases (GolevelDB), the data often does not get physically removed from storage due to the DB backend
not triggering compaction. In these cases it is necessary to enable forced compaction and set the compaction interval accordingly.

### storage.discard_abci_responses
Discard ABCI responses from the state store, which can save a considerable amount of disk space.
```toml
discard_abci_responses = false
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `false` |
|                     | `true`  |

If set to `false` ABCI responses are maintained, if set to `true` ABCI responses will be pruned.

ABCI responses are required for the `/block_results` RPC queries.

### storage.experimental_db_key_layout

The representation of keys in the database. The current representation of keys in Comet's stores is considered to be `v1`.

Users can experiment with a different layout by setting this field to `v2`. Note that this is an experimental feature
and switching back from `v2` to `v1` is not supported by CometBFT.

If the database was initially created with `v1`, it is necessary to migrate the DB before switching to `v2`. The migration
is not done automatically.

```toml
experimental_db_key_layout = 'v1'
```

| Value type          | string |
|:--------------------|:-------|
| **Possible values** | `v1`   |
|                     | `v2`   |

- `v1` - The legacy layout existing in Comet prior to v1.
- `v2` - Order preserving representation ordering entries by height.

If not specified, the default value `v1` will be used.

### storage.compact

If set to true, CometBFT will force compaction to happen for databases that support this feature and save on storage space.

Setting this to true is most beneficial when used in combination with pruning as it will physically delete the entries marked for deletion.

```toml
compact = false
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `false` |
|                     | `true`  |

`false` is the default value (forcing compaction is disabled).

### storage.compaction_interval

To avoid forcing compaction every time, this parameter instructs CometBFT to wait the given amount of blocks to be
pruned before triggering compaction.

It should be tuned depending on the number of items. If your retain height is 1 block, it is too much of an overhead
to try compaction every block. But it should also not be a very large multiple of your retain height as it might incur
bigger overheads.

| Value type          | string (# blocks) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0"`       |

```toml
compaction_interval = '1000'
```

### storage.pruning.interval
The time period between automated background pruning operations.
```toml
interval = "10s"
```

| Value type          | string (duration) |
|:--------------------|:------------------|
| **Possible values** | &gt;= `"0s"`      |

### storage.pruning.data_companion.enabled
Tell the automatic pruning function to respect values set by the data companion.

```toml
enabled = false
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `false` |
|                     | `true`  |

If disabled, only the application retain height will influence block pruning (but not block results pruning).

Only enabling this at a later stage will potentially mean that blocks below the application-set retain height at the
time will not be available to the data companion.

### storage.pruning.data_companion.initial_block_retain_height
The initial value for the data companion block retain height if the data companion has not yet explicitly set one.
If the data companion has already set a block retain height, this is ignored.
```toml
double_sign_check_height = 0
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

### storage.pruning.data_companion.initial_block_results_retain_height
The initial value for the data companion block results retain height if the data companion has not yet explicitly set
one. If the data companion has already set a block results retain height, this is ignored.
```toml
initial_block_results_retain_height = 0
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |


## Transaction indexer
Transaction indexer settings.

The application will set which txs to index.
In some cases, a node operator will be able to decide which txs to index based on the configuration set in the application.

### tx_index.indexer
What indexer to use for transactions.
```toml
indexer = "kv"
```

| Value type          | string   |
|:--------------------|:---------|
| **Possible values** | `"kv"`   |
|                     | `"null"` |
|                     | `"psql"` |

`"null"` indexer disables indexing.

`"kv"` is the simplest possible indexer, backed by a key-value storage.
The key-value storage database backend is defined in [`db_backend`](#db_backend).

`"psql"` indexer is backed by an external PostgreSQL server.
The server connection string is defined in [`tx_index.psql-conn`](#tx_indexpsql-conn).

The transaction height and transaction hash is always indexed, except with the `"null"` indexer.

### tx_index.psql-conn
The PostgreSQL connection configuration.
```toml
psql-conn = ""
```

| Value type          | string                                                       |
|:--------------------|:-------------------------------------------------------------|
| **Possible values** | `"postgresql://<user>:<password>@<host>:<port>/<db>?<opts>"` |
|                     | `""`                                                         |

### tx_index.table_*
Table names used by the PostgreSQL-backed indexer.

This setting is optional and only applies when `indexer`  is set to `psql`.

| Field         | default value               |
|:--------------------|:---------------------|
| `"table_blocks"` | `"blocks"`     |
| `"table_tx_results"` | `"tx_results"` |
| `"table_events"`    | `"events"`     |
| `"table_attributes"` | `"table_attributes"` |

## Prometheus Instrumentation
An extensive amount of Prometheus metrics are built into CometBFT.

### instrumentation.prometheus
Enable or disable presenting the Prometheus metrics at an endpoint.
```toml
prometheus = false
```

| Value type          | boolean |
|:--------------------|:--------|
| **Possible values** | `false` |
|                     | `true`  |

When enabled, metrics are served under the `/metrics` endpoint on the
[instrumentation.prometheus_listen_addr](#instrumentationprometheus_listen_addr) address.

### instrumentation.prometheus_listen_addr
Address to listen for Prometheus collector(s) connections.
```toml
prometheus_listen_addr = ":26660"
```

| Value type          | string                                |
|:--------------------|:--------------------------------------|
| **Possible values** | Network address (`"127.0.0.1:26657"`) |

If the IP address is omitted (see e.g. the default value) then the listening socket is bound to INADDR_ANY (`0.0.0.0`).

The metrics endpoint only supports HTTP.

### instrumentation.max_open_connections
Maximum number of simultaneous connections.
```toml
max_open_connections = 3
```

| Value type          | integer |
|:--------------------|:--------|
| **Possible values** | &gt;= 0 |

`0` allows unlimited connections.

### instrumentation.namespace
Instrumentation namespace
```toml
namespace = "cometbft"
```

| Value type          | string                    |
|:--------------------|:--------------------------|
| **Possible values** | Prometheus namespace name |

## Consensus timeouts explained

There's a variety of information about timeouts in [Running in
production](../../explanation/core/running-in-production.md#configuration-parameters).

You can also find more detailed explanation in the paper describing
the Tendermint consensus algorithm, adopted by CometBFT: [The latest
gossip on BFT consensus](https://arxiv.org/abs/1807.04938).

```toml
[consensus]
...

timeout_propose = "3s"
timeout_propose_delta = "500ms"
timeout_prevote = "1s"
timeout_prevote_delta = "500ms"
timeout_precommit = "1s"
timeout_precommit_delta = "500ms"
timeout_commit = "1s"
```

Note that in a successful round, the only timeout that we absolutely wait no
matter what is `timeout_commit`.

Here's a brief summary of the timeouts:

- `timeout_propose` = how long a validator should wait for a proposal block before prevoting nil
- `timeout_propose_delta` = how much `timeout_propose` increases with each round
- `timeout_prevote` = how long a validator should wait after receiving +2/3 prevotes for
  anything (ie. not a single block or nil)
- `timeout_prevote_delta` = how much the `timeout_prevote` increases with each round
- `timeout_precommit` = how long a validator should wait after receiving +2/3 precommits for
  anything (ie. not a single block or nil)
- `timeout_precommit_delta` = how much the `timeout_precommit` increases with each round
- `timeout_commit` = how long a validator should wait after committing a block, before starting
  on the new height (this gives us a chance to receive some more precommits,
  even though we already have +2/3)

### The adverse effect of using inconsistent `timeout_propose` in a network

Here's an interesting question. What happens if a particular validator sets a
very small `timeout_propose`, as compared to the rest of the network?

Imagine there are only two validators in your network: Alice and Bob. Bob sets
`timeout_propose` to 0s. Alice uses the default value of 3s. Let's say they
both have an equal voting power. Given the proposer selection algorithm is a
weighted round-robin, you may expect Alice and Bob to take turns proposing
blocks, and the result like:

```
#1 block - Alice
#2 block - Bob
#3 block - Alice
#4 block - Bob
...
```

What happens in reality is, however, a little bit different:

```
#1 block - Bob
#2 block - Bob
#3 block - Bob
#4 block - Bob
```

That's because Bob doesn't wait for a proposal from Alice (prevotes `nil`).
This leaves Alice no chances to commit a block. Note that every block Bob
creates needs a vote from Alice to constitute 2/3+. Bob always gets one because
Alice has `timeout_propose` set to 3s. Alice never gets one because Bob has it
set to 0s.

Imagine now there are ten geographically distributed validators. One of them
(Bob) sets `timeout_propose` to 0s. Others have it set to 3s. Now, Bob won't be
able to move with his own speed because it still needs 2/3 votes of the other
validators and it takes time to propagate those. I.e., the network moves with
the speed of time to accumulate 2/3+ of votes (prevotes & precommits), not with
the speed of the fastest proposer.

> Isn't block production determined by voting power?

If it were determined solely by voting power, it wouldn't be possible to ensure
liveness. Timeouts exist because the network can't rely on a single proposer
being available and must move on if such is not responding.

> How can we address situations where someone arbitrarily adjusts their block
> production time to gain an advantage?

The impact shown above is negligible in a decentralized network with enough
decentralization.

### The adverse effect of using inconsistent `timeout_commit` in a network

Let's look at the same scenario as before. There are ten geographically
distributed validators. One of them (Bob) sets `timeout_commit` to 0s. Others
have it set to 1s (the default value). Now, Bob will be the fastest producer
because he doesn't wait for additional precommits after creating a block. If
waiting for precommits (`timeout_commit`) is not incentivized, Bob will accrue
more rewards compared to the other 9 validators.

This is because Bob has the advantage of broadcasting its proposal early (1
second earlier than the others). But it also makes it possible for Bob to miss
a proposal from another validator and prevote `nil` due to him starting
`timeout_propose` earlier. I.e., if Bob's `timeout_commit` is too low comparing
to other validators, then he might miss some proposals and get slashed for
inactivity.
