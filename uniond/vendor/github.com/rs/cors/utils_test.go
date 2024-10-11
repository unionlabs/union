package cors

import (
	"strings"
	"testing"
)

func TestWildcard(t *testing.T) {
	w := wildcard{"foo", "bar"}
	if !w.match("foobar") {
		t.Error("foo*bar should match foobar")
	}
	if !w.match("foobazbar") {
		t.Error("foo*bar should match foobazbar")
	}
	if w.match("foobaz") {
		t.Error("foo*bar should not match foobaz")
	}

	w = wildcard{"foo", "oof"}
	if w.match("foof") {
		t.Error("foo*oof should not match foof")
	}
}

func TestConvert(t *testing.T) {
	s := convert([]string{"A", "b", "C"}, strings.ToLower)
	e := []string{"a", "b", "c"}
	if s[0] != e[0] || s[1] != e[1] || s[2] != e[2] {
		t.Errorf("%v != %v", s, e)
	}
}

func BenchmarkWildcard(b *testing.B) {
	w := wildcard{"foo", "bar"}
	b.Run("match", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			w.match("foobazbar")
		}
	})
	b.Run("too short", func(b *testing.B) {
		b.ReportAllocs()
		for i := 0; i < b.N; i++ {
			w.match("fobar")
		}
	})
}
