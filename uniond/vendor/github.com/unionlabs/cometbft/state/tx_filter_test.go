package state_test

import (
	"os"
	"testing"

	"github.com/stretchr/testify/require"

	dbm "github.com/cometbft/cometbft-db"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
	sm "github.com/cometbft/cometbft/state"
	"github.com/cometbft/cometbft/types"
)

func TestTxFilter(t *testing.T) {
	genDoc := randomGenesisDoc()
	genDoc.ConsensusParams.Block.MaxBytes = 3000
	genDoc.ConsensusParams.Evidence.MaxBytes = 1500

	// Max size of Txs is much smaller than size of block,
	// since we need to account for commits and evidence.
	testCases := []struct {
		tx    types.Tx
		isErr bool
	}{
		{types.Tx(cmtrand.Bytes(2122)), false},
		{types.Tx(cmtrand.Bytes(2123)), true},
		{types.Tx(cmtrand.Bytes(3000)), true},
	}

	for i, tc := range testCases {
		stateDB, err := dbm.NewDB("state", "memdb", os.TempDir())
		require.NoError(t, err)
		stateStore := sm.NewStore(stateDB, sm.StoreOptions{
			DiscardABCIResponses: false,
		})
		state, err := stateStore.LoadFromDBOrGenesisDoc(genDoc)
		require.NoError(t, err)

		f := sm.TxPreCheck(state)
		if tc.isErr {
			require.Error(t, f(tc.tx), "#%v", i)
		} else {
			require.NoError(t, f(tc.tx), "#%v", i)
		}
	}
}
