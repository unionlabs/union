package ratelimit

import (
	"net/http"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
)

func TestFromResponse(t *testing.T) {
	tests := []struct {
		name     string
		response *http.Response
		want     Map
	}{
		{
			"200 no rate limit",
			&http.Response{
				StatusCode: http.StatusOK,
			},
			Map{},
		},
		{
			"200 ignored Retry-After",
			&http.Response{
				StatusCode: http.StatusOK,
				Header: http.Header{
					"Retry-After": []string{"100"}, // ignored
				},
			},
			Map{},
		},
		{
			"200 Retry-After + X-Sentry-Rate-Limits",
			&http.Response{
				StatusCode: http.StatusOK,
				Header: http.Header{
					"Retry-After":          []string{"100"}, // ignored
					"X-Sentry-Rate-Limits": []string{"50:transaction"},
				},
			},
			Map{CategoryTransaction: Deadline(now.Add(50 * time.Second))},
		},
		{
			"200 X-Sentry-Rate-Limits",
			&http.Response{
				StatusCode: http.StatusOK,
				Header: http.Header{
					"X-Sentry-Rate-Limits": []string{"50:transaction"},
				},
			},
			Map{CategoryTransaction: Deadline(now.Add(50 * time.Second))},
		},
		{
			"429 no rate limit, use default",
			&http.Response{
				StatusCode: http.StatusTooManyRequests,
			},
			Map{CategoryAll: Deadline(now.Add(defaultRetryAfter))},
		},
		{
			"429 Retry-After",
			&http.Response{
				StatusCode: http.StatusTooManyRequests,
				Header: http.Header{
					"Retry-After": []string{"100"},
				},
			},
			Map{CategoryAll: Deadline(now.Add(100 * time.Second))},
		},
		{
			"429 X-Sentry-Rate-Limits",
			&http.Response{
				StatusCode: http.StatusTooManyRequests,
				Header: http.Header{
					"X-Sentry-Rate-Limits": []string{"50:error"},
				},
			},
			Map{CategoryError: Deadline(now.Add(50 * time.Second))},
		},
		{
			"429 Retry-After + X-Sentry-Rate-Limits",
			&http.Response{
				StatusCode: http.StatusTooManyRequests,
				Header: http.Header{
					"Retry-After":          []string{"100"}, // ignored
					"X-Sentry-Rate-Limits": []string{"50:error"},
				},
			},
			Map{CategoryError: Deadline(now.Add(50 * time.Second))},
		},
	}
	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			got := fromResponse(tt.response, now)
			if diff := cmp.Diff(tt.want, got); diff != "" {
				t.Errorf("(-want +got):\n%s", diff)
			}
		})
	}
}

func TestMapDeadlineIsRateLimited(t *testing.T) {
	noDeadline := Deadline{}
	plus5s := Deadline(now.Add(5 * time.Second))
	plus10s := Deadline(now.Add(10 * time.Second))
	future := now.Add(time.Hour)

	tests := []struct {
		name string
		m    Map
		want map[Category]Deadline
	}{
		{
			"Empty map = no deadlines",
			Map{},
			map[Category]Deadline{
				CategoryAll:         noDeadline,
				CategoryError:       noDeadline,
				CategoryTransaction: noDeadline,
				Category("unknown"): noDeadline,
			},
		},
		{
			"Only one category",
			Map{
				CategoryError: plus5s,
			},
			map[Category]Deadline{
				CategoryAll:         noDeadline,
				CategoryError:       plus5s,
				CategoryTransaction: noDeadline,
				Category("unknown"): noDeadline,
			},
		},
		{
			"Only CategoryAll",
			Map{
				CategoryAll: plus5s,
			},
			map[Category]Deadline{
				CategoryAll:         plus5s,
				CategoryError:       plus5s,
				CategoryTransaction: plus5s,
				Category("unknown"): plus5s,
			},
		},
		{
			"Two categories",
			Map{
				CategoryError:       plus5s,
				CategoryTransaction: plus10s,
			},
			map[Category]Deadline{
				CategoryAll:         noDeadline,
				CategoryError:       plus5s,
				CategoryTransaction: plus10s,
				Category("unknown"): noDeadline,
			},
		},
		{
			"CategoryAll earlier",
			Map{
				CategoryAll:         plus5s,
				CategoryTransaction: plus10s,
			},
			map[Category]Deadline{
				CategoryAll:         plus5s,
				CategoryError:       plus5s,
				CategoryTransaction: plus10s,
				Category("unknown"): plus5s,
			},
		},
		{
			"CategoryAll later",
			Map{
				CategoryAll:         plus10s,
				CategoryTransaction: plus5s,
			},
			map[Category]Deadline{
				CategoryAll:         plus10s,
				CategoryError:       plus10s,
				CategoryTransaction: plus10s,
				Category("unknown"): plus10s,
			},
		},
	}
	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			for c, want := range tt.want {
				got := tt.m.Deadline(c)
				if got != want {
					t.Fatalf("Deadline(%v): got %v, want %v", c, got, want)
				}
				limited := tt.m.isRateLimited(c, now)
				wantLimited := want != noDeadline
				if limited != wantLimited {
					t.Errorf("isRateLimited(%v, now): got %v, want %v", c, limited, wantLimited)
				}
				// Nothing should be rate-limited in the future
				limited = tt.m.isRateLimited(c, future)
				wantLimited = false
				if limited != wantLimited {
					t.Errorf("isRateLimited(%v, future): got %v, want %v", c, limited, wantLimited)
				}
			}
		})
	}
}

func TestMapMerge(t *testing.T) {
	tests := []struct {
		name     string
		old, new Map
		want     Map
	}{
		{
			name: "both empty",
			old:  Map{},
			new:  Map{},
			want: Map{},
		},
		{
			name: "old empty",
			old:  Map{},
			new:  Map{CategoryError: Deadline(now)},
			want: Map{CategoryError: Deadline(now)},
		},
		{
			name: "new empty",
			old:  Map{CategoryError: Deadline(now)},
			new:  Map{},
			want: Map{CategoryError: Deadline(now)},
		},
		{
			name: "no overlap = union",
			old:  Map{CategoryTransaction: Deadline(now)},
			new:  Map{CategoryError: Deadline(now)},
			want: Map{
				CategoryTransaction: Deadline(now),
				CategoryError:       Deadline(now),
			},
		},
		{
			name: "overlap keep old",
			old:  Map{CategoryError: Deadline(now.Add(time.Minute))},
			new:  Map{CategoryError: Deadline(now)},
			want: Map{CategoryError: Deadline(now.Add(time.Minute))},
		},
		{
			name: "overlap replace with new",
			old:  Map{CategoryError: Deadline(now)},
			new:  Map{CategoryError: Deadline(now.Add(time.Minute))},
			want: Map{CategoryError: Deadline(now.Add(time.Minute))},
		},
	}
	for _, tt := range tests {
		tt := tt
		t.Run(tt.name, func(t *testing.T) {
			tt.old.Merge(tt.new)
			if diff := cmp.Diff(tt.want, tt.old); diff != "" {
				t.Errorf("(-want +got):\n%s", diff)
			}
		})
	}
}
