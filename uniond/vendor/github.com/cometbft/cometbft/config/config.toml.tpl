# This is a TOML config file.
# For more information, see https://github.com/toml-lang/toml

# NOTE: Any path below can be absolute (e.g. "/var/myawesomeapp/data") or
# relative to the home directory (e.g. "data"). The home directory is
# "$HOME/.cometbft" by default, but could be changed via $CMTHOME env variable
# or --home cmd flag.

# The version of the CometBFT binary that created or
# last modified the config file. Do not modify this.
version = "{{ .BaseConfig.Version }}"

#######################################################################
###                   Main Base Config Options                      ###
#######################################################################

# TCP or UNIX socket address of the ABCI application,
# or the name of an ABCI application compiled in with the CometBFT binary
proxy_app = "{{ .BaseConfig.ProxyApp }}"

# A custom human readable name for this node
moniker = "{{ .BaseConfig.Moniker }}"

# Database backend: goleveldb | cleveldb | boltdb | rocksdb | badgerdb | pebbledb
# * goleveldb (github.com/syndtr/goleveldb)
#   - UNMAINTAINED
#   - stable
#   - pure go
# * cleveldb (uses levigo wrapper)
#   - DEPRECATED
#   - requires gcc
#   - use cleveldb build tag (go build -tags cleveldb)
# * boltdb (uses etcd's fork of bolt - github.com/etcd-io/bbolt)
#   - DEPRECATED
#   - EXPERIMENTAL
#   - stable
#   - use boltdb build tag (go build -tags boltdb)
# * rocksdb (uses github.com/linxGnu/grocksdb)
#   - EXPERIMENTAL
#   - requires gcc
#   - use rocksdb build tag (go build -tags rocksdb)
# * badgerdb (uses github.com/dgraph-io/badger)
#   - EXPERIMENTAL
#   - stable
#   - use badgerdb build tag (go build -tags badgerdb)
# * pebbledb (uses github.com/cockroachdb/pebble)
#   - EXPERIMENTAL
#   - stable
#   - pure go
#   - use pebbledb build tag (go build -tags pebbledb)
db_backend = "{{ .BaseConfig.DBBackend }}"

# Database directory
db_dir = "{{ js .BaseConfig.DBPath }}"

# Output level for logging, including package level options
log_level = "{{ .BaseConfig.LogLevel }}"

# Output format: 'plain' (colored text) or 'json'
log_format = "{{ .BaseConfig.LogFormat }}"

##### additional base config options #####

# Path to the JSON file containing the initial validator set and other meta data
genesis_file = "{{ js .BaseConfig.Genesis }}"

# Path to the JSON file containing the private key to use as a validator in the consensus protocol
priv_validator_key_file = "{{ js .BaseConfig.PrivValidatorKey }}"

# Path to the JSON file containing the last sign state of a validator
priv_validator_state_file = "{{ js .BaseConfig.PrivValidatorState }}"

# TCP or UNIX socket address for CometBFT to listen on for
# connections from an external PrivValidator process
priv_validator_laddr = "{{ .BaseConfig.PrivValidatorListenAddr }}"

# Path to the JSON file containing the private key to use for node authentication in the p2p protocol
node_key_file = "{{ js .BaseConfig.NodeKey }}"

# Mechanism to connect to the ABCI application: socket | grpc
abci = "{{ .BaseConfig.ABCI }}"

# If true, query the ABCI app on connecting to a new peer
# so the app can decide if we should keep the connection or not
filter_peers = {{ .BaseConfig.FilterPeers }}


#######################################################################
###                 Advanced Configuration Options                  ###
#######################################################################

#######################################################
###       RPC Server Configuration Options          ###
#######################################################
[rpc]

# TCP or UNIX socket address for the RPC server to listen on
laddr = "{{ .RPC.ListenAddress }}"

# A list of origins a cross-domain request can be executed from
# Default value '[]' disables cors support
# Use '["*"]' to allow any origin
cors_allowed_origins = [{{ range .RPC.CORSAllowedOrigins }}{{ printf "%q, " . }}{{end}}]

# A list of methods the client is allowed to use with cross-domain requests
cors_allowed_methods = [{{ range .RPC.CORSAllowedMethods }}{{ printf "%q, " . }}{{end}}]

