package main

import (
	"context"
	"errors"
	"fmt"
	"net/http"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"github.com/spf13/viper"

	"github.com/cometbft/cometbft/abci/server"
	"github.com/cometbft/cometbft/config"
	"github.com/cometbft/cometbft/crypto/ed25519"
	cmtnet "github.com/cometbft/cometbft/internal/net"
	cmtflags "github.com/cometbft/cometbft/libs/cli/flags"
	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/light"
	lproxy "github.com/cometbft/cometbft/light/proxy"
	lrpc "github.com/cometbft/cometbft/light/rpc"
	dbs "github.com/cometbft/cometbft/light/store/db"
	"github.com/cometbft/cometbft/node"
	"github.com/cometbft/cometbft/p2p"
	"github.com/cometbft/cometbft/privval"
	"github.com/cometbft/cometbft/proxy"
	rpcserver "github.com/cometbft/cometbft/rpc/jsonrpc/server"
	"github.com/cometbft/cometbft/test/e2e/app"
	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
)

var logger = log.NewTMLogger(log.NewSyncWriter(os.Stdout))

// main is the binary entrypoint.
func main() {
	if len(os.Args) != 2 {
		fmt.Printf("Usage: %v <configfile>", os.Args[0])
		return
	}
	configFile := ""
	if len(os.Args) == 2 {
		configFile = os.Args[1]
	}

	if err := run(configFile); err != nil {
		logger.Error(err.Error())
		os.Exit(1)
	}
}

// run runs the application - basically like main() with error handling.
func run(configFile string) error {
	cfg, err := LoadConfig(configFile)
	if err != nil {
		return err
	}

	// Start remote signer (must start before node if running builtin).
	if cfg.PrivValServer != "" {
		if err = startSigner(cfg); err != nil {
			return err
		}
		if cfg.Protocol == "builtin" || cfg.Protocol == "builtin_connsync" {
			time.Sleep(1 * time.Second)
		}
	}

	// Start app server.
	switch cfg.Protocol {
	case "socket", "grpc":
		err = startApp(cfg)
	case "builtin", "builtin_connsync":
		if cfg.Mode == string(e2e.ModeLight) {
			err = startLightClient(cfg)
		} else {
			err = startNode(cfg)
		}
	default:
		err = fmt.Errorf("invalid protocol %q", cfg.Protocol)
	}
	if err != nil {
		return err
	}

	// Apparently there's no way to wait for the server, so we just sleep
	for {
		time.Sleep(1 * time.Hour)
	}
}

// startApp starts the application server, listening for connections from CometBFT.
func startApp(cfg *Config) error {
	app, err := app.NewApplication(cfg.App())
	if err != nil {
		return err
	}
	server, err := server.NewServer(cfg.Listen, cfg.Protocol, app)
	if err != nil {
		return err
	}
	err = server.Start()
	if err != nil {
		return err
	}
	logger.Info("start app", "msg", log.NewLazySprintf("Server listening on %v (%v protocol)", cfg.Listen, cfg.Protocol))
	return nil
}

// startNode starts a CometBFT node running the application directly. It assumes the CometBFT
// configuration is in $CMTHOME/config/cometbft.toml.
//
// FIXME There is no way to simply load the configuration from a file, so we need to pull in Viper.
func startNode(cfg *Config) error {
	app, err := app.NewApplication(cfg.App())
	if err != nil {
		return err
	}

	cmtcfg, nodeLogger, nodeKey, err := setupNode()
	if err != nil {
		return fmt.Errorf("failed to setup config: %w", err)
	}

	var clientCreator proxy.ClientCreator
	if cfg.Protocol == string(e2e.ProtocolBuiltinConnSync) {
		clientCreator = proxy.NewConnSyncLocalClientCreator(app)
		nodeLogger.Info("Using connection-synchronized local client creator")
	} else {
		clientCreator = proxy.NewLocalClientCreator(app)
		nodeLogger.Info("Using default (synchronized) local client creator")
	}

	if cfg.ExperimentalKeyLayout != "" {
		cmtcfg.Storage.ExperimentalKeyLayout = cfg.ExperimentalKeyLayout
	}

	// We hardcode ed25519 here because the priv validator files have already been set up in the setup step
	pv, err := privval.LoadOrGenFilePV(cmtcfg.PrivValidatorKeyFile(), cmtcfg.PrivValidatorStateFile(), nil)
	if err != nil {
		return err
	}
	n, err := node.NewNode(context.Background(), cmtcfg,
		pv,
		nodeKey,
		clientCreator,
		node.DefaultGenesisDocProviderFunc(cmtcfg),
		config.DefaultDBProvider,
		node.DefaultMetricsProvider(cmtcfg.Instrumentation),
		nodeLogger,
	)
	if err != nil {
		return err
	}
	return n.Start()
}

