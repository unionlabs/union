package server

// DONTCOVER

import (
	"errors"
	"fmt"
	"net"
	"net/http"
	"os"
	"runtime/pprof"
	"time"

	"github.com/cometbft/cometbft/abci/server"
	tcmd "github.com/cometbft/cometbft/cmd/cometbft/commands"
	"github.com/cometbft/cometbft/node"
	"github.com/cometbft/cometbft/p2p"
	pvm "github.com/cometbft/cometbft/privval"
	"github.com/cometbft/cometbft/proxy"
	"github.com/cometbft/cometbft/rpc/client/local"
	"github.com/spf13/cobra"
	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"

	"cosmossdk.io/tools/rosetta"
	crgserver "cosmossdk.io/tools/rosetta/lib/server"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/server/api"
	serverconfig "github.com/cosmos/cosmos-sdk/server/config"
	servergrpc "github.com/cosmos/cosmos-sdk/server/grpc"
	"github.com/cosmos/cosmos-sdk/server/types"
	pruningtypes "github.com/cosmos/cosmos-sdk/store/pruning/types"
	"github.com/cosmos/cosmos-sdk/telemetry"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/mempool"
)

const (
	// Tendermint full-node start flags
	flagWithTendermint     = "with-tendermint"
	flagAddress            = "address"
	flagTransport          = "transport"
	flagTraceStore         = "trace-store"
	flagCPUProfile         = "cpu-profile"
	FlagMinGasPrices       = "minimum-gas-prices"
	FlagHaltHeight         = "halt-height"
	FlagHaltTime           = "halt-time"
	FlagInterBlockCache    = "inter-block-cache"
	FlagUnsafeSkipUpgrades = "unsafe-skip-upgrades"
	FlagTrace              = "trace"
	FlagInvCheckPeriod     = "inv-check-period"

	FlagPruning             = "pruning"
	FlagPruningKeepRecent   = "pruning-keep-recent"
	FlagPruningInterval     = "pruning-interval"
	FlagIndexEvents         = "index-events"
	FlagMinRetainBlocks     = "min-retain-blocks"
	FlagIAVLCacheSize       = "iavl-cache-size"
	FlagDisableIAVLFastNode = "iavl-disable-fastnode"
	FlagIAVLLazyLoading     = "iavl-lazy-loading"

	// state sync-related flags
	FlagStateSyncSnapshotInterval   = "state-sync.snapshot-interval"
	FlagStateSyncSnapshotKeepRecent = "state-sync.snapshot-keep-recent"

	// api-related flags
	FlagAPIEnable             = "api.enable"
	FlagAPISwagger            = "api.swagger"
	FlagAPIAddress            = "api.address"
	FlagAPIMaxOpenConnections = "api.max-open-connections"
	FlagRPCReadTimeout        = "api.rpc-read-timeout"
	FlagRPCWriteTimeout       = "api.rpc-write-timeout"
	FlagRPCMaxBodyBytes       = "api.rpc-max-body-bytes"
	FlagAPIEnableUnsafeCORS   = "api.enabled-unsafe-cors"

	// gRPC-related flags
	flagGRPCOnly       = "grpc-only"
	flagGRPCEnable     = "grpc.enable"
	flagGRPCAddress    = "grpc.address"
	flagGRPCWebEnable  = "grpc-web.enable"
	flagGRPCWebAddress = "grpc-web.address"

	// mempool flags
	FlagMempoolMaxTxs = "mempool.max-txs"
)

