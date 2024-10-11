package validate

import (
	"fmt"

	modulev1 "cosmossdk.io/api/cosmos/validate/module/v1"
	appmodulev2 "cosmossdk.io/core/appmodule/v2"
	"cosmossdk.io/core/server"
	"cosmossdk.io/core/transaction"
	"cosmossdk.io/depinject"
	"cosmossdk.io/depinject/appconfig"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/ante"
	"github.com/cosmos/cosmos-sdk/x/auth/ante/unorderedtx"
	"github.com/cosmos/cosmos-sdk/x/auth/posthandler"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

// flagMinGasPricesV2 is the flag name for the minimum gas prices in the main server v2 component.
const flagMinGasPricesV2 = "server.minimum-gas-prices"

func init() {
	appconfig.RegisterModule(&modulev1.Module{},
		appconfig.Provide(ProvideModule),
	)
}

type ModuleInputs struct {
	depinject.In

	ModuleConfig  *modulev1.Module
	Environment   appmodulev2.Environment
	TxConfig      client.TxConfig
	DynamicConfig server.DynamicConfig `optional:"true"`

	AccountKeeper            ante.AccountKeeper
	BankKeeper               authtypes.BankKeeper
	ConsensusKeeper          ante.ConsensusKeeper
	FeeGrantKeeper           ante.FeegrantKeeper                       `optional:"true"`
	AccountAbstractionKeeper ante.AccountAbstractionKeeper             `optional:"true"`
	ExtraTxValidators        []appmodulev2.TxValidator[transaction.Tx] `optional:"true"`
	UnorderedTxManager       *unorderedtx.Manager                      `optional:"true"`
	TxFeeChecker             ante.TxFeeChecker                         `optional:"true"`
}

type ModuleOutputs struct {
	depinject.Out

	Module        appmodulev2.AppModule // Only useful for chains using server/v2. It setup tx validators that don't belong to other modules.
	BaseAppOption runtime.BaseAppOption // Only useful for chains using baseapp. Server/v2 chains use TxValidator.
}

func ProvideModule(in ModuleInputs) ModuleOutputs {
	svd := ante.NewSigVerificationDecorator(
		in.AccountKeeper,
		in.TxConfig.SignModeHandler(),
		ante.DefaultSigVerificationGasConsumer,
		in.AccountAbstractionKeeper, // can be nil
	)

	var (
		err                  error
		minGasPrices         sdk.DecCoins
		feeTxValidator       *ante.DeductFeeDecorator
		unorderedTxValidator *ante.UnorderedTxDecorator
	)

	if in.DynamicConfig != nil {
		minGasPricesStr := in.DynamicConfig.GetString(flagMinGasPricesV2)
		minGasPrices, err = sdk.ParseDecCoins(minGasPricesStr)
		if err != nil {
			panic(fmt.Sprintf("invalid minimum gas prices: %v", err))
		}

		feeTxValidator = ante.NewDeductFeeDecorator(in.AccountKeeper, in.BankKeeper, in.FeeGrantKeeper, in.TxFeeChecker)
		feeTxValidator.SetMinGasPrices(minGasPrices) // set min gas price in deduct fee decorator
	}

	if in.UnorderedTxManager != nil {
		unorderedTxValidator = ante.NewUnorderedTxDecorator(unorderedtx.DefaultMaxTimeoutDuration, in.UnorderedTxManager, in.Environment, ante.DefaultSha256Cost)
	}

	return ModuleOutputs{
		Module:        NewAppModule(svd, feeTxValidator, unorderedTxValidator, in.ExtraTxValidators...),
		BaseAppOption: newBaseAppOption(in),
	}
}

// newBaseAppOption returns baseapp option that sets the ante handler and post handler
// and set the tx encoder and decoder on baseapp.
func newBaseAppOption(in ModuleInputs) func(app *baseapp.BaseApp) {
	return func(app *baseapp.BaseApp) {
		anteHandler, err := newAnteHandler(in)
		if err != nil {
			panic(err)
		}
		app.SetAnteHandler(anteHandler)

		// PostHandlers
		// In v0.46, the SDK introduces _postHandlers_. PostHandlers are like
		// antehandlers, but are run _after_ the `runMsgs` execution. They are also
		// defined as a chain, and have the same signature as antehandlers.
		//
		// In baseapp, postHandlers are run in the same store branch as `runMsgs`,
		// meaning that both `runMsgs` and `postHandler` state will be committed if
		// both are successful, and both will be reverted if any of the two fails.
		//
		// The SDK exposes a default empty postHandlers chain.
		//
		// Please note that changing any of the anteHandler or postHandler chain is
		// likely to be a state-machine breaking change, which needs a coordinated
		// upgrade.
		postHandler, err := posthandler.NewPostHandler(
			posthandler.HandlerOptions{},
		)
		if err != nil {
			panic(err)
		}
		app.SetPostHandler(postHandler)
	}
}

func newAnteHandler(in ModuleInputs) (sdk.AnteHandler, error) {
	anteHandler, err := ante.NewAnteHandler(
		ante.HandlerOptions{
			Environment:        in.Environment,
			AccountKeeper:      in.AccountKeeper,
			ConsensusKeeper:    in.ConsensusKeeper,
			BankKeeper:         in.BankKeeper,
			SignModeHandler:    in.TxConfig.SignModeHandler(),
			FeegrantKeeper:     in.FeeGrantKeeper,
			SigGasConsumer:     ante.DefaultSigVerificationGasConsumer,
			UnorderedTxManager: in.UnorderedTxManager,
		},
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create ante handler: %w", err)
	}

	return anteHandler, nil
}
