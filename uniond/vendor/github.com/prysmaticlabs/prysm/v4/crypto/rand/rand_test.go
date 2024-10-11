package rand

import (
	"math/rand"
	"testing"
)

func TestNewGenerator(_ *testing.T) {
	// Make sure that generation works, no panics.
	randGen := NewGenerator()
	_ = randGen.Int63()
	_ = randGen.Uint64()
	_ = randGen.Intn(32)
	var _ = rand.Source64(randGen)
}

func TestNewDeterministicGenerator(_ *testing.T) {
	// Make sure that generation works, no panics.
	randGen := NewDeterministicGenerator()
	_ = randGen.Int63()
	_ = randGen.Uint64()
	_ = randGen.Intn(32)
	var _ = rand.Source64(randGen)
}
