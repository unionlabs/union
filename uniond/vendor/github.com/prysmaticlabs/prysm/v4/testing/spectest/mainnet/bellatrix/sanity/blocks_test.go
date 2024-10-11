package sanity

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/spectest/shared/bellatrix/sanity"
)

func TestMainnet_Bellatrix_Sanity_Blocks(t *testing.T) {
	sanity.RunBlockProcessingTest(t, "mainnet", "sanity/blocks/pyspec_tests")
}
