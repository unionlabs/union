package cachekv

import "testing"

func TestFindStartIndex(t *testing.T) {
	tests := []struct {
		name    string
		sortedL []string
		query   string
		want    int
	}{
		{
			name:    "non-existent value",
			sortedL: []string{"a", "b", "c", "d", "e", "l", "m", "n", "u", "v", "w", "x", "y", "z"},
			query:   "o",
			want:    8,
		},
		{
			name:    "dupes start at index 0",
			sortedL: []string{"a", "a", "a", "b", "c", "d", "e", "l", "m", "n", "u", "v", "w", "x", "y", "z"},
			query:   "a",
			want:    0,
		},
		{
			name:    "dupes start at non-index 0",
			sortedL: []string{"a", "c", "c", "c", "c", "d", "e", "l", "m", "n", "u", "v", "w", "x", "y", "z"},
			query:   "c",
			want:    1,
		},
		{
			name:    "at end",
			sortedL: []string{"a", "e", "u", "v", "w", "x", "y", "z"},
			query:   "z",
			want:    7,
		},
		{
			name:    "dupes at end",
			sortedL: []string{"a", "e", "u", "v", "w", "x", "y", "z", "z", "z", "z"},
			query:   "z",
			want:    7,
		},
		{
			name:    "entirely dupes",
			sortedL: []string{"z", "z", "z", "z", "z"},
			query:   "z",
			want:    0,
		},
		{
			name:    "non-existent but within >=start",
			sortedL: []string{"z", "z", "z", "z", "z"},
			query:   "p",
			want:    0,
		},
		{
			name:    "non-existent and out of range",
			sortedL: []string{"d", "e", "f", "g", "h"},
			query:   "z",
			want:    -1,
		},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			body := tt.sortedL
			got := findStartIndex(body, tt.query)
			if got != tt.want {
				t.Fatalf("Got: %d, want: %d", got, tt.want)
			}
		})
	}
}

func TestFindEndIndex(t *testing.T) {
	tests := []struct {
		name    string
		sortedL []string
		query   string
		want    int
	}{
		{
			name:    "non-existent value",
			sortedL: []string{"a", "b", "c", "d", "e", "l", "m", "n", "u", "v", "w", "x", "y", "z"},
			query:   "o",
			want:    7,
		},
		{
			name:    "dupes start at index 0",
			sortedL: []string{"a", "a", "a", "b", "c", "d", "e", "l", "m", "n", "u", "v", "w", "x", "y", "z"},
			query:   "a",
			want:    0,
		},
		{
			name:    "dupes start at non-index 0",
			sortedL: []string{"a", "c", "c", "c", "c", "d", "e", "l", "m", "n", "u", "v", "w", "x", "y", "z"},
			query:   "c",
			want:    1,
		},
		{
			name:    "at end",
			sortedL: []string{"a", "e", "u", "v", "w", "x", "y", "z"},
			query:   "z",
			want:    7,
		},
		{
			name:    "dupes at end",
			sortedL: []string{"a", "e", "u", "v", "w", "x", "y", "z", "z", "z", "z"},
			query:   "z",
			want:    7,
		},
		{
			name:    "entirely dupes",
			sortedL: []string{"z", "z", "z", "z", "z"},
			query:   "z",
			want:    0,
		},
		{
			name:    "non-existent and out of range",
			sortedL: []string{"z", "z", "z", "z", "z"},
			query:   "p",
			want:    -1,
		},
		{
			name:    "non-existent and out of range",
			sortedL: []string{"d", "e", "f", "g", "h"},
			query:   "z",
			want:    4,
		},
	}

	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			body := tt.sortedL
			got := findEndIndex(body, tt.query)
			if got != tt.want {
				t.Fatalf("Got: %d, want: %d", got, tt.want)
			}
		})
	}
}
