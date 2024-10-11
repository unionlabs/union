package startup

import (
	"time"

	types "github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// Nower is a function that can return the current time.
// In Clock, Now() will use time.Now by default, but a Nower can be set using WithNower in NewClock
// to customize the return value for Now() in tests.
type Nower func() time.Time

// Clock abstracts important time-related concerns in the beacon chain:
//   - provides a time.Now() construct that can be overridden in tests
//   - GenesisTime() to know the genesis time or use genesis time determination as a synchronization point.
//   - CurrentSlot: convenience conversion for current time -> slot
//     (support backwards compatibility with the TimeFetcher interface)
//   - GenesisValidatorsRoot: is determined at the same point as genesis time and is needed by some of the same code,
//     so it is also bundled for convenience.
type Clock struct {
	t   time.Time
	vr  [32]byte
	now Nower
}

// GenesisTime returns the genesis timestamp.
func (g *Clock) GenesisTime() time.Time {
	return g.t
}

// GenesisValidatorsRoot returns the genesis state validator root
func (g *Clock) GenesisValidatorsRoot() [32]byte {
	return g.vr
}

// CurrentSlot returns the current slot relative to the time.Time value that Clock embeds.
func (g *Clock) CurrentSlot() types.Slot {
	now := g.now()
	return slots.Duration(g.t, now)
}

// SlotStart computes the time the given slot begins.
func (g *Clock) SlotStart(slot types.Slot) time.Time {
	return slots.BeginsAt(slot, g.t)
}

// Now provides a value for time.Now() that can be overridden in tests.
func (g *Clock) Now() time.Time {
	return g.now()
}

// ClockOpt is a functional option to change the behavior of a clock value made by NewClock.
// It is primarily intended as a way to inject an alternate time.Now() callback (WithNower) for testing.
type ClockOpt func(*Clock)

// WithNower allows tests in particular to inject an alternate implementation of time.Now (vs using system time)
func WithNower(n Nower) ClockOpt {
	return func(g *Clock) {
		g.now = n
	}
}

// NewClock constructs a Clock value from a genesis timestamp (t) and a Genesis Validator Root (vr).
// The WithNower ClockOpt can be used in tests to specify an alternate `time.Now` implementation,
// for instance to return a value for `Now` spanning a certain number of slots from genesis time, to control the current slot.
func NewClock(t time.Time, vr [32]byte, opts ...ClockOpt) *Clock {
	c := &Clock{
		t:  t,
		vr: vr,
	}
	for _, o := range opts {
		o(c)
	}
	if c.now == nil {
		c.now = time.Now
	}
	return c
}
