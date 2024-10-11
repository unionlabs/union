package keeper_test

import (
	"context"
	"encoding/hex"
	"fmt"
	"time"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"

	"cosmossdk.io/collections"
	coreaddress "cosmossdk.io/core/address"
	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/evidence"
	"cosmossdk.io/x/evidence/exported"
	"cosmossdk.io/x/evidence/keeper"
	evidencetestutil "cosmossdk.io/x/evidence/testutil"
	"cosmossdk.io/x/evidence/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	"github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

var (
	pubkeys = []cryptotypes.PubKey{
		newPubKey("0B485CFC0EECC619440448436F8FC9DF40566F2369E72400281454CB552AFB50"),
		newPubKey("0B485CFC0EECC619440448436F8FC9DF40566F2369E72400281454CB552AFB51"),
		newPubKey("0B485CFC0EECC619440448436F8FC9DF40566F2369E72400281454CB552AFB52"),
	}

	valAddress = sdk.ValAddress(pubkeys[0].Address())
)

func newPubKey(pk string) (res cryptotypes.PubKey) {
	pkBytes, err := hex.DecodeString(pk)
	if err != nil {
		panic(err)
	}

	pubkey := &ed25519.PubKey{Key: pkBytes}

	return pubkey
}

func testEquivocationHandler(_ interface{}) types.Handler {
	return func(ctx context.Context, e exported.Evidence) error {
		if err := e.ValidateBasic(); err != nil {
			return err
		}

		ee, ok := e.(*types.Equivocation)
		if !ok {
			return fmt.Errorf("unexpected evidence type: %T", e)
		}
		if ee.Height%2 == 0 {
			return fmt.Errorf("unexpected even evidence height: %d", ee.Height)
		}

		return nil
	}
}

type KeeperTestSuite struct {
	suite.Suite

	ctx sdk.Context

	addressCodec     coreaddress.Codec
	consAddressCodec coreaddress.ConsensusAddressCodec

	evidenceKeeper keeper.Keeper
	accountKeeper  *evidencetestutil.MockAccountKeeper
	slashingKeeper *evidencetestutil.MockSlashingKeeper
	stakingKeeper  *evidencetestutil.MockStakingKeeper
	queryClient    types.QueryClient
	encCfg         moduletestutil.TestEncodingConfig
	msgServer      types.MsgServer
}

func (suite *KeeperTestSuite) SetupTest() {
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, evidence.AppModule{})
	key := storetypes.NewKVStoreKey(types.StoreKey)
	env := runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger())
	tkey := storetypes.NewTransientStoreKey("evidence_transient_store")
	testCtx := testutil.DefaultContextWithDB(suite.T(), key, tkey)
	suite.ctx = testCtx.Ctx
	suite.addressCodec = address.NewBech32Codec("cosmos")
	suite.consAddressCodec = address.NewBech32Codec("cosmosvalcons")

	ctrl := gomock.NewController(suite.T())

	stakingKeeper := evidencetestutil.NewMockStakingKeeper(ctrl)
	slashingKeeper := evidencetestutil.NewMockSlashingKeeper(ctrl)
	accountKeeper := evidencetestutil.NewMockAccountKeeper(ctrl)
	ck := evidencetestutil.NewMockConsensusKeeper(ctrl)

	evidenceKeeper := keeper.NewKeeper(
		encCfg.Codec,
		env,
		stakingKeeper,
		slashingKeeper,
		ck,
		address.NewBech32Codec("cosmos"),
	)

	suite.stakingKeeper = stakingKeeper
	suite.slashingKeeper = slashingKeeper

	router := types.NewRouter()
	router = router.AddRoute(types.RouteEquivocation, testEquivocationHandler(evidenceKeeper))
	evidenceKeeper.SetRouter(router)

	suite.ctx = testCtx.Ctx.WithHeaderInfo(header.Info{Height: 1})
	suite.encCfg = moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, evidence.AppModule{})

	suite.accountKeeper = accountKeeper

	queryHelper := baseapp.NewQueryServerTestHelper(suite.ctx, suite.encCfg.InterfaceRegistry)
	types.RegisterQueryServer(queryHelper, keeper.NewQuerier(evidenceKeeper))
	suite.queryClient = types.NewQueryClient(queryHelper)
	suite.evidenceKeeper = *evidenceKeeper
	suite.msgServer = keeper.NewMsgServerImpl(suite.evidenceKeeper)
}

