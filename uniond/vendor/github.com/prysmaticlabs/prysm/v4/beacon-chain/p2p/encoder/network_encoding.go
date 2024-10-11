package encoder

import (
	"io"

	ssz "github.com/prysmaticlabs/fastssz"
)

// NetworkEncoding represents an encoder compatible with Ethereum consensus p2p.
type NetworkEncoding interface {
	// DecodeGossip to the provided gossip message. The interface must be a pointer to the decoding destination.
	DecodeGossip([]byte, ssz.Unmarshaler) error
	// DecodeWithMaxLength a bytes from a reader with a varint length prefix. The interface must be a pointer to the
	// decoding destination. The length of the message should not be more than the provided limit.
	DecodeWithMaxLength(io.Reader, ssz.Unmarshaler) error
	// EncodeGossip an arbitrary gossip message to the provided writer. The interface must be a pointer object to encode.
	EncodeGossip(io.Writer, ssz.Marshaler) (int, error)
	// EncodeWithMaxLength an arbitrary message to the provided writer with a varint length prefix. The interface must be
	// a pointer object to encode. The encoded message should not be bigger than the provided limit.
	EncodeWithMaxLength(io.Writer, ssz.Marshaler) (int, error)
	// ProtocolSuffix returns the last part of the protocol ID to indicate the encoding scheme.
	ProtocolSuffix() string
}
