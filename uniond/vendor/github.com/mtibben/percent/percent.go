// Package percent escapes strings using percent-encoding
package percent

import (
	"strings"
)

const upperhex = "0123456789ABCDEF"

func unhex(c byte) byte {
	switch {
	case '0' <= c && c <= '9':
		return c - '0'
	case 'a' <= c && c <= 'f':
		return c - 'a' + 10
	case 'A' <= c && c <= 'F':
		return c - 'A' + 10
	}
	return 0
}

func ishex(c byte) bool {
	switch {
	case '0' <= c && c <= '9':
		return true
	case 'a' <= c && c <= 'f':
		return true
	case 'A' <= c && c <= 'F':
		return true
	}
	return false
}

// Encode escapes the string using percent-encoding, converting the runes found in charsToEncode with hex-encoded %AB sequences
func Encode(s string, charsToEncode string) string {
	var t strings.Builder
	for _, c := range s {
		if strings.IndexRune(charsToEncode, c) != -1 || c == '%' {
			for _, b := range []byte(string(c)) {
				t.WriteByte('%')
				t.WriteByte(upperhex[b>>4])
				t.WriteByte(upperhex[b&15])
			}
		} else {
			t.WriteRune(c)
		}
	}
	return t.String()
}

// Decode does the inverse transformation of Encode, converting each 3-byte encoded substring of the form "%AB" into the hex-decoded byte 0xAB
func Decode(s string) string {
	var t []byte
	for i := 0; i < len(s); i++ {
		// check next 2 chars are valid
		if s[i] == '%' && i+2 < len(s) && ishex(s[i+1]) && ishex(s[i+2]) {
			t = append(t, (unhex(s[i+1])<<4 | unhex(s[i+2])))
			i += 2
		} else {
			t = append(t, s[i])
		}
	}
	return string(t)
}
