package encoding

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/bls12381"
	"github.com/cometbft/cometbft/crypto/ed25519"
	"github.com/cometbft/cometbft/crypto/secp256k1"
)

type unsupportedPubKey struct{}

func (unsupportedPubKey) Address() crypto.Address             { return nil }
func (unsupportedPubKey) Bytes() []byte                       { return nil }
func (unsupportedPubKey) VerifySignature([]byte, []byte) bool { return false }
func (unsupportedPubKey) Type() string                        { return "unsupportedPubKey" }

func TestPubKeyToFromProto(t *testing.T) {
	// ed25519
	pk := ed25519.GenPrivKey().PubKey()
	proto, err := PubKeyToProto(pk)
	require.NoError(t, err)

	pubkey, err := PubKeyFromProto(proto)
	require.NoError(t, err)
	assert.Equal(t, pk, pubkey)

	// secp256k1
	pk = secp256k1.GenPrivKey().PubKey()
	proto, err = PubKeyToProto(pk)
	require.NoError(t, err)

	pubkey, err = PubKeyFromProto(proto)
	require.NoError(t, err)
	assert.Equal(t, pk, pubkey)

	// bls12381
	if bls12381.Enabled {
		privKey, err := bls12381.GenPrivKey()
		require.NoError(t, err)
		defer privKey.Zeroize()
		pk = privKey.PubKey()
		proto, err := PubKeyToProto(pk)
		require.NoError(t, err)

		pubkey, err := PubKeyFromProto(proto)
		require.NoError(t, err)
		assert.Equal(t, pk, pubkey)
	} else {
		_, err = PubKeyToProto(bls12381.PubKey{})
		assert.Error(t, err)
	}

	// unsupported key type
	_, err = PubKeyToProto(unsupportedPubKey{})
	require.Error(t, err)
	assert.Equal(t, ErrUnsupportedKey{KeyType: unsupportedPubKey{}.Type()}, err)
}

func TestPubKeyFromTypeAndBytes(t *testing.T) {
	// ed25519
	pk := ed25519.GenPrivKey().PubKey()
	pubkey, err := PubKeyFromTypeAndBytes(pk.Type(), pk.Bytes())
	assert.NoError(t, err)
	assert.Equal(t, pk, pubkey)

	// ed25519 invalid size
	_, err = PubKeyFromTypeAndBytes(pk.Type(), pk.Bytes()[:10])
	assert.Error(t, err)

	// secp256k1
	pk = secp256k1.GenPrivKey().PubKey()
	pubkey, err = PubKeyFromTypeAndBytes(pk.Type(), pk.Bytes())
	assert.NoError(t, err)
	assert.Equal(t, pk, pubkey)

	// secp256k1 invalid size
	_, err = PubKeyFromTypeAndBytes(pk.Type(), pk.Bytes()[:10])
	assert.Error(t, err)

	// bls12381
	if bls12381.Enabled {
		privKey, err := bls12381.GenPrivKey()
		require.NoError(t, err)
		pk := privKey.PubKey()
		pubkey, err = PubKeyFromTypeAndBytes(pk.Type(), pk.Bytes())
		assert.NoError(t, err)
		assert.Equal(t, pk, pubkey)

		// bls12381 invalid size
		_, err = PubKeyFromTypeAndBytes(pk.Type(), pk.Bytes()[:10])
		assert.Error(t, err)
	} else {
		_, err = PubKeyFromTypeAndBytes(bls12381.KeyType, []byte{})
		assert.Error(t, err)
	}
}
