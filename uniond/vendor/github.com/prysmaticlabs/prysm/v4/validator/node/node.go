// Package node is the main process which handles the lifecycle of
// the runtime services in a validator client process, gracefully shutting
// everything down upon close.
package node

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"net/http"
	"net/url"
	"os"
	"os/signal"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"syscall"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/gorilla/mux"
	gwruntime "github.com/grpc-ecosystem/grpc-gateway/v2/runtime"
	"github.com/pkg/errors"
	fastssz "github.com/prysmaticlabs/fastssz"
	"github.com/prysmaticlabs/prysm/v4/api"
	"github.com/prysmaticlabs/prysm/v4/api/gateway"
	"github.com/prysmaticlabs/prysm/v4/api/server"
	"github.com/prysmaticlabs/prysm/v4/async/event"
	"github.com/prysmaticlabs/prysm/v4/cmd"
	"github.com/prysmaticlabs/prysm/v4/cmd/validator/flags"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	validatorServiceConfig "github.com/prysmaticlabs/prysm/v4/config/validator/service"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/validator"
	"github.com/prysmaticlabs/prysm/v4/container/slice"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	"github.com/prysmaticlabs/prysm/v4/monitoring/backup"
	"github.com/prysmaticlabs/prysm/v4/monitoring/prometheus"
	tracing2 "github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	validatorpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/validator-client"
	"github.com/prysmaticlabs/prysm/v4/runtime"
	"github.com/prysmaticlabs/prysm/v4/runtime/debug"
	"github.com/prysmaticlabs/prysm/v4/runtime/prereqs"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/validator/accounts/wallet"
	"github.com/prysmaticlabs/prysm/v4/validator/client"
	"github.com/prysmaticlabs/prysm/v4/validator/db/iface"
	"github.com/prysmaticlabs/prysm/v4/validator/db/kv"
	g "github.com/prysmaticlabs/prysm/v4/validator/graffiti"
	"github.com/prysmaticlabs/prysm/v4/validator/keymanager/local"
	remoteweb3signer "github.com/prysmaticlabs/prysm/v4/validator/keymanager/remote-web3signer"
	"github.com/prysmaticlabs/prysm/v4/validator/rpc"
	"github.com/prysmaticlabs/prysm/v4/validator/web"
	"github.com/sirupsen/logrus"
	"github.com/urfave/cli/v2"
	"google.golang.org/protobuf/encoding/protojson"
	"k8s.io/apimachinery/pkg/util/yaml"
)

// ValidatorClient defines an instance of an Ethereum validator that manages
// the entire lifecycle of services attached to it participating in proof of stake.
type ValidatorClient struct {
	cliCtx            *cli.Context
	ctx               context.Context
	cancel            context.CancelFunc
	db                *kv.Store
	services          *runtime.ServiceRegistry // Lifecycle and service store.
	lock              sync.RWMutex
	wallet            *wallet.Wallet
	walletInitialized *event.Feed
	stop              chan struct{} // Channel to wait for termination notifications.
}

// NewValidatorClient creates a new instance of the Prysm validator client.
func NewValidatorClient(cliCtx *cli.Context) (*ValidatorClient, error) {
	// TODO(#9883) - Maybe we can pass in a new validator client config instead of the cliCTX to abstract away the use of flags here .
	if err := tracing2.Setup(
		"validator", // service name
		cliCtx.String(cmd.TracingProcessNameFlag.Name),
		cliCtx.String(cmd.TracingEndpointFlag.Name),
		cliCtx.Float64(cmd.TraceSampleFractionFlag.Name),
		cliCtx.Bool(cmd.EnableTracingFlag.Name),
	); err != nil {
		return nil, err
	}

	verbosity := cliCtx.String(cmd.VerbosityFlag.Name)
	level, err := logrus.ParseLevel(verbosity)
	if err != nil {
		return nil, err
	}
	logrus.SetLevel(level)

	// Warn if user's platform is not supported
	prereqs.WarnIfPlatformNotSupported(cliCtx.Context)

	registry := runtime.NewServiceRegistry()
	ctx, cancel := context.WithCancel(cliCtx.Context)
	validatorClient := &ValidatorClient{
		cliCtx:            cliCtx,
		ctx:               ctx,
		cancel:            cancel,
		services:          registry,
		walletInitialized: new(event.Feed),
		stop:              make(chan struct{}),
	}

	if err := features.ConfigureValidator(cliCtx); err != nil {
		return nil, err
	}
	if err := cmd.ConfigureValidator(cliCtx); err != nil {
		return nil, err
	}

	if cliCtx.IsSet(cmd.ChainConfigFileFlag.Name) {
		chainConfigFileName := cliCtx.String(cmd.ChainConfigFileFlag.Name)
		if err := params.LoadChainConfigFile(chainConfigFileName, nil); err != nil {
			return nil, err
		}
	}

	configureFastSSZHashingAlgorithm()

	// initialize router used for endpoints
	router := newRouter(cliCtx)
	// If the --web flag is enabled to administer the validator
	// client via a web portal, we start the validator client in a different way.
	// Change Web flag name to enable keymanager API, look at merging initializeFromCLI and initializeForWeb maybe after WebUI DEPRECATED.
	if cliCtx.IsSet(flags.EnableWebFlag.Name) {
		if cliCtx.IsSet(flags.Web3SignerURLFlag.Name) || cliCtx.IsSet(flags.Web3SignerPublicValidatorKeysFlag.Name) {
			log.Warn("Remote Keymanager API enabled. Prysm web does not properly support web3signer at this time")
		}
		log.Info("Enabling web portal to manage the validator client")
		if err := validatorClient.initializeForWeb(cliCtx, router); err != nil {
			return nil, err
		}
		return validatorClient, nil
	}

	if err := validatorClient.initializeFromCLI(cliCtx, router); err != nil {
		return nil, err
	}

	return validatorClient, nil
}