# A list of non simple headers the client is allowed to use with cross-domain requests
cors_allowed_headers = [{{ range .RPC.CORSAllowedHeaders }}{{ printf "%q, " . }}{{end}}]

# Activate unsafe RPC commands like /dial_seeds and /unsafe_flush_mempool
unsafe = {{ .RPC.Unsafe }}

# Maximum number of simultaneous connections (including WebSocket).
# If you want to accept a larger number than the default, make sure
# you increase your OS limits.
# 0 - unlimited.
# Should be < {ulimit -Sn} - {MaxNumInboundPeers} - {MaxNumOutboundPeers} - {N of wal, db and other open files}
# 1024 - 40 - 10 - 50 = 924 = ~900
max_open_connections = {{ .RPC.MaxOpenConnections }}

# Maximum number of unique clientIDs that can /subscribe.
# If you're using /broadcast_tx_commit, set to the estimated maximum number
# of broadcast_tx_commit calls per block.
max_subscription_clients = {{ .RPC.MaxSubscriptionClients }}

# Maximum number of unique queries a given client can /subscribe to.
# If you're using /broadcast_tx_commit, set to the estimated maximum number
# of broadcast_tx_commit calls per block.
max_subscriptions_per_client = {{ .RPC.MaxSubscriptionsPerClient }}

# Experimental parameter to specify the maximum number of events a node will
# buffer, per subscription, before returning an error and closing the
# subscription. Must be set to at least 100, but higher values will accommodate
# higher event throughput rates (and will use more memory).
experimental_subscription_buffer_size = {{ .RPC.SubscriptionBufferSize }}

# Experimental parameter to specify the maximum number of RPC responses that
# can be buffered per WebSocket client. If clients cannot read from the
# WebSocket endpoint fast enough, they will be disconnected, so increasing this
# parameter may reduce the chances of them being disconnected (but will cause
# the node to use more memory).
#
# Must be at least the same as "experimental_subscription_buffer_size",
# otherwise connections could be dropped unnecessarily. This value should
# ideally be somewhat higher than "experimental_subscription_buffer_size" to
# accommodate non-subscription-related RPC responses.
experimental_websocket_write_buffer_size = {{ .RPC.WebSocketWriteBufferSize }}

# If a WebSocket client cannot read fast enough, at present we may
# silently drop events instead of generating an error or disconnecting the
# client.
#
# Enabling this experimental parameter will cause the WebSocket connection to
# be closed instead if it cannot read fast enough, allowing for greater
# predictability in subscription behavior.
experimental_close_on_slow_client = {{ .RPC.CloseOnSlowClient }}

# How long to wait for a tx to be committed during /broadcast_tx_commit.
# WARNING: Using a value larger than 10s will result in increasing the
# global HTTP write timeout, which applies to all connections and endpoints.
# See https://github.com/tendermint/tendermint/issues/3435
timeout_broadcast_tx_commit = "{{ .RPC.TimeoutBroadcastTxCommit }}"

# Maximum number of requests that can be sent in a batch
# If the value is set to '0' (zero-value), then no maximum batch size will be
# enforced for a JSON-RPC batch request.
max_request_batch_size = {{ .RPC.MaxRequestBatchSize }}

# Maximum size of request body, in bytes
max_body_bytes = {{ .RPC.MaxBodyBytes }}

# Maximum size of request header, in bytes
max_header_bytes = {{ .RPC.MaxHeaderBytes }}

# The path to a file containing certificate that is used to create the HTTPS server.
# Might be either absolute path or path related to CometBFT's config directory.
# If the certificate is signed by a certificate authority,
# the certFile should be the concatenation of the server's certificate, any intermediates,
# and the CA's certificate.
# NOTE: both tls_cert_file and tls_key_file must be present for CometBFT to create HTTPS server.
# Otherwise, HTTP server is run.
tls_cert_file = "{{ .RPC.TLSCertFile }}"

# The path to a file containing matching private key that is used to create the HTTPS server.
# Might be either absolute path or path related to CometBFT's config directory.
# NOTE: both tls_cert_file and tls_key_file must be present for CometBFT to create HTTPS server.
# Otherwise, HTTP server is run.
tls_key_file = "{{ .RPC.TLSKeyFile }}"

# pprof listen address (https://golang.org/pkg/net/http/pprof)
pprof_laddr = "{{ .RPC.PprofListenAddress }}"

