package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"

// RateLimiter is used to control write rate of flush and
// compaction.
type RateLimiter struct {
	c *C.rocksdb_ratelimiter_t
}

// NewRateLimiter creates a default RateLimiter object.
func NewRateLimiter(rateBytesPerSec, refillPeriodMicros int64, fairness int32) *RateLimiter {
	cR := C.rocksdb_ratelimiter_create(
		C.int64_t(rateBytesPerSec),
		C.int64_t(refillPeriodMicros),
		C.int32_t(fairness),
	)
	return newNativeRateLimiter(cR)
}

// NewNativeRateLimiter creates a native RateLimiter object.
func newNativeRateLimiter(c *C.rocksdb_ratelimiter_t) *RateLimiter {
	return &RateLimiter{c: c}
}

// Destroy deallocates the RateLimiter object.
func (r *RateLimiter) Destroy() {
	C.rocksdb_ratelimiter_destroy(r.c)
	r.c = nil
}
