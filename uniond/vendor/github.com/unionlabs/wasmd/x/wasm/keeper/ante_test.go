package keeper_test

import (
	"testing"
	"time"

	cmtproto "github.com/cometbft/cometbft/api/cometbft/types/v1"
	dbm "github.com/cosmos/cosmos-db"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"cosmossdk.io/log"
	"cosmossdk.io/store"
	storemetrics "cosmossdk.io/store/metrics"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/runtime"
	sdk "github.com/cosmos/cosmos-sdk/types"

	"github.com/CosmWasm/wasmd/x/wasm/keeper"
	"github.com/CosmWasm/wasmd/x/wasm/keeper/wasmtesting"
	"github.com/CosmWasm/wasmd/x/wasm/types"
)

func TestCountTxDecorator(t *testing.T) {
	keyWasm := storetypes.NewKVStoreKey(types.StoreKey)
	db := dbm.NewMemDB()
	ms := store.NewCommitMultiStore(db, log.NewTestLogger(t), storemetrics.NewNoOpMetrics())
	ms.MountStoreWithDB(keyWasm, storetypes.StoreTypeIAVL, db)
	require.NoError(t, ms.LoadLatestVersion())
	const myCurrentBlockHeight = 100

	specs := map[string]struct {
		setupDB        func(t *testing.T, ctx sdk.Context)
		simulate       bool
		nextAssertAnte func(ctx sdk.Context, tx sdk.Tx, simulate bool) (sdk.Context, error)
	}{
		"no initial counter set": {
			setupDB: func(t *testing.T, ctx sdk.Context) {},
			nextAssertAnte: func(ctx sdk.Context, tx sdk.Tx, simulate bool) (sdk.Context, error) {
				gotCounter, ok := types.TXCounter(ctx)
				require.True(t, ok)
				assert.Equal(t, uint32(0), gotCounter)
				// and stored +1
				bz := ctx.MultiStore().GetKVStore(keyWasm).Get(types.TXCounterPrefix)
				assert.Equal(t, []byte{0, 0, 0, 0, 0, 0, 0, myCurrentBlockHeight, 0, 0, 0, 1}, bz)
				return ctx, nil
			},
		},
		"persistent counter incremented - big endian": {
			setupDB: func(t *testing.T, ctx sdk.Context) {
				bz := []byte{0, 0, 0, 0, 0, 0, 0, myCurrentBlockHeight, 1, 0, 0, 2}
				ctx.MultiStore().GetKVStore(keyWasm).Set(types.TXCounterPrefix, bz)
			},
			nextAssertAnte: func(ctx sdk.Context, tx sdk.Tx, simulate bool) (sdk.Context, error) {
				gotCounter, ok := types.TXCounter(ctx)
				require.True(t, ok)
				assert.Equal(t, uint32(1<<24+2), gotCounter)
				// and stored +1
				bz := ctx.MultiStore().GetKVStore(keyWasm).Get(types.TXCounterPrefix)
				assert.Equal(t, []byte{0, 0, 0, 0, 0, 0, 0, myCurrentBlockHeight, 1, 0, 0, 3}, bz)
				return ctx, nil
			},
		},
		"old height counter replaced": {
			setupDB: func(t *testing.T, ctx sdk.Context) {
				previousHeight := byte(myCurrentBlockHeight - 1)
				bz := []byte{0, 0, 0, 0, 0, 0, 0, previousHeight, 0, 0, 0, 1}
				ctx.MultiStore().GetKVStore(keyWasm).Set(types.TXCounterPrefix, bz)
			},
			nextAssertAnte: func(ctx sdk.Context, tx sdk.Tx, simulate bool) (sdk.Context, error) {
				gotCounter, ok := types.TXCounter(ctx)
				require.True(t, ok)
				assert.Equal(t, uint32(0), gotCounter)
				// and stored +1
				bz := ctx.MultiStore().GetKVStore(keyWasm).Get(types.TXCounterPrefix)
				assert.Equal(t, []byte{0, 0, 0, 0, 0, 0, 0, myCurrentBlockHeight, 0, 0, 0, 1}, bz)
				return ctx, nil
			},
		},
		"simulation not persisted": {
			setupDB: func(t *testing.T, ctx sdk.Context) {
			},
			simulate: true,
			nextAssertAnte: func(ctx sdk.Context, tx sdk.Tx, simulate bool) (sdk.Context, error) {
				_, ok := types.TXCounter(ctx)
				assert.False(t, ok)
				require.True(t, simulate)
				// and not stored
				assert.False(t, ctx.MultiStore().GetKVStore(keyWasm).Has(types.TXCounterPrefix))
				return ctx, nil
			},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			ctx := sdk.NewContext(ms.CacheMultiStore(), cmtproto.Header{
				Height: myCurrentBlockHeight,
				Time:   time.Date(2021, time.September, 27, 12, 0, 0, 0, time.UTC),
			}, false, log.NewNopLogger())

			spec.setupDB(t, ctx)
			var anyTx sdk.Tx

			// when
			ante := keeper.NewCountTXDecorator(runtime.NewKVStoreService(keyWasm))
			_, gotErr := ante.AnteHandle(ctx, anyTx, spec.simulate, spec.nextAssertAnte)
			require.NoError(t, gotErr)
		})
	}
}

