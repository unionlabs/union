package p2p

import (
	"bytes"
	"os"
	"path/filepath"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/crypto/ed25519"
	cmtrand "github.com/cometbft/cometbft/internal/rand"
)

func TestLoadOrGenNodeKey(t *testing.T) {
	filePath := filepath.Join(os.TempDir(), cmtrand.Str(12)+"_peer_id.json")

	nodeKey, err := LoadOrGenNodeKey(filePath)
	require.NoError(t, err)

	nodeKey2, err := LoadOrGenNodeKey(filePath)
	require.NoError(t, err)

	assert.Equal(t, nodeKey, nodeKey2)
}

func TestLoadNodeKey(t *testing.T) {
	filePath := filepath.Join(os.TempDir(), cmtrand.Str(12)+"_peer_id.json")

	_, err := LoadNodeKey(filePath)
	assert.True(t, os.IsNotExist(err))

	_, err = LoadOrGenNodeKey(filePath)
	require.NoError(t, err)

	nodeKey, err := LoadNodeKey(filePath)
	require.NoError(t, err)
	assert.NotNil(t, nodeKey)
}

func TestNodeKeySaveAs(t *testing.T) {
	filePath := filepath.Join(os.TempDir(), cmtrand.Str(12)+"_peer_id.json")

	assert.NoFileExists(t, filePath)

	privKey := ed25519.GenPrivKey()
	nodeKey := &NodeKey{
		PrivKey: privKey,
	}
	err := nodeKey.SaveAs(filePath)
	require.NoError(t, err)
	assert.FileExists(t, filePath)
}

// ----------------------------------------------------------

func padBytes(bz []byte) []byte {
	targetBytes := 20
	return append(bz, bytes.Repeat([]byte{0xFF}, targetBytes-len(bz))...)
}

func TestPoWTarget(t *testing.T) {
	cases := []struct {
		difficulty uint
		target     []byte
	}{
		{0, padBytes([]byte{})},
		{1, padBytes([]byte{127})},
		{8, padBytes([]byte{0})},
		{9, padBytes([]byte{0, 127})},
		{10, padBytes([]byte{0, 63})},
		{16, padBytes([]byte{0, 0})},
		{17, padBytes([]byte{0, 0, 127})},
	}

	for _, c := range cases {
		assert.Equal(t, MakePoWTarget(c.difficulty, 20*8), c.target)
	}
}
