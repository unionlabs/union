package msgpack

import (
	"io"

	"github.com/shamaton/msgpack/v2/internal/encoding"
	streamencoding "github.com/shamaton/msgpack/v2/internal/stream/encoding"
)

// MarshalAsMap encodes data as map format.
// This is the same thing that StructAsArray sets false.
func MarshalAsMap(v interface{}) ([]byte, error) {
	return encoding.Encode(v, false)
}

// MarshalAsArray encodes data as array format.
// This is the same thing that StructAsArray sets true.
func MarshalAsArray(v interface{}) ([]byte, error) {
	return encoding.Encode(v, true)
}

// MarshalWriteAsMap writes map format encoded data to writer.
// This is the same thing that StructAsArray sets false.
func MarshalWriteAsMap(w io.Writer, v interface{}) error {
	return streamencoding.Encode(w, v, false)
}

// MarshalWriteAsArray writes array format encoded data to writer.
// This is the same thing that StructAsArray sets true.
func MarshalWriteAsArray(w io.Writer, v interface{}) error {
	return streamencoding.Encode(w, v, true)
}
