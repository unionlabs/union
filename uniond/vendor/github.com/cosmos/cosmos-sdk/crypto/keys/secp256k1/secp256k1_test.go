package secp256k1_test

import (
	"crypto/ecdsa"
	"encoding/base64"
	"encoding/hex"
	"math/big"
	"testing"

	btcSecp256k1 "github.com/btcsuite/btcd/btcec/v2"
	btcecdsa "github.com/btcsuite/btcd/btcec/v2/ecdsa"
	"github.com/cometbft/cometbft/crypto"
	tmsecp256k1 "github.com/cometbft/cometbft/crypto/secp256k1"
	"github.com/cosmos/btcutil/base58"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/codec"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

type keyData struct {
	priv string
	pub  string
	addr string
}

var secpDataTable = []keyData{
	{
		priv: "a96e62ed3955e65be32703f12d87b6b5cf26039ecfa948dc5107a495418e5330",
		pub:  "02950e1cdfcb133d6024109fd489f734eeb4502418e538c28481f22bce276f248c",
		addr: "1CKZ9Nx4zgds8tU7nJHotKSDr4a9bYJCa3",
	},
}

func TestPubKeySecp256k1Address(t *testing.T) {
	for _, d := range secpDataTable {
		privB, _ := hex.DecodeString(d.priv)
		pubB, _ := hex.DecodeString(d.pub)
		addrBbz, _, _ := base58.CheckDecode(d.addr)
		addrB := crypto.Address(addrBbz)

		priv := secp256k1.PrivKey{Key: privB}

		pubKey := priv.PubKey()
		pubT, _ := pubKey.(*secp256k1.PubKey)

		addr := pubKey.Address()
		assert.Equal(t, pubT, &secp256k1.PubKey{Key: pubB}, "Expected pub keys to match")
		assert.Equal(t, addr, addrB, "Expected addresses to match")
	}
}

func TestSignAndValidateSecp256k1(t *testing.T) {
	privKey := secp256k1.GenPrivKey()
	pubKey := privKey.PubKey()

	msg := crypto.CRandBytes(1000)
	sig, err := privKey.Sign(msg)
	require.Nil(t, err)
	assert.True(t, pubKey.VerifySignature(msg, sig))

	// ----
	// Test cross packages verification
	msgHash := crypto.Sha256(msg)
	btcPrivKey, btcPubKey := btcSecp256k1.PrivKeyFromBytes(privKey.Key)
	// This fails: malformed signature: no header magic
	//   btcSig, err := secp256k1.ParseSignature(sig, secp256k1.S256())
	//   require.NoError(t, err)
	//   assert.True(t, btcSig.Verify(msgHash, btcPubKey))
	// So we do a hacky way:
	r := new(big.Int)
	s := new(big.Int)
	r.SetBytes(sig[:32])
	s.SetBytes(sig[32:])
	ok := ecdsa.Verify(btcPubKey.ToECDSA(), msgHash, r, s)
	require.True(t, ok)

	sig2, err := btcecdsa.SignCompact(btcPrivKey, msgHash, false)
	// Chop off compactSigRecoveryCode.
	sig2 = sig2[1:]
	require.NoError(t, err)
	pubKey.VerifySignature(msg, sig2)

	// ----
	// Mutate the signature, just one bit.
	sig[3] ^= byte(0x01)
	assert.False(t, pubKey.VerifySignature(msg, sig))
}

// This test is intended to justify the removal of calls to the underlying library
// in creating the privkey.
func TestSecp256k1LoadPrivkeyAndSerializeIsIdentity(t *testing.T) {
	numberOfTests := 256
	for i := 0; i < numberOfTests; i++ {
		// Seed the test case with some random bytes
		privKeyBytes := [32]byte{}
		copy(privKeyBytes[:], crypto.CRandBytes(32))

		// This function creates a private and public key in the underlying libraries format.
		// The private key is basically calling new(big.Int).SetBytes(pk), which removes leading zero bytes
		priv, _ := btcSecp256k1.PrivKeyFromBytes(privKeyBytes[:])
		// this takes the bytes returned by `(big int).Bytes()`, and if the length is less than 32 bytes,
		// pads the bytes from the left with zero bytes. Therefore these two functions composed
		// result in the identity function on privKeyBytes, hence the following equality check
		// always returning true.
		serializedBytes := priv.Serialize()
		require.Equal(t, privKeyBytes[:], serializedBytes)
	}
}

func TestGenPrivKeyFromSecret(t *testing.T) {
	// curve oder N
	N := btcSecp256k1.S256().N
	tests := []struct {
		name   string
		secret []byte
	}{
		{"empty secret", []byte{}},
		{
			"some long secret",
			[]byte("We live in a society exquisitely dependent on science and technology, " +
				"in which hardly anyone knows anything about science and technology."),
		},
		{"another seed used in cosmos tests #1", []byte{0}},
		{"another seed used in cosmos tests #2", []byte("mySecret")},
		{"another seed used in cosmos tests #3", []byte("")},
	}
	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			gotPrivKey := secp256k1.GenPrivKeyFromSecret(tt.secret)
			require.NotNil(t, gotPrivKey)
			// interpret as a big.Int and make sure it is a valid field element:
			fe := new(big.Int).SetBytes(gotPrivKey.Key[:])
			require.True(t, fe.Cmp(N) < 0)
			require.True(t, fe.Sign() > 0)
		})
	}
}

