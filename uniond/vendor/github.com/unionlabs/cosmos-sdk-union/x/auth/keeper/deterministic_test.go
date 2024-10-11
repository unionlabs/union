package keeper_test

import (
	"context"
	"encoding/hex"
	"sort"
	"sync/atomic"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/stretchr/testify/suite"
	"pgregory.net/rapid"

	"cosmossdk.io/core/appmodule"
	"cosmossdk.io/core/header"
	coretesting "cosmossdk.io/core/testing"
	storetypes "cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/baseapp"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/runtime"
	"github.com/cosmos/cosmos-sdk/testutil"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/auth"
	authcodec "github.com/cosmos/cosmos-sdk/x/auth/codec"
	"github.com/cosmos/cosmos-sdk/x/auth/keeper"
	authtestutil "github.com/cosmos/cosmos-sdk/x/auth/testutil"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

type DeterministicTestSuite struct {
	suite.Suite

	accountNumberLanes uint64

	key            *storetypes.KVStoreKey
	environment    appmodule.Environment
	ctx            sdk.Context
	queryClient    types.QueryClient
	accountKeeper  keeper.AccountKeeper
	acctsModKeeper *authtestutil.MockAccountsModKeeper
	encCfg         moduletestutil.TestEncodingConfig
	maccPerms      map[string][]string
}

var (
	addr        = sdk.MustAccAddressFromBech32("cosmos1j364pjm8jkxxmujj0vp2xjg0y7w8tyveuamfm6")
	pub, _      = hex.DecodeString("01090C02812F010C25200ED40E004105160196E801F70005070EA21603FF06001E")
	permissions = []string{"burner", "minter", "staking", "random"}
)

func TestDeterministicTestSuite(t *testing.T) {
	suite.Run(t, new(DeterministicTestSuite))
}

func (suite *DeterministicTestSuite) SetupTest() {
	suite.encCfg = moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, auth.AppModule{})

	suite.Require()
	key := storetypes.NewKVStoreKey(types.StoreKey)
	storeService := runtime.NewKVStoreService(key)
	env := runtime.NewEnvironment(storeService, coretesting.NewNopLogger())
	testCtx := testutil.DefaultContextWithDB(suite.T(), key, storetypes.NewTransientStoreKey("transient_test"))
	suite.ctx = testCtx.Ctx.WithHeaderInfo(header.Info{})

	// gomock initializations
	ctrl := gomock.NewController(suite.T())
	acctsModKeeper := authtestutil.NewMockAccountsModKeeper(ctrl)
	suite.acctsModKeeper = acctsModKeeper
	accNum := uint64(0)
	suite.acctsModKeeper.EXPECT().NextAccountNumber(gomock.Any()).AnyTimes().DoAndReturn(func(ctx context.Context) (uint64, error) {
		currNum := accNum
		accNum++
		return currNum, nil
	})

	maccPerms := map[string][]string{
		"fee_collector":          nil,
		"mint":                   {"minter"},
		"bonded_tokens_pool":     {"burner", "staking"},
		"not_bonded_tokens_pool": {"burner", "staking"},
		multiPerm:                {"burner", "minter", "staking"},
		randomPerm:               {"random"},
	}

	suite.accountKeeper = keeper.NewAccountKeeper(
		env,
		suite.encCfg.Codec,
		types.ProtoBaseAccount,
		suite.acctsModKeeper,
		maccPerms,
		authcodec.NewBech32Codec("cosmos"),
		"cosmos",
		types.NewModuleAddress("gov").String(),
	)

	queryHelper := baseapp.NewQueryServerTestHelper(suite.ctx, suite.encCfg.InterfaceRegistry)
	types.RegisterQueryServer(queryHelper, keeper.NewQueryServer(suite.accountKeeper))
	suite.queryClient = types.NewQueryClient(queryHelper)

	suite.key = key
	suite.environment = env
	suite.maccPerms = maccPerms
	suite.accountNumberLanes = 1
}

