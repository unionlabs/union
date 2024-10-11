package ecdsa

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/sha256"
	"math/big"
	"testing"

	"github.com/cometbft/cometbft/crypto"
	"github.com/stretchr/testify/suite"
)

func TestSKSuite(t *testing.T) {
	suite.Run(t, new(SKSuite))
}

type SKSuite struct{ CommonSuite }

func (suite *SKSuite) TestString() {
	const prefix = "abc"
	suite.Require().Equal(prefix+"{-}", suite.sk.String(prefix))
}

func (suite *SKSuite) TestPubKey() {
	pk := suite.sk.PubKey()
	suite.True(suite.sk.PublicKey.Equal(&pk.PublicKey))
}

func (suite *SKSuite) TestBytes() {
	bz := suite.sk.Bytes()
	suite.Len(bz, 32)
	var sk *PrivKey
	suite.Nil(sk.Bytes())
}

func (suite *SKSuite) TestMarshal() {
	require := suite.Require()
	const size = 32

	buffer := make([]byte, size)
	_, err := suite.sk.MarshalTo(buffer)
	require.NoError(err)

	sk := new(PrivKey)
	err = sk.Unmarshal(buffer, secp256r1, size)
	require.NoError(err)
	require.True(sk.Equal(&suite.sk.PrivateKey))
}

func (suite *SKSuite) TestSign() {
	require := suite.Require()

	msg := crypto.CRandBytes(1000)
	sig, err := suite.sk.Sign(msg)
	require.NoError(err)
	sigCpy := make([]byte, len(sig))
	copy(sigCpy, sig)
	require.True(suite.pk.VerifySignature(msg, sigCpy))

	// Mutate the signature
	for i := range sig {
		sigCpy[i] ^= byte(i + 1)
		require.False(suite.pk.VerifySignature(msg, sigCpy))
	}

	// mutate the signature by scalar neg'ing the s value
	// to give a high-s signature, valid ECDSA but should
	// be invalid with Cosmos signatures.
	// code mostly copied from privkey/pubkey.go

	// extract the r, s values from sig
	r := new(big.Int).SetBytes(sig[:32])
	lowS := new(big.Int).SetBytes(sig[32:64])

	// test that NormalizeS simply returns an already
	// normalized s
	require.Equal(NormalizeS(lowS), lowS)

	// flip the s value into high order of curve P256
	// leave r untouched!
	highS := new(big.Int).Mod(new(big.Int).Neg(lowS), elliptic.P256().Params().N)

	require.False(suite.pk.VerifySignature(msg, signatureRaw(r, highS)))

	// Valid signature using low_s, but too long
	sigCpy = make([]byte, len(sig)+2)
	copy(sigCpy, sig)
	sigCpy[65] = byte('A')

	require.False(suite.pk.VerifySignature(msg, sigCpy))

	// check whether msg can be verified with same key, and high_s
	// value using "regular" ecdsa signature
	hash := sha256.Sum256(msg)
	require.True(ecdsa.Verify(&suite.pk.PublicKey, hash[:], r, highS))

	// Mutate the message
	msg[1] ^= byte(2)
	require.False(suite.pk.VerifySignature(msg, sig))
}
