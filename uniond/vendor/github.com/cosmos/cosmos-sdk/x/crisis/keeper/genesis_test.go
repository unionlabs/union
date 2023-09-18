package keeper_test

import (
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/crisis"
	"github.com/cosmos/cosmos-sdk/x/crisis/keeper"
	crisistestutil "github.com/cosmos/cosmos-sdk/x/crisis/testutil"
	"github.com/cosmos/cosmos-sdk/x/crisis/types"
)

type GenesisTestSuite struct {
	suite.Suite

	sdkCtx sdk.Context
	keeper keeper.Keeper
	cdc    codec.BinaryCodec
}

func TestGenesisTestSuite(t *testing.T) {
	suite.Run(t, new(GenesisTestSuite))
}

func (s *GenesisTestSuite) SetupTest() {
	key := sdk.NewKVStoreKey(types.StoreKey)
	testCtx := testutil.DefaultContextWithDB(s.T(), key, sdk.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(crisis.AppModuleBasic{})

	// gomock initializations
	ctrl := gomock.NewController(s.T())
	s.cdc = codec.NewProtoCodec(encCfg.InterfaceRegistry)
	s.sdkCtx = testCtx.Ctx

	supplyKeeper := crisistestutil.NewMockSupplyKeeper(ctrl)

	s.keeper = *keeper.NewKeeper(s.cdc, key, 5, supplyKeeper, "", "")
}

func (s *GenesisTestSuite) TestImportExportGenesis() {
	// default params
	constantFee := sdk.NewCoin(sdk.DefaultBondDenom, sdk.NewInt(1000))
	err := s.keeper.SetConstantFee(s.sdkCtx, constantFee)
	s.Require().NoError(err)
	genesis := s.keeper.ExportGenesis(s.sdkCtx)

	// set constant fee to zero
	constantFee = sdk.NewCoin(sdk.DefaultBondDenom, sdk.NewInt(0))
	err = s.keeper.SetConstantFee(s.sdkCtx, constantFee)
	s.Require().NoError(err)

	s.keeper.InitGenesis(s.sdkCtx, genesis)
	newGenesis := s.keeper.ExportGenesis(s.sdkCtx)
	s.Require().Equal(genesis, newGenesis)
}

func (s *GenesisTestSuite) TestInitGenesis() {
	genesisState := types.DefaultGenesisState()
	genesisState.ConstantFee = sdk.NewCoin(sdk.DefaultBondDenom, sdk.NewInt(1000))
	s.keeper.InitGenesis(s.sdkCtx, genesisState)

	constantFee := s.keeper.GetConstantFee(s.sdkCtx)
	s.Require().Equal(genesisState.ConstantFee, constantFee)
}