func TestPubKeyEquals(t *testing.T) {
	secp256K1PubKey := secp256k1.GenPrivKey().PubKey().(*secp256k1.PubKey)

	testCases := []struct {
		msg      string
		pubKey   cryptotypes.PubKey
		other    cryptotypes.PubKey
		expectEq bool
	}{
		{
			"different bytes",
			secp256K1PubKey,
			secp256k1.GenPrivKey().PubKey(),
			false,
		},
		{
			"equals",
			secp256K1PubKey,
			&secp256k1.PubKey{
				Key: secp256K1PubKey.Key,
			},
			true,
		},
		{
			"different types",
			secp256K1PubKey,
			ed25519.GenPrivKey().PubKey(),
			false,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.msg, func(t *testing.T) {
			eq := tc.pubKey.Equals(tc.other)
			require.Equal(t, eq, tc.expectEq)
		})
	}
}

func TestPrivKeyEquals(t *testing.T) {
	secp256K1PrivKey := secp256k1.GenPrivKey()

	testCases := []struct {
		msg      string
		privKey  cryptotypes.PrivKey
		other    cryptotypes.PrivKey
		expectEq bool
	}{
		{
			"different bytes",
			secp256K1PrivKey,
			secp256k1.GenPrivKey(),
			false,
		},
		{
			"equals",
			secp256K1PrivKey,
			&secp256k1.PrivKey{
				Key: secp256K1PrivKey.Key,
			},
			true,
		},
		{
			"different types",
			secp256K1PrivKey,
			ed25519.GenPrivKey(),
			false,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.msg, func(t *testing.T) {
			eq := tc.privKey.Equals(tc.other)
			require.Equal(t, eq, tc.expectEq)
		})
	}
}

func TestMarshalAmino(t *testing.T) {
	aminoCdc := codec.NewLegacyAmino()
	privKey := secp256k1.GenPrivKey()
	pubKey := privKey.PubKey().(*secp256k1.PubKey)

	testCases := []struct {
		desc      string
		msg       codec.AminoMarshaler
		typ       interface{}
		expBinary []byte
		expJSON   string
	}{
		{
			"secp256k1 private key",
			privKey,
			&secp256k1.PrivKey{},
			append([]byte{32}, privKey.Bytes()...), // Length-prefixed.
			"\"" + base64.StdEncoding.EncodeToString(privKey.Bytes()) + "\"",
		},
		{
			"secp256k1 public key",
			pubKey,
			&secp256k1.PubKey{},
			append([]byte{33}, pubKey.Bytes()...), // Length-prefixed.
			"\"" + base64.StdEncoding.EncodeToString(pubKey.Bytes()) + "\"",
		},
	}

	for _, tc := range testCases {
		t.Run(tc.desc, func(t *testing.T) {
			// Do a round trip of encoding/decoding binary.
			bz, err := aminoCdc.Marshal(tc.msg)
			require.NoError(t, err)
			require.Equal(t, tc.expBinary, bz)

			err = aminoCdc.Unmarshal(bz, tc.typ)
			require.NoError(t, err)

			require.Equal(t, tc.msg, tc.typ)

			// Do a round trip of encoding/decoding JSON.
			bz, err = aminoCdc.MarshalJSON(tc.msg)
			require.NoError(t, err)
			require.Equal(t, tc.expJSON, string(bz))

			err = aminoCdc.UnmarshalJSON(bz, tc.typ)
			require.NoError(t, err)

			require.Equal(t, tc.msg, tc.typ)
		})
	}
}

func TestMarshalAmino_BackwardsCompatibility(t *testing.T) {
	aminoCdc := codec.NewLegacyAmino()
	// Create Tendermint keys.
	tmPrivKey := tmsecp256k1.GenPrivKey()
	tmPubKey := tmPrivKey.PubKey()
	// Create our own keys, with the same private key as Tendermint's.
	privKey := &secp256k1.PrivKey{Key: []byte(tmPrivKey)}
	pubKey := privKey.PubKey().(*secp256k1.PubKey)

	testCases := []struct {
		desc      string
		tmKey     interface{}
		ourKey    interface{}
		marshalFn func(o interface{}) ([]byte, error)
	}{
		{
			"secp256k1 private key, binary",
			tmPrivKey,
			privKey,
			aminoCdc.Marshal,
		},
		{
			"secp256k1 private key, JSON",
			tmPrivKey,
			privKey,
			aminoCdc.MarshalJSON,
		},
		{
			"secp256k1 public key, binary",
			tmPubKey,
			pubKey,
			aminoCdc.Marshal,
		},
		{
			"secp256k1 public key, JSON",
			tmPubKey,
			pubKey,
			aminoCdc.MarshalJSON,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.desc, func(t *testing.T) {
			// Make sure Amino encoding override is not breaking backwards compatibility.
			bz1, err := tc.marshalFn(tc.tmKey)
			require.NoError(t, err)
			bz2, err := tc.marshalFn(tc.ourKey)
			require.NoError(t, err)
			require.Equal(t, bz1, bz2)
		})
	}
}
