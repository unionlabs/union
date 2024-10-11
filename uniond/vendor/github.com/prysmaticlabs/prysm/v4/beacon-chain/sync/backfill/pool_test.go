package backfill

import (
	"context"
	"testing"
	"time"

	"github.com/libp2p/go-libp2p/core/peer"
	p2ptest "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

type mockAssigner struct {
	err    error
	assign []peer.ID
}

// Assign satisfies the PeerAssigner interface so that mockAssigner can be used in tests
// in place of the concrete p2p implementation of PeerAssigner.
func (m mockAssigner) Assign(busy map[peer.ID]bool, n int) ([]peer.ID, error) {
	if m.err != nil {
		return nil, m.err
	}
	return m.assign, nil
}

var _ PeerAssigner = &mockAssigner{}

func TestPoolDetectAllEnded(t *testing.T) {
	nw := 5
	p2p := p2ptest.NewTestP2P(t)
	ctx := context.Background()
	ma := &mockAssigner{}
	pool := newP2PBatchWorkerPool(p2p, nw)
	st, err := util.NewBeaconState()
	require.NoError(t, err)
	keys, err := st.PublicKeys()
	require.NoError(t, err)
	v, err := newBackfillVerifier(st.GenesisValidatorsRoot(), keys)
	require.NoError(t, err)
	pool.spawn(ctx, nw, startup.NewClock(time.Now(), [32]byte{}), ma, v)
	br := batcher{min: 10, size: 10}
	endSeq := br.before(0)
	require.Equal(t, batchEndSequence, endSeq.state)
	for i := 0; i < nw; i++ {
		pool.todo(endSeq)
	}
	b, err := pool.complete()
	require.ErrorIs(t, err, errEndSequence)
	require.Equal(t, b.end, endSeq.end)
}

type mockPool struct {
	spawnCalled  []int
	finishedChan chan batch
	finishedErr  chan error
	todoChan     chan batch
}

func (m *mockPool) spawn(_ context.Context, _ int, _ *startup.Clock, _ PeerAssigner, _ *verifier) {
}

func (m *mockPool) todo(b batch) {
	m.todoChan <- b
}

func (m *mockPool) complete() (batch, error) {
	select {
	case b := <-m.finishedChan:
		return b, nil
	case err := <-m.finishedErr:
		return batch{}, err
	}
}

var _ batchWorkerPool = &mockPool{}
