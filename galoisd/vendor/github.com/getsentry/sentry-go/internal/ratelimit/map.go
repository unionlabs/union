package ratelimit

import (
	"net/http"
	"time"
)

// Map maps categories to rate limit deadlines.
//
// A rate limit is in effect for a given category if either the category's
// deadline or the deadline for the special CategoryAll has not yet expired.
//
// Use IsRateLimited to check whether a category is rate-limited.
type Map map[Category]Deadline

// IsRateLimited returns true if the category is currently rate limited.
func (m Map) IsRateLimited(c Category) bool {
	return m.isRateLimited(c, time.Now())
}

func (m Map) isRateLimited(c Category, now time.Time) bool {
	return m.Deadline(c).After(Deadline(now))
}

// Deadline returns the deadline when the rate limit for the given category or
// the special CategoryAll expire, whichever is furthest into the future.
func (m Map) Deadline(c Category) Deadline {
	categoryDeadline := m[c]
	allDeadline := m[CategoryAll]
	if categoryDeadline.After(allDeadline) {
		return categoryDeadline
	}
	return allDeadline
}

// Merge merges the other map into m.
//
// If a category appears in both maps, the deadline that is furthest into the
// future is preserved.
func (m Map) Merge(other Map) {
	for c, d := range other {
		if d.After(m[c]) {
			m[c] = d
		}
	}
}

// FromResponse returns a rate limit map from an HTTP response.
func FromResponse(r *http.Response) Map {
	return fromResponse(r, time.Now())
}

func fromResponse(r *http.Response, now time.Time) Map {
	s := r.Header.Get("X-Sentry-Rate-Limits")
	if s != "" {
		return parseXSentryRateLimits(s, now)
	}
	if r.StatusCode == http.StatusTooManyRequests {
		s := r.Header.Get("Retry-After")
		deadline, _ := parseRetryAfter(s, now)
		return Map{CategoryAll: deadline}
	}
	return Map{}
}
