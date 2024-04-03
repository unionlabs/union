package grocksdb

// #include <stdlib.h>
// #include "rocksdb/c.h"
import "C"

// RateLimiter is used to control write rate of flush and
// compaction.
type RateLimiter struct {
	c *C.rocksdb_ratelimiter_t
}

// NewRateLimiter creates a RateLimiter object, which can be shared among RocksDB instances to
// control write rate of flush and compaction.
//
// @rate_bytes_per_sec: this is the only parameter you want to set most of the
// time. It controls the total write rate of compaction and flush in bytes per
// second. Currently, RocksDB does not enforce rate limit for anything other
// than flush and compaction, e.g. write to WAL.
//
// @refill_period_us: this controls how often tokens are refilled. For example,
// when rate_bytes_per_sec is set to 10MB/s and refill_period_us is set to
// 100ms, then 1MB is refilled every 100ms internally. Larger value can lead to
// burstier writes while smaller value introduces more CPU overhead.
// The default should work for most cases.
//
// @fairness: RateLimiter accepts high-pri requests and low-pri requests.
// A low-pri request is usually blocked in favor of hi-pri request. Currently,
// RocksDB assigns low-pri to request from compaction and high-pri to request
// from flush. Low-pri requests can get blocked if flush requests come in
// continuously. This fairness parameter grants low-pri requests permission by
// 1/fairness chance even though high-pri requests exist to avoid starvation.
// You should be good by leaving it at default 10.
func NewRateLimiter(rateBytesPerSec, refillPeriodMicros int64, fairness int32) *RateLimiter {
	cR := C.rocksdb_ratelimiter_create(
		C.int64_t(rateBytesPerSec),
		C.int64_t(refillPeriodMicros),
		C.int32_t(fairness),
	)
	return newNativeRateLimiter(cR)
}

// NewAutoTunedRateLimiter similar to NewRateLimiter, enables dynamic adjustment of rate
// limit within the range `[rate_bytes_per_sec / 20, rate_bytes_per_sec]`, according to
// the recent demand for background I/O.
func NewAutoTunedRateLimiter(rateBytesPerSec, refillPeriodMicros int64, fairness int32) *RateLimiter {
	cR := C.rocksdb_ratelimiter_create_auto_tuned(
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
