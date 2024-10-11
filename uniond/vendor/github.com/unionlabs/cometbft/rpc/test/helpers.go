package rpctest

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	abci "github.com/cometbft/cometbft/abci/types"
	cfg "github.com/cometbft/cometbft/config"
	cmtnet "github.com/cometbft/cometbft/internal/net"
	"github.com/cometbft/cometbft/internal/test"
	"github.com/cometbft/cometbft/libs/log"
	nm "github.com/cometbft/cometbft/node"
	"github.com/cometbft/cometbft/p2p"
	"github.com/cometbft/cometbft/privval"
	"github.com/cometbft/cometbft/proxy"
	ctypes "github.com/cometbft/cometbft/rpc/core/types"
	rpcclient "github.com/cometbft/cometbft/rpc/jsonrpc/client"
)

// Options helps with specifying some parameters for our RPC testing for greater
// control.
type Options struct {
	suppressStdout  bool
	recreateConfig  bool
	maxReqBatchSize int
}

var (
	globalConfig   *cfg.Config
	defaultOptions = Options{
		suppressStdout: false,
		recreateConfig: false,
	}
)

func waitForRPC() {
	laddr := GetConfig().RPC.ListenAddress
	client, err := rpcclient.New(laddr)
	if err != nil {
		panic(err)
	}
	result := new(ctypes.ResultStatus)
	for {
		_, err := client.Call(context.Background(), "status", map[string]any{}, result)
		if err == nil {
			return
		}

		fmt.Println("error", err)
		time.Sleep(time.Millisecond)
	}
}

// f**ing long, but unique for each test.
func makePathname() string {
	// get path
	p, err := os.Getwd()
	if err != nil {
		panic(err)
	}
	// fmt.Println(p)
	sep := string(filepath.Separator)
	return strings.ReplaceAll(p, sep, "_")
}

func randPort() int {
	port, err := cmtnet.GetFreePort()
	if err != nil {
		panic(err)
	}
	return port
}

func makeAddr() string {
	return fmt.Sprintf("tcp://127.0.0.1:%d", randPort())
}

func createConfig() *cfg.Config {
	pathname := makePathname()
	c := test.ResetTestRoot(pathname)

	// and we use random ports to run in parallel
	c.P2P.ListenAddress = makeAddr()
	c.RPC.ListenAddress = makeAddr()
	c.RPC.CORSAllowedOrigins = []string{"https://cometbft.com/"}
	c.GRPC.ListenAddress = makeAddr()
	c.GRPC.VersionService.Enabled = true
	c.GRPC.Privileged.ListenAddress = makeAddr()
	c.GRPC.Privileged.PruningService.Enabled = true
	// Set pruning interval to a value lower than the default for some of the
	// tests that rely on pruning to occur quickly
	c.Storage.Pruning.Interval = 100 * time.Millisecond
	return c
}

// GetConfig returns a config for the test cases as a singleton.
func GetConfig(forceCreate ...bool) *cfg.Config {
	if globalConfig == nil || (len(forceCreate) > 0 && forceCreate[0]) {
		globalConfig = createConfig()
	}
	return globalConfig
}

// StartCometBFT starts a test CometBFT server in a go routine and returns when it is initialized.
func StartCometBFT(app abci.Application, opts ...func(*Options)) *nm.Node {
	nodeOpts := defaultOptions
	for _, opt := range opts {
		opt(&nodeOpts)
	}
	node := NewCometBFT(app, &nodeOpts)
	err := node.Start()
	if err != nil {
		panic(err)
	}

	// wait for rpc
	waitForRPC()

	if !nodeOpts.suppressStdout {
		fmt.Println("CometBFT running!")
	}

	return node
}

// StopCometBFT stops a test CometBFT server, waits until it's stopped and
// cleans up test/config files.
func StopCometBFT(node *nm.Node) {
	if err := node.Stop(); err != nil {
		node.Logger.Error("Error when trying to stop node", "err", err)
	}
	node.Wait()
	os.RemoveAll(node.Config().RootDir)
}

// NewCometBFT creates a new CometBFT server and sleeps forever.
func NewCometBFT(app abci.Application, opts *Options) *nm.Node {
	// Create & start node
	config := GetConfig(opts.recreateConfig)
	var logger log.Logger
	if opts.suppressStdout {
		logger = log.NewNopLogger()
	} else {
		logger = log.NewTMLogger(log.NewSyncWriter(os.Stdout))
		logger = log.NewFilter(logger, log.AllowError())
	}
	if opts.maxReqBatchSize > 0 {
		config.RPC.MaxRequestBatchSize = opts.maxReqBatchSize
	}
	pvKeyFile := config.PrivValidatorKeyFile()
	pvKeyStateFile := config.PrivValidatorStateFile()
	pv, err := privval.LoadOrGenFilePV(pvKeyFile, pvKeyStateFile, nil)
	if err != nil {
		panic(err)
	}
	papp := proxy.NewLocalClientCreator(app)
	nodeKey, err := p2p.LoadOrGenNodeKey(config.NodeKeyFile())
	if err != nil {
		panic(err)
	}
	node, err := nm.NewNode(context.Background(), config, pv, nodeKey, papp,
		nm.DefaultGenesisDocProviderFunc(config),
		cfg.DefaultDBProvider,
		nm.DefaultMetricsProvider(config.Instrumentation),
		logger)
	if err != nil {
		panic(err)
	}
	return node
}

// SuppressStdout is an option that tries to make sure the RPC test CometBFT
// node doesn't log anything to stdout.
func SuppressStdout(o *Options) {
	o.suppressStdout = true
}

// RecreateConfig instructs the RPC test to recreate the configuration each
// time, instead of treating it as a global singleton.
func RecreateConfig(o *Options) {
	o.recreateConfig = true
}

// MaxReqBatchSize is an option to limit the maximum number of requests per batch.
func MaxReqBatchSize(o *Options) {
	o.maxReqBatchSize = 2
}
