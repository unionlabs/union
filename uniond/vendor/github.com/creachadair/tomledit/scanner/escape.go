// Copyright (C) 2022 Michael J. Fromberger. All Rights Reserved.

package scanner

import (
	"bytes"
	"errors"
	"fmt"
	"unicode/utf8"
)

var controlEsc = [...]byte{
	'\b': 'b',
	'\f': 'f',
	'\n': 'n',
	'\r': 'r',
	'\t': 't',
	' ':  ' ', // sentinel
}

var hexDigit = []byte("0123456789abcdef")

// IsWord reports whether s can be encoded as a bare (unquoted) word in a TOML
// key-value pair or table name.
func IsWord(s string) bool {
	for _, r := range s {
		if !isWordRune(r) {
			return false
		}
	}
	return s != ""
}

// Escape encodes a string to escape characters for a TOML basic string.
// The result is not quoted, the caller must add quotation marks.
func Escape(src string) []byte { return escape(src, false) }

// EscapeMultiline encodes a string to escape characters for a TOML multi-line
// basic string. The result is not quoted, the caller must add quotation marks.
func EscapeMultiline(src string) []byte { return escape(src, true) }

func escape(src string, isMulti bool) []byte {
	var buf bytes.Buffer
	for _, r := range src {
		if r < utf8.RuneSelf {
			if isMulti && r == '\n' {
				// emit newlines literally in a multi-line string
			} else if r < ' ' {
				buf.WriteByte('\\')
				if b := controlEsc[r]; b != 0 {
					buf.WriteByte(b)
				} else {
					buf.WriteString("u00")
					buf.WriteByte(hexDigit[int(r>>4)])
					buf.WriteByte(hexDigit[int(r&15)])
				}
				continue
			} else if r == '\\' || r == '"' {
				buf.WriteByte('\\')
			}
			buf.WriteByte(byte(r))
			continue
		}

		switch r {
		case '\ufffd': // replacement rune
			buf.WriteString(`\ufffd`)
		case '\u2028': // line separator
			buf.WriteString(`\u2028`)
		case '\u2029': // paragraph separator
			buf.WriteString(`\u2029`)
		default:
			buf.WriteRune(r)
		}
	}
	return buf.Bytes()
}

// Unescape decodes a byte slice containing a TOML basic string. The input must
// have the enclosing double quotation marks already removed.
//
// Escape sequences are replaced with their unescaped equivalents. Invalid
// escapes are replaced by the Unicode replacement rune. Unescape reports an
// error for an incomplete escape sequence.
func Unescape(src []byte) ([]byte, error) {
	if !bytes.ContainsRune(src, '\\') {
		return src, nil
	}

	dec := bytes.NewBuffer(make([]byte, 0, len(src)))
	for len(src) != 0 {
		i := bytes.IndexRune(src, '\\')
		if i < 0 {
			dec.Write(src)
			break
		}
		dec.Write(src[:i])

		// Decode the next rune after the escape to figure out what to
		// substitute. There should not be errors here, but if there are, insert
		// replacement runes (utf8.RuneError == '\ufffd').
		src = src[i+1:]
		if len(src) == 0 {
			return nil, errors.New("incomplete escape sequence")
		}
		r, n := utf8.DecodeRune(src)
		if n == 0 {
			n++
		}

		src = src[n:]
		switch r {
		case '"', '\\', '/':
			dec.WriteByte(byte(r))
		case '\n':
			dec.WriteString("\\\n")
		case 'b':
			dec.WriteByte('\b')
		case 'f':
			dec.WriteByte('\f')
		case 'n':
			dec.WriteByte('\n')
		case 'r':
			dec.WriteByte('\r')
		case 't':
			dec.WriteByte('\t')
		case 'u', 'U':
			n := 4
			if r == 'U' {
				n = 8
			}
			if len(src) < n {
				return nil, errors.New("incomplete Unicode escape")
			}
			v, err := parseHex(src[:n])
			if err != nil {
				dec.WriteRune(utf8.RuneError)
			} else {
				dec.WriteRune(rune(v))
			}
			src = src[n:]
		default:
			dec.WriteRune(utf8.RuneError)
		}
	}
	return dec.Bytes(), nil
}

func parseHex(data []byte) (int64, error) {
	var v int64
	for _, b := range data {
		v <<= 4
		if '0' <= b && b <= '9' {
			v += int64(b - '0')
		} else if 'a' <= b && b <= 'f' {
			v += int64(b - 'a' + 10)
		} else if 'A' <= b && b <= 'F' {
			v += int64(b - 'A' + 10)
		} else {
			return 0, fmt.Errorf("invalid hex digit %q", b)
		}
	}
	return v, nil
}
