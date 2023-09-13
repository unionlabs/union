// +build !binary_log

package zerolog

// encoder_json.go file contains bindings to generate
// JSON encoded byte stream.

import (
	"encoding/base64"
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
func appendCBOR(dst []byte, cbor []byte) []byte {
	dst = append(dst, []byte("\"data:application/cbor;base64,")...)
	l := len(dst)
	enc := base64.StdEncoding
	n := enc.EncodedLen(len(cbor))
	for i := 0; i < n; i++ {
		dst = append(dst, '.')
	}
	enc.Encode(dst[l:], cbor)
	return append(dst, '"')
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
