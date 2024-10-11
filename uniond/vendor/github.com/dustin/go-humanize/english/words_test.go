package english

import (
	"testing"
)

func TestPluralWord(t *testing.T) {
	tests := []struct {
		n                int
		singular, plural string
		want             string
	}{
		{0, "object", "", "objects"},
		{1, "object", "", "object"},
		{-1, "object", "", "objects"},
		{42, "object", "", "objects"},
		{2, "vax", "vaxen", "vaxen"},

		// special cases
		{2, "index", "", "indices"},

		// ending in a sibilant sound
		{2, "bus", "", "buses"},
		{2, "bush", "", "bushes"},
		{2, "watch", "", "watches"},
		{2, "box", "", "boxes"},

		// ending with 'o' preceded by a consonant
		{2, "hero", "", "heroes"},

		// ending with 'y' preceded by a consonant
		{2, "lady", "", "ladies"},
		{2, "day", "", "days"},
	}
	for _, tt := range tests {
		if got := PluralWord(tt.n, tt.singular, tt.plural); got != tt.want {
			t.Errorf("PluralWord(%d, %q, %q)=%q; want: %q", tt.n, tt.singular, tt.plural, got, tt.want)
		}
	}
}

func TestPlural(t *testing.T) {
	tests := []struct {
		n                int
		singular, plural string
		want             string
	}{
		{1, "object", "", "1 object"},
		{42, "object", "", "42 objects"},
		{1234567, "object", "", "1,234,567 objects"},
	}
	for _, tt := range tests {
		if got := Plural(tt.n, tt.singular, tt.plural); got != tt.want {
			t.Errorf("Plural(%d, %q, %q)=%q; want: %q", tt.n, tt.singular, tt.plural, got, tt.want)
		}
	}
}

func TestWordSeries(t *testing.T) {
	tests := []struct {
		words       []string
		conjunction string
		want        string
	}{
		{[]string{}, "and", ""},
		{[]string{"foo"}, "and", "foo"},
		{[]string{"foo", "bar"}, "and", "foo and bar"},
		{[]string{"foo", "bar", "baz"}, "and", "foo, bar and baz"},
		{[]string{"foo", "bar", "baz"}, "or", "foo, bar or baz"},
	}
	for _, tt := range tests {
		if got := WordSeries(tt.words, tt.conjunction); got != tt.want {
			t.Errorf("WordSeries(%q, %q)=%q; want: %q", tt.words, tt.conjunction, got, tt.want)
		}
	}
}

func TestOxfordWordSeries(t *testing.T) {
	tests := []struct {
		words       []string
		conjunction string
		want        string
	}{
		{[]string{}, "and", ""},
		{[]string{"foo"}, "and", "foo"},
		{[]string{"foo", "bar"}, "and", "foo and bar"},
		{[]string{"foo", "bar", "baz"}, "and", "foo, bar, and baz"},
		{[]string{"foo", "bar", "baz"}, "or", "foo, bar, or baz"},
	}
	for _, tt := range tests {
		if got := OxfordWordSeries(tt.words, tt.conjunction); got != tt.want {
			t.Errorf("OxfordWordSeries(%q, %q)=%q; want: %q", tt.words, tt.conjunction, got, tt.want)
		}
	}
}
