package backfill

import (
	"context"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	p2ptest "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/proto/dbval"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

type mockMinimumSlotter struct {
	min primitives.Slot
}

var _ minimumSlotter = &mockMinimumSlotter{}

func (m mockMinimumSlotter) minimumSlot() primitives.Slot {
	return m.min
}

func (m mockMinimumSlotter) setClock(*startup.Clock) {
}

func TestServiceInit(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), time.Second*300)
	defer cancel()
	db := &mockBackfillDB{}
	su, err := NewUpdater(ctx, db)
	require.NoError(t, err)
	nWorkers := 5
	var batchSize uint64 = 100
	nBatches := nWorkers * 2
	var high uint64 = 11235
	originRoot := [32]byte{}
	origin, err := util.NewBeaconState()
	require.NoError(t, err)
	db.states = map[[32]byte]state.BeaconState{originRoot: origin}
	su.bs = &dbval.BackfillStatus{
		LowSlot:    high,
		OriginRoot: originRoot[:],
	}
	remaining := nBatches
	cw := startup.NewClockSynchronizer()
	require.NoError(t, cw.SetClock(startup.NewClock(time.Now(), [32]byte{})))
	pool := &mockPool{todoChan: make(chan batch, nWorkers), finishedChan: make(chan batch, nWorkers)}
	p2pt := p2ptest.NewTestP2P(t)
	srv, err := NewService(ctx, su, cw, p2pt, &mockAssigner{}, WithBatchSize(batchSize), WithWorkerCount(nWorkers), WithEnableBackfill(true))
	require.NoError(t, err)
	srv.ms = mockMinimumSlotter{min: primitives.Slot(high - batchSize*uint64(nBatches))}
	srv.pool = pool
	srv.batchImporter = func(context.Context, batch, *Store) (*dbval.BackfillStatus, error) {
		return &dbval.BackfillStatus{}, nil
	}
	go srv.Start()
	todo := make([]batch, 0)
	todo = testReadN(t, ctx, pool.todoChan, nWorkers, todo)
	require.Equal(t, nWorkers, len(todo))
	for i := 0; i < remaining; i++ {
		b := todo[i]
		if b.state == batchSequenced {
			b.state = batchImportable
		}
		pool.finishedChan <- b
		todo = testReadN(t, ctx, pool.todoChan, 1, todo)
	}
	require.Equal(t, remaining+nWorkers, len(todo))
	for i := remaining; i < remaining+nWorkers; i++ {
		require.Equal(t, batchEndSequence, todo[i].state)
	}
}

func TestMinimumBackfillSlot(t *testing.T) {
	oe := helpers.MinEpochsForBlockRequests()

	currSlot := (oe + 100).Mul(uint64(params.BeaconConfig().SlotsPerEpoch))
	minSlot := minimumBackfillSlot(primitives.Slot(currSlot))
	require.Equal(t, 100*params.BeaconConfig().SlotsPerEpoch, minSlot)

	oe = helpers.MinEpochsForBlockRequests()

	currSlot = oe.Mul(uint64(params.BeaconConfig().SlotsPerEpoch))
	minSlot = minimumBackfillSlot(primitives.Slot(currSlot))
	require.Equal(t, primitives.Slot(1), minSlot)
}

func testReadN(t *testing.T, ctx context.Context, c chan batch, n int, into []batch) []batch {
	for i := 0; i < n; i++ {
		select {
		case b := <-c:
			into = append(into, b)
		case <-ctx.Done():
			// this means we hit the timeout, so something went wrong.
			require.Equal(t, true, false)
		}
	}
	return into
}
