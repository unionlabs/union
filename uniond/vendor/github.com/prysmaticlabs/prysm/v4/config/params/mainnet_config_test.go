package params

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

func TestMaxRequestBlock(t *testing.T) {
	testCases := []struct {
		epoch            primitives.Epoch
		expectedMaxBlock uint64
		description      string
	}{
		{
			epoch:            primitives.Epoch(mainnetDenebForkEpoch - 1), // Assuming the fork epoch is not 0
			expectedMaxBlock: mainnetBeaconConfig.MaxRequestBlocks,
		},
		{
			epoch:            primitives.Epoch(mainnetDenebForkEpoch),
			expectedMaxBlock: mainnetBeaconConfig.MaxRequestBlocksDeneb,
		},
	}

	for _, tc := range testCases {
		t.Run(tc.description, func(t *testing.T) {
			maxBlocks := MaxRequestBlock(tc.epoch)
			if maxBlocks != tc.expectedMaxBlock {
				t.Errorf("For epoch %d, expected max blocks %d, got %d", tc.epoch, tc.expectedMaxBlock, maxBlocks)
			}
		})
	}
}