func newRouter(cliCtx *cli.Context) *mux.Router {
	var allowedOrigins []string
	if cliCtx.IsSet(flags.GPRCGatewayCorsDomain.Name) {
		allowedOrigins = strings.Split(cliCtx.String(flags.GPRCGatewayCorsDomain.Name), ",")
	} else {
		allowedOrigins = strings.Split(flags.GPRCGatewayCorsDomain.Value, ",")
	}
	r := mux.NewRouter()
	r.Use(server.NormalizeQueryValuesHandler)
	r.Use(server.CorsHandler(allowedOrigins))
	return r
}

// Start every service in the validator client.
func (c *ValidatorClient) Start() {
	c.lock.Lock()

	log.WithFields(logrus.Fields{
		"version": version.Version(),
	}).Info("Starting validator node")

	c.services.StartAll()

	stop := c.stop
	c.lock.Unlock()

	go func() {
		sigc := make(chan os.Signal, 1)
		signal.Notify(sigc, syscall.SIGINT, syscall.SIGTERM)
		defer signal.Stop(sigc)
		<-sigc
		log.Info("Got interrupt, shutting down...")
		debug.Exit(c.cliCtx) // Ensure trace and CPU profile data are flushed.
		go c.Close()
		for i := 10; i > 0; i-- {
			<-sigc
			if i > 1 {
				log.WithField("times", i-1).Info("Already shutting down, interrupt more to panic.")
			}
		}
		panic("Panic closing the validator client")
	}()

	// Wait for stop channel to be closed.
	<-stop
}

// Close handles graceful shutdown of the system.
func (c *ValidatorClient) Close() {
	c.lock.Lock()
	defer c.lock.Unlock()

	c.services.StopAll()
	log.Info("Stopping Prysm validator")
	c.cancel()
	close(c.stop)
}

// checkLegacyDatabaseLocation checks is a database exists in the specified location.
// If it does not, it checks if a database exists in the legacy location.
// If it does, it returns the legacy location.
func (c *ValidatorClient) getLegacyDatabaseLocation(
	isInteropNumValidatorsSet bool,
	isWeb3SignerURLFlagSet bool,
	dataDir string,
	dataFile string,
	walletDir string,
) (string, string) {
	if isInteropNumValidatorsSet || dataDir != cmd.DefaultDataDir() || file.Exists(dataFile) || c.wallet == nil {
		return dataDir, dataFile
	}

	// We look in the previous, legacy directories.
	// See https://github.com/prysmaticlabs/prysm/issues/13391
	legacyDataDir := c.wallet.AccountsDir()
	if isWeb3SignerURLFlagSet {
		legacyDataDir = walletDir
	}

	legacyDataFile := filepath.Join(legacyDataDir, kv.ProtectionDbFileName)

	if file.Exists(legacyDataFile) {
		log.Infof(`Database not found in the --datadir directory (%s)
		but found in the --wallet-dir directory (%s),
		which was the legacy default.
		The next time you run the validator client without a database,
		it will be created into the --datadir directory (%s).
		To silence this message, you can move the database from (%s)
		to (%s).`,
			dataDir, legacyDataDir, dataDir, legacyDataFile, dataFile)

		dataDir = legacyDataDir
		dataFile = legacyDataFile
	}

	return dataDir, dataFile
}

func (c *ValidatorClient) initializeFromCLI(cliCtx *cli.Context, router *mux.Router) error {
	dataDir := cliCtx.String(cmd.DataDirFlag.Name)
	dataFile := filepath.Join(dataDir, kv.ProtectionDbFileName)
	walletDir := cliCtx.String(flags.WalletDirFlag.Name)
	isInteropNumValidatorsSet := cliCtx.IsSet(flags.InteropNumValidators.Name)
	isWeb3SignerURLFlagSet := cliCtx.IsSet(flags.Web3SignerURLFlag.Name)

	if !isInteropNumValidatorsSet {
		// Custom Check For Web3Signer
		if isWeb3SignerURLFlagSet {
			c.wallet = wallet.NewWalletForWeb3Signer()
		} else {
			fmt.Println("initializeFromCLI asking for wallet")
			w, err := wallet.OpenWalletOrElseCli(cliCtx, func(cliCtx *cli.Context) (*wallet.Wallet, error) {
				return nil, wallet.ErrNoWalletFound
			})
			if err != nil {
				return errors.Wrap(err, "could not open wallet")
			}
			c.wallet = w
			// TODO(#9883) - Remove this when we have a better way to handle this.
			log.WithFields(logrus.Fields{
				"wallet":          w.AccountsDir(),
				"keymanager-kind": w.KeymanagerKind().String(),
			}).Info("Opened validator wallet")
		}
	}

	// Workaround for https://github.com/prysmaticlabs/prysm/issues/13391
	dataDir, dataFile = c.getLegacyDatabaseLocation(
		isInteropNumValidatorsSet,
		isWeb3SignerURLFlagSet,
		dataDir,
		dataFile,
		walletDir,
	)

	clearFlag := cliCtx.Bool(cmd.ClearDB.Name)
	forceClearFlag := cliCtx.Bool(cmd.ForceClearDB.Name)
	if clearFlag || forceClearFlag {
		if err := clearDB(cliCtx.Context, dataDir, forceClearFlag); err != nil {
			return err
		}
	} else {
		if !file.Exists(dataFile) {
			log.Warnf("Slashing protection file %s is missing.\n"+
				"If you changed your --datadir, please copy your previous \"validator.db\" file into your current --datadir.\n"+
				"Disregard this warning if this is the first time you are running this set of keys.", dataFile)
		}
	}
	log.WithField("databasePath", dataDir).Info("Checking DB")

	valDB, err := kv.NewKVStore(cliCtx.Context, dataDir, &kv.Config{
		PubKeys: nil,
	})
	if err != nil {
		return errors.Wrap(err, "could not initialize db")
	}
	c.db = valDB
	if err := valDB.RunUpMigrations(cliCtx.Context); err != nil {
		return errors.Wrap(err, "could not run database migration")
	}

	if !cliCtx.Bool(cmd.DisableMonitoringFlag.Name) {
		if err := c.registerPrometheusService(cliCtx); err != nil {
			return err
		}
	}
	if err := c.registerValidatorService(cliCtx); err != nil {
		return err
	}
	if cliCtx.Bool(flags.EnableRPCFlag.Name) {
		if err := c.registerRPCService(router); err != nil {
			return err
		}
		if err := c.registerRPCGatewayService(router); err != nil {
			return err
		}
	}
	return nil
}

