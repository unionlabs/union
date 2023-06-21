package bytesutil

import "regexp"

var hexRegex = regexp.MustCompile("^0x[0-9a-fA-F]+$")

// IsHex checks whether the byte array is a hex number prefixed with '0x'.
func IsHex(b []byte) bool {
	if b == nil {
		return false
	}
	return hexRegex.Match(b)
}
