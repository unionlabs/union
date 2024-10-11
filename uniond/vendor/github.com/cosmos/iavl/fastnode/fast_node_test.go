package fastnode

import (
	"bytes"
	"encoding/hex"
	"testing"

	"github.com/stretchr/testify/require"

	iavlrand "github.com/cosmos/iavl/internal/rand"
)

func TestFastNode_encodedSize(t *testing.T) {
	fastNode := &Node{
		key:                  iavlrand.RandBytes(10),
		versionLastUpdatedAt: 1,
		value:                iavlrand.RandBytes(20),
	}

	expectedSize := 1 + len(fastNode.value) + 1

	require.Equal(t, expectedSize, fastNode.EncodedSize())
}

func TestFastNode_encode_decode(t *testing.T) {
	testcases := map[string]struct {
		node        *Node
		expectHex   string
		expectError bool
	}{
		"nil":   {nil, "", true},
		"empty": {&Node{}, "0000", false},
		"inner": {&Node{
			key:                  []byte{0x4},
			versionLastUpdatedAt: 1,
			value:                []byte{0x2},
		}, "020102", false},
	}
	for name, tc := range testcases {
		tc := tc
		t.Run(name, func(t *testing.T) {
			var buf bytes.Buffer
			err := tc.node.WriteBytes(&buf)
			if tc.expectError {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
			require.Equal(t, tc.expectHex, hex.EncodeToString(buf.Bytes()))

			node, err := DeserializeNode(tc.node.key, buf.Bytes())
			require.NoError(t, err)
			// since value and leafHash are always decoded to []byte{} we augment the expected struct here
			if tc.node.value == nil {
				tc.node.value = []byte{}
			}
			require.Equal(t, tc.node, node)
		})
	}
}