func (c *ValidatorClient) initializeForWeb(cliCtx *cli.Context, router *mux.Router) error {
	dataDir := cliCtx.String(cmd.DataDirFlag.Name)
	dataFile := filepath.Join(dataDir, kv.ProtectionDbFileName)
	walletDir := cliCtx.String(flags.WalletDirFlag.Name)
	isInteropNumValidatorsSet := cliCtx.IsSet(flags.InteropNumValidators.Name)
	isWeb3SignerURLFlagSet := cliCtx.IsSet(flags.Web3SignerURLFlag.Name)

	if cliCtx.IsSet(flags.Web3SignerURLFlag.Name) {
		// Custom Check For Web3Signer
		c.wallet = wallet.NewWalletForWeb3Signer()
	} else {
		// Read the wallet password file from the cli context.
		if err := setWalletPasswordFilePath(cliCtx); err != nil {
			return errors.Wrap(err, "could not read wallet password file")
		}

		// Read the wallet from the specified path.
		w, err := wallet.OpenWalletOrElseCli(cliCtx, func(cliCtx *cli.Context) (*wallet.Wallet, error) {
			return nil, nil
		})
		if err != nil {
			return errors.Wrap(err, "could not open wallet")
		}
		c.wallet = w
	}

	// Workaround for https://github.com/prysmaticlabs/prysm/issues/13391
	dataDir, _ = c.getLegacyDatabaseLocation(
		isInteropNumValidatorsSet,
		isWeb3SignerURLFlagSet,
		dataDir,
		dataFile,
		walletDir,
	)

	clearFlag := cliCtx.Bool(cmd.ClearDB.Name)
	forceClearFlag := cliCtx.Bool(cmd.ForceClearDB.Name)

	if clearFlag || forceClearFlag {
		if err := clearDB(cliCtx.Context, dataDir, forceClearFlag); err != nil {
			return err
		}
	}
	log.WithField("databasePath", dataDir).Info("Checking DB")
	valDB, err := kv.NewKVStore(cliCtx.Context, dataDir, &kv.Config{
		PubKeys: nil,
	})
	if err != nil {
		return errors.Wrap(err, "could not initialize db")
	}
	c.db = valDB
	if err := valDB.RunUpMigrations(cliCtx.Context); err != nil {
		return errors.Wrap(err, "could not run database migration")
	}

	if !cliCtx.Bool(cmd.DisableMonitoringFlag.Name) {
		if err := c.registerPrometheusService(cliCtx); err != nil {
			return err
		}
	}
	if err := c.registerValidatorService(cliCtx); err != nil {
		return err
	}

	if err := c.registerRPCService(router); err != nil {
		return err
	}
	if err := c.registerRPCGatewayService(router); err != nil {
		return err
	}
	gatewayHost := cliCtx.String(flags.GRPCGatewayHost.Name)
	gatewayPort := cliCtx.Int(flags.GRPCGatewayPort.Name)
	webAddress := fmt.Sprintf("http://%s:%d", gatewayHost, gatewayPort)
	log.WithField("address", webAddress).Info(
		"Starting Prysm web UI on address, open in browser to access",
	)
	return nil
}

func (c *ValidatorClient) registerPrometheusService(cliCtx *cli.Context) error {
	var additionalHandlers []prometheus.Handler
	if cliCtx.IsSet(cmd.EnableBackupWebhookFlag.Name) {
		additionalHandlers = append(
			additionalHandlers,
			prometheus.Handler{
				Path:    "/db/backup",
				Handler: backup.Handler(c.db, cliCtx.String(cmd.BackupWebhookOutputDir.Name)),
			},
		)
	}
	service := prometheus.NewService(
		fmt.Sprintf("%s:%d", c.cliCtx.String(cmd.MonitoringHostFlag.Name), c.cliCtx.Int(flags.MonitoringPortFlag.Name)),
		c.services,
		additionalHandlers...,
	)
	logrus.AddHook(prometheus.NewLogrusCollector())
	return c.services.RegisterService(service)
}

