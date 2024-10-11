package humanize

import (
	"fmt"
	"math/rand"
	"reflect"
	"regexp"
	"strconv"
	"strings"
	"testing"
	"testing/quick"
)

func TestFtoa(t *testing.T) {
	testList{
		{"200", Ftoa(200), "200"},
		{"20", Ftoa(20.0), "20"},
		{"2", Ftoa(2), "2"},
		{"2.2", Ftoa(2.2), "2.2"},
		{"2.02", Ftoa(2.02), "2.02"},
		{"200.02", Ftoa(200.02), "200.02"},
	}.validate(t)
}

func TestFtoaWithDigits(t *testing.T) {
	testList{
		{"1.23, 0", FtoaWithDigits(1.23, 0), "1"},
		{"20, 0", FtoaWithDigits(20.0, 0), "20"},
		{"1.23, 1", FtoaWithDigits(1.23, 1), "1.2"},
		{"1.23, 2", FtoaWithDigits(1.23, 2), "1.23"},
		{"1.23, 3", FtoaWithDigits(1.23, 3), "1.23"},
	}.validate(t)
}

func TestStripTrailingDigits(t *testing.T) {
	err := quick.Check(func(s string, digits int) bool {
		stripped := stripTrailingDigits(s, digits)

		// A stripped string will always be a prefix of its original string
		if !strings.HasPrefix(s, stripped) {
			return false
		}

		if strings.ContainsRune(s, '.') {
			// If there is a dot, the part on the left of the dot will never change
			a := strings.Split(s, ".")
			b := strings.Split(stripped, ".")
			if a[0] != b[0] {
				return false
			}
		} else {
			// If there's no dot in the input, the output will always be the same as the input.
			if stripped != s {
				return false
			}
		}

		return true
	}, &quick.Config{
		MaxCount: 10000,
		Values: func(v []reflect.Value, r *rand.Rand) {
			rdigs := func(n int) string {
				digs := []rune{'0', '1', '2', '3', '4', '5', '6', '7', '8', '9'}
				var rv []rune
				for i := 0; i < n; i++ {
					rv = append(rv, digs[r.Intn(len(digs))])
				}
				return string(rv)
			}

			ls := r.Intn(20)
			rs := r.Intn(20)
			jc := "."
			if rs == 0 {
				jc = ""
			}
			s := rdigs(ls) + jc + rdigs(rs)
			digits := r.Intn(len(s) + 1)

			v[0] = reflect.ValueOf(s)
			v[1] = reflect.ValueOf(digits)
		},
	})

	if err != nil {
		t.Error(err)
	}
}

func BenchmarkFtoaRegexTrailing(b *testing.B) {
	trailingZerosRegex := regexp.MustCompile(`\.?0+$`)

	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		trailingZerosRegex.ReplaceAllString("2.00000", "")
		trailingZerosRegex.ReplaceAllString("2.0000", "")
		trailingZerosRegex.ReplaceAllString("2.000", "")
		trailingZerosRegex.ReplaceAllString("2.00", "")
		trailingZerosRegex.ReplaceAllString("2.0", "")
		trailingZerosRegex.ReplaceAllString("2", "")
	}
}

func BenchmarkFtoaFunc(b *testing.B) {
	for i := 0; i < b.N; i++ {
		stripTrailingZeros("2.00000")
		stripTrailingZeros("2.0000")
		stripTrailingZeros("2.000")
		stripTrailingZeros("2.00")
		stripTrailingZeros("2.0")
		stripTrailingZeros("2")
	}
}

func BenchmarkFmtF(b *testing.B) {
	for i := 0; i < b.N; i++ {
		_ = fmt.Sprintf("%f", 2.03584)
	}
}

func BenchmarkStrconvF(b *testing.B) {
	for i := 0; i < b.N; i++ {
		strconv.FormatFloat(2.03584, 'f', 6, 64)
	}
}