// createAndSetAccounts creates a random account and sets to the keeper store.
func (suite *DeterministicTestSuite) createAndSetAccounts(t *rapid.T, count int) []sdk.AccountI {
	accs := make([]sdk.AccountI, 0, count)

	// We need all generated account-numbers unique
	accNums := rapid.SliceOfNDistinct(rapid.Uint64(), count, count, func(i uint64) uint64 {
		return i
	}).Draw(t, "acc-nums")

	// then we change account numbers in such a way that there cannot be accounts with the same account number
	lane := atomic.AddUint64(&suite.accountNumberLanes, 1)
	for i := range accNums {
		accNums[i] += lane * 1000
	}

	for i := 0; i < count; i++ {
		pub := pubkeyGenerator(t).Draw(t, "pubkey")
		addr := sdk.AccAddress(pub.Address())
		accNum := accNums[i]
		seq := rapid.Uint64().Draw(t, "sequence")

		acc1 := types.NewBaseAccount(addr, &pub, accNum, seq)
		suite.accountKeeper.SetAccount(suite.ctx, acc1)
		accs = append(accs, acc1)
	}
	return accs
}

func (suite *DeterministicTestSuite) TestGRPCQueryAccount() {
	rapid.Check(suite.T(), func(t *rapid.T) {
		accs := suite.createAndSetAccounts(t, 1)
		req := &types.QueryAccountRequest{Address: accs[0].GetAddress().String()}
		testdata.DeterministicIterations(suite.T(), suite.ctx, req, suite.queryClient.Account, 0, true)
	})

	// Regression tests
	accNum := uint64(10087)
	seq := uint64(98)

	acc1 := types.NewBaseAccount(addr, &secp256k1.PubKey{Key: pub}, accNum, seq)
	suite.accountKeeper.SetAccount(suite.ctx, acc1)

	req := &types.QueryAccountRequest{Address: acc1.GetAddress().String()}

	testdata.DeterministicIterations(suite.T(), suite.ctx, req, suite.queryClient.Account, 1543, false)
}

// pubkeyGenerator creates and returns a random pubkey generator using rapid.
func pubkeyGenerator(_ *rapid.T) *rapid.Generator[secp256k1.PubKey] {
	return rapid.Custom(func(t *rapid.T) secp256k1.PubKey {
		pkBz := rapid.SliceOfN(rapid.Byte(), 33, 33).Draw(t, "hex")
		return secp256k1.PubKey{Key: pkBz}
	})
}

func (suite *DeterministicTestSuite) TestGRPCQueryAccounts() {
	rapid.Check(suite.T(), func(t *rapid.T) {
		numAccs := rapid.IntRange(1, 10).Draw(t, "accounts")
		accs := suite.createAndSetAccounts(t, numAccs)

		req := &types.QueryAccountsRequest{Pagination: testdata.PaginationGenerator(t, uint64(numAccs)).Draw(t, "accounts")}
		testdata.DeterministicIterations(suite.T(), suite.ctx, req, suite.queryClient.Accounts, 0, true)

		for i := 0; i < numAccs; i++ {
			suite.accountKeeper.RemoveAccount(suite.ctx, accs[i])
		}
	})

	// Regression test
	addr1, err := suite.accountKeeper.AddressCodec().StringToBytes("cosmos1892yr6fzlj7ud0kfkah2ctrav3a4p4n060ze8f")
	suite.Require().NoError(err)
	pub1, err := hex.DecodeString("D1002E1B019000010BB7034500E71F011F1CA90D5B000E134BFB0F3603030D0303")
	suite.Require().NoError(err)
	accNum1 := uint64(107)
	seq1 := uint64(10001)

	accNum2 := uint64(100)
	seq2 := uint64(10)

	acc1 := types.NewBaseAccount(addr1, &secp256k1.PubKey{Key: pub1}, accNum1, seq1)
	acc2 := types.NewBaseAccount(addr, &secp256k1.PubKey{Key: pub}, accNum2, seq2)

	suite.accountKeeper.SetAccount(suite.ctx, acc1)
	suite.accountKeeper.SetAccount(suite.ctx, acc2)

	req := &types.QueryAccountsRequest{}
	testdata.DeterministicIterations(suite.T(), suite.ctx, req, suite.queryClient.Accounts, 1716, false)
}

