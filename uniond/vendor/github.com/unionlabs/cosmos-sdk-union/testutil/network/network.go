package network

import (
	"bufio"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/url"
	"os"
	"os/signal"
	"path/filepath"
	"strings"
	"sync"
	"sync/atomic"
	"syscall"
	"testing"
	"time"

	cmtcfg "github.com/cometbft/cometbft/config"
	"github.com/spf13/cobra"
	"github.com/spf13/viper"

	"cosmossdk.io/core/address"
	"cosmossdk.io/core/registry"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/depinject"
	"cosmossdk.io/log"
	sdkmath "cosmossdk.io/math"
	"cosmossdk.io/math/unsafe"
	pruningtypes "cosmossdk.io/store/pruning/types"
	banktypes "cosmossdk.io/x/bank/types"
	stakingtypes "cosmossdk.io/x/staking/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/client/grpc/cmtservice"
	"github.com/cosmos/cosmos-sdk/client/tx"
	"github.com/cosmos/cosmos-sdk/codec"
	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	srvconfig "github.com/cosmos/cosmos-sdk/server/config"
	servertypes "github.com/cosmos/cosmos-sdk/server/types"
	"github.com/cosmos/cosmos-sdk/testutil"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/genutil"
)

// package-wide network lock to only allow one test network at a time
var (
	lock     = new(sync.Mutex)
	portPool = make(chan string, 200)
)

func init() {
	closeFns := []func() error{}
	for i := 0; i < 200; i++ {
		_, port, closeFn, err := FreeTCPAddr()
		if err != nil {
			panic(err)
		}

		portPool <- port
		closeFns = append(closeFns, closeFn)
	}

	for _, closeFn := range closeFns {
		err := closeFn()
		if err != nil {
			panic(err)
		}
	}
}

// AppConstructor defines a function which accepts a network configuration and
// creates an ABCI Application to provide to CometBFT.
type (
	AppConstructor     = func(val ValidatorI) servertypes.Application
	TestFixtureFactory = func() TestFixture
)

type TestFixture struct {
	AppConstructor AppConstructor
	GenesisState   map[string]json.RawMessage
	EncodingConfig moduletestutil.TestEncodingConfig
}

// Config defines the necessary configuration used to bootstrap and start an
// in-process local testing network.
type Config struct {
	Codec             codec.Codec
	LegacyAmino       *codec.LegacyAmino // TODO: Remove!
	InterfaceRegistry codectypes.InterfaceRegistry

	TxConfig         client.TxConfig
	AccountRetriever client.AccountRetriever
	AppConstructor   AppConstructor             // the ABCI application constructor
	GenesisState     map[string]json.RawMessage // custom genesis state to provide
	GenesisTime      time.Time                  // the genesis time
	TimeoutCommit    time.Duration              // the consensus commitment timeout
	ChainID          string                     // the network chain-id
	NumValidators    int                        // the total number of validators to create and bond
	Mnemonics        []string                   // custom user-provided validator operator mnemonics
	BondDenom        string                     // the staking bond denomination
	MinGasPrices     string                     // the minimum gas prices each validator will accept
	AccountTokens    sdkmath.Int                // the amount of unique validator tokens (e.g. 1000node0)
	StakingTokens    sdkmath.Int                // the amount of tokens each validator has available to stake
	BondedTokens     sdkmath.Int                // the amount of tokens each validator stakes
	PruningStrategy  string                     // the pruning strategy each validator will have
	EnableLogging    bool                       // enable logging to STDOUT
	CleanupDir       bool                       // remove base temporary directory during cleanup
	SigningAlgo      string                     // signing algorithm for keys
	KeyringOptions   []keyring.Option           // keyring configuration options
	RPCAddress       string                     // RPC listen address (including port)
	APIAddress       string                     // REST API listen address (including port)
	GRPCAddress      string                     // GRPC server listen address (including port)
	PrintMnemonic    bool                       // print the mnemonic of first validator as log output for testing

	// Address codecs
	AddressCodec          address.Codec                 // address codec
	ValidatorAddressCodec address.ValidatorAddressCodec // validator address codec
	ConsensusAddressCodec address.ConsensusAddressCodec // consensus address codec
}

