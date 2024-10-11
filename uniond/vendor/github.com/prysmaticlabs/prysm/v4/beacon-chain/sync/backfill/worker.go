package backfill

import (
	"context"
	"time"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/sync"
	log "github.com/sirupsen/logrus"
)

type workerId int

type p2pWorker struct {
	id   workerId
	todo chan batch
	done chan batch
	p2p  p2p.P2P
	v    *verifier
	c    *startup.Clock
}

func (w *p2pWorker) run(ctx context.Context) {
	for {
		select {
		case b := <-w.todo:
			log.WithFields(b.logFields()).WithField("backfill_worker", w.id).Debug("Backfill worker received batch.")
			w.done <- w.handle(ctx, b)
		case <-ctx.Done():
			log.WithField("backfill_worker", w.id).Info("Backfill worker exiting after context canceled.")
			return
		}
	}
}

func (w *p2pWorker) handle(ctx context.Context, b batch) batch {
	start := time.Now()
	results, err := sync.SendBeaconBlocksByRangeRequest(ctx, w.c, w.p2p, b.pid, b.request(), nil)
	dlt := time.Now()
	backfillBatchTimeDownloading.Observe(float64(dlt.Sub(start).Milliseconds()))
	if err != nil {
		log.WithError(err).WithFields(b.logFields()).Debug("Batch requesting failed")
		return b.withRetryableError(err)
	}
	vb, err := w.v.verify(results)
	backfillBatchTimeVerifying.Observe(float64(time.Since(dlt).Milliseconds()))
	if err != nil {
		log.WithError(err).WithFields(b.logFields()).Debug("Batch validation failed")
		return b.withRetryableError(err)
	}
	// This is a hack to get the rough size of the batch. This helps us approximate the amount of memory needed
	// to hold batches and relative sizes between batches, but will be inaccurate when it comes to measuring actual
	// bytes downloaded from peers, mainly because the p2p messages are snappy compressed.
	bdl := 0
	for i := range vb {
		bdl += vb[i].SizeSSZ()
	}
	backfillBatchApproximateBytes.Add(float64(bdl))
	log.WithField("dlbytes", bdl).Debug("backfill batch bytes downloaded")
	b.results = vb
	return b.withState(batchImportable)
}

func newP2pWorker(id workerId, p p2p.P2P, todo, done chan batch, c *startup.Clock, v *verifier) *p2pWorker {
	return &p2pWorker{
		id:   id,
		todo: todo,
		done: done,
		p2p:  p,
		v:    v,
		c:    c,
	}
}
