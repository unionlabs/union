package epoch_processing

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/spectest/shared/bellatrix/epoch_processing"
)

func TestMinimal_Bellatrix_EpochProcessing_HistoricalRootsUpdate(t *testing.T) {
	epoch_processing.RunHistoricalRootsUpdateTests(t, "minimal")
}
