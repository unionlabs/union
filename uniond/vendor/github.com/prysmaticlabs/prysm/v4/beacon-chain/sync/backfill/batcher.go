package backfill

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

var errMaxBatches = errors.New("backfill batch requested in excess of max outstanding batches")
var errEndSequence = errors.New("sequence has terminated, no more backfill batches will be produced")
var errCannotDecreaseMinimum = errors.New("the minimum backfill slot can only be increased, not decreased")

type batchSequencer struct {
	batcher batcher
	seq     []batch
}

// sequence() is meant as a verb "arrange in a particular order".
// sequence determines the next set of batches that should be worked on based on the state of the batches
// in its internal view. sequence relies on update() for updates to its view of the
// batches it has previously sequenced.
func (c *batchSequencer) sequence() ([]batch, error) {
	s := make([]batch, 0)
	// batch start slots are in descending order, c.seq[n].begin == c.seq[n+1].end
	for i := range c.seq {
		switch c.seq[i].state {
		case batchInit, batchErrRetryable:
			c.seq[i] = c.seq[i].withState(batchSequenced)
			s = append(s, c.seq[i])
		case batchNil:
			// batchNil is the zero value of the batch type.
			// This case means that we are initializing a batch that was created by the
			// initial allocation of the seq slice, so batcher need to compute its bounds.
			var b batch
			if i == 0 {
				// The first item in the list is a special case, subsequent items are initialized
				// relative to the preceding batches.
				b = c.batcher.before(c.batcher.max)
			} else {
				b = c.batcher.beforeBatch(c.seq[i-1])
			}
			c.seq[i] = b.withState(batchSequenced)
			s = append(s, c.seq[i])
		case batchEndSequence:
			if len(s) == 0 {
				s = append(s, c.seq[i])
			}
		default:
			continue
		}
	}
	if len(s) == 0 {
		return nil, errMaxBatches
	}

	return s, nil
}

// update serves 2 roles.
//   - updating batchSequencer's copy of the given batch.
//   - removing batches that are completely imported from the sequence,
//     so that they are not returned the next time import() is called, and populating
//     seq with new batches that are ready to be worked on.
func (c *batchSequencer) update(b batch) {
	done := 0
	for i := 0; i < len(c.seq); i++ {
		if b.replaces(c.seq[i]) {
			c.seq[i] = b
		}
		// Assumes invariant that batches complete and update is called in order.
		// This should be true because the code using the sequencer doesn't know the expected parent
		// for a batch until it imports the previous batch.
		if c.seq[i].state == batchImportComplete {
			done += 1
			continue
		}
		// Move the unfinished batches to overwrite the finished ones.
		// eg consider [a,b,c,d,e] where a,b are done
		// when i==2, done==2 (since done was incremented for a and b)
		// so we want to copy c to a, then on i=3, d to b, then on i=4 e to c.
		c.seq[i-done] = c.seq[i]
	}
	// Overwrite the moved batches with the next ones in the sequence.
	// Continuing the example in the comment above, len(c.seq)==5, done=2, so i=3.
	// We want to replace index 3 with the batch that should be processed after index 2,
	// which was previously the earliest known batch, and index 4 with the batch that should
	// be processed after index 3, the new earliest batch.
	for i := len(c.seq) - done; i < len(c.seq); i++ {
		c.seq[i] = c.batcher.beforeBatch(c.seq[i-1])
	}
}

// importable returns all batches that are ready to be imported. This means they satisfy 2 conditions:
//   - They are in state batchImportable, which means their data has been downloaded and proposer signatures have been verified.
//   - There are no batches that are not in state batchImportable between them and the start of the slice. This ensures that they
//     can be connected to the canonical chain, either because the root of the last block in the batch matches the parent_root of
//     the oldest block in the canonical chain, or because the root of the last block in the batch matches the parent_root of the
//     new block preceding them in the slice (which must connect to the batch before it, or to the canonical chain if it is first).
func (c *batchSequencer) importable() []batch {
	imp := make([]batch, 0)
	for i := range c.seq {
		if c.seq[i].state == batchImportable {
			imp = append(imp, c.seq[i])
			continue
		}
		// as soon as we hit a batch with a different state, we return everything leading to it.
		// If the first element isn't importable, we'll return an empty slice.
		break
	}
	return imp
}

// moveMinimum enables the backfill service to change the slot where the batcher will start replying with
// batch state batchEndSequence (signaling that no new batches will be produced). This is done in response to
// epochs advancing, which shrinks the gap between <checkpoint slot> and <current slot>-MIN_EPOCHS_FOR_BLOCK_REQUESTS,
// allowing the node to download a smaller number of blocks.
func (c *batchSequencer) moveMinimum(min primitives.Slot) error {
	if min < c.batcher.min {
		return errCannotDecreaseMinimum
	}
	c.batcher.min = min
	return nil
}

// countWithState provides a view into how many batches are in a particular state
// to be used for logging or metrics purposes.
func (c *batchSequencer) countWithState(s batchState) int {
	n := 0
	for i := 0; i < len(c.seq); i++ {
		if c.seq[i].state == s {
			n += 1
		}
	}
	return n
}

// numTodo computes the number of remaining batches for metrics and logging purposes.
func (c *batchSequencer) numTodo() int {
	if len(c.seq) == 0 {
		return 0
	}
	lowest := c.seq[len(c.seq)-1]
	todo := 0
	if lowest.state != batchEndSequence {
		todo = c.batcher.remaining(lowest.begin)
	}
	for _, b := range c.seq {
		switch b.state {
		case batchEndSequence, batchImportComplete, batchNil:
			continue
		default:
			todo += 1
		}
	}
	return todo
}

func newBatchSequencer(seqLen int, min, max, size primitives.Slot) *batchSequencer {
	b := batcher{min: min, max: max, size: size}
	seq := make([]batch, seqLen)
	return &batchSequencer{batcher: b, seq: seq}
}

type batcher struct {
	min  primitives.Slot
	max  primitives.Slot
	size primitives.Slot
}

func (r batcher) remaining(upTo primitives.Slot) int {
	if r.min >= upTo {
		return 0
	}
	delta := upTo - r.min
	if delta%r.size != 0 {
		return int(delta/r.size) + 1
	}
	return int(delta / r.size)
}

func (r batcher) beforeBatch(upTo batch) batch {
	return r.before(upTo.begin)
}

func (r batcher) before(upTo primitives.Slot) batch {
	// upTo is an exclusive upper bound. Requesting a batch before the lower bound of backfill signals the end of the
	// backfill process.
	if upTo <= r.min {
		return batch{begin: upTo, end: upTo, state: batchEndSequence}
	}
	begin := r.min
	if upTo > r.size+r.min {
		begin = upTo - r.size
	}

	// batch.end is exclusive, .begin is inclusive, so the prev.end = next.begin
	return batch{begin: begin, end: upTo, state: batchInit}
}
