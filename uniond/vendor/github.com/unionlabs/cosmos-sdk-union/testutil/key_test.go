package testutil

import (
	"testing"

	"github.com/stretchr/testify/require"

	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/module/testutil"
)

func TestGenerateCoinKey(t *testing.T) {
	t.Parallel()
	cdc := testutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec
	addr, mnemonic, err := GenerateCoinKey(hd.Secp256k1, cdc)
	require.NoError(t, err)

	// Test creation
	k, err := keyring.NewInMemory(cdc).NewAccount("xxx", mnemonic, "", hd.NewFundraiserParams(0, types.CoinType, 0).String(), hd.Secp256k1)
	require.NoError(t, err)
	addr1, err := k.GetAddress()
	require.NoError(t, err)
	require.Equal(t, addr, addr1)
}

func TestGenerateSaveCoinKey(t *testing.T) {
	t.Parallel()

	encCfg := testutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})
	kb, err := keyring.New(t.Name(), "test", t.TempDir(), nil, encCfg.Codec)
	require.NoError(t, err)

	addr, mnemonic, err := GenerateSaveCoinKey(kb, "keyname", "", false, hd.Secp256k1, types.GetFullBIP44Path())
	require.NoError(t, err)

	// Test key was actually saved
	k, err := kb.Key("keyname")
	require.NoError(t, err)
	addr1, err := k.GetAddress()
	require.NoError(t, err)
	require.Equal(t, addr, addr1)

	// Test in-memory recovery
	k, err = keyring.NewInMemory(encCfg.Codec).NewAccount("xxx", mnemonic, "", hd.NewFundraiserParams(0, types.CoinType, 0).String(), hd.Secp256k1)
	require.NoError(t, err)
	addr1, err = k.GetAddress()
	require.NoError(t, err)
	require.Equal(t, addr, addr1)
}

func TestGenerateSaveCoinKeyOverwriteFlag(t *testing.T) {
	t.Parallel()

	encCfg := testutil.MakeTestEncodingConfig(codectestutil.CodecOptions{})
	kb, err := keyring.New(t.Name(), "test", t.TempDir(), nil, encCfg.Codec)
	require.NoError(t, err)

	keyname := "justakey"
	addr1, _, err := GenerateSaveCoinKey(kb, keyname, "", false, hd.Secp256k1, types.GetFullBIP44Path())
	require.NoError(t, err)

	// Test overwrite with overwrite=false
	_, _, err = GenerateSaveCoinKey(kb, keyname, "", false, hd.Secp256k1, types.GetFullBIP44Path())
	require.Error(t, err)

	// Test overwrite with overwrite=true
	addr2, _, err := GenerateSaveCoinKey(kb, keyname, "", true, hd.Secp256k1, types.GetFullBIP44Path())
	require.NoError(t, err)

	require.NotEqual(t, addr1, addr2)
}
