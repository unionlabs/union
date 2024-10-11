//go:build bls12381

package bls12381_test

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/bls12381"
)

func TestNewPrivateKeyFromBytes(t *testing.T) {
	privKey, err := bls12381.GenPrivKey()
	require.NoError(t, err)
	defer privKey.Zeroize()

	privKeyBytes := privKey.Bytes()
	privKey2, err := bls12381.NewPrivateKeyFromBytes(privKeyBytes)
	require.NoError(t, err)
	defer privKey2.Zeroize()

	assert.Equal(t, privKey, privKey2)

	_, err = bls12381.NewPrivateKeyFromBytes(crypto.CRandBytes(31))
	assert.Error(t, err)
}

func TestGenPrivateKey(t *testing.T) {
	privKey, err := bls12381.GenPrivKey()
	require.NoError(t, err)
	defer privKey.Zeroize()
	assert.NotNil(t, privKey)
}

func TestPrivKeyBytes(t *testing.T) {
	privKey, err := bls12381.GenPrivKey()
	require.NoError(t, err)
	defer privKey.Zeroize()

	privKeyBytes := privKey.Bytes()
	privKey2, err := bls12381.NewPrivateKeyFromBytes(privKeyBytes)
	require.NoError(t, err)
	defer privKey2.Zeroize()

	assert.Equal(t, privKey, privKey2)
}

func TestPrivKeyPubKey(t *testing.T) {
	privKey, err := bls12381.GenPrivKey()
	require.NoError(t, err)
	pubKey := privKey.PubKey()
	assert.NotNil(t, pubKey)
}

func TestPrivKeyType(t *testing.T) {
	privKey, err := bls12381.GenPrivKey()
	require.NoError(t, err)
	defer privKey.Zeroize()

	assert.Equal(t, "bls12_381", privKey.Type())
}

func TestPrivKeySignAndPubKeyVerifySignature(t *testing.T) {
	privKey, err := bls12381.GenPrivKey()
	require.NoError(t, err)
	defer privKey.Zeroize()
	pubKey := privKey.PubKey()

	msg := crypto.CRandBytes(32)
	sig, err := privKey.Sign(msg)
	require.NoError(t, err)

	// Test the signature
	assert.True(t, pubKey.VerifySignature(msg, sig))

	// Mutate the signature, just one bit.
	// TODO: Replace this with a much better fuzzer, tendermint/ed25519/issues/10
	sig[7] ^= byte(0x01)

	assert.False(t, pubKey.VerifySignature(msg, sig))

	msg = crypto.CRandBytes(192)
	sig, err = privKey.Sign(msg)
	require.NoError(t, err)

	// Test the signature
	assert.True(t, pubKey.VerifySignature(msg, sig))
}

func TestPubKey(t *testing.T) {
	privKey, err := bls12381.GenPrivKey()
	require.NoError(t, err)
	defer privKey.Zeroize()
	pubKey := privKey.PubKey()
	assert.NotNil(t, pubKey)
}

func TestPubKeyType(t *testing.T) {
	privKey, err := bls12381.GenPrivKey()
	require.NoError(t, err)
	defer privKey.Zeroize()
	pubKey := privKey.PubKey()

	assert.Equal(t, "bls12_381", pubKey.Type())
}

func TestConst(t *testing.T) {
	privKey, err := bls12381.GenPrivKey()
	require.NoError(t, err)
	defer privKey.Zeroize()
	assert.Equal(t, bls12381.PrivKeySize, len(privKey.Bytes()))

	pubKey := privKey.PubKey()
	assert.Equal(t, bls12381.PubKeySize, len(pubKey.Bytes()))

	msg := crypto.CRandBytes(32)
	sig, err := privKey.Sign(msg)
	require.NoError(t, err)
	assert.Equal(t, bls12381.SignatureLength, len(sig))
}

func TestPrivKey_MarshalJSON(t *testing.T) {
	privKey, err := bls12381.GenPrivKey()
	require.NoError(t, err)
	defer privKey.Zeroize()

	jsonBytes, err := privKey.MarshalJSON()
	require.NoError(t, err)

	privKey2 := new(bls12381.PrivKey)
	err = privKey2.UnmarshalJSON(jsonBytes)
	require.NoError(t, err)
}

func TestPubKey_MarshalJSON(t *testing.T) {
	privKey, err := bls12381.GenPrivKey()
	require.NoError(t, err)
	defer privKey.Zeroize()
	pubKey, _ := privKey.PubKey().(*bls12381.PubKey)

	jsonBytes, err := pubKey.MarshalJSON()
	require.NoError(t, err)

	pubKey2 := new(bls12381.PubKey)
	err = pubKey2.UnmarshalJSON(jsonBytes)
	require.NoError(t, err)
}
