package ratelimit

import (
	"errors"
	"strconv"
	"time"
)

const defaultRetryAfter = 1 * time.Minute

var errInvalidRetryAfter = errors.New("invalid input")

// parseRetryAfter parses a string s as in the standard Retry-After HTTP header
// and returns a deadline until when requests are rate limited and therefore new
// requests should not be sent. The input may be either a date or a non-negative
// integer number of seconds.
//
// See https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Retry-After
//
// parseRetryAfter always returns a usable deadline, even in case of an error.
//
// This is the original rate limiting mechanism used by Sentry, superseeded by
// the X-Sentry-Rate-Limits response header.
func parseRetryAfter(s string, now time.Time) (Deadline, error) {
	if s == "" {
		goto invalid
	}
	if n, err := strconv.Atoi(s); err == nil {
		if n < 0 {
			goto invalid
		}
		d := time.Duration(n) * time.Second
		return Deadline(now.Add(d)), nil
	}
	if date, err := time.Parse(time.RFC1123, s); err == nil {
		return Deadline(date), nil
	}
invalid:
	return Deadline(now.Add(defaultRetryAfter)), errInvalidRetryAfter
}