// StartCmd runs the service passed in, either stand-alone or in-process with
// Tendermint.
func StartCmd(appCreator types.AppCreator, defaultNodeHome string) *cobra.Command {
	cmd := &cobra.Command{
		Use:   "start",
		Short: "Run the full node",
		Long: `Run the full node application with Tendermint in or out of process. By
default, the application will run with Tendermint in process.

Pruning options can be provided via the '--pruning' flag or alternatively with '--pruning-keep-recent', and
'pruning-interval' together.

For '--pruning' the options are as follows:

default: the last 362880 states are kept, pruning at 10 block intervals
nothing: all historic states will be saved, nothing will be deleted (i.e. archiving node)
everything: 2 latest states will be kept; pruning at 10 block intervals.
custom: allow pruning options to be manually specified through 'pruning-keep-recent', and 'pruning-interval'

Node halting configurations exist in the form of two flags: '--halt-height' and '--halt-time'. During
the ABCI Commit phase, the node will check if the current block height is greater than or equal to
the halt-height or if the current block time is greater than or equal to the halt-time. If so, the
node will attempt to gracefully shutdown and the block will not be committed. In addition, the node
will not be able to commit subsequent blocks.

For profiling and benchmarking purposes, CPU profiling can be enabled via the '--cpu-profile' flag
which accepts a path for the resulting pprof file.

The node may be started in a 'query only' mode where only the gRPC and JSON HTTP
API services are enabled via the 'grpc-only' flag. In this mode, Tendermint is
bypassed and can be used when legacy queries are needed after an on-chain upgrade
is performed. Note, when enabled, gRPC will also be automatically enabled.
`,
		PreRunE: func(cmd *cobra.Command, _ []string) error {
			serverCtx := GetServerContextFromCmd(cmd)

			// Bind flags to the Context's Viper so the app construction can set
			// options accordingly.
			if err := serverCtx.Viper.BindPFlags(cmd.Flags()); err != nil {
				return err
			}

			_, err := GetPruningOptionsFromFlags(serverCtx.Viper)
			return err
		},
		RunE: func(cmd *cobra.Command, _ []string) error {
			serverCtx := GetServerContextFromCmd(cmd)
			clientCtx, err := client.GetClientQueryContext(cmd)
			if err != nil {
				return err
			}

			withTM, _ := cmd.Flags().GetBool(flagWithTendermint)
			if !withTM {
				serverCtx.Logger.Info("starting ABCI without Tendermint")
				return wrapCPUProfile(serverCtx, func() error {
					return startStandAlone(serverCtx, appCreator)
				})
			}

			// amino is needed here for backwards compatibility of REST routes
			err = wrapCPUProfile(serverCtx, func() error {
				return startInProcess(serverCtx, clientCtx, appCreator)
			})
			errCode, ok := err.(ErrorCode)
			if !ok {
				return err
			}

			serverCtx.Logger.Debug(fmt.Sprintf("received quit signal: %d", errCode.Code))
			return nil
		},
	}

	cmd.Flags().String(flags.FlagHome, defaultNodeHome, "The application home directory")
	cmd.Flags().Bool(flagWithTendermint, true, "Run abci app embedded in-process with tendermint")
	cmd.Flags().String(flagAddress, "tcp://0.0.0.0:26658", "Listen address")
	cmd.Flags().String(flagTransport, "socket", "Transport protocol: socket, grpc")
	cmd.Flags().String(flagTraceStore, "", "Enable KVStore tracing to an output file")
	cmd.Flags().String(FlagMinGasPrices, "", "Minimum gas prices to accept for transactions; Any fee in a tx must meet this minimum (e.g. 0.01photino;0.0001stake)")
	cmd.Flags().IntSlice(FlagUnsafeSkipUpgrades, []int{}, "Skip a set of upgrade heights to continue the old binary")
	cmd.Flags().Uint64(FlagHaltHeight, 0, "Block height at which to gracefully halt the chain and shutdown the node")
	cmd.Flags().Uint64(FlagHaltTime, 0, "Minimum block time (in Unix seconds) at which to gracefully halt the chain and shutdown the node")
	cmd.Flags().Bool(FlagInterBlockCache, true, "Enable inter-block caching")
	cmd.Flags().String(flagCPUProfile, "", "Enable CPU profiling and write to the provided file")
	cmd.Flags().Bool(FlagTrace, false, "Provide full stack traces for errors in ABCI Log")
	cmd.Flags().String(FlagPruning, pruningtypes.PruningOptionDefault, "Pruning strategy (default|nothing|everything|custom)")
	cmd.Flags().Uint64(FlagPruningKeepRecent, 0, "Number of recent heights to keep on disk (ignored if pruning is not 'custom')")
	cmd.Flags().Uint64(FlagPruningInterval, 0, "Height interval at which pruned heights are removed from disk (ignored if pruning is not 'custom')")
	cmd.Flags().Uint(FlagInvCheckPeriod, 0, "Assert registered invariants every N blocks")
	cmd.Flags().Uint64(FlagMinRetainBlocks, 0, "Minimum block height offset during ABCI commit to prune Tendermint blocks")

	cmd.Flags().Bool(FlagAPIEnable, false, "Define if the API server should be enabled")
	cmd.Flags().Bool(FlagAPISwagger, false, "Define if swagger documentation should automatically be registered (Note: the API must also be enabled)")
	cmd.Flags().String(FlagAPIAddress, serverconfig.DefaultAPIAddress, "the API server address to listen on")
	cmd.Flags().Uint(FlagAPIMaxOpenConnections, 1000, "Define the number of maximum open connections")
	cmd.Flags().Uint(FlagRPCReadTimeout, 10, "Define the Tendermint RPC read timeout (in seconds)")
	cmd.Flags().Uint(FlagRPCWriteTimeout, 0, "Define the Tendermint RPC write timeout (in seconds)")
	cmd.Flags().Uint(FlagRPCMaxBodyBytes, 1000000, "Define the Tendermint maximum request body (in bytes)")
	cmd.Flags().Bool(FlagAPIEnableUnsafeCORS, false, "Define if CORS should be enabled (unsafe - use it at your own risk)")

	cmd.Flags().Bool(flagGRPCOnly, false, "Start the node in gRPC query only mode (no Tendermint process is started)")
	cmd.Flags().Bool(flagGRPCEnable, true, "Define if the gRPC server should be enabled")
	cmd.Flags().String(flagGRPCAddress, serverconfig.DefaultGRPCAddress, "the gRPC server address to listen on")

	cmd.Flags().Bool(flagGRPCWebEnable, true, "Define if the gRPC-Web server should be enabled. (Note: gRPC must also be enabled)")
	cmd.Flags().String(flagGRPCWebAddress, serverconfig.DefaultGRPCWebAddress, "The gRPC-Web server address to listen on")

	cmd.Flags().Uint64(FlagStateSyncSnapshotInterval, 0, "State sync snapshot interval")
	cmd.Flags().Uint32(FlagStateSyncSnapshotKeepRecent, 2, "State sync snapshot to keep")

	cmd.Flags().Bool(FlagDisableIAVLFastNode, false, "Disable fast node for IAVL tree")

	cmd.Flags().Int(FlagMempoolMaxTxs, mempool.DefaultMaxTx, "Sets MaxTx value for the app-side mempool")

	// add support for all Tendermint-specific command line options
	tcmd.AddNodeFlags(cmd)
	return cmd
}

