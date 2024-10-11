package keystore

import (
	"bytes"
	"os"
	"path"
	"testing"

	"github.com/pborman/uuid"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestMarshalAndUnmarshal(t *testing.T) {
	testID := uuid.NewRandom()
	blsKey, err := bls.RandKey()
	require.NoError(t, err)

	key := &Key{
		ID:        testID,
		SecretKey: blsKey,
		PublicKey: blsKey.PublicKey(),
	}
	marshalledObject, err := key.MarshalJSON()
	require.NoError(t, err)
	newKey := &Key{
		ID:        []byte{},
		SecretKey: blsKey,
		PublicKey: blsKey.PublicKey(),
	}

	err = newKey.UnmarshalJSON(marshalledObject)
	require.NoError(t, err)
	require.Equal(t, true, bytes.Equal(newKey.ID, testID))
}

func TestStoreRandomKey(t *testing.T) {
	ks := &Keystore{
		keysDirPath: path.Join(t.TempDir(), "keystore"),
		scryptN:     LightScryptN,
		scryptP:     LightScryptP,
	}
	require.NoError(t, storeNewRandomKey(ks, "password"))
}

func TestNewKeyFromBLS(t *testing.T) {
	b := []byte("hi")
	b32 := bytesutil.ToBytes32(b)
	blskey, err := bls.SecretKeyFromBytes(b32[:])
	require.NoError(t, err)
	key, err := NewKeyFromBLS(blskey)
	require.NoError(t, err)

	expected := blskey.Marshal()
	require.Equal(t, true, bytes.Equal(expected, key.SecretKey.Marshal()))
	_, err = NewKey()
	require.NoError(t, err)
}

func TestWriteFile(t *testing.T) {
	tempDir := path.Join(t.TempDir(), "keystore", "file")
	testKeystore := []byte{'t', 'e', 's', 't'}

	err := writeKeyFile(tempDir, testKeystore)
	require.NoError(t, err)

	keystore, err := os.ReadFile(tempDir)
	require.NoError(t, err)
	require.Equal(t, true, bytes.Equal(keystore, testKeystore))
}
