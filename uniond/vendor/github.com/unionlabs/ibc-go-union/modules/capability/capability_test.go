package capability_test

import (
	"testing"

	dbm "github.com/cosmos/cosmos-db"
	testifysuite "github.com/stretchr/testify/suite"

	"cosmossdk.io/log"
	"cosmossdk.io/store"
	"cosmossdk.io/store/metrics"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	banktypes "cosmossdk.io/x/bank/types"

	tmproto "github.com/cometbft/cometbft/api/cometbft/types/v1"

	"github.com/cosmos/ibc-go/modules/capability"
	"github.com/cosmos/ibc-go/modules/capability/keeper"
	"github.com/cosmos/ibc-go/modules/capability/types"
)

const mockMemStoreKey = "memory:mock"

type CapabilityTestSuite struct {
	testifysuite.Suite

	cdc codec.Codec
	ctx context.Context

	keeper *keeper.Keeper

	storeKey        *storetypes.KVStoreKey
	memStoreKey     *storetypes.MemoryStoreKey
	mockMemStoreKey *storetypes.MemoryStoreKey
}

func (suite *CapabilityTestSuite) SetupTest() {
	encodingCfg := moduletestutil.MakeTestEncodingConfig(capability.AppModuleBasic{})
	suite.cdc = encodingCfg.Codec

	suite.storeKey = storetypes.NewKVStoreKey(types.StoreKey)
	suite.memStoreKey = storetypes.NewMemoryStoreKey(types.MemStoreKey)
	suite.mockMemStoreKey = storetypes.NewMemoryStoreKey(mockMemStoreKey)

	suite.ctx = suite.NewTestContext()
	suite.keeper = keeper.NewKeeper(suite.cdc, suite.storeKey, suite.memStoreKey)
}

func (suite *CapabilityTestSuite) NewTestContext() context.Context {
	db := dbm.NewMemDB()
	cms := store.NewCommitMultiStore(db, log.NewNopLogger(), metrics.NewNoOpMetrics())
	cms.MountStoreWithDB(suite.storeKey, storetypes.StoreTypeIAVL, db)
	cms.MountStoreWithDB(suite.memStoreKey, storetypes.StoreTypeMemory, db)
	cms.MountStoreWithDB(suite.mockMemStoreKey, storetypes.StoreTypeMemory, db)

	err := cms.LoadLatestVersion()
	suite.Require().NoError(err)

	return sdk.NewContext(cms, tmproto.Header{}, false, log.NewNopLogger())
}

// The following test case mocks a specific bug discovered in https://github.com/cosmos/cosmos-sdk/issues/9800
// and ensures that the current code successfully fixes the issue.
// This test emulates statesync by firstly populating persisted state by creating a new scoped keeper and capability.
// In-memory storage is then discarded by creating a new capability keeper and app module using a mock memstore key.
// BeginBlock is then called to populate the new in-memory store using the persisted state.
func (suite *CapabilityTestSuite) TestInitializeMemStore() {
	// create a scoped keeper and instantiate a new capability to populate state
	scopedKeeper := suite.keeper.ScopeToModule(banktypes.ModuleName)

	cap1, err := scopedKeeper.NewCapability(suite.ctx, "transfer")
	suite.Require().NoError(err)
	suite.Require().NotNil(cap1)

	// mock statesync by creating a new keeper and module that shares persisted state
	// but discards in-memory map by using a mock memstore key
	newKeeper := keeper.NewKeeper(suite.cdc, suite.storeKey, suite.mockMemStoreKey)
	newModule := capability.NewAppModule(suite.cdc, *newKeeper, true)

	// reassign the scoped keeper, this will inherit the the mock memstore key used above
	scopedKeeper = newKeeper.ScopeToModule(banktypes.ModuleName)

	// seal the new keeper and ensure the in-memory store is not initialized
	newKeeper.Seal()
	suite.Require().False(newKeeper.IsInitialized(suite.ctx), "memstore initialized flag set before BeginBlock")

	cap1, ok := scopedKeeper.GetCapability(suite.ctx, "transfer")
	suite.Require().False(ok)
	suite.Require().Nil(cap1)

	// add a new block gas meter to the context
	ctx := suite.ctx.WithBlockGasMeter(storetypes.NewGasMeter(50))

	prevGas := ctx.GasMeter().GasConsumed()
	prevBlockGas := ctx.BlockGasMeter().GasConsumed()

	// call app module BeginBlock and ensure that no gas has been consumed
	err = newModule.BeginBlock(ctx)
	suite.Require().NoError(err)

	gasUsed := ctx.GasMeter().GasConsumed()
	blockGasUsed := ctx.BlockGasMeter().GasConsumed()

	suite.Require().Equal(prevBlockGas, blockGasUsed, "ensure beginblocker consumed no block gas during execution")
	suite.Require().Equal(prevGas, gasUsed, "ensure beginblocker consumed no gas during execution")

	// assert that the in-memory store is now initialized
	suite.Require().True(newKeeper.IsInitialized(ctx), "memstore initialized flag not set")

	// ensure that BeginBlock has populated the new in-memory store (using the mock memstore key) and initialized capabilities
	cap1, ok = scopedKeeper.GetCapability(ctx, "transfer")
	suite.Require().True(ok)
	suite.Require().NotNil(cap1)

	// ensure capabilities do not get reinitialized on next BeginBlock by comparing capability pointers
	// and assert that the in-memory store is still initialized
	err = newModule.BeginBlock(ctx)
	suite.Require().NoError(err)

	refreshedCap, ok := scopedKeeper.GetCapability(ctx, "transfer")
	suite.Require().True(ok)
	suite.Require().Equal(cap1, refreshedCap, "capabilities got reinitialized after second BeginBlock")
	suite.Require().True(newKeeper.IsInitialized(ctx), "memstore initialized flag not set")
}

func TestCapabilityTestSuite(t *testing.T) {
	testifysuite.Run(t, new(CapabilityTestSuite))
}
