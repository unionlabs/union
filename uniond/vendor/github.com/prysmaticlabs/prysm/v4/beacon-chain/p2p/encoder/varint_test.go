package encoder

import (
	"bytes"
	"testing"

	"github.com/gogo/protobuf/proto"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestReadVarint(t *testing.T) {
	data := []byte("foobar data")
	prefixedData := append(proto.EncodeVarint(uint64(len(data))), data...)

	vi, err := readVarint(bytes.NewBuffer(prefixedData))
	require.NoError(t, err)
	assert.Equal(t, uint64(len(data)), vi, "Received wrong varint")
}

func TestReadVarint_ExceedsMaxLength(t *testing.T) {
	fByte := byte(1 << 7)
	// Terminating byte.
	tByte := byte(1 << 6)
	var header []byte
	for i := 0; i < 9; i++ {
		header = append(header, fByte)
	}
	header = append(header, tByte)
	_, err := readVarint(bytes.NewBuffer(header))
	require.NoError(t, err, "Expected no error from reading valid header")
	length := len(header)
	// Add an additional byte to make header invalid.
	header = append(header[:length-1], []byte{fByte, tByte}...)

	_, err = readVarint(bytes.NewBuffer(header))
	assert.ErrorContains(t, errExcessMaxLength.Error(), err, "Expected error from reading invalid header")
}
