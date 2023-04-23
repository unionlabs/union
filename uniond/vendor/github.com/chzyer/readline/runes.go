package readline

import (
	"bytes"
	"unicode"
	"unicode/utf8"
)

var runes = Runes{}
var TabWidth = 4

type Runes struct{}

func (Runes) EqualRune(a, b rune, fold bool) bool {
	if a == b {
		return true
	}
	if !fold {
		return false
	}
	if a > b {
		a, b = b, a
	}
	if b < utf8.RuneSelf && 'A' <= a && a <= 'Z' {
		if b == a+'a'-'A' {
			return true
		}
	}
	return false
}

func (r Runes) EqualRuneFold(a, b rune) bool {
	return r.EqualRune(a, b, true)
}

func (r Runes) EqualFold(a, b []rune) bool {
	if len(a) != len(b) {
		return false
	}
	for i := 0; i < len(a); i++ {
		if r.EqualRuneFold(a[i], b[i]) {
			continue
		}
		return false
	}

	return true
}

func (Runes) Equal(a, b []rune) bool {
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

func (rs Runes) IndexAllBckEx(r, sub []rune, fold bool) int {
	for i := len(r) - len(sub); i >= 0; i-- {
		found := true
		for j := 0; j < len(sub); j++ {
			if !rs.EqualRune(r[i+j], sub[j], fold) {
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

// Search in runes from end to front
func (rs Runes) IndexAllBck(r, sub []rune) int {
	return rs.IndexAllBckEx(r, sub, false)
}

// Search in runes from front to end
func (rs Runes) IndexAll(r, sub []rune) int {
	return rs.IndexAllEx(r, sub, false)
}

func (rs Runes) IndexAllEx(r, sub []rune, fold bool) int {
	for i := 0; i < len(r); i++ {
		found := true
		if len(r[i:]) < len(sub) {
			return -1
		}
		for j := 0; j < len(sub); j++ {
			if !rs.EqualRune(r[i+j], sub[j], fold) {
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

func (Runes) Index(r rune, rs []rune) int {
	for i := 0; i < len(rs); i++ {
		if rs[i] == r {
			return i
		}
	}
	return -1
}

func (Runes) ColorFilter(r []rune) []rune {
	newr := make([]rune, 0, len(r))
	for pos := 0; pos < len(r); pos++ {
		if r[pos] == '\033' && r[pos+1] == '[' {
			idx := runes.Index('m', r[pos+2:])
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

func (Runes) Width(r rune) int {
	if r == '\t' {
		return TabWidth
	}
	if unicode.IsOneOf(zeroWidth, r) {
		return 0
	}
	if unicode.IsOneOf(doubleWidth, r) {
		return 2
	}
	return 1
}

func (Runes) WidthAll(r []rune) (length int) {
	for i := 0; i < len(r); i++ {
		length += runes.Width(r[i])
	}
	return
}

func (Runes) Backspace(r []rune) []byte {
	return bytes.Repeat([]byte{'\b'}, runes.WidthAll(r))
}

func (Runes) Copy(r []rune) []rune {
	n := make([]rune, len(r))
	copy(n, r)
	return n
}

func (Runes) HasPrefixFold(r, prefix []rune) bool {
	if len(r) < len(prefix) {
		return false
	}
	return runes.EqualFold(r[:len(prefix)], prefix)
}

func (Runes) HasPrefix(r, prefix []rune) bool {
	if len(r) < len(prefix) {
		return false
	}
	return runes.Equal(r[:len(prefix)], prefix)
}

func (Runes) Aggregate(candicate [][]rune) (same []rune, size int) {
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
		same = runes.Copy(candicate[0][:size])
		for i := 0; i < len(candicate); i++ {
			n := runes.Copy(candicate[i])
			copy(n, n[size:])
			candicate[i] = n[:len(n)-size]
		}
	}
	return
}

func (Runes) TrimSpaceLeft(in []rune) []rune {
	firstIndex := len(in)
	for i, r := range in {
		if unicode.IsSpace(r) == false {
			firstIndex = i
			break
		}
	}
	return in[firstIndex:]
}