func (c *ValidatorClient) registerValidatorService(cliCtx *cli.Context) error {
	var (
		endpoint             string        = c.cliCtx.String(flags.BeaconRPCProviderFlag.Name)
		dataDir              string        = c.cliCtx.String(cmd.DataDirFlag.Name)
		logValidatorBalances bool          = !c.cliCtx.Bool(flags.DisablePenaltyRewardLogFlag.Name)
		emitAccountMetrics   bool          = !c.cliCtx.Bool(flags.DisableAccountMetricsFlag.Name)
		cert                 string        = c.cliCtx.String(flags.CertFlag.Name)
		graffiti             string        = c.cliCtx.String(flags.GraffitiFlag.Name)
		maxCallRecvMsgSize   int           = c.cliCtx.Int(cmd.GrpcMaxCallRecvMsgSizeFlag.Name)
		grpcRetries          uint          = c.cliCtx.Uint(flags.GrpcRetriesFlag.Name)
		grpcRetryDelay       time.Duration = c.cliCtx.Duration(flags.GrpcRetryDelayFlag.Name)

		interopKeysConfig *local.InteropKeymanagerConfig
		err               error
	)

	// Configure interop.
	if c.cliCtx.IsSet(flags.InteropNumValidators.Name) {
		interopKeysConfig = &local.InteropKeymanagerConfig{
			Offset:           cliCtx.Uint64(flags.InteropStartIndex.Name),
			NumValidatorKeys: cliCtx.Uint64(flags.InteropNumValidators.Name),
		}
	}

	// Configure graffiti.
	graffitiStruct := &g.Graffiti{}
	if c.cliCtx.IsSet(flags.GraffitiFileFlag.Name) {
		graffitiFilePath := c.cliCtx.String(flags.GraffitiFileFlag.Name)

		graffitiStruct, err = g.ParseGraffitiFile(graffitiFilePath)
		if err != nil {
			log.WithError(err).Warn("Could not parse graffiti file")
		}
	}

	web3signerConfig, err := Web3SignerConfig(c.cliCtx)
	if err != nil {
		return err
	}

	proposerSettings, err := proposerSettings(c.cliCtx, c.db)
	if err != nil {
		return err
	}

	validatorService, err := client.NewValidatorService(c.cliCtx.Context, &client.Config{
		Endpoint:                   endpoint,
		DataDir:                    dataDir,
		LogValidatorBalances:       logValidatorBalances,
		EmitAccountMetrics:         emitAccountMetrics,
		CertFlag:                   cert,
		GraffitiFlag:               g.ParseHexGraffiti(graffiti),
		GrpcMaxCallRecvMsgSizeFlag: maxCallRecvMsgSize,
		GrpcRetriesFlag:            grpcRetries,
		GrpcRetryDelay:             grpcRetryDelay,
		GrpcHeadersFlag:            c.cliCtx.String(flags.GrpcHeadersFlag.Name),
		ValDB:                      c.db,
		UseWeb:                     c.cliCtx.Bool(flags.EnableWebFlag.Name),
		InteropKeysConfig:          interopKeysConfig,
		Wallet:                     c.wallet,
		WalletInitializedFeed:      c.walletInitialized,
		GraffitiStruct:             graffitiStruct,
		Web3SignerConfig:           web3signerConfig,
		ProposerSettings:           proposerSettings,
		BeaconApiTimeout:           time.Second * 30,
		BeaconApiEndpoint:          c.cliCtx.String(flags.BeaconRESTApiProviderFlag.Name),
		ValidatorsRegBatchSize:     c.cliCtx.Int(flags.ValidatorsRegistrationBatchSizeFlag.Name),
	})
	if err != nil {
		return errors.Wrap(err, "could not initialize validator service")
	}

	return c.services.RegisterService(validatorService)
}

func Web3SignerConfig(cliCtx *cli.Context) (*remoteweb3signer.SetupConfig, error) {
	var web3signerConfig *remoteweb3signer.SetupConfig
	if cliCtx.IsSet(flags.Web3SignerURLFlag.Name) {
		urlStr := cliCtx.String(flags.Web3SignerURLFlag.Name)
		u, err := url.ParseRequestURI(urlStr)
		if err != nil {
			return nil, errors.Wrapf(err, "web3signer url %s is invalid", urlStr)
		}
		if u.Scheme == "" || u.Host == "" {
			return nil, fmt.Errorf("web3signer url must be in the format of http(s)://host:port url used: %v", urlStr)
		}
		web3signerConfig = &remoteweb3signer.SetupConfig{
			BaseEndpoint:          u.String(),
			GenesisValidatorsRoot: nil,
		}
		if cliCtx.IsSet(flags.WalletPasswordFileFlag.Name) {
			log.Warnf("%s was provided while using web3signer and will be ignored", flags.WalletPasswordFileFlag.Name)
		}

		if publicKeysSlice := cliCtx.StringSlice(flags.Web3SignerPublicValidatorKeysFlag.Name); len(publicKeysSlice) > 0 {
			pks := make([]string, 0)
			if len(publicKeysSlice) == 1 {
				pURL, err := url.ParseRequestURI(publicKeysSlice[0])
				if err == nil && pURL.Scheme != "" && pURL.Host != "" {
					web3signerConfig.PublicKeysURL = publicKeysSlice[0]
				} else {
					pks = strings.Split(publicKeysSlice[0], ",")
				}
			} else if len(publicKeysSlice) > 1 {
				pks = publicKeysSlice
			}
			if len(pks) > 0 {
				pks = slice.Unique[string](pks)
				var validatorKeys [][48]byte
				for _, key := range pks {
					decodedKey, decodeErr := hexutil.Decode(key)
					if decodeErr != nil {
						return nil, errors.Wrapf(decodeErr, "could not decode public key for web3signer: %s", key)
					}
					validatorKeys = append(validatorKeys, bytesutil.ToBytes48(decodedKey))
				}
				web3signerConfig.ProvidedPublicKeys = validatorKeys
			}
		}
	}
	return web3signerConfig, nil
}

