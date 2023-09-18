package capability_test

import (
	"testing"

	abci "github.com/cometbft/cometbft/abci/types"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/runtime"
	storetypes "github.com/cosmos/cosmos-sdk/store/types"
	simtestutil "github.com/cosmos/cosmos-sdk/testutil/sims"
	sdk "github.com/cosmos/cosmos-sdk/types"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	"github.com/cosmos/cosmos-sdk/x/capability"
	"github.com/cosmos/cosmos-sdk/x/capability/keeper"
	"github.com/cosmos/cosmos-sdk/x/capability/testutil"
	"github.com/cosmos/cosmos-sdk/x/capability/types"
)

type CapabilityTestSuite struct {
	suite.Suite

	app    *runtime.App
	cdc    codec.Codec
	ctx    sdk.Context
	keeper *keeper.Keeper
	memKey *storetypes.MemoryStoreKey
}

func (suite *CapabilityTestSuite) SetupTest() {
	suite.memKey = storetypes.NewMemoryStoreKey("testingkey")

	startupCfg := simtestutil.DefaultStartUpConfig()
	startupCfg.BaseAppOption = func(ba *baseapp.BaseApp) {
		ba.MountStores(suite.memKey)
	}

	app, err := simtestutil.SetupWithConfiguration(testutil.AppConfig,
		startupCfg,
		&suite.cdc,
		&suite.keeper,
	)
	suite.Require().NoError(err)

	suite.app = app
	suite.ctx = app.BaseApp.NewContext(false, tmproto.Header{Height: 1})
}

// The following test case mocks a specific bug discovered in https://github.com/cosmos/cosmos-sdk/issues/9800
// and ensures that the current code successfully fixes the issue.
func (suite *CapabilityTestSuite) TestInitializeMemStore() {
	sk1 := suite.keeper.ScopeToModule(banktypes.ModuleName)

	cap1, err := sk1.NewCapability(suite.ctx, "transfer")
	suite.Require().NoError(err)
	suite.Require().NotNil(cap1)

	// mock statesync by creating new keeper that shares persistent state but loses in-memory map
	newKeeper := keeper.NewKeeper(suite.cdc, suite.app.UnsafeFindStoreKey(types.StoreKey).(*storetypes.KVStoreKey), suite.memKey)
	newSk1 := newKeeper.ScopeToModule(banktypes.ModuleName)

	// Mock App startup
	ctx := suite.app.BaseApp.NewUncachedContext(false, tmproto.Header{})
	newKeeper.Seal()
	suite.Require().False(newKeeper.IsInitialized(ctx), "memstore initialized flag set before BeginBlock")

	// Mock app beginblock and ensure that no gas has been consumed and memstore is initialized
	ctx = suite.app.BaseApp.NewContext(false, tmproto.Header{}).WithBlockGasMeter(sdk.NewGasMeter(50))
	prevGas := ctx.BlockGasMeter().GasConsumed()
	restartedModule := capability.NewAppModule(suite.cdc, *newKeeper, true)
	restartedModule.BeginBlock(ctx, abci.RequestBeginBlock{})
	suite.Require().True(newKeeper.IsInitialized(ctx), "memstore initialized flag not set")
	gasUsed := ctx.BlockGasMeter().GasConsumed()

	suite.Require().Equal(prevGas, gasUsed, "beginblocker consumed gas during execution")

	// Mock the first transaction getting capability and subsequently failing
	// by using a cached context and discarding all cached writes.
	cacheCtx, _ := ctx.CacheContext()
	_, ok := newSk1.GetCapability(cacheCtx, "transfer")
	suite.Require().True(ok)

	// Ensure that the second transaction can still receive capability even if first tx fails.
	ctx = suite.app.BaseApp.NewContext(false, tmproto.Header{})

	cap1, ok = newSk1.GetCapability(ctx, "transfer")
	suite.Require().True(ok)

	// Ensure the capabilities don't get reinitialized on next BeginBlock
	// by testing to see if capability returns same pointer
	// also check that initialized flag is still set
	restartedModule.BeginBlock(ctx, abci.RequestBeginBlock{})
	recap, ok := newSk1.GetCapability(ctx, "transfer")
	suite.Require().True(ok)
	suite.Require().Equal(cap1, recap, "capabilities got reinitialized after second BeginBlock")
	suite.Require().True(newKeeper.IsInitialized(ctx), "memstore initialized flag not set")
}

func TestCapabilityTestSuite(t *testing.T) {
	suite.Run(t, new(CapabilityTestSuite))
}
