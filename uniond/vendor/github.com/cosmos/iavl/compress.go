package iavl

import (
	"encoding/binary"
	"fmt"
)

type NodeExporter interface {
	Next() (*ExportNode, error)
}

type NodeImporter interface {
	Add(*ExportNode) error
}

// CompressExporter wraps the normal exporter to apply some compressions on `ExportNode`:
// - branch keys are skipped
// - leaf keys are encoded with delta compared with the previous leaf
// - branch node's version are encoded with delta compared with the max version in it's children
type CompressExporter struct {
	inner        NodeExporter
	lastKey      []byte
	versionStack []int64
}

var _ NodeExporter = (*CompressExporter)(nil)

func NewCompressExporter(exporter NodeExporter) NodeExporter {
	return &CompressExporter{inner: exporter}
}

func (e *CompressExporter) Next() (*ExportNode, error) {
	n, err := e.inner.Next()
	if err != nil {
		return nil, err
	}

	if n.Height == 0 {
		// apply delta encoding to leaf keys
		n.Key, e.lastKey = deltaEncode(n.Key, e.lastKey), n.Key

		e.versionStack = append(e.versionStack, n.Version)
	} else {
		// branch keys can be derived on the fly when import, safe to skip
		n.Key = nil

		// delta encode the version
		maxVersion := maxInt64(e.versionStack[len(e.versionStack)-1], e.versionStack[len(e.versionStack)-2])
		e.versionStack = e.versionStack[:len(e.versionStack)-1]
		e.versionStack[len(e.versionStack)-1] = n.Version
		n.Version -= maxVersion
	}

	return n, nil
}

// CompressImporter wraps the normal importer to do de-compressions before hand.
type CompressImporter struct {
	inner        NodeImporter
	lastKey      []byte
	minKeyStack  [][]byte
	versionStack []int64
}

var _ NodeImporter = (*CompressImporter)(nil)

func NewCompressImporter(importer NodeImporter) NodeImporter {
	return &CompressImporter{inner: importer}
}

func (i *CompressImporter) Add(node *ExportNode) error {
	if node.Height == 0 {
		key, err := deltaDecode(node.Key, i.lastKey)
		if err != nil {
			return err
		}
		node.Key = key
		i.lastKey = key

		i.minKeyStack = append(i.minKeyStack, key)
		i.versionStack = append(i.versionStack, node.Version)
	} else {
		// use the min-key in right branch as the node key
		node.Key = i.minKeyStack[len(i.minKeyStack)-1]
		// leave the min-key in left branch in the stack
		i.minKeyStack = i.minKeyStack[:len(i.minKeyStack)-1]

		// decode branch node version
		maxVersion := maxInt64(i.versionStack[len(i.versionStack)-1], i.versionStack[len(i.versionStack)-2])
		node.Version += maxVersion
		i.versionStack = i.versionStack[:len(i.versionStack)-1]
		i.versionStack[len(i.versionStack)-1] = node.Version
	}

	return i.inner.Add(node)
}

func deltaEncode(key, lastKey []byte) []byte {
	var sizeBuf [binary.MaxVarintLen64]byte
	shared := diffOffset(lastKey, key)
	n := binary.PutUvarint(sizeBuf[:], uint64(shared))
	return append(sizeBuf[:n], key[shared:]...)
}

func deltaDecode(key, lastKey []byte) ([]byte, error) {
	shared, n := binary.Uvarint(key)
	if n <= 0 {
		return nil, fmt.Errorf("uvarint parse failed %d", n)
	}

	key = key[n:]
	if shared == 0 {
		return key, nil
	}

	newKey := make([]byte, shared+uint64(len(key)))
	copy(newKey, lastKey[:shared])
	copy(newKey[shared:], key)
	return newKey, nil
}

// diffOffset returns the index of first byte that's different in two bytes slice.
func diffOffset(a, b []byte) int {
	var off int
	var l int
	if len(a) < len(b) {
		l = len(a)
	} else {
		l = len(b)
	}
	for ; off < l; off++ {
		if a[off] != b[off] {
			break
		}
	}
	return off
}

func maxInt64(a, b int64) int64 {
	if a > b {
		return a
	}
	return b
}
