package types_test

import (
	"encoding/json"
	"errors"
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/testutil/testdata"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/x/auth/types"
)

func TestBaseAddressPubKey(t *testing.T) {
	_, pub1, addr1 := testdata.KeyTestPubAddr()
	_, pub2, addr2 := testdata.KeyTestPubAddr()
	acc := types.NewBaseAccountWithAddress(addr1)

	// check the address (set) and pubkey (not set)
	require.EqualValues(t, addr1, acc.GetAddress())
	require.EqualValues(t, nil, acc.GetPubKey())

	// can't override address
	err := acc.SetAddress(addr2)
	require.NotNil(t, err)
	require.EqualValues(t, addr1, acc.GetAddress())

	// set the pubkey
	err = acc.SetPubKey(pub1)
	require.Nil(t, err)
	require.Equal(t, pub1, acc.GetPubKey())

	// can override pubkey
	err = acc.SetPubKey(pub2)
	require.Nil(t, err)
	require.Equal(t, pub2, acc.GetPubKey())

	//------------------------------------

	// can set address on empty account
	acc2 := types.BaseAccount{}
	err = acc2.SetAddress(addr2)
	require.Nil(t, err)
	require.EqualValues(t, addr2, acc2.GetAddress())

	// no panic on calling string with an account with pubkey
	require.NotEmpty(t, acc.String())
	require.NotPanics(t, func() { _ = acc.String() })
}

func TestBaseSequence(t *testing.T) {
	_, _, addr := testdata.KeyTestPubAddr()
	acc := types.NewBaseAccountWithAddress(addr)
	seq := uint64(7)

	err := acc.SetSequence(seq)
	require.Nil(t, err)
	require.Equal(t, seq, acc.GetSequence())
}

func TestGenesisAccountValidate(t *testing.T) {
	pubkey := secp256k1.GenPrivKey().PubKey()
	addr := sdk.AccAddress(pubkey.Address())
	baseAcc := types.NewBaseAccount(addr, pubkey, 0, 0)

	tests := []struct {
		name   string
		acc    types.GenesisAccount
		expErr bool
	}{
		{
			"valid base account",
			baseAcc,
			false,
		},
		{
			"invalid base valid account",
			types.NewBaseAccount(addr, secp256k1.GenPrivKey().PubKey(), 0, 0),
			true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			require.Equal(t, tt.expErr, tt.acc.Validate() != nil)
		})
	}
}

func TestModuleAccountString(t *testing.T) {
	name := "test"
	moduleAcc := types.NewEmptyModuleAccount(name, types.Minter, types.Burner, types.Staking)
	want := `base_account:<address:"cosmos1n7rdpqvgf37ktx30a2sv2kkszk3m7ncmg5drhe" > name:"test" permissions:"minter" permissions:"burner" permissions:"staking" `
	require.Equal(t, want, moduleAcc.String())
	err := moduleAcc.SetSequence(10)
	require.NoError(t, err)
	want = `base_account:<address:"cosmos1n7rdpqvgf37ktx30a2sv2kkszk3m7ncmg5drhe" sequence:10 > name:"test" permissions:"minter" permissions:"burner" permissions:"staking" `
	require.Equal(t, want, moduleAcc.String())
}

func TestHasPermissions(t *testing.T) {
	name := "test"
	macc := types.NewEmptyModuleAccount(name, types.Staking, types.Minter, types.Burner)
	cases := []struct {
		permission string
		expectHas  bool
	}{
		{types.Staking, true},
		{types.Minter, true},
		{types.Burner, true},
		{"other", false},
	}

	for i, tc := range cases {
		hasPerm := macc.HasPermission(tc.permission)
		if tc.expectHas {
			require.True(t, hasPerm, "test case #%d", i)
		} else {
			require.False(t, hasPerm, "test case #%d", i)
		}
	}
}

func TestValidate(t *testing.T) {
	addr := sdk.AccAddress(secp256k1.GenPrivKey().PubKey().Address())
	baseAcc := types.NewBaseAccount(addr, nil, 0, 0)
	tests := []struct {
		name   string
		acc    types.GenesisAccount
		expErr error
	}{
		{
			"valid module account",
			types.NewEmptyModuleAccount("test"),
			nil,
		},
		{
			"invalid name and address pair",
			types.NewModuleAccount(baseAcc, "test"),
			fmt.Errorf("address %s cannot be derived from the module name 'test'", addr),
		},
		{
			"empty module account name",
			types.NewModuleAccount(baseAcc, "    "),
			errors.New("module account name cannot be blank"),
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			err := tt.acc.Validate()
			require.Equal(t, tt.expErr, err)
		})
	}
}

func TestModuleAccountJSON(t *testing.T) {
	pubkey := secp256k1.GenPrivKey().PubKey()
	addr := sdk.AccAddress(pubkey.Address())
	baseAcc := types.NewBaseAccount(addr, nil, 10, 50)
	acc := types.NewModuleAccount(baseAcc, "test", "burner")

	bz, err := json.Marshal(acc)
	require.NoError(t, err)

	bz1, err := acc.MarshalJSON()
	require.NoError(t, err)
	require.Equal(t, string(bz1), string(bz))

	var a types.ModuleAccount
	require.NoError(t, json.Unmarshal(bz, &a))
	require.Equal(t, acc.String(), a.String())
}

func TestGenesisAccountsContains(t *testing.T) {
	pubkey := secp256k1.GenPrivKey().PubKey()
	addr := sdk.AccAddress(pubkey.Address())
	acc := types.NewBaseAccount(addr, secp256k1.GenPrivKey().PubKey(), 0, 0)

	genAccounts := types.GenesisAccounts{}
	require.False(t, genAccounts.Contains(acc.GetAddress()))

	genAccounts = append(genAccounts, acc)
	require.True(t, genAccounts.Contains(acc.GetAddress()))
}

func TestNewModuleAddressOrBech32Address(t *testing.T) {
	input := "cosmos1cwwv22j5ca08ggdv9c2uky355k908694z577tv"
	require.Equal(t, input, types.NewModuleAddressOrBech32Address(input).String())
	require.Equal(t, "cosmos1jv65s3grqf6v6jl3dp4t6c9t9rk99cd88lyufl", types.NewModuleAddressOrBech32Address("distribution").String())
}

func TestModuleAccountValidateNilBaseAccount(t *testing.T) {
	ma := &types.ModuleAccount{Name: "foo"}
	_ = ma.Validate()
}
