package lv

import (
	"strings"
	"testing"
)

func TestWith(t *testing.T) {
	var a LabelValues
	b := a.With("a", "1")
	c := a.With("b", "2", "c", "3")

	if want, have := "", strings.Join(a, ""); want != have {
		t.Errorf("With appears to mutate the original LabelValues: want %q, have %q", want, have)
	}
	if want, have := "a1", strings.Join(b, ""); want != have {
		t.Errorf("With does not appear to return the right thing: want %q, have %q", want, have)
	}
	if want, have := "b2c3", strings.Join(c, ""); want != have {
		t.Errorf("With does not appear to return the right thing: want %q, have %q", want, have)
	}
}
