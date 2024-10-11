package secp256k1_test

import (
	"crypto/ecdsa"
	"encoding/base64"
	"encoding/hex"
	"math/big"
	"testing"

	"github.com/cometbft/cometbft/crypto"
	tmsecp256k1 "github.com/cometbft/cometbft/crypto/secp256k1"
	"github.com/cosmos/btcutil/base58"
	secp "github.com/decred/dcrd/dcrec/secp256k1/v4"
	btcecdsa "github.com/decred/dcrd/dcrec/secp256k1/v4/ecdsa"
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

/*
The following code snippet has been used to generate test vectors. The purpose of these vectors are to check our
implementation of secp256k1 against go-ethereum's one. It has been commented to avoid dependencies.

	github.com/btcsuite/btcutil v1.0.2
	github.com/ethereum/go-ethereum v1.10.26
	golang.org/x/crypto v0.0.0-20210921155107-089bfa567519

---

	import (
		"crypto/ecdsa"
		"crypto/sha256"
		"encoding/hex"
		"fmt"
		"github.com/btcsuite/btcutil/base58"
		"github.com/ethereum/go-ethereum/crypto"
		"golang.org/x/crypto/ripemd160"
	)

	func ethereumKeys() keyData {
		// Generate private key with the go-ethereum
		priv, err := crypto.GenerateKey()
		if err != nil {
			panic(err)
		}
		encPriv := make([]byte, len(priv.D.Bytes())*2)
		hex.Encode(encPriv, priv.D.Bytes())

		// Get go-ethereum public key
		ethPub, ok := priv.Public().(*ecdsa.PublicKey)
		if !ok {
			panic(err)
		}
		ethPublicKeyBytes := crypto.FromECDSAPub(ethPub)

		// Format byte depending on the oddness of the Y coordinate.
		format := 0x02
		if ethPub.Y.Bit(0) != 0 {
			format = 0x03
		}

		// Public key in the 33-byte compressed format.
		pub := ethPublicKeyBytes[:33]
		encPub := make([]byte, len(pub)*2)
		pub[0] = byte(format)
		hex.Encode(encPub, pub)

		//  Bitcoin style addresses
		sha := sha256.Sum256(pub)
		hasherRIPEMD160 := ripemd160.New()
		hasherRIPEMD160.Write(sha[:])
		addr := hasherRIPEMD160.Sum(nil)
		return keyData{
			priv: string(encPriv),
			pub: string(encPub),
			addr: base58.CheckEncode(addr[:], 0),
		}
	}
*/

/*
generateKeyForCheckingConsistency was used to create test vectors that matches consistency against prior versions.
Here are the specific versions used to generate the vectors.

github.com/cosmos/btcutil v1.0.5
github.com/cosmos/cosmos-sdk v0.46.8
*/
var _ = func() keyData {
	priv := secp256k1.GenPrivKey()
	encPriv := make([]byte, len(priv.Key)*2)
	hex.Encode(encPriv, priv.Key)
	pub := priv.PubKey()
	encPub := make([]byte, len(pub.Bytes())*2)
	hex.Encode(encPub, pub.Bytes())
	addr := pub.Address()
	return keyData{
		priv: string(encPriv),
		pub:  string(encPub),
		addr: base58.CheckEncode(addr, 0),
	}
}

var secpDataTable = []keyData{
	{
		priv: "a96e62ed3955e65be32703f12d87b6b5cf26039ecfa948dc5107a495418e5330",
		pub:  "02950e1cdfcb133d6024109fd489f734eeb4502418e538c28481f22bce276f248c",
		addr: "1CKZ9Nx4zgds8tU7nJHotKSDr4a9bYJCa3",
	},
	// matches consistency against a prior version of this library. Generated with generateKeyForCheckingConsistency
	{
		priv: "9af074dc32fe3e7173802cd72dcb1110582879a1990c90bdac60f2739986aa06",
		pub:  "0285592121e2a5e0eb970a1a9d1879c5fa7b33badf7dbb61c44b1bfced94649efb",
		addr: "1Q4mWVk2hotRVDEdGGtGf6waz622rEwvib",
	},
	// matches consistency against a prior version of this library. Generated with generateKeyForCheckingConsistency
	{
		priv: "ef9edc836bc4d47e9bc3cfab446836a737c41d60abb1d5f76a6d53ffe5b35f76",
		pub:  "02f5bf88d72172cc2f9a52919b6b1b74a01ca606cad75d5f4f93aa1a6ff0374aaf",
		addr: "1KtiSApteeKdLi5cdZVpnkNW1t5Eteksvf",
	},
	// matches consistency against a prior version of this library. Generated with generateKeyForCheckingConsistency
	{
		priv: "ab7715a1dd7cea7898c45b1f291550b83a6897fbdf0ec48330dd50187059b74b",
		pub:  "028f3003b3e6cb40897138dba5858207357a6d116cc5bf556c942cf6081b58d5fe",
		addr: "RnM1o5grgCHAmm45wt5vzGsQoCJdPK2n2",
	},
	// matches consistency against a prior version of this library. Generated with generateKeyForCheckingConsistency
	{
		priv: "db6b914d9a2d6ae4bab8f9b43de3b1e83940e1a309521128b13fdaf3cd15009a",
		pub:  "022f8e4e07ae2705a3c425eafea16027041bcdc87a193b01ea6c36c1c7a0bfc300",
		addr: "16MpKTksSpGABuHqMqU9RPBz26DfwY8cLY",
	},
	// matches consistency against go-ethereum's implementation. Generated with ethereumKeys
	{
		priv: "42ba4249f6fd9f1e31f8876a8d3d3bdef989fcc906164290c0be237f69f53718",
		pub:  "033c2f6ea7a678f0afbb43d0fe7a2b2706a75c2fdea08c3b90fd038c8219b42959",
		addr: "18iz5wdTdwzq6cGzoVhooZCPRAx61GfUMR",
	},
	// matches consistency against go-ethereum's implementation. Generated with ethereumKeys
	{
		priv: "86192b60369616574daabe8d7d6067f14ec3f0648cded5633c566c25c48e1f31",
		pub:  "03ad9e97842d0f6f57804f29f55aac9bba207d2b24b98aaabc7d106250389e6d46",
		addr: "1K31NqmdMBZiLeUiP4kfjLNnWSmx17a9aE",
	},
	// matches consistency against go-ethereum's implementation. Generated with ethereumKeys
	{
		priv: "1856b3a581aa1bf83daf61b1f8f4bb52b5223033f710e61d7e0b3086f48ba09a",
		pub:  "03d681bb11e5ebc14d5d2f72881cb0b2a693ef12bc72fe863f980fc6542eafbd40",
		addr: "1K29nsfH6qwmE3MzsoHpLcWLA4mQLstGgx",
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
	btcPrivKey := secp.PrivKeyFromBytes(privKey.Key)
	btcPubKey := btcPrivKey.PubKey()
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

	sig2 := btcecdsa.SignCompact(btcPrivKey, msgHash, false)
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
		priv := secp.PrivKeyFromBytes(privKeyBytes[:])
		// this takes the bytes returned by `(big int).Bytes()`, and if the length is less than 32 bytes,
		// pads the bytes from the left with zero bytes. Therefore these two functions composed
		// result in the identity function on privKeyBytes, hence the following equality check
		// always returning true.
		serializedBytes := priv.Serialize()
		require.Equal(t, privKeyBytes[:], serializedBytes)
	}
}

func TestGenPrivKeyFromSecret(t *testing.T) {
	// curve order N
	N := secp.S256().N
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
		t.Run(tt.name, func(t *testing.T) {
			gotPrivKey := secp256k1.GenPrivKeyFromSecret(tt.secret)
			require.NotNil(t, gotPrivKey)
			// interpret as a big.Int and make sure it is a valid field element:
			fe := new(big.Int).SetBytes(gotPrivKey.Key)
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

func TestLegacyKeyGenerationAgainstConstantTime(t *testing.T) {
	privKey := secp256k1.GenPrivKey()

	pubKey := privKey.PubKey()

	nonConstantTimePk := secp.PrivKeyFromBytes(privKey.Key).PubKey().SerializeCompressed() // Legacy functionality from pubkey
	legacyPubKey := &secp256k1.PubKey{Key: nonConstantTimePk}

	require.Equal(t, legacyPubKey, pubKey)
}