func TestLimitSimulationGasDecorator(t *testing.T) {
	var (
		hundred storetypes.Gas = 100
		zero    storetypes.Gas = 0
	)
	specs := map[string]struct {
		customLimit *storetypes.Gas
		consumeGas  storetypes.Gas
		maxBlockGas int64
		simulation  bool
		expErr      interface{}
	}{
		"custom limit set": {
			customLimit: &hundred,
			consumeGas:  hundred + 1,
			maxBlockGas: -1,
			simulation:  true,
			expErr:      storetypes.ErrorOutOfGas{Descriptor: "testing"},
		},
		"block limit set": {
			maxBlockGas: 100,
			consumeGas:  hundred + 1,
			simulation:  true,
			expErr:      storetypes.ErrorOutOfGas{Descriptor: "testing"},
		},
		"no limits set": {
			maxBlockGas: -1,
			consumeGas:  hundred + 1,
			simulation:  true,
		},
		"both limits set, custom applies": {
			customLimit: &hundred,
			consumeGas:  hundred - 1,
			maxBlockGas: 10,
			simulation:  true,
		},
		"not a simulation": {
			customLimit: &hundred,
			consumeGas:  hundred + 1,
			simulation:  false,
		},
		"zero custom limit": {
			customLimit: &zero,
			simulation:  true,
			expErr:      "gas limit must not be zero",
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			nextAnte := consumeGasAnteHandler(spec.consumeGas)
			ctx := sdk.Context{}.
				WithGasMeter(storetypes.NewInfiniteGasMeter()).
				WithConsensusParams(cmtproto.ConsensusParams{
					Block: &cmtproto.BlockParams{MaxGas: spec.maxBlockGas},
				})
			// when
			if spec.expErr != nil {
				require.PanicsWithValue(t, spec.expErr, func() {
					ante := keeper.NewLimitSimulationGasDecorator(spec.customLimit)
					_, err := ante.AnteHandle(ctx, nil, spec.simulation, nextAnte)
					require.NoError(t, err)
				})
				return
			}
			ante := keeper.NewLimitSimulationGasDecorator(spec.customLimit)
			_, err := ante.AnteHandle(ctx, nil, spec.simulation, nextAnte)
			require.NoError(t, err)
		})
	}
}

func consumeGasAnteHandler(gasToConsume storetypes.Gas) sdk.AnteHandler {
	return func(ctx sdk.Context, tx sdk.Tx, simulate bool) (sdk.Context, error) {
		ctx.GasMeter().ConsumeGas(gasToConsume, "testing")
		return ctx, nil
	}
}

func TestGasRegisterDecorator(t *testing.T) {
	db := dbm.NewMemDB()
	ms := store.NewCommitMultiStore(db, log.NewTestLogger(t), storemetrics.NewNoOpMetrics())

	specs := map[string]struct {
		simulate       bool
		nextAssertAnte func(ctx sdk.Context, tx sdk.Tx, simulate bool) (sdk.Context, error)
	}{
		"simulation": {
			simulate: true,
			nextAssertAnte: func(ctx sdk.Context, tx sdk.Tx, simulate bool) (sdk.Context, error) {
				_, ok := types.GasRegisterFromContext(ctx)
				assert.True(t, ok)
				require.True(t, simulate)
				return ctx, nil
			},
		},
		"not simulation": {
			simulate: false,
			nextAssertAnte: func(ctx sdk.Context, tx sdk.Tx, simulate bool) (sdk.Context, error) {
				_, ok := types.GasRegisterFromContext(ctx)
				assert.True(t, ok)
				require.False(t, simulate)
				return ctx, nil
			},
		},
	}
	for name, spec := range specs {
		t.Run(name, func(t *testing.T) {
			ctx := sdk.NewContext(ms, cmtproto.Header{
				Height: 100,
				Time:   time.Now(),
			}, false, log.NewNopLogger())
			var anyTx sdk.Tx

			// when
			ante := keeper.NewGasRegisterDecorator(&wasmtesting.MockGasRegister{})
			_, gotErr := ante.AnteHandle(ctx, anyTx, spec.simulate, spec.nextAssertAnte)

			// then
			require.NoError(t, gotErr)
		})
	}
}
