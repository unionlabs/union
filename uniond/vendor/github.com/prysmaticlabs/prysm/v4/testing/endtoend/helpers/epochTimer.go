package helpers

import (
	"time"

	prysmTime "github.com/prysmaticlabs/prysm/v4/time"
)

// EpochTicker is a special ticker for timing epoch changes.
// The channel emits over the epoch interval, and ensures that
// the ticks are in line with the genesis time. This means that
// the duration between the ticks and the genesis time are always a
// multiple of the epoch duration.
// In addition, the channel returns the new epoch number.
type EpochTicker struct {
	c    chan uint64
	done chan struct{}
}

// C returns the ticker channel. Call Cancel afterwards to ensure
// that the goroutine exits cleanly.
func (s *EpochTicker) C() <-chan uint64 {
	return s.c
}

// Done should be called to clean up the ticker.
func (s *EpochTicker) Done() {
	go func() {
		s.done <- struct{}{}
	}()
}

// NewEpochTicker starts the EpochTicker.
func NewEpochTicker(genesisTime time.Time, secondsPerEpoch uint64) *EpochTicker {
	ticker := &EpochTicker{
		c:    make(chan uint64),
		done: make(chan struct{}),
	}
	ticker.start(genesisTime, secondsPerEpoch, prysmTime.Since, prysmTime.Until, time.After)
	return ticker
}

func (s *EpochTicker) start(
	genesisTime time.Time,
	secondsPerEpoch uint64,
	since, until func(time.Time) time.Duration,
	after func(time.Duration) <-chan time.Time) {
	d := time.Duration(secondsPerEpoch) * time.Second

	go func() {
		sinceGenesis := since(genesisTime)

		var nextTickTime time.Time
		var epoch uint64
		if sinceGenesis < 0 {
			// Handle when the current time is before the genesis time.
			nextTickTime = genesisTime
			epoch = 0
		} else {
			nextTick := sinceGenesis.Truncate(d) + d
			nextTickTime = genesisTime.Add(nextTick)
			epoch = uint64(nextTick / d)
		}

		for {
			waitTime := until(nextTickTime)
			select {
			case <-after(waitTime):
				s.c <- epoch
				epoch++
				nextTickTime = nextTickTime.Add(d)
			case <-s.done:
				return
			}
		}
	}()
}
