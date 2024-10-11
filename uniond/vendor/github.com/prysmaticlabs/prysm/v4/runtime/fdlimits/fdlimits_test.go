package fdlimits_test

import (
	"testing"

	gethLimit "github.com/ethereum/go-ethereum/common/fdlimit"
	"github.com/prysmaticlabs/prysm/v4/runtime/fdlimits"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestSetMaxFdLimits(t *testing.T) {
	assert.NoError(t, fdlimits.SetMaxFdLimits())

	curr, err := gethLimit.Current()
	assert.NoError(t, err)

	max, err := gethLimit.Maximum()
	assert.NoError(t, err)

	assert.Equal(t, max, curr, "current and maximum file descriptor limits do not match up.")

}
