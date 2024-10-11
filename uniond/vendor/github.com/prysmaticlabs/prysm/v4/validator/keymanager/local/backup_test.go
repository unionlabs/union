package local

import (
	"context"
	"encoding/hex"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestLocalKeymanager_ExtractKeystores(t *testing.T) {
	secretKeysCache = make(map[[fieldparams.BLSPubkeyLength]byte]bls.SecretKey)
	dr := &Keymanager{}
	validatingKeys := make([]bls.SecretKey, 10)
	for i := 0; i < len(validatingKeys); i++ {
		secretKey, err := bls.RandKey()
		require.NoError(t, err)
		validatingKeys[i] = secretKey
		secretKeysCache[bytesutil.ToBytes48(secretKey.PublicKey().Marshal())] = secretKey
	}
	ctx := context.Background()
	password := "password"

	// Extracting 0 public keys should return 0 keystores.
	keystores, err := dr.ExtractKeystores(ctx, nil, password)
	require.NoError(t, err)
	assert.Equal(t, 0, len(keystores))

	// We attempt to extract a few indices.
	keystores, err = dr.ExtractKeystores(
		ctx,
		[]bls.PublicKey{
			validatingKeys[3].PublicKey(),
			validatingKeys[5].PublicKey(),
			validatingKeys[7].PublicKey(),
		},
		password,
	)
	require.NoError(t, err)
	receivedPubKeys := make([][]byte, len(keystores))
	for i, k := range keystores {
		pubKeyBytes, err := hex.DecodeString(k.Pubkey)
		require.NoError(t, err)
		receivedPubKeys[i] = pubKeyBytes
	}
	assert.DeepEqual(t, receivedPubKeys, [][]byte{
		validatingKeys[3].PublicKey().Marshal(),
		validatingKeys[5].PublicKey().Marshal(),
		validatingKeys[7].PublicKey().Marshal(),
	})
}
