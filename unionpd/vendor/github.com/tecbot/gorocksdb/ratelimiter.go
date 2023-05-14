package gorocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"

// RateLimiter, is used to control write rate of flush and
// compaction.
type RateLimiter struct {
	c *C.rocksdb_ratelimiter_t
}

// NewDefaultRateLimiter creates a default RateLimiter object.
func NewRateLimiter(rate_bytes_per_sec, refill_period_us int64, fairness int32) *RateLimiter {
	return NewNativeRateLimiter(C.rocksdb_ratelimiter_create(
		C.int64_t(rate_bytes_per_sec),
		C.int64_t(refill_period_us),
		C.int32_t(fairness),
	))
}

// NewNativeRateLimiter creates a native RateLimiter object.
func NewNativeRateLimiter(c *C.rocksdb_ratelimiter_t) *RateLimiter {
	return &RateLimiter{c}
}

// Destroy deallocates the RateLimiter object.
func (self *RateLimiter) Destroy() {
	C.rocksdb_ratelimiter_destroy(self.c)
	self.c = nil
}
