package baseapp

import (
	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"

	errorsmod "cosmossdk.io/errors"

	sdk "github.com/cosmos/cosmos-sdk/types"
	sdkerrors "github.com/cosmos/cosmos-sdk/types/errors"
)

// SimCheck defines a CheckTx helper function that used in tests and simulations.
func (app *BaseApp) SimCheck(txEncoder sdk.TxEncoder, tx sdk.Tx) (sdk.GasInfo, *sdk.Result, error) {
	// runTx expects tx bytes as argument, so we encode the tx argument into
	// bytes. Note that runTx will actually decode those bytes again. But since
	// this helper is only used in tests/simulation, it's fine.
	bz, err := txEncoder(tx)
	if err != nil {
		return sdk.GasInfo{}, nil, errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "%s", err)
	}

	gasInfo, result, _, err := app.runTx(execModeCheck, bz, tx)
	return gasInfo, result, err
}

// Simulate executes a tx in simulate mode to get result and gas info.
func (app *BaseApp) Simulate(txBytes []byte) (sdk.GasInfo, *sdk.Result, error) {
	gasInfo, result, _, err := app.runTx(execModeSimulate, txBytes, nil)
	return gasInfo, result, err
}

func (app *BaseApp) SimDeliver(txEncoder sdk.TxEncoder, tx sdk.Tx) (sdk.GasInfo, *sdk.Result, error) {
	// See comment for Check().
	bz, err := txEncoder(tx)
	if err != nil {
		return sdk.GasInfo{}, nil, errorsmod.Wrapf(sdkerrors.ErrInvalidRequest, "%s", err)
	}

	gasInfo, result, _, err := app.runTx(execModeFinalize, bz, tx)
	return gasInfo, result, err
}

// SimWriteState is an entrypoint for simulations only. They are not executed during the normal ABCI finalize
// block step but later. Therefore, an extra call to the root multi-store (app.cms) is required to write the changes.
func (app *BaseApp) SimWriteState() {
	app.finalizeBlockState.ms.Write()
}

// NewContextLegacy returns a new sdk.Context with the provided header
func (app *BaseApp) NewContextLegacy(isCheckTx bool, header cmtproto.Header) sdk.Context {
	if isCheckTx {
		return sdk.NewContext(app.checkState.ms, true, app.logger).
			WithMinGasPrices(app.minGasPrices).WithBlockHeader(header)
	}

	return sdk.NewContext(app.finalizeBlockState.ms, false, app.logger).WithBlockHeader(header)
}

// NewContext returns a new sdk.Context with a empty header
func (app *BaseApp) NewContext(isCheckTx bool) sdk.Context {
	return app.NewContextLegacy(isCheckTx, cmtproto.Header{})
}

func (app *BaseApp) NewUncachedContext(isCheckTx bool, header cmtproto.Header) sdk.Context {
	return sdk.NewContext(app.cms, isCheckTx, app.logger).WithBlockHeader(header)
}

func (app *BaseApp) GetContextForFinalizeBlock(txBytes []byte) sdk.Context {
	return app.getContextForTx(execModeFinalize, txBytes)
}

func (app *BaseApp) GetContextForCheckTx(txBytes []byte) sdk.Context {
	return app.getContextForTx(execModeCheck, txBytes)
}
