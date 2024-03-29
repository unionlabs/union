package fastnode

import (
	"errors"
	"fmt"
	"io"

	"github.com/cosmos/iavl/cache"
	"github.com/cosmos/iavl/internal/encoding"
)

// NOTE: This file favors int64 as opposed to int for size/counts.
// The Tree on the other hand favors int.  This is intentional.

type Node struct {
	key                  []byte
	versionLastUpdatedAt int64
	value                []byte
}

var _ cache.Node = (*Node)(nil)

// NewNode returns a new fast node from a value and version.
func NewNode(key []byte, value []byte, version int64) *Node {
	return &Node{
		key:                  key,
		versionLastUpdatedAt: version,
		value:                value,
	}
}

// DeserializeNode constructs an *FastNode from an encoded byte slice.
// It assumes we do not mutate this input []byte.
func DeserializeNode(key []byte, buf []byte) (*Node, error) {
	ver, n, err := encoding.DecodeVarint(buf)
	if err != nil {
		return nil, fmt.Errorf("decoding fastnode.version, %w", err)
	}
	buf = buf[n:]

	val, _, err := encoding.DecodeBytes(buf)
	if err != nil {
		return nil, fmt.Errorf("decoding fastnode.value, %w", err)
	}

	fastNode := &Node{
		key:                  key,
		versionLastUpdatedAt: ver,
		value:                val,
	}

	return fastNode, nil
}

func (fn *Node) GetKey() []byte {
	return fn.key
}

func (fn *Node) EncodedSize() int {
	n := encoding.EncodeVarintSize(fn.versionLastUpdatedAt) + encoding.EncodeBytesSize(fn.value)
	return n
}

func (fn *Node) GetValue() []byte {
	return fn.value
}

func (fn *Node) GetVersionLastUpdatedAt() int64 {
	return fn.versionLastUpdatedAt
}

// WriteBytes writes the FastNode as a serialized byte slice to the supplied io.Writer.
func (fn *Node) WriteBytes(w io.Writer) error {
	if fn == nil {
		return errors.New("cannot write nil node")
	}
	err := encoding.EncodeVarint(w, fn.versionLastUpdatedAt)
	if err != nil {
		return fmt.Errorf("writing version last updated at, %w", err)
	}
	err = encoding.EncodeBytes(w, fn.value)
	if err != nil {
		return fmt.Errorf("writing value, %w", err)
	}
	return nil
}
