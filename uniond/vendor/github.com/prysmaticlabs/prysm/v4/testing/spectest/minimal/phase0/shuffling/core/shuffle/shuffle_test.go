package shuffle

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/spectest/shared/phase0/shuffling/core/shuffle"
)

func TestMinimal_Phase0_Shuffling_Core_Shuffle(t *testing.T) {
	shuffle.RunShuffleTests(t, "minimal")
}