func (suite *DeterministicTestSuite) TestGRPCQueryAccountAddressByID() {
	rapid.Check(suite.T(), func(t *rapid.T) {
		accs := suite.createAndSetAccounts(t, 1)
		req := &types.QueryAccountAddressByIDRequest{AccountId: accs[0].GetAccountNumber()}
		testdata.DeterministicIterations(suite.T(), suite.ctx, req, suite.queryClient.AccountAddressByID, 0, true)
	})

	// Regression test
	accNum := uint64(10087)
	seq := uint64(0)

	acc1 := types.NewBaseAccount(addr, &secp256k1.PubKey{Key: pub}, accNum, seq)

	suite.accountKeeper.SetAccount(suite.ctx, acc1)
	req := &types.QueryAccountAddressByIDRequest{AccountId: accNum}
	testdata.DeterministicIterations(suite.T(), suite.ctx, req, suite.queryClient.AccountAddressByID, 1123, false)
}

func (suite *DeterministicTestSuite) TestGRPCQueryParameters() {
	rapid.Check(suite.T(), func(t *rapid.T) {
		params := types.NewParams(
			rapid.Uint64Min(1).Draw(t, "max-memo-characters"),
			rapid.Uint64Min(1).Draw(t, "tx-sig-limit"),
			rapid.Uint64Min(1).Draw(t, "tx-size-cost-per-byte"),
			rapid.Uint64Min(1).Draw(t, "sig-verify-cost-ed25519"),
			rapid.Uint64Min(1).Draw(t, "sig-verify-cost-Secp256k1"),
		)
		err := suite.accountKeeper.Params.Set(suite.ctx, params)
		suite.Require().NoError(err)

		req := &types.QueryParamsRequest{}
		testdata.DeterministicIterations(suite.T(), suite.ctx, req, suite.queryClient.Params, 0, true)
	})

	// Regression test
	params := types.NewParams(15, 167, 100, 1, 21457)

	err := suite.accountKeeper.Params.Set(suite.ctx, params)
	suite.Require().NoError(err)

	req := &types.QueryParamsRequest{}
	testdata.DeterministicIterations(suite.T(), suite.ctx, req, suite.queryClient.Params, 1042, false)
}

func (suite *DeterministicTestSuite) TestGRPCQueryAccountInfo() {
	rapid.Check(suite.T(), func(t *rapid.T) {
		accs := suite.createAndSetAccounts(t, 1)
		suite.Require().Len(accs, 1)

		req := &types.QueryAccountInfoRequest{Address: accs[0].GetAddress().String()}
		testdata.DeterministicIterations(suite.T(), suite.ctx, req, suite.queryClient.AccountInfo, 0, true)
	})

	// Regression test
	accNum := uint64(10087)
	seq := uint64(10)

	acc := types.NewBaseAccount(addr, &secp256k1.PubKey{Key: pub}, accNum, seq)

	suite.accountKeeper.SetAccount(suite.ctx, acc)
	req := &types.QueryAccountInfoRequest{Address: acc.GetAddress().String()}
	testdata.DeterministicIterations(suite.T(), suite.ctx, req, suite.queryClient.AccountInfo, 1543, false)
}

func (suite *DeterministicTestSuite) createAndReturnQueryClient(ak keeper.AccountKeeper) types.QueryClient {
	queryHelper := baseapp.NewQueryServerTestHelper(suite.ctx, suite.encCfg.InterfaceRegistry)
	types.RegisterQueryServer(queryHelper, keeper.NewQueryServer(ak))
	return types.NewQueryClient(queryHelper)
}

func (suite *DeterministicTestSuite) setModuleAccounts(
	ctx sdk.Context, ak keeper.AccountKeeper, maccs []string,
) []sdk.AccountI {
	sort.Strings(maccs)
	moduleAccounts := make([]sdk.AccountI, 0, len(maccs))
	for _, m := range maccs {
		acc, _ := ak.GetModuleAccountAndPermissions(ctx, m)
		acc1, ok := acc.(sdk.AccountI)
		suite.Require().True(ok)
		moduleAccounts = append(moduleAccounts, acc1)
	}

	return moduleAccounts
}

