package query_test

import (
	"fmt"
	"testing"

	"cosmossdk.io/math"
	fuzz "github.com/google/gofuzz"

	"github.com/cosmos/cosmos-sdk/store/prefix"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/address"
	"github.com/cosmos/cosmos-sdk/types/query"
	"github.com/cosmos/cosmos-sdk/x/bank/testutil"
	"github.com/cosmos/cosmos-sdk/x/bank/types"
)

type fuzzTestSuite struct {
	paginationTestSuite
}

func FuzzPagination(f *testing.F) {
	if testing.Short() {
		f.Skip("In -short mode")
	}

	suite := new(fuzzTestSuite)
	suite.SetT(new(testing.T))
	suite.SetupTest()

	gf := fuzz.New()
	// 1. Set up some seeds.
	seeds := []*query.PageRequest{
		new(query.PageRequest),
		{
			Offset: 0,
			Limit:  10,
		},
	}

	// 1.5. Use the inprocess fuzzer to mutate variables.
	for i := 0; i < 1000; i++ {
		qr := new(query.PageRequest)
		gf.Fuzz(qr)
		seeds = append(seeds, qr)
	}

	// 2. Now serialize the fuzzers to bytes so that future mutations
	// can occur.
	for _, seed := range seeds {
		seedBlob, err := suite.cdc.Marshal(seed)
		if err == nil { // Some seeds could have been invalid so only add those that marshal.
			f.Add(seedBlob)
		}
	}

	// 3. Setup the keystore.
	var balances sdk.Coins
	for i := 0; i < 5; i++ {
		denom := fmt.Sprintf("foo%ddenom", i)
		balances = append(balances, sdk.NewInt64Coin(denom, int64(100+i)))
	}

	balances = balances.Sort()
	addr1 := sdk.AccAddress([]byte("addr1"))
	acc1 := suite.accountKeeper.NewAccountWithAddress(suite.ctx, addr1)
	suite.accountKeeper.SetAccount(suite.ctx, acc1)
	err := testutil.FundAccount(suite.bankKeeper, suite.ctx, addr1, balances)
	if err != nil { // should return no error
		f.Fatal(err)
	}

	// 4. Now run that fuzzer!
	f.Fuzz(func(t *testing.T, pagBz []byte) {
		qr := new(query.PageRequest)
		if err := suite.cdc.Unmarshal(pagBz, qr); err != nil {
			// Some pagination requests won't unmarshal and that's okay.
			return
		}

		// Now try to paginate it.
		req := types.NewQueryAllBalancesRequest(addr1, qr)
		balResult := sdk.NewCoins()
		authStore := suite.ctx.KVStore(suite.app.UnsafeFindStoreKey(types.StoreKey))
		balancesStore := prefix.NewStore(authStore, types.BalancesPrefix)
		accountStore := prefix.NewStore(balancesStore, address.MustLengthPrefix(addr1))
		_, _ = query.Paginate(accountStore, req.Pagination, func(key []byte, value []byte) error {
			var amount math.Int
			err := amount.Unmarshal(value)
			if err != nil {
				return err
			}
			balResult = append(balResult, sdk.NewCoin(string(key), amount))
			return nil
		})
	})
}
