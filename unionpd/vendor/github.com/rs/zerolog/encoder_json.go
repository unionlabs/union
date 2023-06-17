// +build !binary_log

package zerolog

// encoder_json.go file contains bindings to generate
// JSON encoded byte stream.

import (
	"github.com/rs/zerolog/internal/json"
)

var (
	_ encoder = (*json.Encoder)(nil)

	enc = json.Encoder{}
)

func init() {
	// using closure to reflect the changes at runtime.
	json.JSONMarshalFunc = func(v interface{}) ([]byte, error) {
		return InterfaceMarshalFunc(v)
	}
}

func appendJSON(dst []byte, j []byte) []byte {
	return append(dst, j...)
}

func decodeIfBinaryToString(in []byte) string {
	return string(in)
}

func decodeObjectToStr(in []byte) string {
	return string(in)
}

func decodeIfBinaryToBytes(in []byte) []byte {
	return in
}
