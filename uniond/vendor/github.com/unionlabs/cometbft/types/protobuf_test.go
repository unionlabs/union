package types

import (
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	abci "github.com/cometbft/cometbft/abci/types"
	"github.com/cometbft/cometbft/crypto"
	"github.com/cometbft/cometbft/crypto/ed25519"
	cryptoenc "github.com/cometbft/cometbft/crypto/encoding"
)

func TestPubKey(t *testing.T) {
	pk := ed25519.GenPrivKey().PubKey()

	// to proto
	abciPubKey, err := cryptoenc.PubKeyToProto(pk)
	require.NoError(t, err)

	// from proto
	pk2, err := cryptoenc.PubKeyFromProto(abciPubKey)
	require.NoError(t, err)

	require.Equal(t, pk, pk2)
}

type pubKeyEddie struct{}

func (pubKeyEddie) Address() Address                    { return []byte{} }
func (pubKeyEddie) Bytes() []byte                       { return []byte{} }
func (pubKeyEddie) VerifySignature([]byte, []byte) bool { return false }
func (pubKeyEddie) Equals(crypto.PubKey) bool           { return false }
func (pubKeyEddie) String() string                      { return "" }
func (pubKeyEddie) Type() string                        { return "pubKeyEddie" }

func TestPubKey_UnknownType(t *testing.T) {
	pk := pubKeyEddie{}

	// to proto
	_, err := cryptoenc.PubKeyToProto(pk)
	require.Error(t, err)
}

func TestValidatorUpdates(t *testing.T) {
	pkEd := ed25519.GenPrivKey().PubKey()
	cmtValExpected := NewValidator(pkEd, 10)
	abciVal := abci.NewValidatorUpdate(pkEd, 10)

	// from proto
	cmtVals, err := PB2TM.ValidatorUpdates([]abci.ValidatorUpdate{abciVal})
	require.NoError(t, err)
	assert.Equal(t, cmtValExpected, cmtVals[0])

	// to proto
	abciVals := TM2PB.ValidatorUpdates(NewValidatorSet(cmtVals))
	assert.Equal(t, []abci.ValidatorUpdate{abciVal}, abciVals)
}

func TestValidator_WithoutPubKey(t *testing.T) {
	pkEd := ed25519.GenPrivKey().PubKey()

	abciVal := TM2PB.Validator(NewValidator(pkEd, 10))

	// pubkey must be nil
	cmtValExpected := abci.Validator{
		Address: pkEd.Address(),
		Power:   10,
	}

	assert.Equal(t, cmtValExpected, abciVal)
}
