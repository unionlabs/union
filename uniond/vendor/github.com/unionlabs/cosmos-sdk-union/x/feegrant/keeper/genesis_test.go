package keeper_test

import (
	"errors"
	"testing"

	"github.com/golang/mock/gomock"
	"gotest.tools/v3/assert"

	coretesting "cosmossdk.io/core/testing"
	"cosmossdk.io/math"
	storetypes "cosmossdk.io/store/types"
	"cosmossdk.io/x/feegrant"
	"cosmossdk.io/x/feegrant/keeper"
	"cosmossdk.io/x/feegrant/module"
	feegranttestutil "cosmossdk.io/x/feegrant/testutil"

	"github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	authtypes "github.com/cosmos/cosmos-sdk/x/auth/types"
)

var (
	granteePub  = secp256k1.GenPrivKey().PubKey()
	granterPub  = secp256k1.GenPrivKey().PubKey()
	granteeAddr = sdk.AccAddress(granteePub.Address())
	granterAddr = sdk.AccAddress(granterPub.Address())
)

type genesisFixture struct {
	ctx            sdk.Context
	feegrantKeeper keeper.Keeper
	accountKeeper  *feegranttestutil.MockAccountKeeper
}

func initFixture(t *testing.T) *genesisFixture {
	t.Helper()
	key := storetypes.NewKVStoreKey(feegrant.StoreKey)
	testCtx := testutil.DefaultContextWithDB(t, key, storetypes.NewTransientStoreKey("transient_test"))
	encCfg := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, module.AppModule{})

	ctrl := gomock.NewController(t)
	accountKeeper := feegranttestutil.NewMockAccountKeeper(ctrl)
	accountKeeper.EXPECT().AddressCodec().Return(address.NewBech32Codec("cosmos")).AnyTimes()

	return &genesisFixture{
		ctx:            testCtx.Ctx,
		feegrantKeeper: keeper.NewKeeper(runtime.NewEnvironment(runtime.NewKVStoreService(key), coretesting.NewNopLogger()), encCfg.Codec, accountKeeper),
		accountKeeper:  accountKeeper,
	}
}

func TestImportExportGenesis(t *testing.T) {
	f := initFixture(t)

	f.accountKeeper.EXPECT().GetAccount(gomock.Any(), granteeAddr).Return(authtypes.NewBaseAccountWithAddress(granteeAddr)).AnyTimes()

	coins := sdk.NewCoins(sdk.NewCoin("foo", math.NewInt(1_000)))
	now := f.ctx.HeaderInfo().Time
	oneYear := now.AddDate(1, 0, 0)
	msgSrvr := keeper.NewMsgServerImpl(f.feegrantKeeper)

	allowance := &feegrant.BasicAllowance{SpendLimit: coins, Expiration: &oneYear}
	err := f.feegrantKeeper.GrantAllowance(f.ctx, granterAddr, granteeAddr, allowance)
	assert.NilError(t, err)

	genesis, err := f.feegrantKeeper.ExportGenesis(f.ctx)
	assert.NilError(t, err)

	granter, err := f.accountKeeper.AddressCodec().BytesToString(granterAddr.Bytes())
	assert.NilError(t, err)
	grantee, err := f.accountKeeper.AddressCodec().BytesToString(granteeAddr.Bytes())
	assert.NilError(t, err)

	// revoke fee allowance
	_, err = msgSrvr.RevokeAllowance(f.ctx, &feegrant.MsgRevokeAllowance{
		Granter: granter,
		Grantee: grantee,
	})
	assert.NilError(t, err)

	err = f.feegrantKeeper.InitGenesis(f.ctx, genesis)
	assert.NilError(t, err)

	newGenesis, err := f.feegrantKeeper.ExportGenesis(f.ctx)
	assert.NilError(t, err)
	assert.DeepEqual(t, genesis, newGenesis)
}

func TestInitGenesis(t *testing.T) {
	any, err := codectypes.NewAnyWithValue(&testdata.Dog{})
	assert.NilError(t, err)

	ac := address.NewBech32Codec("cosmos")

	granter, err := ac.BytesToString(granterAddr.Bytes())
	assert.NilError(t, err)
	grantee, err := ac.BytesToString(granteeAddr.Bytes())
	assert.NilError(t, err)

	testCases := []struct {
		name          string
		feeAllowances []feegrant.Grant
		invalidAddr   bool
	}{
		{
			"invalid granter",
			[]feegrant.Grant{
				{
					Granter: "invalid granter",
					Grantee: grantee,
				},
			},
			true,
		},
		{
			"invalid grantee",
			[]feegrant.Grant{
				{
					Granter: granter,
					Grantee: "invalid grantee",
				},
			},
			true,
		},
		{
			"invalid allowance",
			[]feegrant.Grant{
				{
					Granter:   granter,
					Grantee:   grantee,
					Allowance: any,
				},
			},
			false,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			f := initFixture(t)
			if !tc.invalidAddr {
				err := f.feegrantKeeper.InitGenesis(f.ctx, &feegrant.GenesisState{Allowances: tc.feeAllowances})
				assert.ErrorContains(t, err, "failed to get allowance: no allowance")
			} else {
				expectedErr := errors.New("decoding bech32 failed")
				err := f.feegrantKeeper.InitGenesis(f.ctx, &feegrant.GenesisState{Allowances: tc.feeAllowances})
				assert.ErrorContains(t, err, expectedErr.Error())
			}
		})
	}
}
