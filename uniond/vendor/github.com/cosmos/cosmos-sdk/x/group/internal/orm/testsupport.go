package orm

import (
	"fmt"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cosmos/cosmos-sdk/store"
	"github.com/cosmos/cosmos-sdk/store/gaskv"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

type MockContext struct {
	db    *dbm.MemDB
	store storetypes.CommitMultiStore
}

func NewMockContext() *MockContext {
	db := dbm.NewMemDB()
	return &MockContext{
		db:    dbm.NewMemDB(),
		store: store.NewCommitMultiStore(db),
	}
}

func (m MockContext) KVStore(key storetypes.StoreKey) sdk.KVStore {
	if s := m.store.GetCommitKVStore(key); s != nil {
		return s
	}
	m.store.MountStoreWithDB(key, storetypes.StoreTypeIAVL, m.db)
	if err := m.store.LoadLatestVersion(); err != nil {
		panic(err)
	}
	return m.store.GetCommitKVStore(key)
}

type debuggingGasMeter struct {
	g storetypes.GasMeter
}

func (d debuggingGasMeter) GasConsumed() storetypes.Gas {
	return d.g.GasConsumed()
}

func (d debuggingGasMeter) GasRemaining() storetypes.Gas {
	return d.g.GasRemaining()
}

func (d debuggingGasMeter) GasConsumedToLimit() storetypes.Gas {
	return d.g.GasConsumedToLimit()
}

func (d debuggingGasMeter) RefundGas(amount uint64, descriptor string) {
	d.g.RefundGas(amount, descriptor)
}

func (d debuggingGasMeter) Limit() storetypes.Gas {
	return d.g.Limit()
}

func (d debuggingGasMeter) ConsumeGas(amount storetypes.Gas, descriptor string) {
	fmt.Printf("++ Consuming gas: %q :%d\n", descriptor, amount)
	d.g.ConsumeGas(amount, descriptor)
}

func (d debuggingGasMeter) IsPastLimit() bool {
	return d.g.IsPastLimit()
}

func (d debuggingGasMeter) IsOutOfGas() bool {
	return d.g.IsOutOfGas()
}

func (d debuggingGasMeter) String() string {
	return d.g.String()
}

type GasCountingMockContext struct {
	GasMeter sdk.GasMeter
}

func NewGasCountingMockContext() *GasCountingMockContext {
	return &GasCountingMockContext{
		GasMeter: &debuggingGasMeter{sdk.NewInfiniteGasMeter()},
	}
}

func (g GasCountingMockContext) KVStore(store sdk.KVStore) sdk.KVStore {
	return gaskv.NewStore(store, g.GasMeter, storetypes.KVGasConfig())
}

func (g GasCountingMockContext) GasConsumed() storetypes.Gas {
	return g.GasMeter.GasConsumed()
}

func (g GasCountingMockContext) GasRemaining() storetypes.Gas {
	return g.GasMeter.GasRemaining()
}

func (g *GasCountingMockContext) ResetGasMeter() {
	g.GasMeter = sdk.NewInfiniteGasMeter()
}