// DefaultConfig returns a sane default configuration suitable for nearly all
// testing requirements.
func DefaultConfig(factory TestFixtureFactory) Config {
	fixture := factory()

	return Config{
		Codec:                 fixture.EncodingConfig.Codec,
		TxConfig:              fixture.EncodingConfig.TxConfig,
		LegacyAmino:           fixture.EncodingConfig.Amino,
		InterfaceRegistry:     fixture.EncodingConfig.InterfaceRegistry,
		AccountRetriever:      authtypes.AccountRetriever{},
		AppConstructor:        fixture.AppConstructor,
		GenesisState:          fixture.GenesisState,
		TimeoutCommit:         2 * time.Second,
		ChainID:               "chain-" + unsafe.Str(6),
		NumValidators:         4,
		BondDenom:             sdk.DefaultBondDenom,
		MinGasPrices:          fmt.Sprintf("0.000006%s", sdk.DefaultBondDenom),
		AccountTokens:         sdk.TokensFromConsensusPower(1000, sdk.DefaultPowerReduction),
		StakingTokens:         sdk.TokensFromConsensusPower(500, sdk.DefaultPowerReduction),
		BondedTokens:          sdk.TokensFromConsensusPower(100, sdk.DefaultPowerReduction),
		PruningStrategy:       pruningtypes.PruningOptionNothing,
		CleanupDir:            true,
		SigningAlgo:           string(hd.Secp256k1Type),
		KeyringOptions:        []keyring.Option{},
		PrintMnemonic:         false,
		AddressCodec:          addresscodec.NewBech32Codec("cosmos"),
		ValidatorAddressCodec: addresscodec.NewBech32Codec("cosmosvaloper"),
		ConsensusAddressCodec: addresscodec.NewBech32Codec("cosmosvalcons"),
	}
}

func DefaultConfigWithAppConfig(appConfig depinject.Config, baseappOpts ...func(*baseapp.BaseApp)) (Config, error) {
	var (
		appBuilder            *runtime.AppBuilder
		txConfig              client.TxConfig
		legacyAmino           registry.AminoRegistrar
		cdc                   codec.Codec
		interfaceRegistry     codectypes.InterfaceRegistry
		addressCodec          address.Codec
		validatorAddressCodec address.ValidatorAddressCodec
		consensusAddressCodec address.ConsensusAddressCodec
	)

	if err := depinject.Inject(
		depinject.Configs(
			appConfig,
			depinject.Supply(log.NewNopLogger()),
		),
		&appBuilder,
		&txConfig,
		&cdc,
		&legacyAmino,
		&interfaceRegistry,
		&addressCodec,
		&validatorAddressCodec,
		&consensusAddressCodec,
	); err != nil {
		return Config{}, err
	}

	cfg := DefaultConfig(func() TestFixture {
		return TestFixture{}
	})
	cfg.Codec = cdc
	cfg.TxConfig = txConfig
	amino, ok := legacyAmino.(*codec.LegacyAmino)
	if !ok {
		return Config{}, errors.New("legacyAmino must be a *codec.LegacyAmino")
	}
	cfg.LegacyAmino = amino
	cfg.InterfaceRegistry = interfaceRegistry
	cfg.GenesisState = appBuilder.DefaultGenesis()
	cfg.AppConstructor = func(val ValidatorI) servertypes.Application {
		// we build a unique app instance for every validator here
		var appBuilder *runtime.AppBuilder
		if err := depinject.Inject(
			depinject.Configs(
				appConfig,
				depinject.Supply(
					val.GetLogger(),
					simtestutil.NewAppOptionsWithFlagHome(val.GetViper().GetString(flags.FlagHome)),
				),
			),
			&appBuilder); err != nil {
			panic(err)
		}
		app := appBuilder.Build(
			coretesting.NewMemDB(),
			nil,
			append(baseappOpts,
				baseapp.SetPruning(pruningtypes.NewPruningOptionsFromString(val.GetAppConfig().Pruning)),
				baseapp.SetMinGasPrices(val.GetAppConfig().MinGasPrices),
				baseapp.SetChainID(cfg.ChainID),
			)...,
		)

		testdata.RegisterQueryServer(app.GRPCQueryRouter(), testdata.QueryImpl{})

		if err := app.Load(true); err != nil {
			panic(err)
		}

		return app
	}

	cfg.AddressCodec = addressCodec
	cfg.ValidatorAddressCodec = validatorAddressCodec
	cfg.ConsensusAddressCodec = consensusAddressCodec

	return cfg, nil
}

