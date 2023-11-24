package orm

import (
	"fmt"

	dbm "github.com/cosmos/cosmos-db"

	"cosmossdk.io/log"
	"cosmossdk.io/store"
	"cosmossdk.io/store/gaskv"
	"cosmossdk.io/store/metrics"
	storetypes "cosmossdk.io/store/types"
)

type MockContext struct {
	db    *dbm.MemDB
	store storetypes.CommitMultiStore
}

func NewMockContext() *MockContext {
	db := dbm.NewMemDB()
	return &MockContext{
		db:    dbm.NewMemDB(),
		store: store.NewCommitMultiStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics()),
	}
}

func (m MockContext) KVStore(key storetypes.StoreKey) storetypes.KVStore {
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
	GasMeter storetypes.GasMeter
}

func NewGasCountingMockContext() *GasCountingMockContext {
	return &GasCountingMockContext{
		GasMeter: &debuggingGasMeter{storetypes.NewInfiniteGasMeter()},
	}
}

func (g GasCountingMockContext) KVStore(store storetypes.KVStore) storetypes.KVStore {
	return gaskv.NewStore(store, g.GasMeter, storetypes.KVGasConfig())
}

func (g GasCountingMockContext) GasConsumed() storetypes.Gas {
	return g.GasMeter.GasConsumed()
}

func (g GasCountingMockContext) GasRemaining() storetypes.Gas {
	return g.GasMeter.GasRemaining()
}

func (g *GasCountingMockContext) ResetGasMeter() {
	g.GasMeter = storetypes.NewInfiniteGasMeter()
}