func (suite *DeterministicTestSuite) TestGRPCQueryModuleAccounts() {
	rapid.Check(suite.T(), func(t *rapid.T) {
		maccsCount := rapid.IntRange(1, 10).Draw(t, "accounts")
		maccs := make([]string, maccsCount)

		for i := 0; i < maccsCount; i++ {
			maccs[i] = rapid.StringMatching(`[a-z]{5,}`).Draw(t, "module-name")
		}

		maccPerms := make(map[string][]string)
		for i := 0; i < maccsCount; i++ {
			mPerms := make([]string, 0, 4)
			for _, permission := range permissions {
				if rapid.Bool().Draw(t, "permissions") {
					mPerms = append(mPerms, permission)
				}
			}

			if len(mPerms) == 0 {
				num := rapid.IntRange(0, 3).Draw(t, "num")
				mPerms = append(mPerms, permissions[num])
			}

			maccPerms[maccs[i]] = mPerms
		}

		ak := keeper.NewAccountKeeper(
			suite.environment,
			suite.encCfg.Codec,
			types.ProtoBaseAccount,
			suite.acctsModKeeper,
			maccPerms,
			authcodec.NewBech32Codec("cosmos"),
			"cosmos",
			types.NewModuleAddress("gov").String(),
		)
		suite.setModuleAccounts(suite.ctx, ak, maccs)

		queryClient := suite.createAndReturnQueryClient(ak)
		req := &types.QueryModuleAccountsRequest{}
		testdata.DeterministicIterations(suite.T(), suite.ctx, req, queryClient.ModuleAccounts, 0, true)
	})

	maccs := make([]string, 0, len(suite.maccPerms))
	for k := range suite.maccPerms {
		maccs = append(maccs, k)
	}

	suite.setModuleAccounts(suite.ctx, suite.accountKeeper, maccs)

	queryClient := suite.createAndReturnQueryClient(suite.accountKeeper)
	req := &types.QueryModuleAccountsRequest{}
	testdata.DeterministicIterations(suite.T(), suite.ctx, req, queryClient.ModuleAccounts, 8565, false)
}

func (suite *DeterministicTestSuite) TestGRPCQueryModuleAccountByName() {
	rapid.Check(suite.T(), func(t *rapid.T) {
		mName := rapid.StringMatching(`[a-z]{5,}`).Draw(t, "module-name")

		maccPerms := make(map[string][]string)
		mPerms := make([]string, 0, 4)
		for _, permission := range permissions {
			if rapid.Bool().Draw(t, "permissions") {
				mPerms = append(mPerms, permission)
			}
		}

		if len(mPerms) == 0 {
			num := rapid.IntRange(0, 3).Draw(t, "num")
			mPerms = append(mPerms, permissions[num])
		}

		maccPerms[mName] = mPerms

		ak := keeper.NewAccountKeeper(
			suite.environment,
			suite.encCfg.Codec,
			types.ProtoBaseAccount,
			suite.acctsModKeeper,
			maccPerms,
			authcodec.NewBech32Codec("cosmos"),
			"cosmos",
			types.NewModuleAddress("gov").String(),
		)
		suite.setModuleAccounts(suite.ctx, ak, []string{mName})

		queryClient := suite.createAndReturnQueryClient(ak)
		req := &types.QueryModuleAccountByNameRequest{Name: mName}
		testdata.DeterministicIterations(suite.T(), suite.ctx, req, queryClient.ModuleAccountByName, 0, true)
	})

	maccs := make([]string, 0, len(suite.maccPerms))
	for k := range suite.maccPerms {
		maccs = append(maccs, k)
	}

	suite.setModuleAccounts(suite.ctx, suite.accountKeeper, maccs)

	queryClient := suite.createAndReturnQueryClient(suite.accountKeeper)
	req := &types.QueryModuleAccountByNameRequest{Name: "mint"}
	testdata.DeterministicIterations(suite.T(), suite.ctx, req, queryClient.ModuleAccountByName, 1372, false)
}