type (
	// Network defines a local in-process testing network using SimApp. It can be
	// configured to start any number of validators, each with its own RPC and API
	// clients. Typically, this test network would be used in client and integration
	// testing where user input is expected.
	//
	// Note, due to CometBFT constraints in regards to RPC functionality, there
	// may only be one test network running at a time. Thus, any caller must be
	// sure to Cleanup after testing is finished in order to allow other tests
	// to create networks. In addition, only the first validator will have a valid
	// RPC and API server/client.
	Network struct {
		Logger     Logger
		BaseDir    string
		Validators []*Validator

		Config Config
	}

	// Logger is a network logger interface that exposes testnet-level Log() methods for an in-process testing network
	// This is not to be confused with logging that may happen at an individual node or validator level
	Logger interface {
		Log(args ...interface{})
		Logf(format string, args ...interface{})
	}
)

var (
	_ Logger = (*testing.T)(nil)
	_ Logger = (*CLILogger)(nil)
)

// CLILogger wraps a cobra.Command and provides command logging methods.
type CLILogger struct {
	cmd *cobra.Command
}

// Log logs given args.
func (s CLILogger) Log(args ...interface{}) {
	s.cmd.Println(args...)
}

// Logf logs given args according to a format specifier.
func (s CLILogger) Logf(format string, args ...interface{}) {
	s.cmd.Printf(format, args...)
}

// NewCLILogger creates a new CLILogger.
func NewCLILogger(cmd *cobra.Command) CLILogger {
	return CLILogger{cmd}
}

