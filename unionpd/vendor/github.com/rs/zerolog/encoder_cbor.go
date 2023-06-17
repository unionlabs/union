// +build binary_log

package zerolog

// This file contains bindings to do binary encoding.

import (
	"github.com/rs/zerolog/internal/cbor"
)

var (
	_ encoder = (*cbor.Encoder)(nil)

	enc = cbor.Encoder{}
)

func init() {
	// using closure to reflect the changes at runtime.
	cbor.JSONMarshalFunc = func(v interface{}) ([]byte, error) {
		return InterfaceMarshalFunc(v)
	}
}

func appendJSON(dst []byte, j []byte) []byte {
	return cbor.AppendEmbeddedJSON(dst, j)
}

// decodeIfBinaryToString - converts a binary formatted log msg to a
// JSON formatted String Log message.
func decodeIfBinaryToString(in []byte) string {
	return cbor.DecodeIfBinaryToString(in)
}

func decodeObjectToStr(in []byte) string {
	return cbor.DecodeObjectToStr(in)
}

// decodeIfBinaryToBytes - converts a binary formatted log msg to a
// JSON formatted Bytes Log message.
func decodeIfBinaryToBytes(in []byte) []byte {
	return cbor.DecodeIfBinaryToBytes(in)
}
