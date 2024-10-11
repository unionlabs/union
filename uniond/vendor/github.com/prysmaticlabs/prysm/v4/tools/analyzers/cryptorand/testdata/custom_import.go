package testdata

import (
	foobar "math/rand"
	mathRand "math/rand"
	"time"
)

// UseRandNewCustomImport --
func UseRandNewCustomImport() {
	// #nosec G404
	source := mathRand.NewSource(time.Now().UnixNano()) // want "crypto-secure RNGs are required, use CSPRNG or PRNG defined in github.com/prysmaticlabs/prysm/crypto/rand"
	// #nosec G404
	randGenerator := mathRand.New(source) // want "crypto-secure RNGs are required, use CSPRNG or PRNG defined in github.com/prysmaticlabs/prysm/crypto/rand"
	start := uint64(randGenerator.Intn(32))
	_ = start

	// #nosec G404
	source = mathRand.NewSource(time.Now().UnixNano()) // want "crypto-secure RNGs are required, use CSPRNG or PRNG defined in github.com/prysmaticlabs/prysm/crypto/rand"
	// #nosec G404
	randGenerator = mathRand.New(source) // want "crypto-secure RNGs are required, use CSPRNG or PRNG defined in github.com/prysmaticlabs/prysm/crypto/rand"
}

// UseWithoutSeeCustomImport --
func UseWithoutSeeCustomImport() {
	// #nosec G404
	assignedIndex := mathRand.Intn(128) // want "crypto-secure RNGs are required, use CSPRNG or PRNG defined in github.com/prysmaticlabs/prysm/crypto/rand"
	_ = assignedIndex
	// #nosec G404
	foobar.Shuffle(10, func(i, j int) { // want "crypto-secure RNGs are required, use CSPRNG or PRNG defined in github.com/prysmaticlabs/prysm/crypto/rand"

	})
}