func startLightClient(cfg *Config) error {
	cmtcfg, nodeLogger, _, err := setupNode()
	if err != nil {
		return err
	}

	dbContext := &config.DBContext{ID: "light", Config: cmtcfg}
	lightDB, err := config.DefaultDBProvider(dbContext)
	if err != nil {
		return err
	}

	providers := rpcEndpoints(cmtcfg.P2P.PersistentPeers)

	c, err := light.NewHTTPClient(
		context.Background(),
		cfg.ChainID,
		light.TrustOptions{
			Period: cmtcfg.StateSync.TrustPeriod,
			Height: cmtcfg.StateSync.TrustHeight,
			Hash:   cmtcfg.StateSync.TrustHashBytes(),
		},
		providers[0],
		providers[1:],
		dbs.NewWithDBVersion(lightDB, "light", cfg.ExperimentalKeyLayout),
		light.Logger(nodeLogger),
	)
	if err != nil {
		return err
	}

	rpccfg := rpcserver.DefaultConfig()
	rpccfg.MaxBodyBytes = cmtcfg.RPC.MaxBodyBytes
	rpccfg.MaxHeaderBytes = cmtcfg.RPC.MaxHeaderBytes
	rpccfg.MaxOpenConnections = cmtcfg.RPC.MaxOpenConnections
	// If necessary adjust global WriteTimeout to ensure it's greater than
	// TimeoutBroadcastTxCommit.
	// See https://github.com/tendermint/tendermint/issues/3435
	if rpccfg.WriteTimeout <= cmtcfg.RPC.TimeoutBroadcastTxCommit {
		rpccfg.WriteTimeout = cmtcfg.RPC.TimeoutBroadcastTxCommit + 1*time.Second
	}

	p, err := lproxy.NewProxy(c, cmtcfg.RPC.ListenAddress, providers[0], rpccfg, nodeLogger,
		lrpc.KeyPathFn(lrpc.DefaultMerkleKeyPathFn()))
	if err != nil {
		return err
	}

	logger.Info("Starting proxy...", "laddr", cmtcfg.RPC.ListenAddress)
	if err := p.ListenAndServe(); err != http.ErrServerClosed {
		// Error starting or closing listener:
		logger.Error("proxy ListenAndServe", "err", err)
	}

	return nil
}

// startSigner starts a signer server connecting to the given endpoint.
func startSigner(cfg *Config) error {
	filePV := privval.LoadFilePV(cfg.PrivValKey, cfg.PrivValState)

	protocol, address := cmtnet.ProtocolAndAddress(cfg.PrivValServer)
	var dialFn privval.SocketDialer
	switch protocol {
	case "tcp":
		dialFn = privval.DialTCPFn(address, 3*time.Second, ed25519.GenPrivKey())
	case "unix":
		dialFn = privval.DialUnixFn(address)
	default:
		return fmt.Errorf("invalid privval protocol %q", protocol)
	}

	endpoint := privval.NewSignerDialerEndpoint(logger, dialFn,
		privval.SignerDialerEndpointRetryWaitInterval(1*time.Second),
		privval.SignerDialerEndpointConnRetries(100))
	err := privval.NewSignerServer(endpoint, cfg.ChainID, filePV).Start()
	if err != nil {
		return err
	}
	logger.Info("start signer", "msg", log.NewLazySprintf("Remote signer connecting to %v", cfg.PrivValServer))
	return nil
}

func setupNode() (*config.Config, log.Logger, *p2p.NodeKey, error) {
	var cmtcfg *config.Config

	home := os.Getenv("CMTHOME")
	if home == "" {
		return nil, nil, nil, errors.New("CMTHOME not set")
	}

	viper.AddConfigPath(filepath.Join(home, "config"))
	viper.SetConfigName("config")

	if err := viper.ReadInConfig(); err != nil {
		return nil, nil, nil, err
	}

	cmtcfg = config.DefaultConfig()

	if err := viper.Unmarshal(cmtcfg); err != nil {
		return nil, nil, nil, err
	}

	cmtcfg.SetRoot(home)

	if err := cmtcfg.ValidateBasic(); err != nil {
		return nil, nil, nil, fmt.Errorf("error in config file: %w", err)
	}

	if cmtcfg.LogFormat == config.LogFormatJSON {
		logger = log.NewTMJSONLogger(log.NewSyncWriter(os.Stdout))
	}

	nodeLogger, err := cmtflags.ParseLogLevel(cmtcfg.LogLevel, logger, config.DefaultLogLevel)
	if err != nil {
		return nil, nil, nil, err
	}

	nodeLogger = nodeLogger.With("module", "main")

	nodeKey, err := p2p.LoadOrGenNodeKey(cmtcfg.NodeKeyFile())
	if err != nil {
		return nil, nil, nil, fmt.Errorf("failed to load or gen node key %s: %w", cmtcfg.NodeKeyFile(), err)
	}

	return cmtcfg, nodeLogger, nodeKey, nil
}

// rpcEndpoints takes a list of persistent peers and splits them into a list of rpc endpoints
// using 26657 as the port number.
func rpcEndpoints(peers string) []string {
	arr := strings.Split(peers, ",")
	endpoints := make([]string, len(arr))
	for i, v := range arr {
		urlString := strings.SplitAfter(v, "@")[1]
		hostName := strings.Split(urlString, ":26656")[0]
		// use RPC port instead
		port := 26657
		rpcEndpoint := "http://" + hostName + ":" + strconv.Itoa(port)
		endpoints[i] = rpcEndpoint
	}
	return endpoints
}