#######################################################
###       gRPC Server Configuration Options         ###
#######################################################

#
# Note that the gRPC server is exposed unauthenticated. It is critical that
# this server not be exposed directly to the public internet. If this service
# must be accessed via the public internet, please ensure that appropriate
# precautions are taken (e.g. fronting with a reverse proxy like nginx with TLS
# termination and authentication, using DDoS protection services like
# CloudFlare, etc.).
#

[grpc]

# TCP or UNIX socket address for the RPC server to listen on. If not specified,
# the gRPC server will be disabled.
laddr = "{{ .GRPC.ListenAddress }}"

#
# Each gRPC service can be turned on/off, and in some cases configured,
# individually. If the gRPC server is not enabled, all individual services'
# configurations are ignored.
#

# The gRPC version service provides version information about the node and the
# protocols it uses.
[grpc.version_service]
enabled = {{ .GRPC.VersionService.Enabled }}

# The gRPC block service returns block information
[grpc.block_service]
enabled = {{ .GRPC.BlockService.Enabled }}

# The gRPC block results service returns block results for a given height. If no height
# is given, it will return the block results from the latest height.
[grpc.block_results_service]
enabled = {{ .GRPC.BlockResultsService.Enabled }}

#
# Configuration for privileged gRPC endpoints, which should **never** be exposed
# to the public internet.
#
[grpc.privileged]
# The host/port on which to expose privileged gRPC endpoints.
laddr = "{{ .GRPC.Privileged.ListenAddress }}"

#
# Configuration specifically for the gRPC pruning service, which is considered a
# privileged service.
#
[grpc.privileged.pruning_service]

# Only controls whether the pruning service is accessible via the gRPC API - not
# whether a previously set pruning service retain height is honored by the
# node. See the [storage.pruning] section for control over pruning.
#
# Disabled by default.
enabled = {{ .GRPC.Privileged.PruningService.Enabled }}

#######################################################
###           P2P Configuration Options             ###
#######################################################
[p2p]

# Address to listen for incoming connections
laddr = "{{ .P2P.ListenAddress }}"

# Address to advertise to peers for them to dial. If empty, will use the same
# port as the laddr, and will introspect on the listener to figure out the
# address. IP and port are required. Example: 159.89.10.97:26656
external_address = "{{ .P2P.ExternalAddress }}"

# Comma separated list of seed nodes to connect to
seeds = "{{ .P2P.Seeds }}"

# Comma separated list of nodes to keep persistent connections to
persistent_peers = "{{ .P2P.PersistentPeers }}"

# Path to address book
addr_book_file = "{{ js .P2P.AddrBook }}"

# Set true for strict address routability rules
# Set false for private or local networks
addr_book_strict = {{ .P2P.AddrBookStrict }}

# Maximum number of inbound peers
max_num_inbound_peers = {{ .P2P.MaxNumInboundPeers }}

# Maximum number of outbound peers to connect to, excluding persistent peers
max_num_outbound_peers = {{ .P2P.MaxNumOutboundPeers }}

# List of node IDs, to which a connection will be (re)established ignoring any existing limits
unconditional_peer_ids = "{{ .P2P.UnconditionalPeerIDs }}"

# Maximum pause when redialing a persistent peer (if zero, exponential backoff is used)
persistent_peers_max_dial_period = "{{ .P2P.PersistentPeersMaxDialPeriod }}"

# Time to wait before flushing messages out on the connection
flush_throttle_timeout = "{{ .P2P.FlushThrottleTimeout }}"

# Maximum size of a message packet payload, in bytes
max_packet_msg_payload_size = {{ .P2P.MaxPacketMsgPayloadSize }}

# Rate at which packets can be sent, in bytes/second
send_rate = {{ .P2P.SendRate }}

# Rate at which packets can be received, in bytes/second
recv_rate = {{ .P2P.RecvRate }}

# Set true to enable the peer-exchange reactor
pex = {{ .P2P.PexReactor }}

# Seed mode, in which node constantly crawls the network and looks for
# peers. If another node asks it for addresses, it responds and disconnects.
#
# Does not work if the peer-exchange reactor is disabled.
seed_mode = {{ .P2P.SeedMode }}

