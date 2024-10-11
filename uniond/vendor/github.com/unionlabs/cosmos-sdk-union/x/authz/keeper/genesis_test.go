package keeper_test

import (
	"testing"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/log"
	sdkmath "cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/authz/keeper"
	authzmodule "cosmossdk.io/x/authz/module"
	authztestutil "cosmossdk.io/x/authz/testutil"
	bank "cosmossdk.io/x/bank/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

var (
	granteePub  = secp256k1.GenPrivKey().PubKey()
	granterPub  = secp256k1.GenPrivKey().PubKey()
	granteeAddr = sdk.AccAddress(granteePub.Address())
	granterAddr = sdk.AccAddress(granterPub.Address())
)

type GenesisTestSuite struct {
	suite.Suite

	ctx           sdk.Context
	keeper        keeper.Keeper
	baseApp       *baseapp.BaseApp
	accountKeeper *authztestutil.MockAccountKeeper
	encCfg        moduletestutil.TestEncodingConfig
}

func (suite *GenesisTestSuite) SetupTest() {
	key := storetypes.NewKVStoreKey(keeper.StoreKey)
	storeService := runtime.NewKVStoreService(key)
	testCtx := testutil.DefaultContextWithDB(suite.T(), key, storetypes.NewTransientStoreKey("transient_test"))
	suite.ctx = testCtx.Ctx.WithHeaderInfo(header.Info{Height: 1})

	suite.encCfg = moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, authzmodule.AppModule{})

	// gomock initializations
	ctrl := gomock.NewController(suite.T())
	suite.accountKeeper = authztestutil.NewMockAccountKeeper(ctrl)
	suite.accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	suite.baseApp = baseapp.NewBaseApp(
		"authz",
		log.NewNopLogger(),
		testCtx.DB,
		suite.encCfg.TxConfig.TxDecoder(),
	)
	suite.baseApp.SetCMS(testCtx.CMS)

	bank.RegisterInterfaces(suite.encCfg.InterfaceRegistry)

	msr := suite.baseApp.MsgServiceRouter()
	msr.SetInterfaceRegistry(suite.encCfg.InterfaceRegistry)
	env := runtime.NewEnvironment(storeService, coretesting.NewNopLogger(), runtime.EnvWithMsgRouterService(msr))

	suite.keeper = keeper.NewKeeper(env, suite.encCfg.Codec, suite.accountKeeper)
}

func (suite *GenesisTestSuite) TestImportExportGenesis() {
	coins := sdk.NewCoins(sdk.NewCoin("foo", sdkmath.NewInt(1_000)))

	now := suite.ctx.HeaderInfo().Time
	expires := now.Add(time.Hour)
	grant := &bank.SendAuthorization{SpendLimit: coins}
	err := suite.keeper.SaveGrant(suite.ctx, granteeAddr, granterAddr, grant, &expires)
	suite.Require().NoError(err)
	genesis, err := suite.keeper.ExportGenesis(suite.ctx)
	suite.Require().NoError(err)
	// Clear keeper
	err = suite.keeper.DeleteGrant(suite.ctx, granteeAddr, granterAddr, grant.MsgTypeURL())
	suite.Require().NoError(err)
	newGenesis, err := suite.keeper.ExportGenesis(suite.ctx)
	suite.Require().NoError(err)
	suite.Require().NotEqual(genesis, newGenesis)
	suite.Require().Empty(newGenesis)

	err = suite.keeper.InitGenesis(suite.ctx, genesis)
	suite.Require().NoError(err)
	newGenesis, err = suite.keeper.ExportGenesis(suite.ctx)
	suite.Require().NoError(err)
	suite.Require().Equal(genesis, newGenesis)
}

func TestGenesisTestSuite(t *testing.T) {
	suite.Run(t, new(GenesisTestSuite))
}