// New creates a new Network for integration tests or in-process testnets run via the CLI
func New(l Logger, baseDir string, cfg Config) (NetworkI, error) {
	// only one caller/test can create and use a network at a time
	l.Log("acquiring test network lock")
	lock.Lock()

	network := &Network{
		Logger:     l,
		BaseDir:    baseDir,
		Validators: make([]*Validator, cfg.NumValidators),
		Config:     cfg,
	}

	l.Logf("preparing test network with chain-id \"%s\"\n", cfg.ChainID)

	monikers := make([]string, cfg.NumValidators)
	nodeIDs := make([]string, cfg.NumValidators)
	valPubKeys := make([]cryptotypes.PubKey, cfg.NumValidators)
	cmtConfigs := make([]*cmtcfg.Config, cfg.NumValidators)

	var (
		genAccounts []authtypes.GenesisAccount
		genBalances []banktypes.Balance
		genFiles    []string
	)

	buf := bufio.NewReader(os.Stdin)

	// generate private keys, node IDs, and initial transactions
	for i := 0; i < cfg.NumValidators; i++ {
		appCfg := srvconfig.DefaultConfig()
		appCfg.Pruning = cfg.PruningStrategy
		appCfg.MinGasPrices = cfg.MinGasPrices
		appCfg.API.Enable = true
		appCfg.API.Swagger = false
		appCfg.Telemetry.Enabled = false

		viper := viper.New()
		// Create default cometbft config for each validator
		cmtCfg := client.GetConfigFromViper(viper)

		cmtCfg.Consensus.TimeoutCommit = cfg.TimeoutCommit

		// Only allow the first validator to expose an RPC, API and gRPC
		// server/client due to CometBFT in-process constraints.
		apiAddr := ""
		cmtCfg.RPC.ListenAddress = ""
		appCfg.GRPC.Enable = false
		apiListenAddr := ""
		if i == 0 {
			if cfg.APIAddress != "" {
				apiListenAddr = cfg.APIAddress
			} else {
				if len(portPool) == 0 {
					return nil, fmt.Errorf("failed to get port for API server")
				}
				port := <-portPool
				apiListenAddr = fmt.Sprintf("tcp://127.0.0.1:%s", port)
			}

			appCfg.API.Address = apiListenAddr
			apiURL, err := url.Parse(apiListenAddr)
			if err != nil {
				return nil, err
			}
			apiAddr = fmt.Sprintf("http://%s:%s", apiURL.Hostname(), apiURL.Port())

			if cfg.RPCAddress != "" {
				cmtCfg.RPC.ListenAddress = cfg.RPCAddress
			} else {
				if len(portPool) == 0 {
					return nil, fmt.Errorf("failed to get port for RPC server")
				}
				port := <-portPool
				cmtCfg.RPC.ListenAddress = fmt.Sprintf("tcp://127.0.0.1:%s", port)
			}

			if cfg.GRPCAddress != "" {
				appCfg.GRPC.Address = cfg.GRPCAddress
			} else {
				if len(portPool) == 0 {
					return nil, fmt.Errorf("failed to get port for GRPC server")
				}
				port := <-portPool
				appCfg.GRPC.Address = fmt.Sprintf("127.0.0.1:%s", port)
			}
			appCfg.GRPC.Enable = true
		}

		logger := log.NewNopLogger()
		if cfg.EnableLogging {
			logger = log.NewLogger(os.Stdout) // TODO(mr): enable selection of log destination.
		}

		nodeDirName := fmt.Sprintf("node%d", i)
		nodeDir := filepath.Join(network.BaseDir, nodeDirName, "simd")
		clientDir := filepath.Join(network.BaseDir, nodeDirName, "simcli")
		gentxsDir := filepath.Join(network.BaseDir, "gentxs")

		err := os.MkdirAll(filepath.Join(nodeDir, "config"), 0o755)
		if err != nil {
			return nil, err
		}

		err = os.MkdirAll(clientDir, 0o755)
		if err != nil {
			return nil, err
		}

		cmtCfg.SetRoot(nodeDir)
		cmtCfg.Moniker = nodeDirName
		monikers[i] = nodeDirName

		if len(portPool) == 0 {
			return nil, fmt.Errorf("failed to get port for Proxy server")
		}
		port := <-portPool
		proxyAddr := fmt.Sprintf("tcp://127.0.0.1:%s", port)
		cmtCfg.ProxyApp = proxyAddr

		if len(portPool) == 0 {
			return nil, fmt.Errorf("failed to get port for Proxy server")
		}
		port = <-portPool
		p2pAddr := fmt.Sprintf("tcp://127.0.0.1:%s", port)
		cmtCfg.P2P.ListenAddress = p2pAddr
		cmtCfg.P2P.AddrBookStrict = false
		cmtCfg.P2P.AllowDuplicateIP = true

		cmtConfigs[i] = cmtCfg

		var mnemonic string
		if i < len(cfg.Mnemonics) {
			mnemonic = cfg.Mnemonics[i]
		}

		nodeID, pubKey, err := genutil.InitializeNodeValidatorFilesFromMnemonic(cmtCfg, mnemonic, ed25519.PrivKeyName)
		if err != nil {
			return nil, err
		}

		nodeIDs[i] = nodeID
		valPubKeys[i] = pubKey

		kb, err := keyring.New(sdk.KeyringServiceName(), keyring.BackendTest, clientDir, buf, cfg.Codec, cfg.KeyringOptions...)
		if err != nil {
			return nil, err
		}

		keyringAlgos, _ := kb.SupportedAlgorithms()
		algo, err := keyring.NewSigningAlgoFromString(cfg.SigningAlgo, keyringAlgos)
		if err != nil {
			return nil, err
		}

		addr, secret, err := testutil.GenerateSaveCoinKey(kb, nodeDirName, mnemonic, true, algo, sdk.GetFullBIP44Path())
		if err != nil {
			return nil, err
		}

		// if PrintMnemonic is set to true, we print the first validator node's secret to the network's logger
		// for debugging and manual testing
		if cfg.PrintMnemonic && i == 0 {
			printMnemonic(l, secret)
		}

		info := map[string]string{"secret": secret}
		infoBz, err := json.Marshal(info)
		if err != nil {
			return nil, err
		}

		// save private key seed words
		err = writeFile(fmt.Sprintf("%v.json", "key_seed"), clientDir, infoBz)
		if err != nil {
			return nil, err
		}

		balances := sdk.NewCoins(
			sdk.NewCoin(fmt.Sprintf("%stoken", nodeDirName), cfg.AccountTokens),
			sdk.NewCoin(cfg.BondDenom, cfg.StakingTokens),
		)

		genFiles = append(genFiles, cmtCfg.GenesisFile())
		genBalances = append(genBalances, banktypes.Balance{Address: addr.String(), Coins: balances.Sort()})
		genAccounts = append(genAccounts, authtypes.NewBaseAccount(addr, nil, 0, 0))

		commission, err := sdkmath.LegacyNewDecFromStr("0.5")
		if err != nil {
			return nil, err
		}

		createValMsg, err := stakingtypes.NewMsgCreateValidator(
			sdk.ValAddress(addr).String(),
			valPubKeys[i],
			sdk.NewCoin(cfg.BondDenom, cfg.BondedTokens),
			stakingtypes.NewDescription(nodeDirName, "", "", "", ""),
			stakingtypes.NewCommissionRates(commission, sdkmath.LegacyOneDec(), sdkmath.LegacyOneDec()),
			sdkmath.OneInt(),
		)
		if err != nil {
			return nil, err
		}

		p2pURL, err := url.Parse(p2pAddr)
		if err != nil {
			return nil, err
		}

		memo := fmt.Sprintf("%s@%s:%s", nodeIDs[i], p2pURL.Hostname(), p2pURL.Port())
		fee := sdk.NewCoins(sdk.NewCoin(fmt.Sprintf("%stoken", nodeDirName), sdkmath.NewInt(0)))
		txBuilder := cfg.TxConfig.NewTxBuilder()
		err = txBuilder.SetMsgs(createValMsg)
		if err != nil {
			return nil, err
		}
		txBuilder.SetFeeAmount(fee)    // Arbitrary fee
		txBuilder.SetGasLimit(1000000) // Need at least 100386
		txBuilder.SetMemo(memo)

		txFactory := tx.Factory{}
		txFactory = txFactory.
			WithChainID(cfg.ChainID).
			WithMemo(memo).
			WithKeybase(kb).
			WithTxConfig(cfg.TxConfig)

		clientCtx := client.Context{}.
			WithKeyringDir(clientDir).
			WithKeyring(kb).
			WithHomeDir(cmtCfg.RootDir).
			WithChainID(cfg.ChainID).
			WithInterfaceRegistry(cfg.InterfaceRegistry).
			WithCodec(cfg.Codec).
			WithLegacyAmino(cfg.LegacyAmino).
			WithTxConfig(cfg.TxConfig).
			WithAccountRetriever(cfg.AccountRetriever).
			WithAddressCodec(cfg.AddressCodec).
			WithValidatorAddressCodec(cfg.ValidatorAddressCodec).
			WithConsensusAddressCodec(cfg.ConsensusAddressCodec).
			WithNodeURI(cmtCfg.RPC.ListenAddress).
			WithCmdContext(context.TODO())

		if err := tx.Sign(clientCtx, txFactory, nodeDirName, txBuilder, true); err != nil {
			return nil, err
		}

		txBz, err := cfg.TxConfig.TxJSONEncoder()(txBuilder.GetTx())
		if err != nil {
			return nil, err
		}
		err = writeFile(fmt.Sprintf("%v.json", nodeDirName), gentxsDir, txBz)
		if err != nil {
			return nil, err
		}
		err = srvconfig.WriteConfigFile(filepath.Join(nodeDir, "config", "app.toml"), appCfg)
		if err != nil {
			return nil, err
		}

		// Provide ChainID here since we can't modify it in the Comet config.
		viper.Set(flags.FlagChainID, cfg.ChainID)

		network.Validators[i] = &Validator{
			AppConfig:  appCfg,
			clientCtx:  clientCtx,
			viper:      viper,
			logger:     logger,
			dir:        filepath.Join(network.BaseDir, nodeDirName),
			nodeID:     nodeID,
			pubKey:     pubKey,
			moniker:    nodeDirName,
			rPCAddress: cmtCfg.RPC.ListenAddress,
			p2PAddress: cmtCfg.P2P.ListenAddress,
			aPIAddress: apiAddr,
			address:    addr,
			valAddress: sdk.ValAddress(addr),
		}
	}

	err := initGenFiles(cfg, genAccounts, genBalances, genFiles)
	if err != nil {
		return nil, err
	}
	err = collectGenFiles(cfg, network.Validators, cmtConfigs, network.BaseDir)
	if err != nil {
		return nil, err
	}

	l.Log("starting test network...")
	for idx, v := range network.Validators {
		if err := startInProcess(cfg, v); err != nil {
			return nil, err
		}
		l.Log("started validator", idx)
	}

	height, err := network.LatestHeight()
	if err != nil {
		return nil, err
	}

	l.Log("started test network at height:", height)

	// Ensure we cleanup in case any test was abruptly halted (e.g. SIGINT) as any
	// defer in a test would not be called.
	trapSignal(network.Cleanup)

	return network, nil
}

