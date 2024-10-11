package forkchoice

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/shared/common/forkchoice"
)

func TestMinimal_Deneb_Forkchoice(t *testing.T) {
	t.Skip("blocked by go-kzg-4844 minimal trusted setup")
	forkchoice.Run(t, "minimal", version.Deneb)
}
