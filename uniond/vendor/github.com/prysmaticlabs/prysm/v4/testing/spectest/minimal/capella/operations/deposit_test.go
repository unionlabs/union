package operations

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/testing/spectest/shared/capella/operations"
)

func TestMinimal_Capella_Operations_Deposit(t *testing.T) {
	operations.RunDepositTest(t, "minimal")
}
