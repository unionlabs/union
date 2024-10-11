package baseapp

import (
	abci "github.com/cometbft/cometbft/api/cometbft/abci/v1"

	errorsmod "cosmossdk.io/errors"
)

// responseCheckTxWithEvents returns an ABCI ResponseCheckTx object with fields filled in
// from the given error, gas values and events.
func responseCheckTxWithEvents(err error, gw, gu uint64, events []abci.Event, debug bool) *abci.CheckTxResponse {
	space, code, log := errorsmod.ABCIInfo(err, debug)
	return &abci.CheckTxResponse{
		Codespace: space,
		Code:      code,
		Log:       log,
		GasWanted: int64(gw),
		GasUsed:   int64(gu),
		Events:    events,
	}
}

// responseExecTxResultWithEvents returns an ABCI ExecTxResult object with fields
// filled in from the given error, gas values and events.
func responseExecTxResultWithEvents(err error, gw, gu uint64, events []abci.Event, debug bool) *abci.ExecTxResult {
	space, code, log := errorsmod.ABCIInfo(err, debug)
	return &abci.ExecTxResult{
		Codespace: space,
		Code:      code,
		Log:       log,
		GasWanted: int64(gw),
		GasUsed:   int64(gu),
		Events:    events,
	}
}

// queryResult returns a ResponseQuery from an error. It will try to parse ABCI
// info from the error.
func queryResult(err error, debug bool) *abci.QueryResponse {
	space, code, log := errorsmod.ABCIInfo(err, debug)
	return &abci.QueryResponse{
		Codespace: space,
		Code:      code,
		Log:       log,
	}
}
