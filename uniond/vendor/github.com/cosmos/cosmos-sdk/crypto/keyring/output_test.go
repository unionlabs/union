package keyring

import (
	"fmt"
	"testing"

	"github.com/stretchr/testify/require"

	kmultisig "github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
)

func TestBech32KeysOutput(t *testing.T) {
	sk := secp256k1.PrivKey{Key: []byte{154, 49, 3, 117, 55, 232, 249, 20, 205, 216, 102, 7, 136, 72, 177, 2, 131, 202, 234, 81, 31, 208, 46, 244, 179, 192, 167, 163, 142, 117, 246, 13}}
	tmpKey := sk.PubKey()
	multisigPk := kmultisig.NewLegacyAminoPubKey(1, []types.PubKey{tmpKey})

	k, err := NewMultiRecord("multisig", multisigPk)
	require.NotNil(t, k)
	require.NoError(t, err)
	pubKey, err := k.GetPubKey()
	require.NoError(t, err)
	accAddr := sdk.AccAddress(pubKey.Address())
	expectedOutput, err := NewKeyOutput(k.Name, k.GetType(), accAddr, multisigPk)
	require.NoError(t, err)

	out, err := MkAccKeyOutput(k)
	require.NoError(t, err)
	require.Equal(t, expectedOutput, out)
	require.Equal(t, "{Name:multisig Type:multi Address:cosmos1nf8lf6n4wa43rzmdzwe6hkrnw5guekhqt595cw PubKey:{\"@type\":\"/cosmos.crypto.multisig.LegacyAminoPubKey\",\"threshold\":1,\"public_keys\":[{\"@type\":\"/cosmos.crypto.secp256k1.PubKey\",\"key\":\"AurroA7jvfPd1AadmmOvWM2rJSwipXfRf8yD6pLbA2DJ\"}]} Mnemonic:}", fmt.Sprintf("%+v", out))
}
