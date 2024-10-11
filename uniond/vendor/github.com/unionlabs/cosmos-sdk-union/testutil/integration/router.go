package integration

import (
	"context"
	"fmt"

	cmtabcitypes "github.com/cometbft/cometbft/api/cometbft/abci/v1"
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	cmttypes "github.com/cometbft/cometbft/types"

	"cosmossdk.io/collections"
	"cosmossdk.io/core/address"
	"cosmossdk.io/core/appmodule"
	corestore "cosmossdk.io/core/store"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/log"
	"cosmossdk.io/store"
	"cosmossdk.io/store/metrics"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module"
	authtx "github.com/cosmos/cosmos-sdk/x/auth/tx"
)

const (
	appName   = "integration-app"
	consensus = "consensus"
)

// App is a test application that can be used to test the integration of modules.
type App struct {
	*baseapp.BaseApp

	ctx           sdk.Context
	logger        log.Logger
	moduleManager module.Manager
	queryHelper   *baseapp.QueryServiceTestHelper
}

// NewIntegrationApp creates an application for testing purposes. This application
// is able to route messages to their respective handlers.
func NewIntegrationApp(
	sdkCtx sdk.Context,
	logger log.Logger,
	keys map[string]*storetypes.KVStoreKey,
	appCodec codec.Codec,
	addressCodec address.Codec,
	validatorCodec address.Codec,
	modules map[string]appmodule.AppModule,
	msgRouter *baseapp.MsgServiceRouter,
	grpcRouter *baseapp.GRPCQueryRouter,
	baseAppOptions ...func(*baseapp.BaseApp),
) *App {
	db := coretesting.NewMemDB()

	interfaceRegistry := codectypes.NewInterfaceRegistry()
	moduleManager := module.NewManagerFromMap(modules)
	moduleManager.RegisterInterfaces(interfaceRegistry)

	txConfig := authtx.NewTxConfig(codec.NewProtoCodec(interfaceRegistry), addressCodec, validatorCodec, authtx.DefaultSignModes)
	bApp := baseapp.NewBaseApp(appName, logger, db, txConfig.TxDecoder(), append(baseAppOptions, baseapp.SetChainID(appName))...)
	bApp.MountKVStores(keys)

	bApp.SetInitChainer(func(_ sdk.Context, _ *cmtabcitypes.InitChainRequest) (*cmtabcitypes.InitChainResponse, error) {
		for _, mod := range modules {
			if m, ok := mod.(module.HasGenesis); ok {
				if err := m.InitGenesis(sdkCtx, m.DefaultGenesis()); err != nil {
					return nil, err
				}
			}
		}

		return &cmtabcitypes.InitChainResponse{}, nil
	})

	bApp.SetBeginBlocker(func(_ sdk.Context) (sdk.BeginBlock, error) {
		return moduleManager.BeginBlock(sdkCtx)
	})
	bApp.SetEndBlocker(func(_ sdk.Context) (sdk.EndBlock, error) {
		return moduleManager.EndBlock(sdkCtx)
	})

	msgRouter.SetInterfaceRegistry(interfaceRegistry)
	bApp.SetMsgServiceRouter(msgRouter)
	grpcRouter.SetInterfaceRegistry(interfaceRegistry)
	bApp.SetGRPCQueryRouter(grpcRouter)

	if keys[consensus] != nil {
		cps := newParamStore(runtime.NewKVStoreService(keys[consensus]), appCodec)
		bApp.SetParamStore(cps)

		params := cmttypes.ConsensusParamsFromProto(*simtestutil.DefaultConsensusParams) // This fills up missing param sections
		err := cps.Set(sdkCtx, params.ToProto())
		if err != nil {
			panic(fmt.Errorf("failed to set consensus params: %w", err))
		}

		if err := bApp.LoadLatestVersion(); err != nil {
			panic(fmt.Errorf("failed to load application version from store: %w", err))
		}

		if _, err := bApp.InitChain(&cmtabcitypes.InitChainRequest{ChainId: appName, ConsensusParams: simtestutil.DefaultConsensusParams}); err != nil {
			panic(fmt.Errorf("failed to initialize application: %w", err))
		}
	} else {
		if err := bApp.LoadLatestVersion(); err != nil {
			panic(fmt.Errorf("failed to load application version from store: %w", err))
		}

		if _, err := bApp.InitChain(&cmtabcitypes.InitChainRequest{ChainId: appName}); err != nil {
			panic(fmt.Errorf("failed to initialize application: %w", err))
		}
	}

	_, err := bApp.Commit()
	if err != nil {
		panic(err)
	}

	ctx := sdkCtx.WithBlockHeader(cmtproto.Header{ChainID: appName}).WithIsCheckTx(true)

	return &App{
		BaseApp:       bApp,
		logger:        logger,
		ctx:           ctx,
		moduleManager: *moduleManager,
		queryHelper:   baseapp.NewQueryServerTestHelper(ctx, interfaceRegistry),
	}
}

