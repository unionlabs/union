package backfill

import (
	"context"
	"math"
	"time"

	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	log "github.com/sirupsen/logrus"
)

type batchWorkerPool interface {
	spawn(ctx context.Context, n int, clock *startup.Clock, a PeerAssigner, v *verifier)
	todo(b batch)
	complete() (batch, error)
}

type worker interface {
	run(context.Context)
}

type newWorker func(id workerId, in, out chan batch, c *startup.Clock, v *verifier) worker

func defaultNewWorker(p p2p.P2P) newWorker {
	return func(id workerId, in, out chan batch, c *startup.Clock, v *verifier) worker {
		return newP2pWorker(id, p, in, out, c, v)
	}
}

type p2pBatchWorkerPool struct {
	maxBatches  int
	newWorker   newWorker
	toWorkers   chan batch
	fromWorkers chan batch
	toRouter    chan batch
	fromRouter  chan batch
	shutdownErr chan error
	endSeq      []batch
	ctx         context.Context
	cancel      func()
}

var _ batchWorkerPool = &p2pBatchWorkerPool{}

func newP2PBatchWorkerPool(p p2p.P2P, maxBatches int) *p2pBatchWorkerPool {
	nw := defaultNewWorker(p)
	return &p2pBatchWorkerPool{
		newWorker:   nw,
		toRouter:    make(chan batch, maxBatches),
		fromRouter:  make(chan batch, maxBatches),
		toWorkers:   make(chan batch),
		fromWorkers: make(chan batch),
		maxBatches:  maxBatches,
		shutdownErr: make(chan error),
	}
}

func (p *p2pBatchWorkerPool) spawn(ctx context.Context, n int, c *startup.Clock, a PeerAssigner, v *verifier) {
	p.ctx, p.cancel = context.WithCancel(ctx)
	go p.batchRouter(a)
	for i := 0; i < n; i++ {
		go p.newWorker(workerId(i), p.toWorkers, p.fromWorkers, c, v).run(p.ctx)
	}
}

func (p *p2pBatchWorkerPool) todo(b batch) {
	// Intercept batchEndSequence batches so workers can remain unaware of this state.
	// Workers don't know what to do with batchEndSequence batches. They are a signal to the pool that the batcher
	// has stopped producing things for the workers to do and the pool is close to winding down. See complete()
	// to understand how the pool manages the state where all workers are idle
	// and all incoming batches signal end of sequence.
	if b.state == batchEndSequence {
		p.endSeq = append(p.endSeq, b)
		return
	}
	p.toRouter <- b
}

func (p *p2pBatchWorkerPool) complete() (batch, error) {
	if len(p.endSeq) == p.maxBatches {
		return p.endSeq[0], errEndSequence
	}

	select {
	case b := <-p.fromRouter:
		return b, nil
	case err := <-p.shutdownErr:
		return batch{}, errors.Wrap(err, "fatal error from backfill worker pool")
	case <-p.ctx.Done():
		log.WithError(p.ctx.Err()).Info("p2pBatchWorkerPool context canceled, shutting down")
		return batch{}, p.ctx.Err()
	}
}

func (p *p2pBatchWorkerPool) batchRouter(pa PeerAssigner) {
	busy := make(map[peer.ID]bool)
	todo := make([]batch, 0)
	rt := time.NewTicker(time.Second)
	earliest := primitives.Slot(math.MaxUint64)
	for {
		select {
		case b := <-p.toRouter:
			todo = append(todo, b)
		case <-rt.C:
			// Worker assignments can fail if assignBatch can't find a suitable peer.
			// This ticker exists to periodically break out of the channel select
			// to retry failed assignments.
		case b := <-p.fromWorkers:
			pid := b.pid
			busy[pid] = false
			p.fromRouter <- b
		case <-p.ctx.Done():
			log.WithError(p.ctx.Err()).Info("p2pBatchWorkerPool context canceled, shutting down")
			p.shutdown(p.ctx.Err())
			return
		}
		if len(todo) == 0 {
			continue
		}
		// Try to assign as many outstanding batches as possible to peers and feed the assigned batches to workers.
		assigned, err := pa.Assign(busy, len(todo))
		if err != nil {
			if errors.Is(err, peers.ErrInsufficientSuitable) {
				// Transient error resulting from insufficient number of connected peers. Leave batches in
				// queue and get to them whenever the peer situation is resolved.
				continue
			}
			p.shutdown(err)
			return
		}
		for _, pid := range assigned {
			busy[pid] = true
			todo[0].pid = pid
			p.toWorkers <- todo[0].withPeer(pid)
			if todo[0].begin < earliest {
				earliest = todo[0].begin
				oldestBatch.Set(float64(earliest))
			}
			todo = todo[1:]
		}
	}
}

func (p *p2pBatchWorkerPool) shutdown(err error) {
	p.cancel()
	p.shutdownErr <- err
}
