package binary

import (
	"testing"

	"github.com/mmcloughlin/addchain/alg/algtest"
)

func TestRightToLeft(t *testing.T) {
	algtest.ChainAlgorithm(t, RightToLeft{})
}