// trapSignal traps SIGINT and SIGTERM and calls os.Exit once a signal is received.
func trapSignal(cleanupFunc func()) {
	sigs := make(chan os.Signal, 1)
	signal.Notify(sigs, syscall.SIGINT, syscall.SIGTERM)

	go func() {
		sig := <-sigs

		if cleanupFunc != nil {
			cleanupFunc()
		}
		exitCode := 128

		switch sig {
		case syscall.SIGINT:
			exitCode += int(syscall.SIGINT)
		case syscall.SIGTERM:
			exitCode += int(syscall.SIGTERM)
		}

		os.Exit(exitCode)
	}()
}

// LatestHeight returns the latest height of the network or an error if the
// query fails or no validators exist.
func (n *Network) LatestHeight() (int64, error) {
	if len(n.Validators) == 0 {
		return 0, errors.New("no validators available")
	}

	ticker := time.NewTicker(time.Second)
	defer ticker.Stop()

	timeout := time.NewTimer(time.Second * 5)
	defer timeout.Stop()

	var latestHeight atomic.Int64
	val := n.Validators[0]
	queryClient := cmtservice.NewServiceClient(val.clientCtx)

	for {
		select {
		case <-timeout.C:
			return latestHeight.Load(), errors.New("timeout exceeded waiting for block")
		case <-ticker.C:
			done := make(chan struct{})
			go func() {
				res, err := queryClient.GetLatestBlock(context.Background(), &cmtservice.GetLatestBlockRequest{})
				if err == nil && res != nil {
					latestHeight.Store(res.SdkBlock.Header.Height)
				}
				done <- struct{}{}
			}()
			select {
			case <-timeout.C:
				return latestHeight.Load(), errors.New("timeout exceeded waiting for block")
			case <-done:
				if latestHeight.Load() != 0 {
					return latestHeight.Load(), nil
				}
			}
		}
	}
}

