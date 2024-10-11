//go:build ledger || test_ledger_mock
// +build ledger test_ledger_mock

package keyring

import (
	"bytes"
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	"github.com/cosmos/cosmos-sdk/crypto/ledger"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/tx/signing"
)

func TestInMemoryCreateLedger(t *testing.T) {
	cdc := getCodec()
	kb := NewInMemory(cdc)

	k, err := kb.SaveLedgerKey("some_account", hd.Secp256k1, "cosmos", 118, 3, 1)
	if err != nil {
		require.Error(t, err)
		require.Equal(t, "ledger nano S: support for ledger devices is not available in this executable", err.Error())
		require.Nil(t, k)
		t.Skip("ledger nano S: support for ledger devices is not available in this executable")
		return
	}

	// The mock is available, check that the address is correct
	pubKey, err := k.GetPubKey()
	require.NoError(t, err)
	expectedPkStr := "PubKeySecp256k1{03602C0CB4D8C0081FEE794BDE96E7B95FA16F2B5283B764AC070584327B2C7202}"
	require.Equal(t, expectedPkStr, pubKey.String())

	// Check that restoring the key gets the same results
	restoredRecord, err := kb.Key("some_account")
	require.NoError(t, err)
	require.NotNil(t, restoredRecord)
	require.Equal(t, "some_account", restoredRecord.Name)
	pubKey, err = restoredRecord.GetPubKey()
	require.NoError(t, err)
	require.Equal(t, expectedPkStr, pubKey.String())

	ledgerInfo := restoredRecord.GetLedger()
	require.NotNil(t, ledgerInfo)
	path := ledgerInfo.GetPath()
	require.Equal(t, "m/44'/118'/3'/0/1", path.String())
}

