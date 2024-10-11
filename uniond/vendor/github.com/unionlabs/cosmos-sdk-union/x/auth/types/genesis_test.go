package types_test

import (
	"encoding/json"
	"testing"

	"github.com/cosmos/gogoproto/proto"
	"github.com/stretchr/testify/require"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	"github.com/cosmos/cosmos-sdk/x/auth"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

func TestSanitize(t *testing.T) {
	addr1 := sdk.AccAddress(ed25519.GenPrivKey().PubKey().Address())
	authAcc1 := types.NewBaseAccountWithAddress(addr1)
	err := authAcc1.SetAccountNumber(1)
	require.NoError(t, err)

	addr2 := sdk.AccAddress(ed25519.GenPrivKey().PubKey().Address())
	authAcc2 := types.NewBaseAccountWithAddress(addr2)

	genAccs := types.GenesisAccounts{authAcc1, authAcc2}

	require.True(t, genAccs[0].GetAccountNumber() > genAccs[1].GetAccountNumber())
	require.Equal(t, genAccs[1].GetAddress(), addr2)
	genAccs = types.SanitizeGenesisAccounts(genAccs)

	require.False(t, genAccs[0].GetAccountNumber() > genAccs[1].GetAccountNumber())
	require.Equal(t, genAccs[1].GetAddress(), addr1)
}

var (
	pk1   = ed25519.GenPrivKey().PubKey()
	pk2   = ed25519.GenPrivKey().PubKey()
	addr1 = sdk.ValAddress(pk1.Address())
	addr2 = sdk.ValAddress(pk2.Address())
)

// require duplicate accounts fails validation
func TestValidateGenesisDuplicateAccounts(t *testing.T) {
	acc1 := types.NewBaseAccountWithAddress(sdk.AccAddress(addr1))

	genAccs := make(types.GenesisAccounts, 2)
	genAccs[0] = acc1
	genAccs[1] = acc1

	require.Error(t, types.ValidateGenAccounts(genAccs))
}

func TestGenesisAccountIterator(t *testing.T) {
	encodingConfig := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}, auth.AppModule{})
	cdc := encodingConfig.Codec

	acc1 := types.NewBaseAccountWithAddress(sdk.AccAddress(addr1))
	acc2 := types.NewBaseAccountWithAddress(sdk.AccAddress(addr2))

	genAccounts := types.GenesisAccounts{acc1, acc2}

	authGenState := types.DefaultGenesisState()
	accounts, err := types.PackAccounts(genAccounts)
	require.NoError(t, err)
	authGenState.Accounts = accounts

	appGenesis := make(map[string]json.RawMessage)
	authGenStateBz, err := cdc.MarshalJSON(authGenState)
	require.NoError(t, err)

	appGenesis[types.ModuleName] = authGenStateBz

	var addresses []sdk.AccAddress
	types.GenesisAccountIterator{}.IterateGenesisAccounts(
		cdc, appGenesis, func(acc sdk.AccountI) (stop bool) {
			addresses = append(addresses, acc.GetAddress())
			return false
		},
	)

	require.Len(t, addresses, 2)
	require.Equal(t, addresses[0], acc1.GetAddress())
	require.Equal(t, addresses[1], acc2.GetAddress())
}

func TestPackAccountsAny(t *testing.T) {
	var accounts []*codectypes.Any

	testCases := []struct {
		msg      string
		malleate func()
		expPass  bool
	}{
		{
			"expected genesis account",
			func() {
				accounts = []*codectypes.Any{{}}
			},
			false,
		},
		{
			"success",
			func() {
				genAccounts := types.GenesisAccounts{&types.BaseAccount{}}
				accounts = make([]*codectypes.Any, len(genAccounts))

				for i, a := range genAccounts {
					msg, ok := a.(proto.Message)
					require.Equal(t, ok, true)
					any, err := codectypes.NewAnyWithValue(msg)
					require.NoError(t, err)
					accounts[i] = any
				}
			},
			true,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.msg, func(t *testing.T) {
			tc.malleate()

			res, err := types.UnpackAccounts(accounts)

			if tc.expPass {
				require.NoError(t, err)
				require.NotNil(t, res)
				require.Equal(t, len(res), len(accounts))
			} else {
				require.Error(t, err)
				require.Nil(t, res)
			}
		})
	}
}
