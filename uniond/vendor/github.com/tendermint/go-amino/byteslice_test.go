package amino

import (
	"bytes"
	"testing"
)

func TestReadByteSliceEquality(t *testing.T) {

	var encoded []byte
	var err error
	var cdc = NewCodec()

	// Write a byteslice
	var testBytes = []byte("ThisIsSomeTestArray")
	encoded, err = cdc.MarshalBinaryLengthPrefixed(testBytes)
	if err != nil {
		t.Error(err.Error())
	}

	// Read the byteslice, should return the same byteslice
	var testBytes2 []byte
	err = cdc.UnmarshalBinaryLengthPrefixed(encoded, &testBytes2)
	if err != nil {
		t.Error(err.Error())
	}

	if !bytes.Equal(testBytes, testBytes2) {
		t.Error("Returned the wrong bytes")
	}

}

/* XXX
// Issues:
// + https://github.com/tendermint/go-wire/issues/25
// + https://github.com/tendermint/go-wire/issues/37
func TestFuzzBinaryLengthOverflowsCaught(t *testing.T) {
	n, err := int(0), error(nil)
	var x []byte
	bs := ReadBinary(x, bytes.NewReader([]byte{8, 127, 255, 255, 255, 255, 255, 255, 255}), 0, &n, &err)
	require.Equal(t, err, ErrBinaryReadOverflow, "expected to detect a length overflow")
	require.Nil(t, bs, "expecting no bytes read out")
}
*/
