package iavl

import (
	"bytes"
	"crypto/sha256"
	"encoding/hex"
	"math/rand"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	iavlrand "github.com/cosmos/iavl/internal/rand"
)

func TestNode_encodedSize(t *testing.T) {
	nodeKey := &NodeKey{
		version: 1,
		nonce:   1,
	}
	node := &Node{
		key:           iavlrand.RandBytes(10),
		value:         iavlrand.RandBytes(10),
		subtreeHeight: 0,
		size:          100,
		hash:          iavlrand.RandBytes(20),
		nodeKey:       nodeKey,
		leftNodeKey:   nodeKey.GetKey(),
		leftNode:      nil,
		rightNodeKey:  nodeKey.GetKey(),
		rightNode:     nil,
	}

	// leaf node
	require.Equal(t, 25, node.encodedSize())

	// non-leaf node
	node.subtreeHeight = 1
	require.Equal(t, 39, node.encodedSize())
}

func TestNode_encode_decode(t *testing.T) {
	childNodeKey := &NodeKey{
		version: 1,
		nonce:   1,
	}
	childNodeHash := []byte{0x7f, 0x68, 0x90, 0xca, 0x16, 0xde, 0xa6, 0xe8, 0x89, 0x3d, 0x96, 0xf0, 0xa3, 0xd, 0xa, 0x14, 0xe5, 0x55, 0x59, 0xfc, 0x9b, 0x83, 0x4, 0x91, 0xe3, 0xd2, 0x45, 0x1c, 0x81, 0xf6, 0xd1, 0xe}
	testcases := map[string]struct {
		node        *Node
		expectHex   string
		expectError bool
	}{
		"nil": {nil, "", true},
		"inner": {&Node{
			subtreeHeight: 3,
			size:          7,
			key:           []byte("key"),
			nodeKey: &NodeKey{
				version: 2,
				nonce:   1,
			},
			leftNodeKey:  childNodeKey.GetKey(),
			rightNodeKey: childNodeKey.GetKey(),
			hash:         []byte{0x7f, 0x68, 0x90, 0xca, 0x16, 0xde, 0xa6, 0xe8, 0x89, 0x3d, 0x96, 0xf0, 0xa3, 0xd, 0xa, 0x14, 0xe5, 0x55, 0x59, 0xfc, 0x9b, 0x83, 0x4, 0x91, 0xe3, 0xd2, 0x45, 0x1c, 0x81, 0xf6, 0xd1, 0xe},
		}, "060e036b6579207f6890ca16dea6e8893d96f0a30d0a14e55559fc9b830491e3d2451c81f6d10e0002020202", false},
		"inner hybrid": {&Node{
			subtreeHeight: 3,
			size:          7,
			key:           []byte("key"),
			nodeKey: &NodeKey{
				version: 2,
				nonce:   1,
			},
			leftNodeKey:  childNodeKey.GetKey(),
			rightNodeKey: childNodeHash,
			hash:         []byte{0x7f, 0x68, 0x90, 0xca, 0x16, 0xde, 0xa6, 0xe8, 0x89, 0x3d, 0x96, 0xf0, 0xa3, 0xd, 0xa, 0x14, 0xe5, 0x55, 0x59, 0xfc, 0x9b, 0x83, 0x4, 0x91, 0xe3, 0xd2, 0x45, 0x1c, 0x81, 0xf6, 0xd1, 0xe},
		}, "060e036b6579207f6890ca16dea6e8893d96f0a30d0a14e55559fc9b830491e3d2451c81f6d10e040202207f6890ca16dea6e8893d96f0a30d0a14e55559fc9b830491e3d2451c81f6d10e", false},
		"leaf": {&Node{
			subtreeHeight: 0,
			size:          1,
			key:           []byte("key"),
			value:         []byte("value"),
			nodeKey: &NodeKey{
				version: 3,
				nonce:   1,
			},
			hash: []byte{0x7f, 0x68, 0x90, 0xca, 0x16, 0xde, 0xa6, 0xe8, 0x89, 0x3d, 0x96, 0xf0, 0xa3, 0xd, 0xa, 0x14, 0xe5, 0x55, 0x59, 0xfc, 0x9b, 0x83, 0x4, 0x91, 0xe3, 0xd2, 0x45, 0x1c, 0x81, 0xf6, 0xd1, 0xe},
		}, "0002036b65790576616c7565", false},
	}
	for name, tc := range testcases {
		tc := tc
		t.Run(name, func(t *testing.T) {
			var buf bytes.Buffer
			err := tc.node.writeBytes(&buf)
			if tc.expectError {
				require.Error(t, err)
				return
			}
			require.NoError(t, err)
			require.Equal(t, tc.expectHex, hex.EncodeToString(buf.Bytes()))

			node, err := MakeNode(tc.node.GetKey(), buf.Bytes())
			require.NoError(t, err)
			// since key and value is always decoded to []byte{} we augment the expected struct here
			if tc.node.key == nil {
				tc.node.key = []byte{}
			}
			if tc.node.value == nil && tc.node.subtreeHeight == 0 {
				tc.node.value = []byte{}
			}
			require.Equal(t, tc.node, node)
		})
	}
}

