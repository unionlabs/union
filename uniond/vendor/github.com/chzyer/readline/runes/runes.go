// deprecated.
// see https://github.com/chzyer/readline/issues/43
// use github.com/chzyer/readline/runes.go
package runes

import (
	"bytes"
	"unicode"
)

func Equal(a, b []rune) bool {
	if len(a) != len(b) {
		return false
	}
	for i := 0; i < len(a); i++ {
		if a[i] != b[i] {
			return false
		}
	}
	return true
}

// Search in runes from end to front
func IndexAllBck(r, sub []rune) int {
	for i := len(r) - len(sub); i >= 0; i-- {
		found := true
		for j := 0; j < len(sub); j++ {
			if r[i+j] != sub[j] {
				found = false
				break
			}
		}
		if found {
			return i
		}
	}
	return -1
}

// Search in runes from front to end
func IndexAll(r, sub []rune) int {
	for i := 0; i < len(r); i++ {
		found := true
		if len(r[i:]) < len(sub) {
			return -1
		}
		for j := 0; j < len(sub); j++ {
			if r[i+j] != sub[j] {
				found = false
				break
			}
		}
		if found {
			return i
		}
	}
	return -1
}

func Index(r rune, rs []rune) int {
	for i := 0; i < len(rs); i++ {
		if rs[i] == r {
			return i
		}
	}
	return -1
}

func ColorFilter(r []rune) []rune {
	newr := make([]rune, 0, len(r))
	for pos := 0; pos < len(r); pos++ {
		if r[pos] == '\033' && r[pos+1] == '[' {
			idx := Index('m', r[pos+2:])
			if idx == -1 {
				continue
			}
			pos += idx + 2
			continue
		}
		newr = append(newr, r[pos])
	}
	return newr
}

var zeroWidth = []*unicode.RangeTable{
	unicode.Mn,
	unicode.Me,
	unicode.Cc,
	unicode.Cf,
}

var doubleWidth = []*unicode.RangeTable{
	unicode.Han,
	unicode.Hangul,
	unicode.Hiragana,
	unicode.Katakana,
}

func Width(r rune) int {
	if unicode.IsOneOf(zeroWidth, r) {
		return 0
	}
	if unicode.IsOneOf(doubleWidth, r) {
		return 2
	}
	return 1
}

func WidthAll(r []rune) (length int) {
	for i := 0; i < len(r); i++ {
		length += Width(r[i])
	}
	return
}

func Backspace(r []rune) []byte {
	return bytes.Repeat([]byte{'\b'}, WidthAll(r))
}

func Copy(r []rune) []rune {
	n := make([]rune, len(r))
	copy(n, r)
	return n
}

func HasPrefix(r, prefix []rune) bool {
	if len(r) < len(prefix) {
		return false
	}
	return Equal(r[:len(prefix)], prefix)
}

func Aggregate(candicate [][]rune) (same []rune, size int) {
	for i := 0; i < len(candicate[0]); i++ {
		for j := 0; j < len(candicate)-1; j++ {
			if i >= len(candicate[j]) || i >= len(candicate[j+1]) {
				goto aggregate
			}
			if candicate[j][i] != candicate[j+1][i] {
				goto aggregate
			}
		}
		size = i + 1
	}
aggregate:
	if size > 0 {
		same = Copy(candicate[0][:size])
		for i := 0; i < len(candicate); i++ {
			n := Copy(candicate[i])
			copy(n, n[size:])
			candicate[i] = n[:len(n)-size]
		}
	}
	return
}
