package posthandler

import (
	"fmt"

	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

// ValidateBasicDecorator will call tx.ValidateBasic and return any non-nil error.
// If ValidateBasic passes, decorator calls next AnteHandler in chain. Note,
// ValidateBasicDecorator decorator will not get executed on ReCheckTx since it
// is not dependent on application state.
type tipDecorator struct {
	bankKeeper types.BankKeeper
}

// NewTipDecorator returns a new decorator for handling transactions with
// tips.
//
// IMPORTANT: This decorator is still in beta, please use it at your own risk.
func NewTipDecorator(bankKeeper types.BankKeeper) sdk.AnteDecorator {
	return tipDecorator{
		bankKeeper: bankKeeper,
	}
}

func (d tipDecorator) AnteHandle(ctx sdk.Context, tx sdk.Tx, simulate bool, next sdk.AnteHandler) (sdk.Context, error) {
	err := d.transferTip(ctx, tx)
	if err != nil {
		return ctx, err
	}

	return next(ctx, tx, simulate)
}

// transferTip transfers the tip from the tipper to the fee payer.
func (d tipDecorator) transferTip(ctx sdk.Context, sdkTx sdk.Tx) error {
	tipTx, ok := sdkTx.(tx.TipTx)

	// No-op if the tx doesn't have tips.
	if !ok || tipTx.GetTip() == nil {
		return nil
	}

	tipper, err := sdk.AccAddressFromBech32(tipTx.GetTip().Tipper)
	if err != nil {
		return err
	}

	coins := tipTx.GetTip().Amount
	if err := d.bankKeeper.IsSendEnabledCoins(ctx, coins...); err != nil {
		return fmt.Errorf("cannot tip these coins: %w", err)
	}

	return d.bankKeeper.SendCoins(ctx, tipper, tipTx.FeePayer(), coins)
}
