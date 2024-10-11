package consensus

import (
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/internal/consensus/types"
)

func TestTimeoutTicker(t *testing.T) {
	ticker := NewTimeoutTicker()
	err := ticker.Start()
	require.NoError(t, err)
	defer func() {
		err := ticker.Stop()
		require.NoError(t, err)
	}()

	c := ticker.Chan()
	for i := 1; i <= 10; i++ {
		height := int64(i)

		startTime := time.Now()
		// Schedule a timeout for 5ms from now
		negTimeout := timeoutInfo{Duration: -1 * time.Millisecond, Height: height, Round: 0, Step: types.RoundStepNewHeight}
		timeout := timeoutInfo{Duration: 5 * time.Millisecond, Height: height, Round: 0, Step: types.RoundStepNewRound}
		ticker.ScheduleTimeout(negTimeout)
		ticker.ScheduleTimeout(timeout)

		// Wait for the timeout to be received
		to := <-c
		endTime := time.Now()
		elapsedTime := endTime.Sub(startTime)
		if timeout == to {
			require.True(t, elapsedTime >= timeout.Duration, "We got the 5ms timeout. However the timeout happened too quickly. Should be >= 5ms. Got %dms (start time %d end time %d)", elapsedTime.Milliseconds(), startTime.UnixMilli(), endTime.UnixMilli())
		}
	}
}