func proposerSettings(cliCtx *cli.Context, db iface.ValidatorDB) (*validatorServiceConfig.ProposerSettings, error) {
	var fileConfig *validatorpb.ProposerSettingsPayload

	if cliCtx.IsSet(flags.ProposerSettingsFlag.Name) && cliCtx.IsSet(flags.ProposerSettingsURLFlag.Name) {
		return nil, errors.New("cannot specify both " + flags.ProposerSettingsFlag.Name + " and " + flags.ProposerSettingsURLFlag.Name)
	}
	builderConfigFromFlag, err := BuilderSettingsFromFlags(cliCtx)
	if err != nil {
		return nil, err
	}
	// is overridden by file and URL flags
	if cliCtx.IsSet(flags.SuggestedFeeRecipientFlag.Name) &&
		!cliCtx.IsSet(flags.ProposerSettingsFlag.Name) &&
		!cliCtx.IsSet(flags.ProposerSettingsURLFlag.Name) {
		suggestedFee := cliCtx.String(flags.SuggestedFeeRecipientFlag.Name)
		fileConfig = &validatorpb.ProposerSettingsPayload{
			ProposerConfig: nil,
			DefaultConfig: &validatorpb.ProposerOptionPayload{
				FeeRecipient: suggestedFee,
				Builder:      builderConfigFromFlag.ToPayload(),
			},
		}
	}

	if cliCtx.IsSet(flags.ProposerSettingsFlag.Name) {
		if err := unmarshalFromFile(cliCtx.Context, cliCtx.String(flags.ProposerSettingsFlag.Name), &fileConfig); err != nil {
			return nil, err
		}
	}
	if cliCtx.IsSet(flags.ProposerSettingsURLFlag.Name) {
		if err := unmarshalFromURL(cliCtx.Context, cliCtx.String(flags.ProposerSettingsURLFlag.Name), &fileConfig); err != nil {
			return nil, err
		}
	}

	// this condition triggers if SuggestedFeeRecipientFlag,ProposerSettingsFlag or ProposerSettingsURLFlag did not create any settings
	if fileConfig == nil {
		// Checks the db or enable builder settings before starting the node without proposer settings
		// starting the node without proposer settings, will skip API calls for push proposer settings and register validator
		return handleNoProposerSettingsFlagsProvided(cliCtx, db, builderConfigFromFlag)
	}

	// convert file config to proposer config for internal use
	vpSettings := &validatorServiceConfig.ProposerSettings{}

	// default fileConfig is mandatory
	if fileConfig.DefaultConfig == nil {
		return nil, errors.New("default fileConfig is required, proposer settings file is either empty or an incorrect format")
	}
	if !common.IsHexAddress(fileConfig.DefaultConfig.FeeRecipient) {
		return nil, errors.New("default fileConfig fee recipient is not a valid eth1 address")
	}
	psExists, err := db.ProposerSettingsExists(cliCtx.Context)
	if err != nil {
		return nil, err
	}
	if err := warnNonChecksummedAddress(fileConfig.DefaultConfig.FeeRecipient); err != nil {
		return nil, err
	}
	vpSettings.DefaultConfig = &validatorServiceConfig.ProposerOption{
		FeeRecipientConfig: &validatorServiceConfig.FeeRecipientConfig{
			FeeRecipient: common.HexToAddress(fileConfig.DefaultConfig.FeeRecipient),
		},
		BuilderConfig: validatorServiceConfig.ToBuilderConfig(fileConfig.DefaultConfig.Builder),
	}

	if builderConfigFromFlag != nil {
		config := builderConfigFromFlag.Clone()
		if config.GasLimit == validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit) && vpSettings.DefaultConfig.BuilderConfig != nil {
			config.GasLimit = vpSettings.DefaultConfig.BuilderConfig.GasLimit
		}
		vpSettings.DefaultConfig.BuilderConfig = config
	} else if vpSettings.DefaultConfig.BuilderConfig != nil {
		vpSettings.DefaultConfig.BuilderConfig.GasLimit = reviewGasLimit(vpSettings.DefaultConfig.BuilderConfig.GasLimit)
	}

	if psExists {
		// if settings exist update the default
		if err := db.UpdateProposerSettingsDefault(cliCtx.Context, vpSettings.DefaultConfig); err != nil {
			return nil, err
		}
	}

	if fileConfig.ProposerConfig != nil && len(fileConfig.ProposerConfig) != 0 {
		vpSettings.ProposeConfig = make(map[[fieldparams.BLSPubkeyLength]byte]*validatorServiceConfig.ProposerOption)
		for key, option := range fileConfig.ProposerConfig {
			decodedKey, err := hexutil.Decode(key)
			if err != nil {
				return nil, errors.Wrapf(err, "could not decode public key %s", key)
			}
			if len(decodedKey) != fieldparams.BLSPubkeyLength {
				return nil, fmt.Errorf("%v  is not a bls public key", key)
			}
			if err := verifyOption(key, option); err != nil {
				return nil, err
			}
			currentBuilderConfig := validatorServiceConfig.ToBuilderConfig(option.Builder)
			if builderConfigFromFlag != nil {
				config := builderConfigFromFlag.Clone()
				if config.GasLimit == validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit) && currentBuilderConfig != nil {
					config.GasLimit = currentBuilderConfig.GasLimit
				}
				currentBuilderConfig = config
			} else if currentBuilderConfig != nil {
				currentBuilderConfig.GasLimit = reviewGasLimit(currentBuilderConfig.GasLimit)
			}
			o := &validatorServiceConfig.ProposerOption{
				FeeRecipientConfig: &validatorServiceConfig.FeeRecipientConfig{
					FeeRecipient: common.HexToAddress(option.FeeRecipient),
				},
				BuilderConfig: currentBuilderConfig,
			}
			pubkeyB := bytesutil.ToBytes48(decodedKey)
			vpSettings.ProposeConfig[pubkeyB] = o
		}
		if psExists {
			// override the existing saved settings if providing values via fileConfig.ProposerConfig
			if err := db.SaveProposerSettings(cliCtx.Context, vpSettings); err != nil {
				return nil, err
			}
		}
	}
	if !psExists {
		// if no proposer settings ever existed in the db just save the settings
		if err := db.SaveProposerSettings(cliCtx.Context, vpSettings); err != nil {
			return nil, err
		}
	}
	return vpSettings, nil
}

