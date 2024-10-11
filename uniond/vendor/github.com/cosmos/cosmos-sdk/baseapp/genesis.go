package baseapp

import (
	"errors"

	"github.com/cometbft/cometbft/abci/types"
)

// ExecuteGenesisTx implements a genesis TxHandler used to execute a genTxs (from genutil).
func (app *BaseApp) ExecuteGenesisTx(tx []byte) error {
	res := app.deliverTx(tx)

	if res.Code != types.CodeTypeOK {
		return errors.New(res.Log)
	}

	return nil
}
