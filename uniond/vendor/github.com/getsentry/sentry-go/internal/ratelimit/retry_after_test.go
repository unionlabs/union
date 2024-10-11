package ratelimit

import (
	"testing"
	"time"
)

func TestParseRetryAfter(t *testing.T) {
	defaultDeadline := Deadline(now.Add(defaultRetryAfter))
	tests := map[string]struct {
		input   string
		want    Deadline
		wantErr bool // if true, want is set to defaultDeadline
	}{
		// Invalid input
		"Empty": {
			input:   "",
			wantErr: true,
		},
		"BadString": {
			input:   "x",
			wantErr: true,
		},
		"Negative": {
			input:   "-1",
			wantErr: true,
		},
		"Float": {
			input:   "5.0",
			wantErr: true,
		},
		// Valid input
		"Integer": {
			input: "1337",
			want:  Deadline(now.Add(1337 * time.Second)),
		},
		"Date": {
			input: "Fri, 08 Mar 2019 11:17:09 GMT",
			want:  Deadline(time.Date(2019, 3, 8, 11, 17, 9, 0, time.UTC)),
		},
	}
	for name, tt := range tests {
		tt := tt
		t.Run(name, func(t *testing.T) {
			got, err := parseRetryAfter(tt.input, now)
			want := tt.want
			if tt.wantErr {
				want = defaultDeadline
				if err == nil {
					t.Errorf("got err = nil, want non-nil")
				}
			}
			if !got.Equal(want) {
				t.Errorf("got %v, want %v", got, want)
			}
		})
	}
}