// WaitForHeight performs a blocking check where it waits for a block to be
// committed after a given block. If that height is not reached within a timeout,
// an error is returned. Regardless, the latest height queried is returned.
func (n *Network) WaitForHeight(h int64) (int64, error) {
	return n.WaitForHeightWithTimeout(h, 10*time.Second)
}

func (n *Network) GetValidators() []ValidatorI {
	var vals []ValidatorI
	for _, val := range n.Validators {
		vals = append(vals, val)
	}
	return vals
}

// WaitForHeightWithTimeout is the same as WaitForHeight except the caller can
// provide a custom timeout.
func (n *Network) WaitForHeightWithTimeout(h int64, t time.Duration) (int64, error) {
	ticker := time.NewTicker(time.Second)
	defer ticker.Stop()

	timeout := time.NewTimer(t)
	defer timeout.Stop()

	if len(n.Validators) == 0 {
		return 0, errors.New("no validators available")
	}

	var latestHeight int64
	val := n.Validators[0]
	queryClient := cmtservice.NewServiceClient(val.clientCtx)

	for {
		select {
		case <-timeout.C:
			return latestHeight, errors.New("timeout exceeded waiting for block")
		case <-ticker.C:

			res, err := queryClient.GetLatestBlock(context.Background(), &cmtservice.GetLatestBlockRequest{})
			if err == nil && res != nil {
				latestHeight = res.GetSdkBlock().Header.Height
				if latestHeight >= h {
					return latestHeight, nil
				}
			}
		}
	}
}

