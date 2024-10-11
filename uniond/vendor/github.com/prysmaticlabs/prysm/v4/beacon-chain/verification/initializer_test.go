package verification

import (
	"bytes"
	"context"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestInitializerWaiter(t *testing.T) {
	ctx := context.Background()
	vr := bytesutil.ToBytes32([]byte{0, 1, 1, 2, 3, 5})
	gen := time.Now()
	c := startup.NewClock(gen, vr)
	cs := startup.NewClockSynchronizer()
	require.NoError(t, cs.SetClock(c))

	w := NewInitializerWaiter(cs, &mockForkchoicer{}, &mockStateByRooter{})
	ini, err := w.WaitForInitializer(ctx)
	require.NoError(t, err)
	csc, ok := ini.shared.sc.(*sigCache)
	require.Equal(t, true, ok)
	require.Equal(t, true, bytes.Equal(vr[:], csc.valRoot))
}
