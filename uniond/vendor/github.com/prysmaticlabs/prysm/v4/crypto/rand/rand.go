/*
Package rand defines methods of obtaining random number generators.

One is expected to use randomness from this package only, without introducing any other packages.
This limits the scope of code that needs to be hardened.

There are two modes, one for deterministic and another non-deterministic randomness:
1. If deterministic pseudo-random generator is enough, use:

		import "github.com/prysmaticlabs/prysm/v4/crypto/rand"
		randGen := rand.NewDeterministicGenerator()
		randGen.Intn(32) // or any other func defined in math.rand API

	   In this mode, only seed is generated using cryptographically secure source (crypto/rand). So,
	   once seed is obtained, and generator is seeded, the next generations are deterministic, thus fast.
	   However given that we only seed this 63 bits from crypto/rand and use math/rand to generate the outputs,
	   this method is not cryptographically secure. This is directly stated in the math/rand package,
	   https://github.com/golang/go/blob/release-branch.go1.17/src/math/rand/rand.go#L15. For any security
	   sensitive work this particular generator is NOT to be used.

2. For cryptographically secure non-deterministic mode (CSPRNG), use:

		import "github.com/prysmaticlabs/prysm/v4/crypto/rand"
		randGen := rand.NewGenerator()
		randGen.Intn(32) // or any other func defined in math.rand API

	   Again, any of the functions from `math/rand` can be used, however, they all use custom source
	   of randomness (crypto/rand), on every step. This makes randomness non-deterministic. However,
	   you take a performance hit -- as it is an order of magnitude slower.
*/
package rand

import (
	"crypto/rand"
	"encoding/binary"
	mrand "math/rand"
	"sync"
)

type source struct{}

var lock sync.RWMutex
var _ mrand.Source64 = (*source)(nil) // #nosec G404 -- This ensures we meet the interface

// Seed does nothing when crypto/rand is used as source.
func (_ *source) Seed(_ int64) {}

// Int63 returns uniformly-distributed random (as in CSPRNG) int64 value within [0, 1<<63) range.
// Panics if random generator reader cannot return data.
func (s *source) Int63() int64 {
	return int64(s.Uint64() & ^uint64(1<<63))
}

// Uint64 returns uniformly-distributed random (as in CSPRNG) uint64 value within [0, 1<<64) range.
// Panics if random generator reader cannot return data.
func (_ *source) Uint64() (val uint64) {
	lock.RLock()
	defer lock.RUnlock()
	if err := binary.Read(rand.Reader, binary.BigEndian, &val); err != nil {
		panic(err)
	}
	return
}

// Rand is alias for underlying random generator.
type Rand = mrand.Rand // #nosec G404

// NewGenerator returns a new generator that uses random values from crypto/rand as a source
// (cryptographically secure random number generator).
// Panics if crypto/rand input cannot be read.
// Use it for everything where crypto secure non-deterministic randomness is required. Performance
// takes a hit, so use sparingly.
func NewGenerator() *Rand {
	return mrand.New(&source{}) // #nosec G404 -- excluded
}

// NewDeterministicGenerator returns a random generator which is only seeded with crypto/rand,
// but is deterministic otherwise (given seed, produces given results, deterministically).
// Panics if crypto/rand input cannot be read.
// Use this method for performance, where deterministic pseudo-random behaviour is enough.
// Otherwise, rely on NewGenerator(). This method is not cryptographically secure as outputs
// can be potentially predicted even without knowledge of the underlying seed.
func NewDeterministicGenerator() *Rand {
	randGen := NewGenerator()
	return mrand.New(mrand.NewSource(randGen.Int63())) // #nosec G404 -- excluded
}
