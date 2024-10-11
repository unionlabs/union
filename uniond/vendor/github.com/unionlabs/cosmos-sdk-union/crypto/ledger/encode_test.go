package ledger

import (
	"os"
	"testing"

	"github.com/stretchr/testify/require"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

func checkAminoJSON(t *testing.T, src, dst interface{}, isNil bool) {
	t.Helper()
	// Marshal to JSON bytes.
	js, err := cdc.MarshalJSON(src)
	require.Nil(t, err, "%+v", err)
	if isNil {
		require.Equal(t, string(js), `null`)
	} else {
		require.Contains(t, string(js), `"type":`)
		require.Contains(t, string(js), `"value":`)
	}
	// Unmarshal.
	err = cdc.UnmarshalJSON(js, dst)
	require.Nil(t, err, "%+v", err)
}

func ExamplePrintRegisteredTypes() { //nolint:govet // ignore for examples
	_ = cdc.PrintTypes(os.Stdout)
	// | Type | Name | Prefix | Length | Notes |
	// | ---- | ---- | ------ | ----- | ------ |
	// | PrivKeyLedgerSecp256k1 | tendermint/PrivKeyLedgerSecp256k1 | 0x10CAB393 | variable |  |
	// | PubKey | tendermint/PubKeyEd25519 | 0x1624DE64 | variable |  |
	// | PubKey | tendermint/PubKeySr25519 | 0x0DFB1005 | variable |  |
	// | PubKey | tendermint/PubKeySecp256k1 | 0xEB5AE987 | variable |  |
	// | PubKeyMultisigThreshold | tendermint/PubKeyMultisigThreshold | 0x22C1F7E2 | variable |  |
	// | PrivKey | tendermint/PrivKeyEd25519 | 0xA3288910 | variable |  |
	// | PrivKey | tendermint/PrivKeySr25519 | 0x2F82D78B | variable |  |
	// | PrivKey | tendermint/PrivKeySecp256k1 | 0xE1B0F79B | variable |  |
}

func TestNilEncodings(t *testing.T) {
	// Check nil Signature.
	var a, b []byte
	checkAminoJSON(t, &a, &b, true)
	require.EqualValues(t, a, b)

	// Check nil PubKey.
	var c, d cryptotypes.PubKey
	checkAminoJSON(t, &c, &d, true)
	require.EqualValues(t, c, d)

	// Check nil PrivKey.
	var e, f cryptotypes.PrivKey
	checkAminoJSON(t, &e, &f, true)
	require.EqualValues(t, e, f)
}
