package colorful

import (
	"math/rand"
	"testing"
	"time"
)

// This is really difficult to test, if you've got a good idea, pull request!

// Check if it returns all valid colors.
func TestColorValidity(t *testing.T) {
	seed := time.Now().UTC().UnixNano()
	rand.Seed(seed)

	for i := 0; i < 100; i++ {
		if col := WarmColor(); !col.IsValid() {
			t.Errorf("Warm color %v is not valid! Seed was: %v", col, seed)
		}

		if col := FastWarmColor(); !col.IsValid() {
			t.Errorf("Fast warm color %v is not valid! Seed was: %v", col, seed)
		}

		if col := HappyColor(); !col.IsValid() {
			t.Errorf("Happy color %v is not valid! Seed was: %v", col, seed)
		}

		if col := FastHappyColor(); !col.IsValid() {
			t.Errorf("Fast happy color %v is not valid! Seed was: %v", col, seed)
		}
	}
}
