package json

import (
	"fmt"
	"unicode/utf8"
)

const hex = "0123456789abcdef"

var noEscapeTable = [256]bool{}

func init() {
	for i := 0; i <= 0x7e; i++ {
		noEscapeTable[i] = i >= 0x20 && i != '\\' && i != '"'
	}
}

// AppendStrings encodes the input strings to json and
// appends the encoded string list to the input byte slice.
func (e Encoder) AppendStrings(dst []byte, vals []string) []byte {
	if len(vals) == 0 {
		return append(dst, '[', ']')
	}
	dst = append(dst, '[')
	dst = e.AppendString(dst, vals[0])
	if len(vals) > 1 {
		for _, val := range vals[1:] {
			dst = e.AppendString(append(dst, ','), val)
		}
	}
	dst = append(dst, ']')
	return dst
}

// AppendString encodes the input string to json and appends
// the encoded string to the input byte slice.
//
// The operation loops though each byte in the string looking
// for characters that need json or utf8 encoding. If the string
// does not need encoding, then the string is appended in its
// entirety to the byte slice.
// If we encounter a byte that does need encoding, switch up
// the operation and perform a byte-by-byte read-encode-append.
func (Encoder) AppendString(dst []byte, s string) []byte {
	// Start with a double quote.
	dst = append(dst, '"')
	// Loop through each character in the string.
	for i := 0; i < len(s); i++ {
		// Check if the character needs encoding. Control characters, slashes,
		// and the double quote need json encoding. Bytes above the ascii
		// boundary needs utf8 encoding.
		if !noEscapeTable[s[i]] {
			// We encountered a character that needs to be encoded. Switch
			// to complex version of the algorithm.
			dst = appendStringComplex(dst, s, i)
			return append(dst, '"')
		}
	}
	// The string has no need for encoding and therefore is directly
	// appended to the byte slice.
	dst = append(dst, s...)
	// End with a double quote
	return append(dst, '"')
}

// AppendStringers encodes the provided Stringer list to json and
// appends the encoded Stringer list to the input byte slice.
func (e Encoder) AppendStringers(dst []byte, vals []fmt.Stringer) []byte {
	if len(vals) == 0 {
		return append(dst, '[', ']')
	}
	dst = append(dst, '[')
	dst = e.AppendStringer(dst, vals[0])
	if len(vals) > 1 {
		for _, val := range vals[1:] {
			dst = e.AppendStringer(append(dst, ','), val)
		}
	}
	return append(dst, ']')
}

// AppendStringer encodes the input Stringer to json and appends the
// encoded Stringer value to the input byte slice.
func (e Encoder) AppendStringer(dst []byte, val fmt.Stringer) []byte {
	if val == nil {
		return e.AppendInterface(dst, nil)
	}
	return e.AppendString(dst, val.String())
}

//// appendStringComplex is used by appendString to take over an in
// progress JSON string encoding that encountered a character that needs
// to be encoded.
func appendStringComplex(dst []byte, s string, i int) []byte {
	start := 0
	for i < len(s) {
		b := s[i]
		if b >= utf8.RuneSelf {
			r, size := utf8.DecodeRuneInString(s[i:])
			if r == utf8.RuneError && size == 1 {
				// In case of error, first append previous simple characters to
				// the byte slice if any and append a replacement character code
				// in place of the invalid sequence.
				if start < i {
					dst = append(dst, s[start:i]...)
				}
				dst = append(dst, `\ufffd`...)
				i += size
				start = i
				continue
			}
			i += size
			continue
		}
		if noEscapeTable[b] {
			i++
			continue
		}
		// We encountered a character that needs to be encoded.
		// Let's append the previous simple characters to the byte slice
		// and switch our operation to read and encode the remainder
		// characters byte-by-byte.
		if start < i {
			dst = append(dst, s[start:i]...)
		}
		switch b {
		case '"', '\\':
			dst = append(dst, '\\', b)
		case '\b':
			dst = append(dst, '\\', 'b')
		case '\f':
			dst = append(dst, '\\', 'f')
		case '\n':
			dst = append(dst, '\\', 'n')
		case '\r':
			dst = append(dst, '\\', 'r')
		case '\t':
			dst = append(dst, '\\', 't')
		default:
			dst = append(dst, '\\', 'u', '0', '0', hex[b>>4], hex[b&0xF])
		}
		i++
		start = i
	}
	if start < len(s) {
		dst = append(dst, s[start:]...)
	}
	return dst
}
