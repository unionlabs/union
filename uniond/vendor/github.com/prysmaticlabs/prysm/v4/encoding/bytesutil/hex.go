package bytesutil

import (
	"fmt"
	"regexp"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/container/slice"
)

var hexRegex = regexp.MustCompile("^0x[0-9a-fA-F]+$")

// IsHex checks whether the byte array is a hex number prefixed with '0x'.
func IsHex(b []byte) bool {
	if b == nil {
		return false
	}
	return hexRegex.Match(b)
}

// DecodeHexWithLength takes a string and a length in bytes,
// and validates whether the string is a hex and has the correct length.
func DecodeHexWithLength(s string, length int) ([]byte, error) {
	bytes, err := hexutil.Decode(s)
	if err != nil {
		return nil, errors.Wrap(err, fmt.Sprintf("%s is not a valid hex", s))
	}
	if len(bytes) != length {
		return nil, fmt.Errorf("%s is not length %d bytes", s, length)
	}
	return bytes, nil
}

// DecodeHexWithMaxLength takes a string and a length in bytes,
// and validates whether the string is a hex and has the correct length.
func DecodeHexWithMaxLength(s string, maxLength int) ([]byte, error) {
	bytes, err := hexutil.Decode(s)
	if err != nil {
		return nil, errors.Wrap(err, fmt.Sprintf("%s is not a valid hex", s))
	}
	err = slice.VerifyMaxLength(bytes, maxLength)
	if err != nil {
		return nil, errors.Wrap(err, fmt.Sprintf("length of %s exceeds max of %d bytes", s, maxLength))
	}
	return bytes, nil
}
