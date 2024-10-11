package offchain

import (
	"reflect"
	"testing"

	"github.com/stretchr/testify/require"
)

func Test_EncodingFuncs(t *testing.T) {
	tests := []struct {
		name       string
		encodeFunc encodingFunc
		digest     []byte
		want       string
	}{
		{
			name:       "No encoding",
			encodeFunc: noEncoding,
			digest:     []byte("Hello!"),
			want:       "Hello!",
		},
		{
			name:       "base64 encoding",
			encodeFunc: base64Encoding,
			digest:     []byte("Hello!"),
			want:       "SGVsbG8h",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := tt.encodeFunc(tt.digest)
			require.NoError(t, err)
			require.Equal(t, got, tt.want)
		})
	}
}

func Test_getEncoder(t *testing.T) {
	tests := []struct {
		name    string
		encoder string
		want    encodingFunc
	}{
		{
			name:    "no encoding",
			encoder: "no-encoding",
			want:    noEncoding,
		},
		{
			name:    "base64",
			encoder: "base64",
			want:    base64Encoding,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got, err := getEncoder(tt.encoder)
			require.NoError(t, err)
			require.Equal(t, reflect.ValueOf(got).Pointer(), reflect.ValueOf(tt.want).Pointer())
		})
	}
}