# Comma separated list of peer IDs to keep private (will not be gossiped to other peers)
private_peer_ids = "{{ .P2P.PrivatePeerIDs }}"

# Toggle to disable guard against peers connecting from the same ip.
allow_duplicate_ip = {{ .P2P.AllowDuplicateIP }}

# Peer connection configuration.
handshake_timeout = "{{ .P2P.HandshakeTimeout }}"
dial_timeout = "{{ .P2P.DialTimeout }}"

#######################################################
###          Mempool Configuration Options          ###
#######################################################
[mempool]

# The type of mempool for this node to use.
#
#  Possible types:
#  - "flood" : concurrent linked list mempool with flooding gossip protocol
#  (default)
#  - "nop"   : nop-mempool (short for no operation; the ABCI app is responsible
#  for storing, disseminating and proposing txs). "create_empty_blocks=false" is
#  not supported.
type = "flood"

# recheck (default: true) defines whether CometBFT should recheck the
# validity for all remaining transaction in the mempool after a block.
# Since a block affects the application state, some transactions in the
# mempool may become invalid. If this does not apply to your application,
# you can disable rechecking.
recheck = {{ .Mempool.Recheck }}

# recheck_timeout is the time the application has during the rechecking process
# to return CheckTx responses, once all requests have been sent. Responses that
# arrive after the timeout expires are discarded. It only applies to
# non-local ABCI clients and when recheck is enabled.
recheck_timeout = "{{ .Mempool.RecheckTimeout }}"

# broadcast (default: true) defines whether the mempool should relay
# transactions to other peers. Setting this to false will stop the mempool
# from relaying transactions to other peers until they are included in a
# block. In other words, if Broadcast is disabled, only the peer you send
# the tx to will see it until it is included in a block.
broadcast = {{ .Mempool.Broadcast }}

# wal_dir (default: "") configures the location of the Write Ahead Log
# (WAL) for the mempool. The WAL is disabled by default. To enable, set
# wal_dir to where you want the WAL to be written (e.g.
# "data/mempool.wal").
wal_dir = "{{ js .Mempool.WalPath }}"

# Maximum number of transactions in the mempool
size = {{ .Mempool.Size }}

# Maximum size in bytes of a single transaction accepted into the mempool.
max_tx_bytes = {{ .Mempool.MaxTxBytes }}

# The maximum size in bytes of all transactions stored in the mempool.
# This is the raw, total transaction size. For example, given 1MB
# transactions and a 5MB maximum mempool byte size, the mempool will
# only accept five transactions.
max_txs_bytes = {{ .Mempool.MaxTxsBytes }}

# Size of the cache (used to filter transactions we saw earlier) in transactions
cache_size = {{ .Mempool.CacheSize }}

# Do not remove invalid transactions from the cache (default: false)
# Set to true if it's not possible for any invalid transaction to become valid
# again in the future.
keep-invalid-txs-in-cache = {{ .Mempool.KeepInvalidTxsInCache }}

# Experimental parameters to limit gossiping txs to up to the specified number of peers.
# We use two independent upper values for persistent and non-persistent peers.
# Unconditional peers are not affected by this feature.
# If we are connected to more than the specified number of persistent peers, only send txs to
# ExperimentalMaxGossipConnectionsToPersistentPeers of them. If one of those
# persistent peers disconnects, activate another persistent peer.
# Similarly for non-persistent peers, with an upper limit of
# ExperimentalMaxGossipConnectionsToNonPersistentPeers.
# If set to 0, the feature is disabled for the corresponding group of peers, that is, the
# number of active connections to that group of peers is not bounded.
# For non-persistent peers, if enabled, a value of 10 is recommended based on experimental
# performance results using the default P2P configuration.
experimental_max_gossip_connections_to_persistent_peers = {{ .Mempool.ExperimentalMaxGossipConnectionsToPersistentPeers }}
experimental_max_gossip_connections_to_non_persistent_peers = {{ .Mempool.ExperimentalMaxGossipConnectionsToNonPersistentPeers }}

#######################################################
###         State Sync Configuration Options        ###
#######################################################
[statesync]
# State sync rapidly bootstraps a new node by discovering, fetching, and restoring a state machine
# snapshot from peers instead of fetching and replaying historical blocks. Requires some peers in
# the network to take and serve state machine snapshots. State sync is not attempted if the node
# has any local state (LastBlockHeight > 0). The node will have a truncated block history,
# starting from the height of the snapshot.
enable = {{ .StateSync.Enable }}