func verifyOption(key string, option *validatorpb.ProposerOptionPayload) error {
	if option == nil {
		return fmt.Errorf("fee recipient is required for proposer %s", key)
	}
	if !common.IsHexAddress(option.FeeRecipient) {
		return errors.New("fee recipient is not a valid eth1 address")
	}
	if err := warnNonChecksummedAddress(option.FeeRecipient); err != nil {
		return err
	}
	return nil
}

func handleNoProposerSettingsFlagsProvided(cliCtx *cli.Context,
	db iface.ValidatorDB,
	builderConfigFromFlag *validatorServiceConfig.BuilderConfig) (*validatorServiceConfig.ProposerSettings, error) {
	log.Info("no proposer settings files have been provided, attempting to load from db.")
	// checks db if proposer settings exist if none is provided.
	settings, err := db.ProposerSettings(cliCtx.Context)
	if err == nil {
		// process any overrides to builder settings
		overrideBuilderSettings(settings, builderConfigFromFlag)
		// if settings are empty
		log.Info("successfully loaded proposer settings from db.")
		return settings, nil
	} else {
		log.WithError(err).Warn("no proposer settings will be loaded from the db")
	}

	if cliCtx.Bool(flags.EnableBuilderFlag.Name) {
		// if there are no proposer settings provided, create a default where fee recipient is not populated, this will be skipped for validator registration on validators that don't have a fee recipient set.
		// skip saving to DB if only builder settings are provided until a trigger like keymanager API updates with fee recipient values
		return &validatorServiceConfig.ProposerSettings{
			DefaultConfig: &validatorServiceConfig.ProposerOption{
				BuilderConfig: builderConfigFromFlag,
			},
		}, nil
	}
	return nil, nil
}

func overrideBuilderSettings(settings *validatorServiceConfig.ProposerSettings, builderConfigFromFlag *validatorServiceConfig.BuilderConfig) {
	// override the db settings with the results based on whether the --enable-builder flag is provided.
	if builderConfigFromFlag == nil {
		log.Infof("proposer settings loaded from db. validator registration to builder is not enabled, please use the --%s flag if you wish to use a builder.", flags.EnableBuilderFlag.Name)
	}
	if settings.ProposeConfig != nil {
		for key := range settings.ProposeConfig {
			settings.ProposeConfig[key].BuilderConfig = builderConfigFromFlag
		}
	}
	if settings.DefaultConfig != nil {
		settings.DefaultConfig.BuilderConfig = builderConfigFromFlag
	}
}

func BuilderSettingsFromFlags(cliCtx *cli.Context) (*validatorServiceConfig.BuilderConfig, error) {
	if cliCtx.Bool(flags.EnableBuilderFlag.Name) {
		gasLimit := validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit)
		sgl := cliCtx.String(flags.BuilderGasLimitFlag.Name)

		if sgl != "" {
			gl, err := strconv.ParseUint(sgl, 10, 64)
			if err != nil {
				return nil, errors.New("Gas Limit is not a uint64")
			}
			gasLimit = reviewGasLimit(validator.Uint64(gl))
		}
		return &validatorServiceConfig.BuilderConfig{
			Enabled:  true,
			GasLimit: gasLimit,
		}, nil
	}
	return nil, nil
}

func warnNonChecksummedAddress(feeRecipient string) error {
	mixedcaseAddress, err := common.NewMixedcaseAddressFromString(feeRecipient)
	if err != nil {
		return errors.Wrapf(err, "could not decode fee recipient %s", feeRecipient)
	}
	if !mixedcaseAddress.ValidChecksum() {
		log.Warnf("Fee recipient %s is not a checksum Ethereum address. "+
			"The checksummed address is %s and will be used as the fee recipient. "+
			"We recommend using a mixed-case address (checksum) "+
			"to prevent spelling mistakes in your fee recipient Ethereum address", feeRecipient, mixedcaseAddress.Address().Hex())
	}
	return nil
}

