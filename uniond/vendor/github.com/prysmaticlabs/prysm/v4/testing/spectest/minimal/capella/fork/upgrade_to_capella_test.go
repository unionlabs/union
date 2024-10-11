package fork

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/spectest/shared/capella/fork"
)

func TestMinimal_Capella_UpgradeToCapella(t *testing.T) {
	fork.RunUpgradeToCapella(t, "minimal")
}
