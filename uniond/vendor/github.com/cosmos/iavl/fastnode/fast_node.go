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
func DeserializeNode(key []byte, buf []byte) (*Node, error) {
	ver, n, cause := encoding.DecodeVarint(buf)
	if cause != nil {
		return nil, fmt.Errorf("decoding fastnode.version, %w", cause)
	}
	buf = buf[n:]

	val, _, cause := encoding.DecodeBytes(buf)
	if cause != nil {
		return nil, fmt.Errorf("decoding fastnode.value, %w", cause)
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
	cause := encoding.EncodeVarint(w, fn.versionLastUpdatedAt)
	if cause != nil {
		return fmt.Errorf("writing version last updated at, %w", cause)
	}
	cause = encoding.EncodeBytes(w, fn.value)
	if cause != nil {
		return fmt.Errorf("writing value, %w", cause)
	}
	return nil
}
