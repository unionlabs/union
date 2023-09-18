package ecdsa

import (
	"crypto/elliptic"
	"encoding/hex"
	"testing"

	"github.com/stretchr/testify/suite"
)

var secp256r1 = elliptic.P256()

func GenSecp256r1() (PrivKey, error) {
	return GenPrivKey(secp256r1)
}

func TestPKSuite(t *testing.T) {
	suite.Run(t, new(PKSuite))
}

type CommonSuite struct {
	suite.Suite
	pk PubKey
	sk PrivKey
}

func (suite *CommonSuite) SetupSuite() {
	sk, err := GenSecp256r1()
	suite.Require().NoError(err)
	suite.sk = sk
	suite.pk = sk.PubKey()
}

type PKSuite struct{ CommonSuite }

func (suite *PKSuite) TestString() {
	assert := suite.Assert()
	require := suite.Require()

	prefix := "abc"
	pkStr := suite.pk.String(prefix)
	assert.Equal(prefix+"{", pkStr[:len(prefix)+1])
	assert.EqualValues('}', pkStr[len(pkStr)-1])

	bz, err := hex.DecodeString(pkStr[len(prefix)+1 : len(pkStr)-1])
	require.NoError(err)
	assert.EqualValues(suite.pk.Bytes(), bz)
}

func (suite *PKSuite) TestBytes() {
	bz := suite.sk.Bytes()
	fieldSize := (suite.sk.Curve.Params().BitSize + 7) / 8
	suite.Len(bz, fieldSize)
	var pk *PubKey
	suite.Nil(pk.Bytes())
}

func (suite *PKSuite) TestMarshal() {
	require := suite.Require()
	const size = 33 // secp256r1 size

	buffer := make([]byte, size)
	n, err := suite.pk.MarshalTo(buffer)
	require.NoError(err)
	require.Equal(size, n)

	pk := new(PubKey)
	err = pk.Unmarshal(buffer, secp256r1, size)
	require.NoError(err)
	require.True(pk.PublicKey.Equal(&suite.pk.PublicKey))
}
