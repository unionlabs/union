package ante

import (
	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/gas"
	errorsmod "cosmossdk.io/errors"
	txsigning "cosmossdk.io/x/tx/signing"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/ante/unorderedtx"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

// HandlerOptions are the options required for constructing a default SDK AnteHandler.
type HandlerOptions struct {
	Environment              appmodule.Environment
	AccountKeeper            AccountKeeper
	AccountAbstractionKeeper AccountAbstractionKeeper
	BankKeeper               types.BankKeeper
	ConsensusKeeper          ConsensusKeeper
	ExtensionOptionChecker   ExtensionOptionChecker
	FeegrantKeeper           FeegrantKeeper
	SignModeHandler          *txsigning.HandlerMap
	SigGasConsumer           func(meter gas.Meter, sig signing.SignatureV2, params types.Params) error
	TxFeeChecker             TxFeeChecker
	UnorderedTxManager       *unorderedtx.Manager
}

// NewAnteHandler returns an AnteHandler that checks and increments sequence
// numbers, checks signatures & account numbers, and deducts fees from the first
// signer.
func NewAnteHandler(options HandlerOptions) (sdk.AnteHandler, error) {
	if options.AccountKeeper == nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrLogic, "account keeper is required for ante builder")
	}

	if options.BankKeeper == nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrLogic, "bank keeper is required for ante builder")
	}

	if options.SignModeHandler == nil {
		return nil, errorsmod.Wrap(sdkerrors.ErrLogic, "sign mode handler is required for ante builder")
	}

	anteDecorators := []sdk.AnteDecorator{
		NewSetUpContextDecorator(options.Environment, options.ConsensusKeeper), // outermost AnteDecorator. SetUpContext must be called first
		NewExtensionOptionsDecorator(options.ExtensionOptionChecker),
		NewValidateBasicDecorator(options.Environment),
		NewTxTimeoutHeightDecorator(options.Environment),
		NewValidateMemoDecorator(options.AccountKeeper),
		NewConsumeGasForTxSizeDecorator(options.AccountKeeper),
		NewDeductFeeDecorator(options.AccountKeeper, options.BankKeeper, options.FeegrantKeeper, options.TxFeeChecker),
		NewValidateSigCountDecorator(options.AccountKeeper),
		NewSigVerificationDecorator(options.AccountKeeper, options.SignModeHandler, options.SigGasConsumer, options.AccountAbstractionKeeper),
	}

	if options.UnorderedTxManager != nil {
		anteDecorators = append(anteDecorators, NewUnorderedTxDecorator(unorderedtx.DefaultMaxTimeoutDuration, options.UnorderedTxManager, options.Environment, DefaultSha256Cost))
	}

	return sdk.ChainAnteDecorators(anteDecorators...), nil
}