func startStandAlone(ctx *Context, appCreator types.AppCreator) error {
	addr := ctx.Viper.GetString(flagAddress)
	transport := ctx.Viper.GetString(flagTransport)
	home := ctx.Viper.GetString(flags.FlagHome)

	db, err := openDB(home, GetAppDBBackend(ctx.Viper))
	if err != nil {
		return err
	}

	traceWriterFile := ctx.Viper.GetString(flagTraceStore)
	traceWriter, err := openTraceWriter(traceWriterFile)
	if err != nil {
		return err
	}

	app := appCreator(ctx.Logger, db, traceWriter, ctx.Viper)

	config, err := serverconfig.GetConfig(ctx.Viper)
	if err != nil {
		return err
	}

	_, err = startTelemetry(config)
	if err != nil {
		return err
	}

	svr, err := server.NewServer(addr, transport, app)
	if err != nil {
		return fmt.Errorf("error creating listener: %v", err)
	}

	svr.SetLogger(ctx.Logger.With("module", "abci-server"))

	err = svr.Start()
	if err != nil {
		fmt.Println(err.Error())
		os.Exit(1)
	}

	defer func() {
		if err = svr.Stop(); err != nil {
			fmt.Println(err.Error())
			os.Exit(1)
		}

		if err = app.Close(); err != nil {
			fmt.Println(err.Error())
			os.Exit(1)
		}
	}()

	// Wait for SIGINT or SIGTERM signal
	return WaitForQuitSignals()
}

