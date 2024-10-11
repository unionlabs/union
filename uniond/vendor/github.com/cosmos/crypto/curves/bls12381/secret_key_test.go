package blst_test

import (
	"bytes"
	"crypto/rand"
	"errors"
	"testing"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	blst "github.com/cosmos/crypto/curves/bls12381"
)

func TestMarshalUnmarshal(t *testing.T) {
	priv, err := blst.RandKey()
	require.NoError(t, err)
	b := priv.Marshal()
	b32 := ToBytes32(b)
	pk, err := blst.SecretKeyFromBytes(b32[:])
	require.NoError(t, err)
	pk2, err := blst.SecretKeyFromBytes(b32[:])
	require.NoError(t, err)
	assert.Equal(t, pk.Marshal(), pk2.Marshal(), "Keys not equal")
}

func TestSecretKeyFromBytes(t *testing.T) {
	tests := []struct {
		name  string
		input []byte
		err   error
	}{
		{
			name: "Nil",
			err:  errors.New("secret key must be 32 bytes"),
		},
		{
			name:  "Empty",
			input: []byte{},
			err:   errors.New("secret key must be 32 bytes"),
		},
		{
			name:  "Short",
			input: []byte{0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
			err:   errors.New("secret key must be 32 bytes"),
		},
		{
			name:  "Long",
			input: []byte{0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00},
			err:   errors.New("secret key must be 32 bytes"),
		},
		{
			name:  "Bad",
			input: []byte{0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff},
			err:   errors.New("could not unmarshal bytes into secret key"),
		},
		{
			name:  "Good",
			input: []byte{0x25, 0x29, 0x5f, 0x0d, 0x1d, 0x59, 0x2a, 0x90, 0xb3, 0x33, 0xe2, 0x6e, 0x85, 0x14, 0x97, 0x08, 0x20, 0x8e, 0x9f, 0x8e, 0x8b, 0xc1, 0x8f, 0x6c, 0x77, 0xbd, 0x62, 0xf8, 0xad, 0x7a, 0x68, 0x66},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			res, err := blst.SecretKeyFromBytes(test.input)
			if test.err != nil {
				assert.NotEqual(t, nil, err, "No error returned")
				assert.ErrorContains(t, test.err, err.Error(), "Unexpected error returned")
			} else {
				assert.NoError(t, err)
				assert.Equal(t, 0, bytes.Compare(res.Marshal(), test.input))
			}
		})
	}
}

func TestSerialize(t *testing.T) {
	rk, err := blst.RandKey()
	require.NoError(t, err)
	b := rk.Marshal()

	_, err = blst.SecretKeyFromBytes(b)
	assert.NoError(t, err)
}

func TestZeroKey(t *testing.T) {
	// Is Zero
	var zKey [32]byte
	assert.Equal(t, true, blst.IsZero(zKey[:]))

	// Is Not Zero
	_, err := rand.Read(zKey[:])
	assert.NoError(t, err)
	assert.Equal(t, false, blst.IsZero(zKey[:]))
}

// PadTo pads a byte slice to the given size. If the byte slice is larger than the given size, the
// original slice is returned.
func PadTo(b []byte, size int) []byte {
	if len(b) >= size {
		return b
	}
	return append(b, make([]byte, size-len(b))...)
}

// ToBytes32 is a convenience method for converting a byte slice to a fix
// sized 32 byte array. This method will truncate the input if it is larger
// than 32 bytes.
func ToBytes32(x []byte) [32]byte {
	return [32]byte(PadTo(x, 32))
}
