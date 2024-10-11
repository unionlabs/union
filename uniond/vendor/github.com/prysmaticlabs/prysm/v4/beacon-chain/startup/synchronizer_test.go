package startup

import (
	"context"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestSynchronizerErrOnSecondSet(t *testing.T) {
	s := NewClockSynchronizer()
	require.NoError(t, s.SetClock(NewClock(time.Now(), [32]byte{})))
	require.ErrorIs(t, s.SetClock(NewClock(time.Now(), [32]byte{})), errClockSet)
}

func TestWaitForClockCanceled(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	cancel()
	s := NewClockSynchronizer()
	c, err := s.WaitForClock(ctx)
	require.Equal(t, true, c == nil)
	require.ErrorIs(t, err, context.Canceled)
}

func TestWaitForClock(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	s := NewClockSynchronizer()
	var vr [32]byte
	copy(vr[:], bytesutil.PadTo([]byte("valroot"), 32))
	genesis := time.Unix(23, 0)
	later := time.Unix(42, 0)
	nower := func() time.Time { return later }
	expect := NewClock(genesis, vr, WithNower(nower))
	go func() {
		// This is just to ensure the test doesn't hang.
		// If we hit this cancellation case, then the happy path failed and the NoError assertion etc below will fail.
		time.Sleep(time.Second)
		cancel()
	}()
	go func() {
		require.NoError(t, s.SetClock(expect))
	}()
	c, err := s.WaitForClock(ctx)
	require.NoError(t, err)
	require.Equal(t, later, c.Now())
	require.Equal(t, genesis, c.GenesisTime())
	require.Equal(t, vr, c.GenesisValidatorsRoot())
}
