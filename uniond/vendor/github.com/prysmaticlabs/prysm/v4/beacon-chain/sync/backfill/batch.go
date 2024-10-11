package backfill

import (
	"fmt"
	"time"

	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	log "github.com/sirupsen/logrus"
)

// ErrChainBroken indicates a backfill batch can't be imported to the db because it is not known to be the ancestor
// of the canonical chain.
var ErrChainBroken = errors.New("batch is not the ancestor of a known finalized root")

type batchState int

func (s batchState) String() string {
	switch s {
	case batchNil:
		return "nil"
	case batchInit:
		return "init"
	case batchSequenced:
		return "sequenced"
	case batchErrRetryable:
		return "error_retryable"
	case batchImportable:
		return "importable"
	case batchImportComplete:
		return "import_complete"
	case batchEndSequence:
		return "end_sequence"
	default:
		return "unknown"
	}
}

const (
	batchNil batchState = iota
	batchInit
	batchSequenced
	batchErrRetryable
	batchImportable
	batchImportComplete
	batchEndSequence
)

type batchId string

type batch struct {
	firstScheduled time.Time
	scheduled      time.Time
	seq            int // sequence identifier, ie how many times has the sequence() method served this batch
	retries        int
	begin          primitives.Slot
	end            primitives.Slot // half-open interval, [begin, end), ie >= start, < end.
	results        VerifiedROBlocks
	err            error
	state          batchState
	pid            peer.ID
}

func (b batch) logFields() log.Fields {
	return map[string]interface{}{
		"batch_id":  b.id(),
		"state":     b.state.String(),
		"scheduled": b.scheduled.String(),
		"seq":       b.seq,
		"retries":   b.retries,
		"begin":     b.begin,
		"end":       b.end,
		"pid":       b.pid,
	}
}

func (b batch) replaces(r batch) bool {
	if r.state == batchImportComplete {
		return false
	}
	if b.begin != r.begin {
		return false
	}
	if b.end != r.end {
		return false
	}
	return b.seq >= r.seq
}

func (b batch) id() batchId {
	return batchId(fmt.Sprintf("%d:%d", b.begin, b.end))
}

func (b batch) ensureParent(expected [32]byte) error {
	tail := b.results[len(b.results)-1]
	if tail.Root() != expected {
		return errors.Wrapf(ErrChainBroken, "last parent_root=%#x, tail root=%#x", expected, tail.Root())
	}
	return nil
}

func (b batch) request() *eth.BeaconBlocksByRangeRequest {
	return &eth.BeaconBlocksByRangeRequest{
		StartSlot: b.begin,
		Count:     uint64(b.end - b.begin),
		Step:      1,
	}
}

func (b batch) withState(s batchState) batch {
	if s == batchSequenced {
		b.scheduled = time.Now()
		switch b.state {
		case batchErrRetryable:
			b.retries += 1
			log.WithFields(b.logFields()).Info("sequencing batch for retry")
		case batchInit, batchNil:
			b.firstScheduled = b.scheduled
		}
	}
	if s == batchImportComplete {
		backfillBatchTimeRoundtrip.Observe(float64(time.Since(b.firstScheduled).Milliseconds()))
		log.WithFields(b.logFields()).Debug("Backfill batch imported.")
	}
	b.state = s
	b.seq += 1
	return b
}

func (b batch) withPeer(p peer.ID) batch {
	b.pid = p
	backfillBatchTimeWaiting.Observe(float64(time.Since(b.scheduled).Milliseconds()))
	return b
}

func (b batch) withRetryableError(err error) batch {
	b.err = err
	return b.withState(batchErrRetryable)
}