// TestSignVerify does some detailed checks on how we sign and validate
// signatures
func TestSignVerifyKeyRingWithLedger(t *testing.T) {
	dir := t.TempDir()
	cdc := getCodec()

	kb, err := New("keybasename", "test", dir, nil, cdc)
	require.NoError(t, err)

	k, err := kb.SaveLedgerKey("key", hd.Secp256k1, "cosmos", 118, 0, 0)
	if err != nil {
		require.Equal(t, "ledger nano S: support for ledger devices is not available in this executable", err.Error())
		t.Skip("ledger nano S: support for ledger devices is not available in this executable")
		return
	}
	require.Equal(t, "key", k.Name)

	d1 := []byte("my first message")
	s1, pub1, err := kb.Sign("key", d1, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	require.NoError(t, err)

	s2, pub2, err := SignWithLedger(k, d1, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	require.NoError(t, err)

	require.True(t, pub1.Equals(pub2))
	require.True(t, bytes.Equal(s1, s2))

	key1, err := k.GetPubKey()
	require.NoError(t, err)

	require.Equal(t, key1, pub1)
	require.Equal(t, key1, pub2)
	require.True(t, pub1.VerifySignature(d1, s1))
	require.True(t, key1.VerifySignature(d1, s1))
	require.True(t, bytes.Equal(s1, s2))

	k, _, err = kb.NewMnemonic("test", English, types.FullFundraiserPath, DefaultBIP39Passphrase, hd.Secp256k1)
	require.NoError(t, err)
	_, _, err = SignWithLedger(k, d1, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
	require.Error(t, err)
	require.Equal(t, "not a ledger object", err.Error())
}

func TestAltKeyring_SaveLedgerKey(t *testing.T) {
	dir := t.TempDir()
	cdc := getCodec()

	kr, err := New(t.Name(), BackendTest, dir, nil, cdc)
	require.NoError(t, err)

	// Test unsupported Algo
	_, err = kr.SaveLedgerKey("key", notSupportedAlgo{}, "cosmos", 118, 0, 0)
	require.True(t, errors.Is(err, ErrUnsupportedSigningAlgo))

	k, err := kr.SaveLedgerKey("some_account", hd.Secp256k1, "cosmos", 118, 3, 1)
	if err != nil {
		require.Equal(t, "ledger nano S: support for ledger devices is not available in this executable", err.Error())
		t.Skip("ledger nano S: support for ledger devices is not available in this executable")
		return
	}

	// The mock is available, check that the address is correct
	require.Equal(t, "some_account", k.Name)
	pubKey, err := k.GetPubKey()
	require.NoError(t, err)
	expectedPkStr := "PubKeySecp256k1{03602C0CB4D8C0081FEE794BDE96E7B95FA16F2B5283B764AC070584327B2C7202}"
	require.Equal(t, expectedPkStr, pubKey.String())

	// Check that restoring the key gets the same results
	restoredRecord, err := kr.Key("some_account")
	require.NoError(t, err)
	require.NotNil(t, restoredRecord)
	require.Equal(t, "some_account", restoredRecord.Name)
	//	require.Equal(t, TypeLedger, restoredRecord.GetType())
	pubKey, err = restoredRecord.GetPubKey()
	require.NoError(t, err)
	require.Equal(t, expectedPkStr, pubKey.String())

	ledgerInfo := k.GetLedger()
	require.NotNil(t, ledgerInfo)

	path := ledgerInfo.GetPath()
	require.Equal(t, "m/44'/118'/3'/0/1", path.String())
}

func TestSignWithLedger(t *testing.T) {
	// Create two distinct Ledger records: recordA and recordB.
	// RecordA is added to the Ledger but recordB is not added.
	pathA := hd.NewFundraiserParams(0, types.CoinType, 0)
	privA, _, err := ledger.NewPrivKeySecp256k1(*pathA, "cosmos")
	require.NoError(t, err)
	recordA, err := NewLedgerRecord("ledgerA", privA.PubKey(), pathA)
	require.NoError(t, err)
	pubA, err := recordA.GetPubKey()
	require.NoError(t, err)

	pathB := hd.NewFundraiserParams(0, types.CoinType, 1)
	// privB won't be added to the Ledger because it doesn't use ledger.NewPrivKeySecp256k1
	privB := secp256k1.GenPrivKey()
	recordB, err := NewLedgerRecord("ledgerB", privB.PubKey(), pathB)
	require.NoError(t, err)
	pubB, err := recordB.GetPubKey()
	require.NoError(t, err)

	require.NotEqual(t, pubA, pubB)
	type testCase struct {
		name            string
		record          *Record
		msg             []byte
		wantSig         []byte
		wantPub         cryptotypes.PubKey
		wantErr         bool
		wantErrContains string
	}
	testCases := []testCase{
		{
			name:    "ordinary ledger tx",
			record:  recordA,
			msg:     []byte("msg"),
			wantSig: []byte{0xfb, 0x93, 0x1b, 0xb9, 0x75, 0x25, 0xe7, 0x99, 0x64, 0xc2, 0x78, 0xf7, 0x94, 0x9a, 0x63, 0x83, 0xe2, 0x59, 0x76, 0x48, 0x1d, 0x2, 0xbc, 0xc2, 0x83, 0x21, 0x24, 0x4b, 0x95, 0x99, 0x25, 0x8b, 0x30, 0x38, 0x6, 0x61, 0x79, 0x9a, 0x9e, 0x8, 0x98, 0xfd, 0x34, 0xc6, 0x7e, 0x47, 0x4d, 0x5f, 0xe, 0xf3, 0xc3, 0xe7, 0xdd, 0xe3, 0x89, 0x80, 0xda, 0x8b, 0x48, 0x15, 0x34, 0xce, 0xdf, 0x1c},
			wantPub: pubA,
			wantErr: false,
		},
		{
			name:            "want error when the public key the user attempted to sign with doesn't match the public key on the ledger",
			record:          recordB,
			msg:             []byte("msg"),
			wantSig:         []byte(nil),
			wantPub:         nil,
			wantErr:         true,
			wantErrContains: "the public key that the user attempted to sign with does not match the public key on the ledger device",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			sig, pub, err := SignWithLedger(tc.record, tc.msg, signing.SignMode_SIGN_MODE_LEGACY_AMINO_JSON)
			assert.Equal(t, tc.wantSig, sig)
			assert.Equal(t, tc.wantPub, pub)
			if tc.wantErr {
				assert.Error(t, err)
				assert.Contains(t, err.Error(), tc.wantErrContains)
			}
		})
	}
}
