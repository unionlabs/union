package misspell

import (
	"math"
	"testing"
)

func TestAlmostEqual(t *testing.T) {
	t.Parallel()

	tcs := []struct {
		inA  string
		inB  string
		want bool
	}{
		{"", "", true},
		{"", "a", true},
		{"a", "a", true},
		{"a", "b", true},
		{"hello", "hell", true},
		{"hello", "jello", true},
		{"hello", "helol", true},
		{"hello", "jelol", false},
	}
	for _, tc := range tcs {
		got := AlmostEqual(tc.inA, tc.inB)
		if got != tc.want {
			t.Errorf("AlmostEqual(%q, %q) = %v, want %v", tc.inA, tc.inB, got, tc.want)
		}
		// two tests for the price of one \o/
		if got != AlmostEqual(tc.inB, tc.inA) {
			t.Errorf("AlmostEqual(%q, %q) == %v != AlmostEqual(%q, %q)", tc.inA, tc.inB, got, tc.inB, tc.inA)
		}
	}
}

func FuzzAlmostEqual(f *testing.F) {
	f.Add("", "")
	f.Add("", "a")
	f.Add("a", "a")
	f.Add("a", "b")
	f.Add("hello", "hell")
	f.Add("hello", "jello")
	f.Add("hello", "helol")
	f.Add("hello", "jelol")
	f.Fuzz(func(t *testing.T, a, b string) {
		if len(a) > 10 || len(b) > 10 {
			// longer strings won't add coverage, but take longer to check
			return
		}
		d := editDistance([]rune(a), []rune(b))
		got := AlmostEqual(a, b)
		if want := d <= 1; got != want {
			t.Errorf("AlmostEqual(%q, %q) = %v, editDistance(%q, %q) = %d", a, b, got, a, b, d)
		}
		if got != AlmostEqual(b, a) {
			t.Errorf("AlmostEqual(%q, %q) == %v != AlmostEqual(%q, %q)", a, b, got, b, a)
		}
	})
}

// editDistance returns the Damerau-Levenshtein distance between a and b. It is
// inefficient, but by keeping almost verbatim to the recursive definition from
// Wikipedia, hopefully "obviously correct" and thus suitable for the fuzzing
// test of AlmostEqual.
func editDistance(a, b []rune) int {
	i, j := len(a), len(b)
	m := math.MaxInt
	if i == 0 && j == 0 {
		return 0
	}
	if i > 0 {
		m = min(m, editDistance(a[1:], b)+1)
	}
	if j > 0 {
		m = min(m, editDistance(a, b[1:])+1)
	}
	if i > 0 && j > 0 {
		d := editDistance(a[1:], b[1:])
		if a[0] != b[0] {
			d += 1
		}
		m = min(m, d)
	}
	if i > 1 && j > 1 && a[0] == b[1] && a[1] == b[0] {
		d := editDistance(a[2:], b[2:])
		if a[0] != b[0] {
			d += 1
		}
		m = min(m, d)
	}
	return m
}

func min(a, b int) int {
	if a < b {
		return a
	}
	return b
}