func reviewGasLimit(gasLimit validator.Uint64) validator.Uint64 {
	// sets gas limit to default if not defined or set to 0
	if gasLimit == 0 {
		return validator.Uint64(params.BeaconConfig().DefaultBuilderGasLimit)
	}
	// TODO(10810): add in warning for ranges
	return gasLimit
}

func (c *ValidatorClient) registerRPCService(router *mux.Router) error {
	var vs *client.ValidatorService
	if err := c.services.FetchService(&vs); err != nil {
		return err
	}
	validatorGatewayHost := c.cliCtx.String(flags.GRPCGatewayHost.Name)
	validatorGatewayPort := c.cliCtx.Int(flags.GRPCGatewayPort.Name)
	validatorMonitoringHost := c.cliCtx.String(cmd.MonitoringHostFlag.Name)
	validatorMonitoringPort := c.cliCtx.Int(flags.MonitoringPortFlag.Name)
	rpcHost := c.cliCtx.String(flags.RPCHost.Name)
	rpcPort := c.cliCtx.Int(flags.RPCPort.Name)
	nodeGatewayEndpoint := c.cliCtx.String(flags.BeaconRPCGatewayProviderFlag.Name)
	beaconClientEndpoint := c.cliCtx.String(flags.BeaconRPCProviderFlag.Name)
	maxCallRecvMsgSize := c.cliCtx.Int(cmd.GrpcMaxCallRecvMsgSizeFlag.Name)
	grpcRetries := c.cliCtx.Uint(flags.GrpcRetriesFlag.Name)
	grpcRetryDelay := c.cliCtx.Duration(flags.GrpcRetryDelayFlag.Name)
	walletDir := c.cliCtx.String(flags.WalletDirFlag.Name)
	grpcHeaders := c.cliCtx.String(flags.GrpcHeadersFlag.Name)
	clientCert := c.cliCtx.String(flags.CertFlag.Name)
	server := rpc.NewServer(c.cliCtx.Context, &rpc.Config{
		ValDB:                    c.db,
		Host:                     rpcHost,
		Port:                     fmt.Sprintf("%d", rpcPort),
		WalletInitializedFeed:    c.walletInitialized,
		ValidatorService:         vs,
		SyncChecker:              vs,
		GenesisFetcher:           vs,
		NodeGatewayEndpoint:      nodeGatewayEndpoint,
		WalletDir:                walletDir,
		Wallet:                   c.wallet,
		ValidatorGatewayHost:     validatorGatewayHost,
		ValidatorGatewayPort:     validatorGatewayPort,
		ValidatorMonitoringHost:  validatorMonitoringHost,
		ValidatorMonitoringPort:  validatorMonitoringPort,
		BeaconClientEndpoint:     beaconClientEndpoint,
		ClientMaxCallRecvMsgSize: maxCallRecvMsgSize,
		ClientGrpcRetries:        grpcRetries,
		ClientGrpcRetryDelay:     grpcRetryDelay,
		ClientGrpcHeaders:        strings.Split(grpcHeaders, ","),
		ClientWithCert:           clientCert,
		Router:                   router,
	})
	return c.services.RegisterService(server)
}

func (c *ValidatorClient) registerRPCGatewayService(router *mux.Router) error {
	gatewayHost := c.cliCtx.String(flags.GRPCGatewayHost.Name)
	if gatewayHost != flags.DefaultGatewayHost {
		log.WithField("web-host", gatewayHost).Warn(
			"You are using a non-default web host. Web traffic is served by HTTP, so be wary of " +
				"changing this parameter if you are exposing this host to the Internet!",
		)
	}
	gatewayPort := c.cliCtx.Int(flags.GRPCGatewayPort.Name)
	rpcHost := c.cliCtx.String(flags.RPCHost.Name)
	rpcPort := c.cliCtx.Int(flags.RPCPort.Name)
	rpcAddr := net.JoinHostPort(rpcHost, fmt.Sprintf("%d", rpcPort))
	gatewayAddress := net.JoinHostPort(gatewayHost, fmt.Sprintf("%d", gatewayPort))
	timeout := c.cliCtx.Int(cmd.ApiTimeoutFlag.Name)
	var allowedOrigins []string
	if c.cliCtx.IsSet(flags.GPRCGatewayCorsDomain.Name) {
		allowedOrigins = strings.Split(c.cliCtx.String(flags.GPRCGatewayCorsDomain.Name), ",")
	} else {
		allowedOrigins = strings.Split(flags.GPRCGatewayCorsDomain.Value, ",")
	}
	maxCallSize := c.cliCtx.Uint64(cmd.GrpcMaxCallRecvMsgSizeFlag.Name)

	registrations := []gateway.PbHandlerRegistration{
		pb.RegisterHealthHandler,
	}
	gwmux := gwruntime.NewServeMux(
		gwruntime.WithMarshalerOption(gwruntime.MIMEWildcard, &gwruntime.HTTPBodyMarshaler{
			Marshaler: &gwruntime.JSONPb{
				MarshalOptions: protojson.MarshalOptions{
					EmitUnpopulated: true,
					UseProtoNames:   true,
				},
				UnmarshalOptions: protojson.UnmarshalOptions{
					DiscardUnknown: true,
				},
			},
		}),
		gwruntime.WithMarshalerOption(
			api.EventStreamMediaType, &gwruntime.EventSourceJSONPb{}, // TODO: remove this
		),
		gwruntime.WithForwardResponseOption(gateway.HttpResponseModifier),
	)

	muxHandler := func(h http.HandlerFunc, w http.ResponseWriter, req *http.Request) {
		// The validator gateway handler requires this special logic as it serves the web APIs and the web UI.
		if strings.HasPrefix(req.URL.Path, "/api") {
			req.URL.Path = strings.Replace(req.URL.Path, "/api", "", 1)
			// Else, we handle with the Prysm API gateway without a middleware.
			h(w, req)
		} else {
			// Finally, we handle with the web server.
			web.Handler(w, req)
		}
	}

	pbHandler := &gateway.PbMux{
		Registrations: registrations,
		Mux:           gwmux,
	}
	opts := []gateway.Option{
		gateway.WithMuxHandler(muxHandler),
		gateway.WithRouter(router), // note some routes are registered in server.go
		gateway.WithRemoteAddr(rpcAddr),
		gateway.WithGatewayAddr(gatewayAddress),
		gateway.WithMaxCallRecvMsgSize(maxCallSize),
		gateway.WithPbHandlers([]*gateway.PbMux{pbHandler}),
		gateway.WithAllowedOrigins(allowedOrigins),
		gateway.WithTimeout(uint64(timeout)),
	}
	gw, err := gateway.New(c.cliCtx.Context, opts...)
	if err != nil {
		return err
	}
	return c.services.RegisterService(gw)
}

