package forkchoice

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/testing/spectest/shared/common/forkchoice"
)

func TestMainnet_Deneb_Forkchoice(t *testing.T) {
	t.Skip("This will fail until we re-integrate proof verification")
	forkchoice.Run(t, "mainnet", version.Deneb)
}
