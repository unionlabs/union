package orm

import (
	"testing"

	"github.com/stretchr/testify/require"
	"pgregory.net/rapid"

	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
)

func TestSequence(t *testing.T) {
	rapid.Check(t, func(rt *rapid.T) {
		// Init sets up the real Sequence, including choosing a random initial value,
		// and initializes the model state
		key := storetypes.NewKVStoreKey("test")
		testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
		store := runtime.NewKVStoreService(key).OpenKVStore(testCtx.Ctx)

		// Create primary key table
		seq := NewSequence(0x1)

		// Choose initial sequence value
		initSeqVal := rapid.Uint64().Draw(rt, "initSeqVal")
		err := seq.InitVal(store, initSeqVal)
		require.NoError(t, err)

		// Create model state
		state := initSeqVal

		rt.Repeat(map[string]func(*rapid.T){
			// NextVal is one of the model commands. It checks that the next value of the
			// sequence matches the model and increments the model state.
			"NextVal": func(t *rapid.T) {
				// Check that the next value in the sequence matches the model
				require.Equal(t, state+1, seq.NextVal(store))
				// Increment the model state
				state++
			},
			// CurVal is one of the model commands. It checks that the current value of the
			// sequence matches the model.
			"CurVal": func(t *rapid.T) {
				// Check the current value matches the model
				require.Equal(t, state, seq.CurVal(store))
			},
			// PeekNextVal is one of the model commands. It checks that the next value of
			// the sequence matches the model without modifying the state.
			"PeekNextVal": func(t *rapid.T) {
				// Check that the next value in the sequence matches the model
				require.Equal(t, state+1, seq.PeekNextVal(store))
			},
		})
	})
}