func setWalletPasswordFilePath(cliCtx *cli.Context) error {
	walletDir := cliCtx.String(flags.WalletDirFlag.Name)
	defaultWalletPasswordFilePath := filepath.Join(walletDir, wallet.DefaultWalletPasswordFile)
	if file.Exists(defaultWalletPasswordFilePath) {
		// Ensure file has proper permissions.
		hasPerms, err := file.HasReadWritePermissions(defaultWalletPasswordFilePath)
		if err != nil {
			return err
		}
		if !hasPerms {
			return fmt.Errorf(
				"wallet password file %s does not have proper 0600 permissions",
				defaultWalletPasswordFilePath,
			)
		}

		// Set the filepath into the cli context.
		if err := cliCtx.Set(flags.WalletPasswordFileFlag.Name, defaultWalletPasswordFilePath); err != nil {
			return errors.Wrap(err, "could not set default wallet password file path")
		}
	}
	return nil
}

func clearDB(ctx context.Context, dataDir string, force bool) error {
	var err error
	clearDBConfirmed := force

	if !force {
		actionText := "This will delete your validator's historical actions database stored in your data directory. " +
			"This may lead to potential slashing - do you want to proceed? (Y/N)"
		deniedText := "The historical actions database will not be deleted. No changes have been made."
		clearDBConfirmed, err = cmd.ConfirmAction(actionText, deniedText)
		if err != nil {
			return errors.Wrapf(err, "Could not clear DB in dir %s", dataDir)
		}
	}

	if clearDBConfirmed {
		valDB, err := kv.NewKVStore(ctx, dataDir, &kv.Config{})
		if err != nil {
			return errors.Wrapf(err, "Could not create DB in dir %s", dataDir)
		}
		if err := valDB.Close(); err != nil {
			return errors.Wrapf(err, "could not close DB in dir %s", dataDir)
		}

		log.Warning("Removing database")
		if err := valDB.ClearDB(); err != nil {
			return errors.Wrapf(err, "Could not clear DB in dir %s", dataDir)
		}
	}

	return nil
}

func unmarshalFromURL(ctx context.Context, from string, to interface{}) error {
	u, err := url.ParseRequestURI(from)
	if err != nil {
		return err
	}
	if u.Scheme == "" || u.Host == "" {
		return fmt.Errorf("invalid URL: %s", from)
	}
	req, reqerr := http.NewRequestWithContext(ctx, http.MethodGet, from, nil)
	if reqerr != nil {
		return errors.Wrap(reqerr, "failed to create http request")
	}
	req.Header.Set("Content-Type", "application/json")
	resp, resperr := http.DefaultClient.Do(req)
	if resperr != nil {
		return errors.Wrap(resperr, "failed to send http request")
	}
	defer func(Body io.ReadCloser) {
		err = Body.Close()
		if err != nil {
			log.WithError(err).Error("failed to close response body")
		}
	}(resp.Body)
	if resp.StatusCode != http.StatusOK {
		return errors.Errorf("http request to %v failed with status code %d", from, resp.StatusCode)
	}
	if decodeerr := json.NewDecoder(resp.Body).Decode(&to); decodeerr != nil {
		return errors.Wrap(decodeerr, "failed to decode http response")
	}
	return nil
}

func unmarshalFromFile(ctx context.Context, from string, to interface{}) error {
	if ctx == nil {
		return errors.New("node: nil context passed to unmarshalFromFile")
	}
	cleanpath := filepath.Clean(from)
	b, err := os.ReadFile(cleanpath)
	if err != nil {
		return errors.Wrap(err, "failed to open file")
	}

	if err := yaml.Unmarshal(b, to); err != nil {
		return errors.Wrap(err, "failed to unmarshal yaml file")
	}

	return nil
}

func configureFastSSZHashingAlgorithm() {
	fastssz.EnableVectorizedHTR = true
}
