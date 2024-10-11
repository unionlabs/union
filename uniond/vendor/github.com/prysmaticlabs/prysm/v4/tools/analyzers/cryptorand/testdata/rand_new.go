package testdata

import (
	"math/rand"
	mathRand "math/rand"
	"time"
)

// UseRandNew --
func UseRandNew() {
	// #nosec G404
	source := rand.NewSource(time.Now().UnixNano()) // want "crypto-secure RNGs are required, use CSPRNG or PRNG defined in github.com/prysmaticlabs/prysm/crypto/rand"
	// #nosec G404
	randGenerator := mathRand.New(source) // want "crypto-secure RNGs are required, use CSPRNG or PRNG defined in github.com/prysmaticlabs/prysm/crypto/rand"
	start := uint64(randGenerator.Intn(32))
	_ = start

	// #nosec G404
	source = rand.NewSource(time.Now().UnixNano()) // want "crypto-secure RNGs are required, use CSPRNG or PRNG defined in github.com/prysmaticlabs/prysm/crypto/rand"
	// #nosec G404
	randGenerator = rand.New(source) // want "crypto-secure RNGs are required, use CSPRNG or PRNG defined in github.com/prysmaticlabs/prysm/crypto/rand"
}

// UseWithoutSeed --
func UseWithoutSeed() {
	// #nosec G404
	assignedIndex := rand.Intn(128) // want "crypto-secure RNGs are required, use CSPRNG or PRNG defined in github.com/prysmaticlabs/prysm/crypto/rand"
	_ = assignedIndex
}
