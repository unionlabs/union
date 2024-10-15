package zerolog

import (
	"math/rand"
	"sync/atomic"
	"time"
)

var (
	// Often samples log every ~ 10 events.
	Often = RandomSampler(10)
	// Sometimes samples log every ~ 100 events.
	Sometimes = RandomSampler(100)
	// Rarely samples log every ~ 1000 events.
	Rarely = RandomSampler(1000)
)

// Sampler defines an interface to a log sampler.
type Sampler interface {
	// Sample returns true if the event should be part of the sample, false if
	// the event should be dropped.
	Sample(lvl Level) bool
}

// RandomSampler use a PRNG to randomly sample an event out of N events,
// regardless of their level.
type RandomSampler uint32

// Sample implements the Sampler interface.
func (s RandomSampler) Sample(lvl Level) bool {
	if s <= 0 {
		return false
	}
	if rand.Intn(int(s)) != 0 {
		return false
	}
	return true
}

// BasicSampler is a sampler that will send every Nth events, regardless of
// their level.
type BasicSampler struct {
	N       uint32
	counter uint32
}

// Sample implements the Sampler interface.
func (s *BasicSampler) Sample(lvl Level) bool {
	n := s.N
	if n == 1 {
		return true
	}
	c := atomic.AddUint32(&s.counter, 1)
	return c%n == 1
}

// BurstSampler lets Burst events pass per Period then pass the decision to
// NextSampler. If Sampler is not set, all subsequent events are rejected.
type BurstSampler struct {
	// Burst is the maximum number of event per period allowed before calling
	// NextSampler.
	Burst uint32
	// Period defines the burst period. If 0, NextSampler is always called.
	Period time.Duration
	// NextSampler is the sampler used after the burst is reached. If nil,
	// events are always rejected after the burst.
	NextSampler Sampler

	counter uint32
	resetAt int64
}

// Sample implements the Sampler interface.
func (s *BurstSampler) Sample(lvl Level) bool {
	if s.Burst > 0 && s.Period > 0 {
		if s.inc() <= s.Burst {
			return true
		}
	}
	if s.NextSampler == nil {
		return false
	}
	return s.NextSampler.Sample(lvl)
}

func (s *BurstSampler) inc() uint32 {
	now := TimestampFunc().UnixNano()
	resetAt := atomic.LoadInt64(&s.resetAt)
	var c uint32
	if now > resetAt {
		c = 1
		atomic.StoreUint32(&s.counter, c)
		newResetAt := now + s.Period.Nanoseconds()
		reset := atomic.CompareAndSwapInt64(&s.resetAt, resetAt, newResetAt)
		if !reset {
			// Lost the race with another goroutine trying to reset.
			c = atomic.AddUint32(&s.counter, 1)
		}
	} else {
		c = atomic.AddUint32(&s.counter, 1)
	}
	return c
}

// LevelSampler applies a different sampler for each level.
type LevelSampler struct {
	TraceSampler, DebugSampler, InfoSampler, WarnSampler, ErrorSampler Sampler
}

func (s LevelSampler) Sample(lvl Level) bool {
	switch lvl {
	case TraceLevel:
		if s.TraceSampler != nil {
			return s.TraceSampler.Sample(lvl)
		}
	case DebugLevel:
		if s.DebugSampler != nil {
			return s.DebugSampler.Sample(lvl)
		}
	case InfoLevel:
		if s.InfoSampler != nil {
			return s.InfoSampler.Sample(lvl)
		}
	case WarnLevel:
		if s.WarnSampler != nil {
			return s.WarnSampler.Sample(lvl)
		}
	case ErrorLevel:
		if s.ErrorSampler != nil {
			return s.ErrorSampler.Sample(lvl)
		}
	}
	return true
}
