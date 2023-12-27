package ratelimit

import (
	"errors"
	"math"
	"strconv"
	"strings"
	"time"
)

var errInvalidXSRLRetryAfter = errors.New("invalid retry-after value")

// parseXSentryRateLimits returns a RateLimits map by parsing an input string in
// the format of the X-Sentry-Rate-Limits header.
//
// Example
//
//	X-Sentry-Rate-Limits: 60:transaction, 2700:default;error;security
//
// This will rate limit transactions for the next 60 seconds and errors for the
// next 2700 seconds.
//
// Limits for unknown categories are ignored.
func parseXSentryRateLimits(s string, now time.Time) Map {
	// https://github.com/getsentry/relay/blob/0424a2e017d193a93918053c90cdae9472d164bf/relay-server/src/utils/rate_limits.rs#L44-L82
	m := make(Map, len(knownCategories))
	for _, limit := range strings.Split(s, ",") {
		limit = strings.TrimSpace(limit)
		if limit == "" {
			continue
		}
		components := strings.Split(limit, ":")
		if len(components) == 0 {
			continue
		}
		retryAfter, err := parseXSRLRetryAfter(strings.TrimSpace(components[0]), now)
		if err != nil {
			continue
		}
		categories := ""
		if len(components) > 1 {
			categories = components[1]
		}
		for _, category := range strings.Split(categories, ";") {
			c := Category(strings.ToLower(strings.TrimSpace(category)))
			if _, ok := knownCategories[c]; !ok {
				// skip unknown categories, keep m small
				continue
			}
			// always keep the deadline furthest into the future
			if retryAfter.After(m[c]) {
				m[c] = retryAfter
			}
		}
	}
	return m
}

// parseXSRLRetryAfter parses a string into a retry-after rate limit deadline.
//
// Valid input is a number, possibly signed and possibly floating-point,
// indicating the number of seconds to wait before sending another request.
// Negative values are treated as zero. Fractional values are rounded to the
// next integer.
func parseXSRLRetryAfter(s string, now time.Time) (Deadline, error) {
	// https://github.com/getsentry/relay/blob/0424a2e017d193a93918053c90cdae9472d164bf/relay-quotas/src/rate_limit.rs#L88-L96
	f, err := strconv.ParseFloat(s, 64)
	if err != nil {
		return Deadline{}, errInvalidXSRLRetryAfter
	}
	d := time.Duration(math.Ceil(math.Max(f, 0.0))) * time.Second
	if d < 0 {
		d = 0
	}
	return Deadline(now.Add(d)), nil
}