func TestNode_validate(t *testing.T) {
	k := []byte("key")
	v := []byte("value")
	nk := &NodeKey{
		version: 1,
		nonce:   1,
	}
	c := &Node{key: []byte("child"), value: []byte("x"), size: 1}

	testcases := map[string]struct {
		node  *Node
		valid bool
	}{
		"nil node":                 {nil, false},
		"leaf":                     {&Node{key: k, value: v, nodeKey: nk, size: 1}, true},
		"leaf with nil key":        {&Node{key: nil, value: v, size: 1}, false},
		"leaf with empty key":      {&Node{key: []byte{}, value: v, nodeKey: nk, size: 1}, true},
		"leaf with nil value":      {&Node{key: k, value: nil, size: 1}, false},
		"leaf with empty value":    {&Node{key: k, value: []byte{}, nodeKey: nk, size: 1}, true},
		"leaf with version 0":      {&Node{key: k, value: v, size: 1}, false},
		"leaf with version -1":     {&Node{key: k, value: v, size: 1}, false},
		"leaf with size 0":         {&Node{key: k, value: v, size: 0}, false},
		"leaf with size 2":         {&Node{key: k, value: v, size: 2}, false},
		"leaf with size -1":        {&Node{key: k, value: v, size: -1}, false},
		"leaf with left node key":  {&Node{key: k, value: v, size: 1, leftNodeKey: nk.GetKey()}, false},
		"leaf with left child":     {&Node{key: k, value: v, size: 1, leftNode: c}, false},
		"leaf with right node key": {&Node{key: k, value: v, size: 1, rightNodeKey: nk.GetKey()}, false},
		"leaf with right child":    {&Node{key: k, value: v, size: 1, rightNode: c}, false},
		"inner":                    {&Node{key: k, size: 1, subtreeHeight: 1, nodeKey: nk, leftNodeKey: nk.GetKey(), rightNodeKey: nk.GetKey()}, true},
		"inner with nil key":       {&Node{key: nil, value: v, size: 1, subtreeHeight: 1, leftNodeKey: nk.GetKey(), rightNodeKey: nk.GetKey()}, false},
		"inner with value":         {&Node{key: k, value: v, size: 1, subtreeHeight: 1, leftNodeKey: nk.GetKey(), rightNodeKey: nk.GetKey()}, false},
		"inner with empty value":   {&Node{key: k, value: []byte{}, size: 1, subtreeHeight: 1, leftNodeKey: nk.GetKey(), rightNodeKey: nk.GetKey()}, false},
		"inner with left child":    {&Node{key: k, size: 1, subtreeHeight: 1, nodeKey: nk, leftNodeKey: nk.GetKey()}, true},
		"inner with right child":   {&Node{key: k, size: 1, subtreeHeight: 1, nodeKey: nk, rightNodeKey: nk.GetKey()}, true},
		"inner with no child":      {&Node{key: k, size: 1, subtreeHeight: 1}, false},
		"inner with height 0":      {&Node{key: k, size: 1, subtreeHeight: 0, leftNodeKey: nk.GetKey(), rightNodeKey: nk.GetKey()}, false},
	}

	for desc, tc := range testcases {
		tc := tc // appease scopelint
		t.Run(desc, func(t *testing.T) {
			err := tc.node.validate()
			if tc.valid {
				assert.NoError(t, err)
			} else {
				assert.Error(t, err)
			}
		})
	}
}

