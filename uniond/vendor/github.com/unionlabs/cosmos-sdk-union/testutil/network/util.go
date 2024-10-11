package network

import (
	"context"
	"encoding/json"
	"fmt"
	"net"
	"os"
	"path/filepath"

	cmtcfg "github.com/cometbft/cometbft/config"
	cmtcrypto "github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/ed25519"
	"github.com/cometbft/cometbft/node"
	"github.com/cometbft/cometbft/p2p"
	pvm "github.com/cometbft/cometbft/privval"
	"github.com/cometbft/cometbft/proxy"
	"github.com/cometbft/cometbft/rpc/client/local"
	cmttime "github.com/cometbft/cometbft/types/time"
	"golang.org/x/sync/errgroup"

	"cosmossdk.io/log"
	banktypes "cosmossdk.io/x/bank/types"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/server"
	"github.com/cosmos/cosmos-sdk/server/api"
	servergrpc "github.com/cosmos/cosmos-sdk/server/grpc"
	servercmtlog "github.com/cosmos/cosmos-sdk/server/log"
	"github.com/cosmos/cosmos-sdk/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
	"github.com/cosmos/cosmos-sdk/x/genutil"
	genutiltypes "github.com/cosmos/cosmos-sdk/x/genutil/types"
)

func startInProcess(cfg Config, val *Validator) error {
	logger := val.GetLogger()
	cmtCfg := client.GetConfigFromViper(val.GetViper())
	cmtCfg.Instrumentation.Prometheus = false

	if err := val.AppConfig.ValidateBasic(); err != nil {
		return err
	}

	nodeKey, err := p2p.LoadOrGenNodeKey(cmtCfg.NodeKeyFile())
	if err != nil {
		return err
	}

	app := cfg.AppConstructor(val)
	val.app = app

	appGenesisProvider := func() (node.ChecksummedGenesisDoc, error) {
		appGenesis, err := genutiltypes.AppGenesisFromFile(cmtCfg.GenesisFile())
		if err != nil {
			return node.ChecksummedGenesisDoc{
				Sha256Checksum: []byte{},
			}, err
		}
		gen, err := appGenesis.ToGenesisDoc()
		if err != nil {
			return node.ChecksummedGenesisDoc{
				Sha256Checksum: []byte{},
			}, err
		}
		return node.ChecksummedGenesisDoc{GenesisDoc: gen, Sha256Checksum: make([]byte, 0)}, nil
	}

	cmtApp := server.NewCometABCIWrapper(app)
	pv, err := pvm.LoadOrGenFilePV(cmtCfg.PrivValidatorKeyFile(), cmtCfg.PrivValidatorStateFile(), func() (cmtcrypto.PrivKey, error) {
		return ed25519.GenPrivKey(), nil
	})
	if err != nil {
		return err
	}

	tmNode, err := node.NewNode( //resleak:notresource
		context.TODO(),
		cmtCfg,
		pv,
		nodeKey,
		proxy.NewLocalClientCreator(cmtApp),
		appGenesisProvider,
		cmtcfg.DefaultDBProvider,
		node.DefaultMetricsProvider(cmtCfg.Instrumentation),
		servercmtlog.CometLoggerWrapper{Logger: logger.With("module", val.moniker)},
	)
	if err != nil {
		return err
	}

	if err := tmNode.Start(); err != nil {
		return err
	}
	val.tmNode = tmNode

	if val.rPCAddress != "" {
		val.rPCClient = local.New(tmNode)
	}

	// We'll need a RPC client if the validator exposes a gRPC or REST endpoint.
	if val.aPIAddress != "" || val.AppConfig.GRPC.Enable {
		val.clientCtx = val.clientCtx.
			WithClient(val.rPCClient)

		app.RegisterTxService(val.clientCtx)
		app.RegisterTendermintService(val.clientCtx)
		app.RegisterNodeService(val.clientCtx, *val.AppConfig)
	}

	ctx := context.Background()
	ctx, val.cancelFn = context.WithCancel(ctx)
	val.errGroup, ctx = errgroup.WithContext(ctx)

	grpcCfg := val.AppConfig.GRPC

	if grpcCfg.Enable {
		grpcSrv, err := servergrpc.NewGRPCServer(val.clientCtx, app, grpcCfg)
		if err != nil {
			return err
		}

		// Start the gRPC server in a goroutine. Note, the provided ctx will ensure
		// that the server is gracefully shut down.
		val.errGroup.Go(func() error {
			return servergrpc.StartGRPCServer(ctx, logger.With(log.ModuleKey, "grpc-server"), grpcCfg, grpcSrv)
		})

		val.grpc = grpcSrv
	}

	if val.aPIAddress != "" {
		apiSrv := api.New(val.clientCtx, logger.With(log.ModuleKey, "api-server"), val.grpc)
		app.RegisterAPIRoutes(apiSrv, val.AppConfig.API)

		val.errGroup.Go(func() error {
			return apiSrv.Start(ctx, *val.AppConfig)
		})

		val.api = apiSrv
	}

	return nil
}