# RPC servers (comma-separated) for light client verification of the synced state machine and
# retrieval of state data for node bootstrapping. Also needs a trusted height and corresponding
# header hash obtained from a trusted source, and a period during which validators can be trusted.
#
# For Cosmos SDK-based chains, trust_period should usually be about 2/3 of the unbonding time (~2
# weeks) during which they can be financially punished (slashed) for misbehavior.
rpc_servers = "{{ StringsJoin .StateSync.RPCServers "," }}"
trust_height = {{ .StateSync.TrustHeight }}
trust_hash = "{{ .StateSync.TrustHash }}"
trust_period = "{{ .StateSync.TrustPeriod }}"

# Time to spend discovering snapshots before initiating a restore.
discovery_time = "{{ .StateSync.DiscoveryTime }}"

# Temporary directory for state sync snapshot chunks, defaults to the OS tempdir (typically /tmp).
# Will create a new, randomly named directory within, and remove it when done.
temp_dir = "{{ .StateSync.TempDir }}"

# The timeout duration before re-requesting a chunk, possibly from a different
# peer (default: 1 minute).
chunk_request_timeout = "{{ .StateSync.ChunkRequestTimeout }}"

# The number of concurrent chunk fetchers to run (default: 1).
chunk_fetchers = "{{ .StateSync.ChunkFetchers }}"

#######################################################
###       Block Sync Configuration Options          ###
#######################################################
[blocksync]

# Block Sync version to use:
#
# In v0.37, v1 and v2 of the block sync protocols were deprecated.
# Please use v0 instead.
#
#   1) "v0" - the default block sync implementation
version = "{{ .BlockSync.Version }}"

#######################################################
###         Consensus Configuration Options         ###
#######################################################
[consensus]

wal_file = "{{ js .Consensus.WalPath }}"

# How long we wait for a proposal block before prevoting nil
timeout_propose = "{{ .Consensus.TimeoutPropose }}"
# How much timeout_propose increases with each round
timeout_propose_delta = "{{ .Consensus.TimeoutProposeDelta }}"
# How long we wait after receiving +2/3 prevotes for “anything” (ie. not a single block or nil)
timeout_prevote = "{{ .Consensus.TimeoutPrevote }}"
# How much the timeout_prevote increases with each round
timeout_prevote_delta = "{{ .Consensus.TimeoutPrevoteDelta }}"
# How long we wait after receiving +2/3 precommits for “anything” (ie. not a single block or nil)
timeout_precommit = "{{ .Consensus.TimeoutPrecommit }}"
# How much the timeout_precommit increases with each round
timeout_precommit_delta = "{{ .Consensus.TimeoutPrecommitDelta }}"
# How long we wait after committing a block, before starting on the new
# height (this gives us a chance to receive some more precommits, even
# though we already have +2/3).
# Set to 0 if you want to make progress as soon as the node has all the precommits.
timeout_commit = "{{ .Consensus.TimeoutCommit }}"

# Deprecated: set `timeout_commit` to 0 instead.
skip_timeout_commit = {{ .Consensus.SkipTimeoutCommit }}

# How many blocks to look back to check existence of the node's consensus votes before joining consensus
# When non-zero, the node will panic upon restart
# if the same consensus key was used to sign {double_sign_check_height} last blocks.
# So, validators should stop the state machine, wait for some blocks, and then restart the state machine to avoid panic.
double_sign_check_height = {{ .Consensus.DoubleSignCheckHeight }}

# EmptyBlocks mode and possible interval between empty blocks
create_empty_blocks = {{ .Consensus.CreateEmptyBlocks }}
create_empty_blocks_interval = "{{ .Consensus.CreateEmptyBlocksInterval }}"

# Reactor sleep duration parameters
peer_gossip_sleep_duration = "{{ .Consensus.PeerGossipSleepDuration }}"
peer_gossip_intraloop_sleep_duration = "{{ .Consensus.PeerGossipIntraloopSleepDuration }}"
peer_query_maj23_sleep_duration = "{{ .Consensus.PeerQueryMaj23SleepDuration }}"

#######################################################
###         Storage Configuration Options           ###
#######################################################
[storage]

