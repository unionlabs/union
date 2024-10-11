package keys

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	cryptocodec "github.com/cosmos/cosmos-sdk/crypto/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	kmultisig "github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

func generatePubKeys(n int) []types.PubKey {
	pks := make([]types.PubKey, n)
	for i := 0; i < n; i++ {
		pks[i] = secp256k1.GenPrivKey().PubKey()
	}
	return pks
}

func TestBech32KeysOutput(t *testing.T) {
	sk := secp256k1.PrivKey{Key: []byte{154, 49, 3, 117, 55, 232, 249, 20, 205, 216, 102, 7, 136, 72, 177, 2, 131, 202, 234, 81, 31, 208, 46, 244, 179, 192, 167, 163, 142, 117, 246, 13}}
	tmpKey := sk.PubKey()
	multisigPk := kmultisig.NewLegacyAminoPubKey(1, []types.PubKey{tmpKey})

	k, err := keyring.NewMultiRecord("multisig", multisigPk)
	require.NotNil(t, k)
	require.NoError(t, err)
	pubKey, err := k.GetPubKey()
	require.NoError(t, err)
	accAddr := sdk.AccAddress(pubKey.Address())
	expectedOutput, err := NewKeyOutput(k.Name, k.GetType(), accAddr, multisigPk, addresscodec.NewBech32Codec("cosmos"))
	require.NoError(t, err)

	out, err := MkAccKeyOutput(k, addresscodec.NewBech32Codec("cosmos"))
	require.NoError(t, err)
	require.Equal(t, expectedOutput, out)
	require.Equal(t, "{Name:multisig Type:multi Address:cosmos1nf8lf6n4wa43rzmdzwe6hkrnw5guekhqt595cw PubKey:{\"@type\":\"/cosmos.crypto.multisig.LegacyAminoPubKey\",\"threshold\":1,\"public_keys\":[{\"@type\":\"/cosmos.crypto.secp256k1.PubKey\",\"key\":\"AurroA7jvfPd1AadmmOvWM2rJSwipXfRf8yD6pLbA2DJ\"}]} Mnemonic:}", fmt.Sprintf("%+v", out))
}

// TestBech32KeysOutputNestedMsig tests that the output of a nested multisig key is correct
func TestBech32KeysOutputNestedMsig(t *testing.T) {
	sk := secp256k1.PrivKey{Key: []byte{154, 49, 3, 117, 55, 232, 249, 20, 205, 216, 102, 7, 136, 72, 177, 2, 131, 202, 234, 81, 31, 208, 46, 244, 179, 192, 167, 163, 142, 117, 246, 13}}
	tmpKey := sk.PubKey()
	nestedMultiSig := kmultisig.NewLegacyAminoPubKey(1, []types.PubKey{tmpKey})
	multisigPk := kmultisig.NewLegacyAminoPubKey(2, []types.PubKey{tmpKey, nestedMultiSig})
	k, err := keyring.NewMultiRecord("multisig", multisigPk)
	require.NotNil(t, k)
	require.NoError(t, err)

	pubKey, err := k.GetPubKey()
	require.NoError(t, err)

	accAddr := sdk.AccAddress(pubKey.Address())
	expectedOutput, err := NewKeyOutput(k.Name, k.GetType(), accAddr, multisigPk, addresscodec.NewBech32Codec("cosmos"))
	require.NoError(t, err)

	out, err := MkAccKeyOutput(k, addresscodec.NewBech32Codec("cosmos"))
	require.NoError(t, err)

	require.Equal(t, expectedOutput, out)
	require.Equal(t, "{Name:multisig Type:multi Address:cosmos1nffp6v2j7wva4y4975exlrv8x5vh39axxt3swz PubKey:{\"@type\":\"/cosmos.crypto.multisig.LegacyAminoPubKey\",\"threshold\":2,\"public_keys\":[{\"@type\":\"/cosmos.crypto.secp256k1.PubKey\",\"key\":\"AurroA7jvfPd1AadmmOvWM2rJSwipXfRf8yD6pLbA2DJ\"},{\"@type\":\"/cosmos.crypto.multisig.LegacyAminoPubKey\",\"threshold\":1,\"public_keys\":[{\"@type\":\"/cosmos.crypto.secp256k1.PubKey\",\"key\":\"AurroA7jvfPd1AadmmOvWM2rJSwipXfRf8yD6pLbA2DJ\"}]}]} Mnemonic:}", fmt.Sprintf("%+v", out))
}

func TestProtoMarshalJSON(t *testing.T) {
	require := require.New(t)
	pubkeys := generatePubKeys(3)
	msig := kmultisig.NewLegacyAminoPubKey(2, pubkeys)

	registry := codectypes.NewInterfaceRegistry()
	cryptocodec.RegisterInterfaces(registry)
	cdc := codec.NewProtoCodec(registry)

	bz, err := cdc.MarshalInterfaceJSON(msig)
	require.NoError(err)

	var pk2 types.PubKey
	err = cdc.UnmarshalInterfaceJSON(bz, &pk2)
	require.NoError(err)
	require.True(pk2.Equals(msig))

	addressCodec := addresscodec.NewBech32Codec("cosmos")

	// Test that we can correctly unmarshal key from output
	k, err := keyring.NewMultiRecord("my multisig", msig)
	require.NoError(err)
	ko, err := MkAccKeyOutput(k, addressCodec)
	require.NoError(err)

	expectedOutput, err := addressCodec.BytesToString(pk2.Address())
	require.NoError(err)

	require.Equal(ko.Address, expectedOutput)
	require.Equal(ko.PubKey, string(bz))
}

func TestNestedMultisigOutput(t *testing.T) {
	cdc := moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec

	sk := secp256k1.PrivKey{Key: []byte{154, 49, 3, 117, 55, 232, 249, 20, 205, 216, 102, 7, 136, 72, 177, 2, 131, 202, 234, 81, 31, 208, 46, 244, 179, 192, 167, 163, 142, 117, 246, 13}}
	tmpKey := sk.PubKey()
	multisigPk := kmultisig.NewLegacyAminoPubKey(1, []types.PubKey{tmpKey})
	multisigPk2 := kmultisig.NewLegacyAminoPubKey(1, []types.PubKey{tmpKey, multisigPk})

	kb, err := keyring.New(t.Name(), keyring.BackendTest, t.TempDir(), nil, cdc)
	require.NoError(t, err)

	_, err = kb.SaveMultisig("multisig", multisigPk2)
	require.NoError(t, err)

	k, err := kb.Key("multisig")
	require.NoError(t, err)

	_, err = MkAccKeyOutput(k, addresscodec.NewBech32Codec("cosmos"))
	require.NoError(t, err)
}