func (suite *KeeperTestSuite) populateEvidence(ctx sdk.Context, numEvidence int) []exported.Evidence {
	evidence := make([]exported.Evidence, numEvidence)

	for i := 0; i < numEvidence; i++ {
		pk := ed25519.GenPrivKey()

		consAddr, err := suite.consAddressCodec.BytesToString(pk.PubKey().Address())
		suite.Require().NoError(err)

		evidence[i] = &types.Equivocation{
			Height:           11,
			Power:            100,
			Time:             time.Now().UTC(),
			ConsensusAddress: consAddr,
		}

		suite.Nil(suite.evidenceKeeper.SubmitEvidence(ctx, evidence[i]))
	}

	return evidence
}

func (suite *KeeperTestSuite) TestSubmitValidEvidence() {
	ctx := suite.ctx.WithIsCheckTx(false)
	pk := ed25519.GenPrivKey()
	consAddr, err := suite.consAddressCodec.BytesToString(pk.PubKey().Address())
	suite.Require().NoError(err)

	e := &types.Equivocation{
		Height:           1,
		Power:            100,
		Time:             time.Now().UTC(),
		ConsensusAddress: consAddr,
	}

	suite.Nil(suite.evidenceKeeper.SubmitEvidence(ctx, e))

	res, err := suite.evidenceKeeper.Evidences.Get(ctx, e.Hash())
	suite.NoError(err)
	suite.Equal(e, res)
}

func (suite *KeeperTestSuite) TestSubmitValidEvidence_Duplicate() {
	ctx := suite.ctx.WithIsCheckTx(false)
	pk := ed25519.GenPrivKey()
	consAddr, err := suite.consAddressCodec.BytesToString(pk.PubKey().Address())
	suite.Require().NoError(err)

	e := &types.Equivocation{
		Height:           1,
		Power:            100,
		Time:             time.Now().UTC(),
		ConsensusAddress: consAddr,
	}

	suite.Nil(suite.evidenceKeeper.SubmitEvidence(ctx, e))
	suite.Error(suite.evidenceKeeper.SubmitEvidence(ctx, e))

	res, err := suite.evidenceKeeper.Evidences.Get(ctx, e.Hash())
	suite.NoError(err)
	suite.Equal(e, res)
}

func (suite *KeeperTestSuite) TestSubmitInvalidEvidence() {
	ctx := suite.ctx.WithIsCheckTx(false)
	pk := ed25519.GenPrivKey()
	consAddr, err := suite.consAddressCodec.BytesToString(pk.PubKey().Address())
	suite.Require().NoError(err)
	e := &types.Equivocation{
		Height:           0,
		Power:            100,
		Time:             time.Now().UTC(),
		ConsensusAddress: consAddr,
	}

	err = suite.evidenceKeeper.SubmitEvidence(ctx, e)
	suite.ErrorIs(err, types.ErrInvalidEvidence)

	res, err := suite.evidenceKeeper.Evidences.Get(ctx, e.Hash())
	suite.ErrorIs(err, collections.ErrNotFound)
	suite.Nil(res)
}

func (suite *KeeperTestSuite) TestIterateEvidence() {
	ctx := suite.ctx.WithIsCheckTx(false)
	numEvidence := 100
	suite.populateEvidence(ctx, numEvidence)

	var evidences []exported.Evidence
	suite.Require().NoError(suite.evidenceKeeper.Evidences.Walk(ctx, nil, func(key []byte, value exported.Evidence) (stop bool, err error) {
		evidences = append(evidences, value)
		return false, nil
	}))
	suite.Len(evidences, numEvidence)
}

func (suite *KeeperTestSuite) TestGetEvidenceHandler() {
	handler, err := suite.evidenceKeeper.GetEvidenceHandler((&types.Equivocation{}).Route())
	suite.NoError(err)
	suite.NotNil(handler)

	handler, err = suite.evidenceKeeper.GetEvidenceHandler("invalidHandler")
	suite.Error(err)
	suite.Nil(handler)
}