// RetryForBlocks will wait for the next block and execute the function provided.
// It will do this until the function returns a nil error or until the number of
// blocks has been reached.
func (n *Network) RetryForBlocks(retryFunc func() error, blocks int) error {
	for i := 0; i < blocks; i++ {
		_ = n.WaitForNextBlock()
		err := retryFunc()
		if err == nil {
			return nil
		}
		// we've reached the last block to wait, return the error
		if i == blocks-1 {
			return err
		}
	}
	return nil
}

// WaitForNextBlock waits for the next block to be committed, returning an error
// upon failure.
func (n *Network) WaitForNextBlock() error {
	lastBlock, err := n.LatestHeight()
	if err != nil {
		return err
	}

	_, err = n.WaitForHeight(lastBlock + 1)
	if err != nil {
		return err
	}

	return err
}

// Cleanup removes the root testing (temporary) directory and stops both the
// CometBFT and API services. It allows other callers to create and start
// test networks. This method must be called when a test is finished, typically
// in a defer.
func (n *Network) Cleanup() {
	defer func() {
		lock.Unlock()
		n.Logger.Log("released test network lock")
	}()

	n.Logger.Log("cleaning up test network...")

	for _, v := range n.Validators {
		// cancel the validator's context which will signal to the gRPC and API
		// goroutines that they should gracefully exit.
		v.cancelFn()

		if err := v.errGroup.Wait(); err != nil {
			n.Logger.Log("unexpected error waiting for validator gRPC and API processes to exit", "err", err)
		}

		if v.tmNode != nil && v.tmNode.IsRunning() {
			if err := v.tmNode.Stop(); err != nil {
				n.Logger.Log("failed to stop validator CometBFT node", "err", err)
			}
		}

		if v.grpcWeb != nil {
			_ = v.grpcWeb.Close()
		}

		if v.app != nil {
			if err := v.app.Close(); err != nil {
				n.Logger.Log("failed to stop validator ABCI application", "err", err)
			}
		}
	}

	time.Sleep(100 * time.Millisecond)

	if n.Config.CleanupDir {
		_ = os.RemoveAll(n.BaseDir)
	}

	n.Logger.Log("finished cleaning up test network")
}

// printMnemonic prints a provided mnemonic seed phrase on a network logger
// for debugging and manual testing
func printMnemonic(l Logger, secret string) {
	lines := []string{
		"THIS MNEMONIC IS FOR TESTING PURPOSES ONLY",
		"DO NOT USE IN PRODUCTION",
		"",
		strings.Join(strings.Fields(secret)[0:8], " "),
		strings.Join(strings.Fields(secret)[8:16], " "),
		strings.Join(strings.Fields(secret)[16:24], " "),
	}

	lineLengths := make([]int, len(lines))
	for i, line := range lines {
		lineLengths[i] = len(line)
	}

	maxLineLength := 0
	for _, lineLen := range lineLengths {
		if lineLen > maxLineLength {
			maxLineLength = lineLen
		}
	}

	l.Log("\n")
	l.Log(strings.Repeat("+", maxLineLength+8))
	for _, line := range lines {
		l.Logf("++  %s  ++\n", centerText(line, maxLineLength))
	}
	l.Log(strings.Repeat("+", maxLineLength+8))
	l.Log("\n")
}

// centerText centers text across a fixed width, filling either side with whitespace buffers
func centerText(text string, width int) string {
	textLen := len(text)
	leftBuffer := strings.Repeat(" ", (width-textLen)/2)
	rightBuffer := strings.Repeat(" ", (width-textLen)/2+(width-textLen)%2)

	return fmt.Sprintf("%s%s%s", leftBuffer, text, rightBuffer)
}