# Set to true to discard ABCI responses from the state store, which can save a
# considerable amount of disk space. Set to false to ensure ABCI responses are
# persisted. ABCI responses are required for /block_results RPC queries, and to
# reindex events in the command-line tool.
discard_abci_responses = {{ .Storage.DiscardABCIResponses}}

# The representation of keys in the database.
# The current representation of keys in Comet's stores is considered to be v1
# Users can experiment with a different layout by setting this field to v2.
# Note that this is an experimental feature and switching back from v2 to v1
# is not supported by CometBFT.
# If the database was initially created with v1, it is necessary to migrate the DB
# before switching to v2. The migration is not done automatically.
# v1 - the legacy layout existing in Comet prior to v1.
# v2 - Order preserving representation ordering entries by height.
experimental_db_key_layout = "{{ .Storage.ExperimentalKeyLayout }}"

# If set to true, CometBFT will force compaction to happen for databases that support this feature.
# and save on storage space. Setting this to true is most benefits when used in combination
# with pruning as it will physically delete the entries marked for deletion.
# false by default (forcing compaction is disabled).
compact = {{ .Storage.Compact }}

# To avoid forcing compaction every time, this parameter instructs CometBFT to wait
# the given amount of blocks to be pruned before triggering compaction.
# It should be tuned depending on the number of items. If your retain height is 1 block,
# it is too much of an overhead to try compaction every block. But it should also not be a very
# large multiple of your retain height as it might occur bigger overheads.
compaction_interval = "{{ .Storage.CompactionInterval }}"

[storage.pruning]

# The time period between automated background pruning operations.
interval = "{{ .Storage.Pruning.Interval }}"

#
# Storage pruning configuration relating only to the data companion.
#
[storage.pruning.data_companion]

# Whether automatic pruning respects values set by the data companion. Disabled
# by default. All other parameters in this section are ignored when this is
# disabled.
#
# If disabled, only the application retain height will influence block pruning
# (but not block results pruning). Only enabling this at a later stage will
# potentially mean that blocks below the application-set retain height at the
# time will not be available to the data companion.
enabled = {{ .Storage.Pruning.DataCompanion.Enabled }}

# The initial value for the data companion block retain height if the data
# companion has not yet explicitly set one. If the data companion has already
# set a block retain height, this is ignored.
initial_block_retain_height = {{ .Storage.Pruning.DataCompanion.InitialBlockRetainHeight }}

# The initial value for the data companion block results retain height if the
# data companion has not yet explicitly set one. If the data companion has
# already set a block results retain height, this is ignored.
initial_block_results_retain_height = {{ .Storage.Pruning.DataCompanion.InitialBlockResultsRetainHeight }}

#######################################################
###   Transaction Indexer Configuration Options     ###
#######################################################
[tx_index]

# What indexer to use for transactions
#
# The application will set which txs to index. In some cases a node operator will be able
# to decide which txs to index based on configuration set in the application.
#
# Options:
#   1) "null"
#   2) "kv" (default) - the simplest possible indexer, backed by key-value storage (defaults to levelDB; see DBBackend).
# 		- When "kv" is chosen "tx.height" and "tx.hash" will always be indexed.
#   3) "psql" - the indexer services backed by PostgreSQL.
# When "kv" or "psql" is chosen "tx.height" and "tx.hash" will always be indexed.
indexer = "{{ .TxIndex.Indexer }}"

# The PostgreSQL connection configuration, the connection format:
#   postgresql://<user>:<password>@<host>:<port>/<db>?<opts>
psql-conn = "{{ .TxIndex.PsqlConn }}"

#######################################################
###       Instrumentation Configuration Options     ###
#######################################################
[instrumentation]

# When true, Prometheus metrics are served under /metrics on
# PrometheusListenAddr.
# Check out the documentation for the list of available metrics.
prometheus = {{ .Instrumentation.Prometheus }}

# Address to listen for Prometheus collector(s) connections
prometheus_listen_addr = "{{ .Instrumentation.PrometheusListenAddr }}"

# Maximum number of simultaneous connections.
# If you want to accept a larger number than the default, make sure
# you increase your OS limits.
# 0 - unlimited.
max_open_connections = {{ .Instrumentation.MaxOpenConnections }}

# Instrumentation namespace
namespace = "{{ .Instrumentation.Namespace }}"
