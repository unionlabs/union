package encoder

import (
	"io"

	"github.com/gogo/protobuf/proto"
	"github.com/pkg/errors"
)

const maxVarintLength = 10

var errExcessMaxLength = errors.Errorf("provided header exceeds the max varint length of %d bytes", maxVarintLength)

// readVarint at the beginning of a byte slice. This varint may be used to indicate
// the length of the remaining bytes in the reader.
func readVarint(r io.Reader) (uint64, error) {
	b := make([]byte, 0, maxVarintLength)
	for i := 0; i < maxVarintLength; i++ {
		b1 := make([]byte, 1)
		n, err := r.Read(b1)
		if err != nil {
			return 0, err
		}
		if n != 1 {
			return 0, errors.New("did not read a byte from stream")
		}
		b = append(b, b1[0])

		// If most significant bit is not set, we have reached the end of the Varint.
		if b1[0]&0x80 == 0 {
			break
		}

		// If the varint is larger than 10 bytes, it is invalid as it would
		// exceed the size of MaxUint64.
		if i+1 >= maxVarintLength {
			return 0, errExcessMaxLength
		}
	}

	vi, n := proto.DecodeVarint(b)
	if n != len(b) {
		return 0, errors.New("varint did not decode entire byte slice")
	}
	return vi, nil
}