// RunMsg provides the ability to run a message and return the response.
// In order to run a message, the application must have a handler for it.
// These handlers are registered on the application message service router.
// The result of the message execution is returned as an Any type.
// That any type can be unmarshaled to the expected response type.
// If the message execution fails, an error is returned.
func (app *App) RunMsg(msg sdk.Msg, option ...Option) (*codectypes.Any, error) {
	// set options
	cfg := &Config{}
	for _, opt := range option {
		opt(cfg)
	}

	if cfg.AutomaticCommit {
		defer func() {
			_, err := app.Commit()
			if err != nil {
				panic(err)
			}
		}()
	}

	if cfg.AutomaticFinalizeBlock {
		height := app.LastBlockHeight() + 1
		if _, err := app.FinalizeBlock(&cmtabcitypes.FinalizeBlockRequest{Height: height, DecidedLastCommit: cmtabcitypes.CommitInfo{Votes: []cmtabcitypes.VoteInfo{{}}}}); err != nil {
			return nil, fmt.Errorf("failed to run finalize block: %w", err)
		}
	}

	app.logger.Info("Running msg", "msg", msg.String())

	handler := app.MsgServiceRouter().Handler(msg)
	if handler == nil {
		return nil, fmt.Errorf("handler is nil, can't route message %s: %+v", sdk.MsgTypeURL(msg), msg)
	}

	msgResult, err := handler(app.ctx, msg)
	if err != nil {
		return nil, fmt.Errorf("failed to execute message %s: %w", sdk.MsgTypeURL(msg), err)
	}

	var response *codectypes.Any
	if len(msgResult.MsgResponses) > 0 {
		msgResponse := msgResult.MsgResponses[0]
		if msgResponse == nil {
			return nil, fmt.Errorf("got nil msg response %s in message result: %s", sdk.MsgTypeURL(msg), msgResult.String())
		}

		response = msgResponse
	}

	return response, nil
}

// Context returns the application context. It can be unwrapped to a sdk.Context,
// with the sdk.UnwrapSDKContext function.
func (app *App) Context() context.Context {
	return app.ctx
}

// QueryHelper returns the application query helper.
// It can be used when registering query services.
func (app *App) QueryHelper() *baseapp.QueryServiceTestHelper {
	return app.queryHelper
}

// CreateMultiStore is a helper for setting up multiple stores for provided modules.
func CreateMultiStore(keys map[string]*storetypes.KVStoreKey, logger log.Logger) storetypes.CommitMultiStore {
	db := coretesting.NewMemDB()
	cms := store.NewCommitMultiStore(db, logger, metrics.NewNoOpMetrics())

	for key := range keys {
		cms.MountStoreWithDB(keys[key], storetypes.StoreTypeIAVL, db)
	}

	_ = cms.LoadLatestVersion()
	return cms
}

type paramStoreService struct {
	ParamsStore collections.Item[cmtproto.ConsensusParams]
}

func newParamStore(storeService corestore.KVStoreService, cdc codec.Codec) paramStoreService {
	sb := collections.NewSchemaBuilder(storeService)
	return paramStoreService{
		ParamsStore: collections.NewItem(sb, collections.NewPrefix("Consensus"), "params", codec.CollValue[cmtproto.ConsensusParams](cdc)),
	}
}

func (pss paramStoreService) Get(ctx context.Context) (cmtproto.ConsensusParams, error) {
	return pss.ParamsStore.Get(ctx)
}

func (pss paramStoreService) Has(ctx context.Context) (bool, error) {
	return pss.ParamsStore.Has(ctx)
}

func (pss paramStoreService) Set(ctx context.Context, cp cmtproto.ConsensusParams) error {
	return pss.ParamsStore.Set(ctx, cp)
}
