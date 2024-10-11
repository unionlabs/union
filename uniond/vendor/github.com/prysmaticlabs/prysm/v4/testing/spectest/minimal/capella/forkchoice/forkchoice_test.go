package forkchoice

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/shared/common/forkchoice"
)

func TestMinimal_Capella_Forkchoice(t *testing.T) {
	forkchoice.Run(t, "minimal", version.Capella)
}
