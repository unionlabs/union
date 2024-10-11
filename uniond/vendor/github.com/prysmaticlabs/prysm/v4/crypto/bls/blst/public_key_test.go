//go:build ((linux && amd64) || (linux && arm64) || (darwin && amd64) || (darwin && arm64) || (windows && amd64)) && !blst_disabled

package blst_test

import (
	"bytes"
	"errors"
	"sync"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/crypto/bls/blst"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls/common"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestPublicKeyFromBytes(t *testing.T) {
	tests := []struct {
		name  string
		input []byte
		err   error
	}{
		{
			name: "Nil",
			err:  errors.New("public key must be 48 bytes"),
		},
		{
			name:  "Empty",
			input: []byte{},
			err:   errors.New("public key must be 48 bytes"),
		},
		{
			name:  "Short",
			input: []byte{0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
			err:   errors.New("public key must be 48 bytes"),
		},
		{
			name:  "Long",
			input: []byte{0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
			err:   errors.New("public key must be 48 bytes"),
		},
		{
			name:  "Bad",
			input: []byte{0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
			err:   errors.New("could not unmarshal bytes into public key"),
		},
		{
			name:  "Good",
			input: []byte{0xa9, 0x9a, 0x76, 0xed, 0x77, 0x96, 0xf7, 0xbe, 0x22, 0xd5, 0xb7, 0xe8, 0x5d, 0xee, 0xb7, 0xc5, 0x67, 0x7e, 0x88, 0xe5, 0x11, 0xe0, 0xb3, 0x37, 0x61, 0x8f, 0x8c, 0x4e, 0xb6, 0x13, 0x49, 0xb4, 0xbf, 0x2d, 0x15, 0x3f, 0x64, 0x9f, 0x7b, 0x53, 0x35, 0x9f, 0xe8, 0xb9, 0x4a, 0x38, 0xe4, 0x4c},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			res, err := blst.PublicKeyFromBytes(test.input)
			if test.err != nil {
				assert.NotEqual(t, nil, err, "No error returned")
				assert.ErrorContains(t, test.err.Error(), err, "Unexpected error returned")
			} else {
				assert.NoError(t, err)
				assert.DeepEqual(t, 0, bytes.Compare(res.Marshal(), test.input))
			}
		})
	}
}

func TestPublicKey_Copy(t *testing.T) {
	priv, err := blst.RandKey()
	require.NoError(t, err)
	pubkeyA := priv.PublicKey()
	pubkeyBytes := pubkeyA.Marshal()

	pubkeyB := pubkeyA.Copy()
	priv2, err := blst.RandKey()
	require.NoError(t, err)
	pubkeyB.Aggregate(priv2.PublicKey())

	require.DeepEqual(t, pubkeyA.Marshal(), pubkeyBytes, "Pubkey was mutated after copy")
}

func TestPublicKey_Aggregate(t *testing.T) {
	priv, err := blst.RandKey()
	require.NoError(t, err)
	pubkeyA := priv.PublicKey()

	pubkeyB := pubkeyA.Copy()
	priv2, err := blst.RandKey()
	require.NoError(t, err)
	resKey := pubkeyB.Aggregate(priv2.PublicKey())

	aggKey := blst.AggregateMultiplePubkeys([]common.PublicKey{priv.PublicKey(), priv2.PublicKey()})

	require.DeepEqual(t, resKey.Marshal(), aggKey.Marshal(), "Pubkey does not match up")
}

func TestPublicKey_Aggregation_NoCorruption(t *testing.T) {
	var pubkeys []common.PublicKey
	for i := 0; i < 100; i++ {
		priv, err := blst.RandKey()
		require.NoError(t, err)
		pubkey := priv.PublicKey()
		pubkeys = append(pubkeys, pubkey)
	}

	var compressedKeys [][]byte
	// Fill up the cache
	for _, pkey := range pubkeys {
		_, err := blst.PublicKeyFromBytes(pkey.Marshal())
		require.NoError(t, err)
		compressedKeys = append(compressedKeys, pkey.Marshal())
	}

	wg := new(sync.WaitGroup)

	// Aggregate different sets of keys.
	wg.Add(1)
	go func() {
		_, err := blst.AggregatePublicKeys(compressedKeys)
		require.NoError(t, err)
		wg.Done()
	}()

	wg.Add(1)
	go func() {
		_, err := blst.AggregatePublicKeys(compressedKeys[:10])
		require.NoError(t, err)
		wg.Done()
	}()

	wg.Add(1)
	go func() {
		_, err := blst.AggregatePublicKeys(compressedKeys[:40])
		require.NoError(t, err)
		wg.Done()
	}()

	wg.Add(1)
	go func() {
		_, err := blst.AggregatePublicKeys(compressedKeys[20:60])
		require.NoError(t, err)
		wg.Done()
	}()

	wg.Add(1)
	go func() {
		_, err := blst.AggregatePublicKeys(compressedKeys[80:])
		require.NoError(t, err)
		wg.Done()
	}()

	wg.Add(1)
	go func() {
		_, err := blst.AggregatePublicKeys(compressedKeys[60:90])
		require.NoError(t, err)
		wg.Done()
	}()

	wg.Add(1)
	go func() {
		_, err := blst.AggregatePublicKeys(compressedKeys[40:99])
		require.NoError(t, err)
		wg.Done()
	}()

	wg.Wait()

	for _, pkey := range pubkeys {
		cachedPubkey, err := blst.PublicKeyFromBytes(pkey.Marshal())
		require.NoError(t, err)
		assert.Equal(t, true, cachedPubkey.Equals(pkey))
	}
}

func TestPublicKeysEmpty(t *testing.T) {
	var pubs [][]byte
	_, err := blst.AggregatePublicKeys(pubs)
	require.ErrorContains(t, "nil or empty public keys", err)
}

func BenchmarkPublicKeyFromBytes(b *testing.B) {
	priv, err := blst.RandKey()
	require.NoError(b, err)
	pubkey := priv.PublicKey()
	pubkeyBytes := pubkey.Marshal()

	b.Run("cache on", func(b *testing.B) {
		blst.EnableCaches()
		for i := 0; i < b.N; i++ {
			_, err := blst.PublicKeyFromBytes(pubkeyBytes)
			require.NoError(b, err)
		}
	})

	b.Run("cache off", func(b *testing.B) {
		blst.DisableCaches()
		for i := 0; i < b.N; i++ {
			_, err := blst.PublicKeyFromBytes(pubkeyBytes)
			require.NoError(b, err)
		}
	})

}
