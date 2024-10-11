package runtime

import (
	"sort"
	"testing"
)

func TestWithIncomingHeaderMatcher_matchedMalformedHeaders(t *testing.T) {
	tests := []struct {
		name    string
		matcher HeaderMatcherFunc
		want    []string
	}{
		{
			"nil matcher returns nothing",
			nil,
			nil,
		},
		{
			"default matcher returns nothing",
			DefaultHeaderMatcher,
			nil,
		},
		{
			"passthrough matcher returns all malformed headers",
			func(s string) (string, bool) {
				return s, true
			},
			[]string{"connection"},
		},
	}

	sliceEqual := func(a, b []string) bool {
		if len(a) != len(b) {
			return false
		}
		sort.Slice(a, func(i, j int) bool {
			return a[i] < a[j]
		})
		sort.Slice(b, func(i, j int) bool {
			return a[i] < a[j]
		})
		for idx := range a {
			if a[idx] != b[idx] {
				return false
			}
		}
		return true
	}

	for _, tt := range tests {
		out := tt.matcher.matchedMalformedHeaders()
		if !sliceEqual(tt.want, out) {
			t.Errorf("matchedMalformedHeaders not match; Want %v; got %v",
				tt.want, out)
		}
	}
}
