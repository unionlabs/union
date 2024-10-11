package startup

import (
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestClock(t *testing.T) {
	vr := [32]byte{}
	cases := []struct {
		name   string
		nSlots primitives.Slot
	}{
		{
			name:   "3 slots",
			nSlots: 3,
		},
		{
			name:   "0 slots",
			nSlots: 0,
		},
		{
			name:   "1 epoch",
			nSlots: params.BeaconConfig().SlotsPerEpoch,
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			genesis, now := testInterval(c.nSlots)
			nower := func() time.Time { return now }
			cl := NewClock(genesis, vr, WithNower(nower))
			require.Equal(t, genesis, cl.GenesisTime())
			require.Equal(t, now, cl.Now())
			require.Equal(t, c.nSlots, cl.CurrentSlot())
		})
	}
}

func testInterval(nSlots primitives.Slot) (time.Time, time.Time) {
	oneSlot := time.Second * time.Duration(params.BeaconConfig().SecondsPerSlot)
	var start uint64 = 23
	endOffset := oneSlot * time.Duration(nSlots)
	startTime := time.Unix(int64(start), 0)
	return startTime, startTime.Add(endOffset)
}
