//nolint:dupword
/*
Package leakybucket implements a scalable leaky bucket algorithm.

There are at least two different definitions of the leaky bucket algorithm.
This package implements the leaky bucket as a meter. For more details see:

https://en.wikipedia.org/wiki/Leaky_bucket#As_a_meter

This means it is the exact mirror of a token bucket.

	// New LeakyBucket that leaks at the rate of 0.5/sec and a total capacity of 10.
	b := NewLeakyBucket(0.5, 10)

	b.Add(5)
	b.Add(5)
	// Bucket is now full!

	n := b.Add(1)
	// n == 0

A Collector is a convenient way to keep track of multiple LeakyBucket's.
Buckets are associated with string keys for fast lookup. It can dynamically
add new buckets and automatically remove them as they become empty, freeing
up resources.

	// New Collector that leaks at 1 MiB/sec, a total capacity of 10 MiB and
	// automatic removal of bucket's when they become empty.
	const megabyte = 1<<20
	c := NewCollector(megabyte, megabyte*10, true)

	// Attempt to add 100 MiB to a bucket associated with an IP.
	n := c.Add("192.168.0.42", megabyte*100)

	// 100 MiB is over the capacity, so only 10 MiB is actually added.
	// n equals 10 MiB.
*/
package leakybucket

import (
	"math"
	"time"
)

// Makes it easy to test time based things.
var now = time.Now

// LeakyBucket represents a bucket that leaks at a constant rate.
type LeakyBucket struct {
	// The identifying key, used for map lookups.
	key string

	// How large the bucket is.
	capacity int64

	// Amount the bucket leaks per time duration.
	rate float64

	// The priority of the bucket in a min-heap priority queue, where p is the
	// exact time the bucket will have leaked enough to be empty. Buckets that
	// are empty or will be the soonest are at the top of the heap. This allows
	// for quick pruning of empty buckets that scales very well. p is adjusted
	// any time an amount is added to the Queue().
	p time.Time

	// The time duration through which the leaky bucket is
	// assessed.
	period time.Duration

	// The index is maintained by the heap.Interface methods.
	index int
}

// NewLeakyBucket creates a new LeakyBucket with the give rate and capacity.
func NewLeakyBucket(rate float64, capacity int64, period time.Duration) *LeakyBucket {
	return &LeakyBucket{
		rate:     rate,
		capacity: capacity,
		period:   period,
		p:        now(),
	}
}

// Count returns the bucket's current count.
func (b *LeakyBucket) Count() int64 {
	if !now().Before(b.p) {
		return 0
	}

	nsRemaining := float64(b.p.Sub(now()))
	nsPerDrip := float64(b.period) / b.rate
	count := int64(math.Ceil(nsRemaining / nsPerDrip))

	return count
}

// Rate returns the amount the bucket leaks per second.
func (b *LeakyBucket) Rate() float64 {
	return b.rate
}

// Capacity returns the bucket's capacity.
func (b *LeakyBucket) Capacity() int64 {
	return b.capacity
}

// Remaining returns the bucket's remaining capacity.
func (b *LeakyBucket) Remaining() int64 {
	return b.capacity - b.Count()
}

// ChangeCapacity changes the bucket's capacity.
//
// If the bucket's current count is greater than the new capacity, the count
// will be decreased to match the new capacity.
func (b *LeakyBucket) ChangeCapacity(capacity int64) {
	diff := float64(capacity - b.capacity)

	if diff < 0 && b.Count() > capacity {
		// We are shrinking the capacity and the new bucket size can't hold all
		// the current contents. Dump the extra and adjust the time till empty.
		nsPerDrip := float64(b.period) / b.rate
		b.p = now().Add(time.Duration(nsPerDrip * float64(capacity)))
	}
	b.capacity = capacity
}

// TillEmpty returns how much time must pass until the bucket is empty.
func (b *LeakyBucket) TillEmpty() time.Duration {
	return b.p.Sub(now())
}

// Add 'amount' to the bucket's count, up to it's capacity. Returns how much
// was added to the bucket. If the return is less than 'amount', then the
// bucket's capacity was reached.
func (b *LeakyBucket) Add(amount int64) int64 {
	count := b.Count()
	if count >= b.capacity {
		// The bucket is full.
		return 0
	}

	if !now().Before(b.p) {
		// The bucket needs to be reset.
		b.p = now()
	}
	remaining := b.capacity - count
	if amount > remaining {
		amount = remaining
	}
	t := time.Duration(float64(b.period) * (float64(amount) / b.rate))
	b.p = b.p.Add(t)

	return amount
}
