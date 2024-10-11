// Package flags defines beacon-node specific runtime flags for
// setting important values such as ports, eth1 endpoints, and more.
package flags

import (
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/urfave/cli/v2"
)

var (
	// MevRelayEndpoint provides an HTTP access endpoint to a MEV builder network.
	MevRelayEndpoint = &cli.StringFlag{
		Name:  "http-mev-relay",
		Usage: "A MEV builder relay string http endpoint, this will be used to interact MEV builder network using API defined in: https://ethereum.github.io/builder-specs/#/Builder",
		Value: "",
	}
	MaxBuilderConsecutiveMissedSlots = &cli.IntFlag{
		Name:  "max-builder-consecutive-missed-slots",
		Usage: "Number of consecutive skip slot to fallback from using relay/builder to local execution engine for block construction",
		Value: 3,
	}
	MaxBuilderEpochMissedSlots = &cli.IntFlag{
		Name:  "max-builder-epoch-missed-slots",
		Usage: "Number of total skip slot to fallback from using relay/builder to local execution engine for block construction in last epoch rolling window",
	}
	// LocalBlockValueBoost sets a percentage boost for local block construction while using a custom builder.
	LocalBlockValueBoost = &cli.Uint64Flag{
		Name: "local-block-value-boost",
		Usage: "A percentage boost for local block construction as a Uint64. This is used to prioritize local block construction over relay/builder block construction" +
			"Boost is an additional percentage to multiple local block value. Use builder block if: builder_bid_value * 100 > local_block_value * (local-block-value-boost + 100)",
	}
	// ExecutionEngineEndpoint provides an HTTP access endpoint to connect to an execution client on the execution layer
	ExecutionEngineEndpoint = &cli.StringFlag{
		Name:  "execution-endpoint",
		Usage: "An execution client http endpoint. Can contain auth header as well in the format",
		Value: "http://localhost:8551",
	}
	// ExecutionEngineHeaders defines a list of HTTP headers to send with all execution client requests.
	ExecutionEngineHeaders = &cli.StringFlag{
		Name: "execution-headers",
		Usage: "A comma separated list of key value pairs to pass as HTTP headers for all execution " +
			"client calls. Example: --execution-headers=key1=value1,key2=value2",
	}
	// ExecutionJWTSecretFlag provides a path to a file containing a hex-encoded string representing a 32 byte secret
	// used to authenticate with an execution node via HTTP. This is required if using an HTTP connection, otherwise all requests
	// to execution nodes for consensus-related calls will fail. This is not required if using an IPC connection.
	ExecutionJWTSecretFlag = &cli.StringFlag{
		Name: "jwt-secret",
		Usage: "REQUIRED if connecting to an execution node via HTTP. Provides a path to a file containing " +
			"a hex-encoded string representing a 32 byte secret used for authentication with an execution node via " +
			"HTTP. If this is not set, all requests to execution nodes via HTTP for consensus-related calls will " +
			"fail, which will prevent your validators from performing their duties. " +
			"This is not required if using an IPC connection.",
		Value: "",
	}
	// JwtId is the id field of the JWT claims. The consensus layer client MAY use this to communicate a unique identifier for the individual consensus layer client
	JwtId = &cli.StringFlag{
		Name:  "jwt-id",
		Usage: "JWT claims id. Could be used to identify the client",
	}
	// DepositContractFlag defines a flag for the deposit contract address.
	DepositContractFlag = &cli.StringFlag{
		Name:  "deposit-contract",
		Usage: "Deposit contract address. Beacon chain node will listen logs coming from the deposit contract to determine when validator is eligible to participate.",
		Value: params.BeaconConfig().DepositContractAddress,
	}
	// RPCHost defines the host on which the RPC server should listen.
	RPCHost = &cli.StringFlag{
		Name:  "rpc-host",
		Usage: "Host on which the RPC server should listen",
		Value: "127.0.0.1",
	}
	// RPCPort defines a beacon node RPC port to open.
	RPCPort = &cli.IntFlag{
		Name:  "rpc-port",
		Usage: "RPC port exposed by a beacon node",
		Value: 4000,
	}
	// MonitoringPortFlag defines the http port used to serve prometheus metrics.
	MonitoringPortFlag = &cli.IntFlag{
		Name:  "monitoring-port",
		Usage: "Port used to listening and respond metrics for Prometheus.",
		Value: 8080,
	}
	// CertFlag defines a flag for the node's TLS certificate.
	CertFlag = &cli.StringFlag{
		Name:  "tls-cert",
		Usage: "Certificate for secure gRPC. Pass this and the tls-key flag in order to use gRPC securely.",
	}
	// KeyFlag defines a flag for the node's TLS key.
	KeyFlag = &cli.StringFlag{
		Name:  "tls-key",
		Usage: "Key for secure gRPC. Pass this and the tls-cert flag in order to use gRPC securely.",
	}
	// HTTPModules define the set of enabled HTTP APIs.
	HTTPModules = &cli.StringFlag{
		Name:  "http-modules",
		Usage: "Comma-separated list of API module names. Possible values: `" + PrysmAPIModule + `,` + EthAPIModule + "`.",
		Value: PrysmAPIModule + `,` + EthAPIModule,
	}
	// DisableGRPCGateway for JSON-HTTP requests to the beacon node.
	DisableGRPCGateway = &cli.BoolFlag{
		Name:  "disable-grpc-gateway",
		Usage: "Disable the gRPC gateway for JSON-HTTP requests",
	}
	// GRPCGatewayHost specifies a gRPC gateway host for Prysm.
	GRPCGatewayHost = &cli.StringFlag{
		Name:  "grpc-gateway-host",
		Usage: "The host on which the gateway server runs on",
		Value: "127.0.0.1",
	}
	// GRPCGatewayPort specifies a gRPC gateway port for Prysm.
	GRPCGatewayPort = &cli.IntFlag{
		Name:  "grpc-gateway-port",
		Usage: "The port on which the gateway server runs on",
		Value: 3500,
	}
	// GPRCGatewayCorsDomain serves preflight requests when serving gRPC JSON gateway.
	GPRCGatewayCorsDomain = &cli.StringFlag{
		Name: "grpc-gateway-corsdomain",
		Usage: "Comma separated list of domains from which to accept cross origin requests " +
			"(browser enforced). This flag has no effect if not used with --grpc-gateway-port.",
		Value: "http://localhost:4200,http://localhost:7500,http://127.0.0.1:4200,http://127.0.0.1:7500,http://0.0.0.0:4200,http://0.0.0.0:7500,http://localhost:3000,http://0.0.0.0:3000,http://127.0.0.1:3000",
	}
	// MinSyncPeers specifies the required number of successful peer handshakes in order
	// to start syncing with external peers.
	MinSyncPeers = &cli.IntFlag{
		Name:  "min-sync-peers",
		Usage: "The required number of valid peers to connect with before syncing.",
		Value: 3,
	}
	// ContractDeploymentBlock is the block in which the eth1 deposit contract was deployed.
	ContractDeploymentBlock = &cli.IntFlag{
		Name:  "contract-deployment-block",
		Usage: "The eth1 block in which the deposit contract was deployed.",
		Value: 11184524,
	}
	// SetGCPercent is the percentage of current live allocations at which the garbage collector is to run.
	SetGCPercent = &cli.IntFlag{
		Name:  "gc-percent",
		Usage: "The percentage of freshly allocated data to live data on which the gc will be run again.",
		Value: 100,
	}
	// SafeSlotsToImportOptimistically is deprecated. It should be removed in the next major release.
	SafeSlotsToImportOptimistically = &cli.IntFlag{
		Name:   "safe-slots-to-import-optimistically",
		Usage:  "DEPRECATED. DO NOT USE",
		Hidden: true,
	}
	// SlotsPerArchivedPoint specifies the number of slots between the archived points, to save beacon state in the cold
	// section of beaconDB.
	SlotsPerArchivedPoint = &cli.IntFlag{
		Name:  "slots-per-archive-point",
		Usage: "The slot durations of when an archived state gets saved in the beaconDB.",
		Value: 2048,
	}
	// BlockBatchLimit specifies the requested block batch size.
	BlockBatchLimit = &cli.IntFlag{
		Name:  "block-batch-limit",
		Usage: "The amount of blocks the local peer is bounded to request and respond to in a batch.",
		Value: 64,
	}
	// BlockBatchLimitBurstFactor specifies the factor by which block batch size may increase.
	BlockBatchLimitBurstFactor = &cli.IntFlag{
		Name:  "block-batch-limit-burst-factor",
		Usage: "The factor by which block batch limit may increase on burst.",
		Value: 2,
	}
	// BlobBatchLimit specifies the requested blob batch size.
	BlobBatchLimit = &cli.IntFlag{
		Name:  "blob-batch-limit",
		Usage: "The amount of blobs the local peer is bounded to request and respond to in a batch.",
		Value: 64,
	}
	// BlobBatchLimitBurstFactor specifies the factor by which blob batch size may increase.
	BlobBatchLimitBurstFactor = &cli.IntFlag{
		Name:  "blob-batch-limit-burst-factor",
		Usage: "The factor by which blob batch limit may increase on burst.",
		Value: 2,
	}
	// EnableDebugRPCEndpoints as /v1/beacon/state.
	EnableDebugRPCEndpoints = &cli.BoolFlag{
		Name:  "enable-debug-rpc-endpoints",
		Usage: "Enables the debug rpc service, containing utility endpoints such as /eth/v1alpha1/beacon/state.",
	}
	// SubscribeToAllSubnets defines a flag to specify whether to subscribe to all possible attestation/sync subnets or not.
	SubscribeToAllSubnets = &cli.BoolFlag{
		Name:  "subscribe-all-subnets",
		Usage: "Subscribe to all possible attestation and sync subnets.",
	}
	// HistoricalSlasherNode is a set of beacon node flags required for performing historical detection with a slasher.
	HistoricalSlasherNode = &cli.BoolFlag{
		Name:  "historical-slasher-node",
		Usage: "Enables required flags for serving historical data to a slasher client. Results in additional storage usage",
	}
	// ChainID defines a flag to set the chain id. If none is set, it derives this value from NetworkConfig
	ChainID = &cli.Uint64Flag{
		Name:  "chain-id",
		Usage: "Sets the chain id of the beacon chain",
	}
	// NetworkID defines a flag to set the network id. If none is set, it derives this value from NetworkConfig
	NetworkID = &cli.Uint64Flag{
		Name:  "network-id",
		Usage: "Sets the network id of the beacon chain.",
	}
	// EngineEndpointTimeoutSeconds defines the seconds to wait before timing out engine endpoints with execution payload execution semantics (newPayload, forkchoiceUpdated).
	// If this flag is not used then default will be used as defined here:
	// https://github.com/ethereum/execution-apis/blob/main/src/engine/specification.md#core
	EngineEndpointTimeoutSeconds = &cli.Uint64Flag{
		Name:  "engine-endpoint-timeout-seconds",
		Usage: "Sets the execution engine timeout (seconds) for execution payload semantics (forkchoiceUpdated, newPayload)",
	}
	// Eth1HeaderReqLimit defines a flag to set the maximum number of headers that a deposit log query can fetch. If none is set, 1000 will be the limit.
	Eth1HeaderReqLimit = &cli.Uint64Flag{
		Name:  "eth1-header-req-limit",
		Usage: "Sets the maximum number of headers that a deposit log query can fetch.",
		Value: uint64(1000),
	}

	// WeakSubjectivityCheckpoint defines the weak subjectivity checkpoint the node must sync through to defend against long range attacks.
	WeakSubjectivityCheckpoint = &cli.StringFlag{
		Name: "weak-subjectivity-checkpoint",
		Usage: "Input in `block_root:epoch_number` format." +
			" This guarantees that syncing leads to the given Weak Subjectivity Checkpoint along the canonical chain. " +
			"If such a sync is not possible, the node will treat it as a critical and irrecoverable failure",
		Value: "",
	}
	// MinPeersPerSubnet defines a flag to set the minimum number of peers that a node will attempt to peer with for a subnet.
	MinPeersPerSubnet = &cli.Uint64Flag{
		Name:  "minimum-peers-per-subnet",
		Usage: "Sets the minimum number of peers that a node will attempt to peer with that are subscribed to a subnet.",
		Value: 6,
	}
	// SuggestedFeeRecipient specifies the fee recipient for the transaction fees.
	SuggestedFeeRecipient = &cli.StringFlag{
		Name:  "suggested-fee-recipient",
		Usage: "Post bellatrix, this address will receive the transaction fees produced by any blocks from this node. Default to junk whilst bellatrix is in development state. Validator client can override this value through the preparebeaconproposer api.",
		Value: params.BeaconConfig().EthBurnAddressHex,
	}
	// TerminalTotalDifficultyOverride specifies the total difficulty to manual overrides the `TERMINAL_TOTAL_DIFFICULTY` parameter.
	TerminalTotalDifficultyOverride = &cli.StringFlag{
		Name: "terminal-total-difficulty-override",
		Usage: "Sets the total difficulty to manual overrides the default TERMINAL_TOTAL_DIFFICULTY value. " +
			"WARNING: This flag should be used only if you have a clear understanding that community has decided to override the terminal difficulty. " +
			"Incorrect usage will result in your node experience consensus failure.",
	}
	// TerminalBlockHashOverride specifies the terminal block hash to manual overrides the `TERMINAL_BLOCK_HASH` parameter.
	TerminalBlockHashOverride = &cli.StringFlag{
		Name: "terminal-block-hash-override",
		Usage: "Sets the block hash to manual overrides the default TERMINAL_BLOCK_HASH value. " +
			"WARNING: This flag should be used only if you have a clear understanding that community has decided to override the terminal block hash. " +
			"Incorrect usage will result in your node experience consensus failure.",
	}
	// TerminalBlockHashActivationEpochOverride specifies the terminal block hash epoch to manual overrides the `TERMINAL_BLOCK_HASH_ACTIVATION_EPOCH` parameter.
	TerminalBlockHashActivationEpochOverride = &cli.Uint64Flag{
		Name: "terminal-block-hash-epoch-override",
		Usage: "Sets the block hash epoch to manual overrides the default TERMINAL_BLOCK_HASH_ACTIVATION_EPOCH value. " +
			"WARNING: This flag should be used only if you have a clear understanding that community has decided to override the terminal block hash activation epoch. " +
			"Incorrect usage will result in your node experience consensus failure.",
	}
	// SlasherDirFlag defines a path on disk where the slasher database is stored.
	SlasherDirFlag = &cli.StringFlag{
		Name:  "slasher-datadir",
		Usage: "Directory for the slasher database",
		Value: cmd.DefaultDataDir(),
	}
)