func startInProcess(ctx *Context, clientCtx client.Context, appCreator types.AppCreator) error {
	cfg := ctx.Config
	home := cfg.RootDir

	db, err := openDB(home, GetAppDBBackend(ctx.Viper))
	if err != nil {
		return err
	}

	traceWriterFile := ctx.Viper.GetString(flagTraceStore)
	traceWriter, err := openTraceWriter(traceWriterFile)
	if err != nil {
		return err
	}

	// Clean up the traceWriter when the server is shutting down.
	var traceWriterCleanup func()
	// if flagTraceStore is not used then traceWriter is nil
	if traceWriter != nil {
		traceWriterCleanup = func() {
			if err = traceWriter.Close(); err != nil {
				ctx.Logger.Error("failed to close trace writer", "err", err)
			}
		}
	}

	config, err := serverconfig.GetConfig(ctx.Viper)
	if err != nil {
		return err
	}

	if err := config.ValidateBasic(); err != nil {
		return err
	}

	app := appCreator(ctx.Logger, db, traceWriter, ctx.Viper)

	nodeKey, err := p2p.LoadOrGenNodeKey(cfg.NodeKeyFile())
	if err != nil {
		return err
	}
	genDocProvider := node.DefaultGenesisDocProviderFunc(cfg)

	var (
		tmNode   *node.Node
		gRPCOnly = ctx.Viper.GetBool(flagGRPCOnly)
	)

	if gRPCOnly {
		ctx.Logger.Info("starting node in gRPC only mode; Tendermint is disabled")
		config.GRPC.Enable = true
	} else {
		ctx.Logger.Info("starting node with ABCI Tendermint in-process")

		tmNode, err = node.NewNode(
			cfg,
			pvm.LoadOrGenFilePV(cfg.PrivValidatorKeyFile(), cfg.PrivValidatorStateFile()),
			nodeKey,
			proxy.NewLocalClientCreator(app),
			genDocProvider,
			node.DefaultDBProvider,
			node.DefaultMetricsProvider(cfg.Instrumentation),
			ctx.Logger,
		)
		if err != nil {
			return err
		}

		if err := tmNode.Start(); err != nil {
			return err
		}
	}

	// Add the tx service to the gRPC router. We only need to register this
	// service if API or gRPC is enabled, and avoid doing so in the general
	// case, because it spawns a new local tendermint RPC client.
	if (config.API.Enable || config.GRPC.Enable) && tmNode != nil {
		// re-assign for making the client available below
		// do not use := to avoid shadowing clientCtx
		clientCtx = clientCtx.WithClient(local.New(tmNode))

		app.RegisterTxService(clientCtx)
		app.RegisterTendermintService(clientCtx)
		app.RegisterNodeService(clientCtx)
	}

	metrics, err := startTelemetry(config)
	if err != nil {
		return err
	}

	var apiSrv *api.Server
	if config.API.Enable {
		genDoc, err := genDocProvider()
		if err != nil {
			return err
		}

		clientCtx := clientCtx.WithHomeDir(home).WithChainID(genDoc.ChainID)

		if config.GRPC.Enable {
			_, port, err := net.SplitHostPort(config.GRPC.Address)
			if err != nil {
				return err
			}

			maxSendMsgSize := config.GRPC.MaxSendMsgSize
			if maxSendMsgSize == 0 {
				maxSendMsgSize = serverconfig.DefaultGRPCMaxSendMsgSize
			}

			maxRecvMsgSize := config.GRPC.MaxRecvMsgSize
			if maxRecvMsgSize == 0 {
				maxRecvMsgSize = serverconfig.DefaultGRPCMaxRecvMsgSize
			}

			grpcAddress := fmt.Sprintf("127.0.0.1:%s", port)

			// If grpc is enabled, configure grpc client for grpc gateway.
			grpcClient, err := grpc.Dial(
				grpcAddress,
				grpc.WithTransportCredentials(insecure.NewCredentials()),
				grpc.WithDefaultCallOptions(
					grpc.ForceCodec(codec.NewProtoCodec(clientCtx.InterfaceRegistry).GRPCCodec()),
					grpc.MaxCallRecvMsgSize(maxRecvMsgSize),
					grpc.MaxCallSendMsgSize(maxSendMsgSize),
				),
			)
			if err != nil {
				return err
			}

			clientCtx = clientCtx.WithGRPCClient(grpcClient)
			ctx.Logger.Debug("grpc client assigned to client context", "target", grpcAddress)
		}

		apiSrv = api.New(clientCtx, ctx.Logger.With("module", "api-server"))
		app.RegisterAPIRoutes(apiSrv, config.API)
		if config.Telemetry.Enabled {
			apiSrv.SetTelemetry(metrics)
		}
		errCh := make(chan error)

		go func() {
			if err := apiSrv.Start(config); err != nil {
				errCh <- err
			}
		}()

		select {
		case err := <-errCh:
			return err

		case <-time.After(types.ServerStartTime): // assume server started successfully
		}
	}

	var (
		grpcSrv    *grpc.Server
		grpcWebSrv *http.Server
	)

	if config.GRPC.Enable {
		grpcSrv, err = servergrpc.StartGRPCServer(clientCtx, app, config.GRPC)
		if err != nil {
			return err
		}
		defer grpcSrv.Stop()
		if config.GRPCWeb.Enable {
			grpcWebSrv, err = servergrpc.StartGRPCWeb(grpcSrv, config)
			if err != nil {
				ctx.Logger.Error("failed to start grpc-web http server: ", err)
				return err
			}
			defer func() {
				if err := grpcWebSrv.Close(); err != nil {
					ctx.Logger.Error("failed to close grpc-web http server: ", err)
				}
			}()
		}
	}

	// At this point it is safe to block the process if we're in gRPC only mode as
	// we do not need to start Rosetta or handle any Tendermint related processes.
	if gRPCOnly {
		// wait for signal capture and gracefully return
		return WaitForQuitSignals()
	}

	var rosettaSrv crgserver.Server
	if config.Rosetta.Enable {
		offlineMode := config.Rosetta.Offline

		// If GRPC is not enabled rosetta cannot work in online mode, so we throw an error.
		if !config.GRPC.Enable && !offlineMode {
			return errors.New("'grpc' must be enable in online mode for Rosetta to work")
		}

		minGasPrices, err := sdk.ParseDecCoins(config.MinGasPrices)
		if err != nil {
			ctx.Logger.Error("failed to parse minimum-gas-prices: ", err)
			return err
		}

		conf := &rosetta.Config{
			Blockchain:          config.Rosetta.Blockchain,
			Network:             config.Rosetta.Network,
			TendermintRPC:       ctx.Config.RPC.ListenAddress,
			GRPCEndpoint:        config.GRPC.Address,
			Addr:                config.Rosetta.Address,
			Retries:             config.Rosetta.Retries,
			Offline:             offlineMode,
			GasToSuggest:        config.Rosetta.GasToSuggest,
			EnableFeeSuggestion: config.Rosetta.EnableFeeSuggestion,
			GasPrices:           minGasPrices.Sort(),
			Codec:               clientCtx.Codec.(*codec.ProtoCodec),
			InterfaceRegistry:   clientCtx.InterfaceRegistry,
		}

		rosettaSrv, err = rosetta.ServerFromConfig(conf)
		if err != nil {
			return err
		}

		errCh := make(chan error)
		go func() {
			if err := rosettaSrv.Start(); err != nil {
				errCh <- err
			}
		}()

		select {
		case err := <-errCh:
			return err

		case <-time.After(types.ServerStartTime): // assume server started successfully
		}
	}

	defer func() {
		if tmNode != nil && tmNode.IsRunning() {
			_ = tmNode.Stop()
			_ = app.Close()
		}

		if traceWriterCleanup != nil {
			traceWriterCleanup()
		}

		if apiSrv != nil {
			_ = apiSrv.Close()
		}

		ctx.Logger.Info("exiting...")
	}()

	// wait for signal capture and gracefully return
	return WaitForQuitSignals()
}

func startTelemetry(cfg serverconfig.Config) (*telemetry.Metrics, error) {
	if !cfg.Telemetry.Enabled {
		return nil, nil
	}
	return telemetry.New(cfg.Telemetry)
}

// wrapCPUProfile runs callback in a goroutine, then wait for quit signals.
func wrapCPUProfile(ctx *Context, callback func() error) error {
	if cpuProfile := ctx.Viper.GetString(flagCPUProfile); cpuProfile != "" {
		f, err := os.Create(cpuProfile)
		if err != nil {
			return err
		}

		ctx.Logger.Info("starting CPU profiler", "profile", cpuProfile)
		if err := pprof.StartCPUProfile(f); err != nil {
			return err
		}

		defer func() {
			ctx.Logger.Info("stopping CPU profiler", "profile", cpuProfile)
			pprof.StopCPUProfile()
			if err := f.Close(); err != nil {
				ctx.Logger.Info("failed to close cpu-profile file", "profile", cpuProfile, "err", err.Error())
			}
		}()
	}

	errCh := make(chan error)
	go func() {
		errCh <- callback()
	}()

	select {
	case err := <-errCh:
		return err

	case <-time.After(types.ServerStartTime):
	}

	return WaitForQuitSignals()
}
