package validate

import (
	"context"

	appmodulev2 "cosmossdk.io/core/appmodule/v2"
	"cosmossdk.io/core/transaction"

	"github.com/cosmos/cosmos-sdk/x/auth/ante"
)

var (
	_ appmodulev2.AppModule                      = AppModule{}
	_ appmodulev2.HasTxValidator[transaction.Tx] = AppModule{}
)

// AppModule is a module that only implements tx validators.
// The goal of this module is to allow extensible registration of tx validators provided by chains without requiring a new modules.
// Additionally, it registers tx validators that do not really have a place in other modules.
// This module is only useful for chains using server/v2. Ante/Post handlers are setup via baseapp options in depinject.
type AppModule struct {
	sigVerification    ante.SigVerificationDecorator
	feeTxValidator     *ante.DeductFeeDecorator
	unorderTxValidator *ante.UnorderedTxDecorator
	// txValidators contains tx validator that can be injected into the module via depinject.
	// tx validators should be module based, but it can happen that you do not want to create a new module
	// and simply depinject-it.
	txValidators []appmodulev2.TxValidator[transaction.Tx]
}

// NewAppModule creates a new AppModule object.
func NewAppModule(
	sigVerification ante.SigVerificationDecorator,
	feeTxValidator *ante.DeductFeeDecorator,
	unorderTxValidator *ante.UnorderedTxDecorator,
	txValidators ...appmodulev2.TxValidator[transaction.Tx],
) AppModule {
	return AppModule{
		sigVerification:    sigVerification,
		feeTxValidator:     feeTxValidator,
		unorderTxValidator: unorderTxValidator,
		txValidators:       txValidators,
	}
}

// IsAppModule implements appmodule.AppModule.
func (a AppModule) IsAppModule() {}

// IsOnePerModuleType implements appmodule.AppModule.
func (a AppModule) IsOnePerModuleType() {}

// TxValidator implements appmodule.HasTxValidator.
func (a AppModule) TxValidator(ctx context.Context, tx transaction.Tx) error {
	for _, txValidator := range a.txValidators {
		if err := txValidator.ValidateTx(ctx, tx); err != nil {
			return err
		}
	}

	if err := a.feeTxValidator.ValidateTx(ctx, tx); err != nil {
		return err
	}

	if a.unorderTxValidator != nil {
		if err := a.unorderTxValidator.ValidateTx(ctx, tx); err != nil {
			return err
		}
	}

	return a.sigVerification.ValidateTx(ctx, tx)
}