func collectGenFiles(cfg Config, vals []*Validator, cmtConfigs []*cmtcfg.Config, outputDir string) error {
	genTime := cfg.GenesisTime
	if genTime.IsZero() {
		genTime = cmttime.Now()
	}

	for i := 0; i < cfg.NumValidators; i++ {
		cmtCfg := cmtConfigs[i]

		nodeDir := filepath.Join(outputDir, vals[i].moniker, "simd")
		gentxsDir := filepath.Join(outputDir, "gentxs")

		cmtCfg.Moniker = vals[i].moniker
		cmtCfg.SetRoot(nodeDir)

		initCfg := genutiltypes.NewInitConfig(cfg.ChainID, gentxsDir, vals[i].nodeID, vals[i].pubKey)

		genFile := cmtCfg.GenesisFile()
		appGenesis, err := genutiltypes.AppGenesisFromFile(genFile)
		if err != nil {
			return err
		}

		appState, err := genutil.GenAppStateFromConfig(cfg.Codec, cfg.TxConfig,
			cmtCfg, initCfg, appGenesis, genutiltypes.DefaultMessageValidator,
			cfg.ValidatorAddressCodec, cfg.AddressCodec)
		if err != nil {
			return err
		}

		// overwrite each validator's genesis file to have a canonical genesis time
		if err := genutil.ExportGenesisFileWithTime(genFile, cfg.ChainID, nil, appState, genTime); err != nil {
			return err
		}

		v := vals[i].GetViper()
		v.Set(flags.FlagHome, nodeDir)
		v.SetConfigType("toml")
		v.SetConfigName("config")
		v.AddConfigPath(filepath.Join(nodeDir, "config"))
		err = v.ReadInConfig()
		if err != nil {
			return err
		}
	}

	return nil
}

func initGenFiles(cfg Config, genAccounts []authtypes.GenesisAccount, genBalances []banktypes.Balance, genFiles []string) error {
	// set the accounts in the genesis state
	var authGenState authtypes.GenesisState
	cfg.Codec.MustUnmarshalJSON(cfg.GenesisState[testutil.AuthModuleName], &authGenState)

	accounts, err := authtypes.PackAccounts(genAccounts)
	if err != nil {
		return err
	}

	authGenState.Accounts = append(authGenState.Accounts, accounts...)
	cfg.GenesisState[testutil.AuthModuleName] = cfg.Codec.MustMarshalJSON(&authGenState)

	// set the balances in the genesis state
	var bankGenState banktypes.GenesisState
	cfg.Codec.MustUnmarshalJSON(cfg.GenesisState[testutil.BankModuleName], &bankGenState)

	bankGenState.Balances = append(bankGenState.Balances, genBalances...)
	cfg.GenesisState[testutil.BankModuleName] = cfg.Codec.MustMarshalJSON(&bankGenState)

	appGenStateJSON, err := json.MarshalIndent(cfg.GenesisState, "", "  ")
	if err != nil {
		return err
	}

	appGenesis := genutiltypes.AppGenesis{
		ChainID:  cfg.ChainID,
		AppState: appGenStateJSON,
		Consensus: &genutiltypes.ConsensusGenesis{
			Validators: nil,
		},
	}

	// generate empty genesis files for each validator and save
	for i := 0; i < cfg.NumValidators; i++ {
		if err := appGenesis.SaveAs(genFiles[i]); err != nil {
			return err
		}
	}

	return nil
}

func writeFile(name, dir string, contents []byte) error {
	file := filepath.Join(dir, name)

	if err := os.MkdirAll(dir, 0o755); err != nil {
		return fmt.Errorf("could not create directory %q: %w", dir, err)
	}

	if err := os.WriteFile(file, contents, 0o600); err != nil {
		return err
	}

	return nil
}

// Get a free address for a test CometBFT server
// protocol is either tcp, http, etc
func FreeTCPAddr() (addr, port string, closeFn func() error, err error) {
	l, err := net.Listen("tcp", "127.0.0.1:0")
	if err != nil {
		return "", "", nil, err
	}

	closeFn = func() error {
		return l.Close()
	}

	portI := l.Addr().(*net.TCPAddr).Port
	port = fmt.Sprintf("%d", portI)
	addr = fmt.Sprintf("tcp://127.0.0.1:%s", port)
	return
}
