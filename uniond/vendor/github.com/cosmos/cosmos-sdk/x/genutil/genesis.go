package genutil

import (
	"context"
	"fmt"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/types/module"
	"github.com/cosmos/cosmos-sdk/x/genutil/types"
)

// TxHandler is an interface that defines how genesis txs are handled.
type TxHandler interface {
	ExecuteGenesisTx([]byte) error
}

// InitGenesis - initialize accounts and deliver genesis transactions
// NOTE: It isn't used in server/v2 applications.
func InitGenesis(
	ctx context.Context, stakingKeeper types.StakingKeeper,
	deliverTx TxHandler, genesisState types.GenesisState,
	txEncodingConfig client.TxEncodingConfig,
) (validatorUpdates []module.ValidatorUpdate, err error) {
	if deliverTx == nil {
		return nil, fmt.Errorf("deliverTx (genesis.TxHandler) not defined, verify x/genutil wiring")
	}

	if len(genesisState.GenTxs) > 0 {
		return DeliverGenTxs(ctx, genesisState.GenTxs, stakingKeeper, deliverTx, txEncodingConfig)
	}
	return
}
