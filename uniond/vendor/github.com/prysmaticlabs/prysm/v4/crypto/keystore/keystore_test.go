package keystore

import (
	"bytes"
	"os"
	"path"
	"testing"

	"github.com/pborman/uuid"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestStoreAndGetKey(t *testing.T) {
	tempDir := path.Join(t.TempDir(), "keystore", "file")
	ks := &Keystore{
		keysDirPath: tempDir,
		scryptN:     LightScryptN,
		scryptP:     LightScryptP,
	}

	key, err := NewKey()
	require.NoError(t, err)
	require.NoError(t, ks.StoreKey(tempDir, key, "password"))

	decryptedKey, err := ks.GetKey(tempDir, "password")
	require.NoError(t, err)
	require.Equal(t, true, bytes.Equal(decryptedKey.SecretKey.Marshal(), key.SecretKey.Marshal()))
}

func TestStoreAndGetKeys(t *testing.T) {
	tempDir := path.Join(t.TempDir(), "keystore")
	ks := &Keystore{
		keysDirPath: tempDir,
		scryptN:     LightScryptN,
		scryptP:     LightScryptP,
	}

	key, err := NewKey()
	require.NoError(t, err)
	require.NoError(t, ks.StoreKey(tempDir+"/test-1", key, "password"))
	key2, err := NewKey()
	require.NoError(t, err)
	require.NoError(t, ks.StoreKey(tempDir+"/test-2", key, "password"))
	decryptedKeys, err := ks.GetKeys(tempDir, "test", "password", false)
	require.NoError(t, err)
	for _, s := range decryptedKeys {
		require.Equal(t, true, bytes.Equal(s.SecretKey.Marshal(), key.SecretKey.Marshal()) && !bytes.Equal(s.SecretKey.Marshal(), key2.SecretKey.Marshal()))
	}
}

func TestEncryptDecryptKey(t *testing.T) {
	newID := uuid.NewRandom()
	b := []byte("hi")
	b32 := bytesutil.ToBytes32(b)
	password := "test"

	pk, err := bls.SecretKeyFromBytes(b32[:])
	require.NoError(t, err)
	key := &Key{
		ID:        newID,
		SecretKey: pk,
		PublicKey: pk.PublicKey(),
	}

	keyJSON, err := EncryptKey(key, password, LightScryptN, LightScryptP)
	require.NoError(t, err)

	decryptedKey, err := DecryptKey(keyJSON, password)
	require.NoError(t, err)
	require.Equal(t, true, bytes.Equal(decryptedKey.ID, newID))
	expected := pk.Marshal()
	require.Equal(t, true, bytes.Equal(decryptedKey.SecretKey.Marshal(), expected))
}

func TestGetSymlinkedKeys(t *testing.T) {
	tempDir := path.Join(t.TempDir(), "keystore")
	ks := &Keystore{
		scryptN: LightScryptN,
		scryptP: LightScryptP,
	}

	key, err := NewKey()
	require.NoError(t, err)
	require.NoError(t, ks.StoreKey(tempDir+"/files/test-1", key, "password"))
	require.NoError(t, os.Symlink(tempDir+"/files/test-1", tempDir+"/test-1"))
	decryptedKeys, err := ks.GetKeys(tempDir, "test", "password", false)
	require.NoError(t, err)
	assert.Equal(t, 1, len(decryptedKeys))
	for _, s := range decryptedKeys {
		require.Equal(t, true, bytes.Equal(s.SecretKey.Marshal(), key.SecretKey.Marshal()))
	}
}