func BenchmarkNode_encodedSize(b *testing.B) {
	nk := &NodeKey{
		version: rand.Int63n(10000000),
		nonce:   uint32(rand.Int31n(10000000)),
	}
	node := &Node{
		key:           iavlrand.RandBytes(25),
		value:         iavlrand.RandBytes(100),
		nodeKey:       nk,
		subtreeHeight: 1,
		size:          rand.Int63n(10000000),
		leftNodeKey:   nk.GetKey(),
		rightNodeKey:  nk.GetKey(),
	}
	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		node.encodedSize()
	}
}

func BenchmarkNode_WriteBytes(b *testing.B) {
	nk := &NodeKey{
		version: rand.Int63n(10000000),
		nonce:   uint32(rand.Int31n(10000000)),
	}
	node := &Node{
		key:           iavlrand.RandBytes(25),
		value:         iavlrand.RandBytes(100),
		nodeKey:       nk,
		subtreeHeight: 1,
		size:          rand.Int63n(10000000),
		leftNodeKey:   nk.GetKey(),
		rightNodeKey:  nk.GetKey(),
	}
	b.ResetTimer()
	b.Run("NoPreAllocate", func(sub *testing.B) {
		sub.ReportAllocs()
		for i := 0; i < sub.N; i++ {
			var buf bytes.Buffer
			_ = node.writeBytes(&buf)
		}
	})
	b.Run("PreAllocate", func(sub *testing.B) {
		sub.ReportAllocs()
		for i := 0; i < sub.N; i++ {
			var buf bytes.Buffer
			buf.Grow(node.encodedSize())
			_ = node.writeBytes(&buf)
		}
	})
}

func BenchmarkNode_HashNode(b *testing.B) {
	node := &Node{
		key:   iavlrand.RandBytes(25),
		value: iavlrand.RandBytes(100),
		nodeKey: &NodeKey{
			version: rand.Int63n(10000000),
			nonce:   uint32(rand.Int31n(10000000)),
		},
		subtreeHeight: 0,
		size:          rand.Int63n(10000000),
		hash:          iavlrand.RandBytes(32),
	}
	b.ResetTimer()
	b.Run("NoBuffer", func(sub *testing.B) {
		sub.ReportAllocs()
		for i := 0; i < sub.N; i++ {
			h := sha256.New()
			require.NoError(b, node.writeHashBytes(h, node.nodeKey.version))
			_ = h.Sum(nil)
		}
	})
	b.Run("PreAllocate", func(sub *testing.B) {
		sub.ReportAllocs()
		for i := 0; i < sub.N; i++ {
			h := sha256.New()
			buf := new(bytes.Buffer)
			buf.Grow(node.encodedSize())
			require.NoError(b, node.writeHashBytes(buf, node.nodeKey.version))
			_, err := h.Write(buf.Bytes())
			require.NoError(b, err)
			_ = h.Sum(nil)
		}
	})
	b.Run("NoPreAllocate", func(sub *testing.B) {
		sub.ReportAllocs()
		for i := 0; i < sub.N; i++ {
			h := sha256.New()
			buf := new(bytes.Buffer)
			require.NoError(b, node.writeHashBytes(buf, node.nodeKey.version))
			_, err := h.Write(buf.Bytes())
			require.NoError(b, err)
			_ = h.Sum(nil)
		}
	})
}
