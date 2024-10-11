package sync

import (
	"context"
	"fmt"
	"sort"
	"time"

	libp2pcore "github.com/libp2p/go-libp2p/core"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filters"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

// blockRangeBatcher encapsulates the logic for splitting up a block range request into fixed-size batches of
// blocks that are retrieved from the database, ensured to be canonical, sequential and unique.
// If a non-nil value for ticker is set, it will be used to pause between batches lookups, as a rate-limiter.
type blockRangeBatcher struct {
	start   primitives.Slot
	end     primitives.Slot
	size    uint64
	db      db.NoHeadAccessDatabase
	limiter *limiter
	ticker  *time.Ticker

	cf      *canonicalFilter
	current *blockBatch
}

func newBlockRangeBatcher(rp rangeParams, bdb db.NoHeadAccessDatabase, limiter *limiter, canonical canonicalChecker, ticker *time.Ticker) (*blockRangeBatcher, error) {
	if bdb == nil {
		return nil, errors.New("nil db param, unable to initialize blockRangeBatcher")
	}
	if limiter == nil {
		return nil, errors.New("nil limiter param, unable to initialize blockRangeBatcher")
	}
	if canonical == nil {
		return nil, errors.New("nil canonicalChecker param, unable to initialize blockRangeBatcher")
	}
	if ticker == nil {
		return nil, errors.New("nil ticker param, unable to initialize blockRangeBatcher")
	}
	if rp.size == 0 {
		return nil, fmt.Errorf("invalid batch size of %d", rp.size)
	}
	if rp.end < rp.start {
		return nil, fmt.Errorf("batch end slot %d is lower than batch start %d", rp.end, rp.start)
	}
	cf := &canonicalFilter{canonical: canonical}
	return &blockRangeBatcher{
		start:   rp.start,
		end:     rp.end,
		size:    rp.size,
		db:      bdb,
		limiter: limiter,
		ticker:  ticker,
		cf:      cf,
	}, nil
}

func (bb *blockRangeBatcher) next(ctx context.Context, stream libp2pcore.Stream) (blockBatch, bool) {
	var nb blockBatch
	var more bool
	// The result of each call to next() is saved in the `current` field.
	// If current is not nil, current.next figures out the next batch based on the previous one.
	// If current is nil, newBlockBatch is used to generate the first batch.
	if bb.current != nil {
		current := *bb.current
		nb, more = current.next(bb.end, bb.size)
	} else {
		nb, more = newBlockBatch(bb.start, bb.end, bb.size)
	}
	// newBlockBatch and next() both return a boolean to indicate whether calling .next() will yield another batch
	// (based on the whether we've gotten to the end slot yet). blockRangeBatcher.next does the same,
	// and returns (zero value, false), to signal the end of the iteration.
	if !more {
		return blockBatch{}, false
	}
	if err := bb.limiter.validateRequest(stream, bb.size); err != nil {
		return blockBatch{err: errors.Wrap(err, "throttled by rate limiter")}, false
	}

	// Wait for the ticker before doing anything expensive, unless this is the first batch.
	if bb.ticker != nil && bb.current != nil {
		<-bb.ticker.C
	}
	filter := filters.NewFilter().SetStartSlot(nb.start).SetEndSlot(nb.end)
	blks, roots, err := bb.db.Blocks(ctx, filter)
	if err != nil {
		return blockBatch{err: errors.Wrap(err, "Could not retrieve blocks")}, false
	}

	rob := make([]blocks.ROBlock, 0)
	if nb.start == 0 {
		gb, err := bb.genesisBlock(ctx)
		if err != nil {
			return blockBatch{err: errors.Wrap(err, "could not retrieve genesis block")}, false
		}
		rob = append(rob, gb)
	}
	for i := 0; i < len(blks); i++ {
		rb, err := blocks.NewROBlockWithRoot(blks[i], roots[i])
		if err != nil {
			return blockBatch{err: errors.Wrap(err, "Could not initialize ROBlock")}, false
		}
		rob = append(rob, rb)
	}

	// Filter and sort our retrieved blocks, so that we only return valid sets of blocks.
	nb.lin, nb.nonlin, nb.err = bb.cf.filter(ctx, rob)

	// Decrease allowed blocks capacity by the number of streamed blocks.
	bb.limiter.add(stream, int64(1+nb.end.SubSlot(nb.start)))
	bb.current = &nb
	return *bb.current, true
}

func (bb *blockRangeBatcher) genesisBlock(ctx context.Context) (blocks.ROBlock, error) {
	b, err := bb.db.GenesisBlock(ctx)
	if err != nil {
		return blocks.ROBlock{}, err
	}
	htr, err := b.Block().HashTreeRoot()
	if err != nil {
		return blocks.ROBlock{}, err
	}
	return blocks.NewROBlockWithRoot(b, htr)
}

type blockBatch struct {
	start  primitives.Slot
	end    primitives.Slot
	lin    []blocks.ROBlock // lin is a linear chain of blocks connected through parent_root. broken tails go in nonlin.
	nonlin []blocks.ROBlock // if there is a break in the chain of parent->child relationships, the tail is stored here.
	err    error
}

func newBlockBatch(start, reqEnd primitives.Slot, size uint64) (blockBatch, bool) {
	if start > reqEnd {
		return blockBatch{}, false
	}
	if size == 0 {
		return blockBatch{}, false
	}
	nb := blockBatch{start: start, end: start.Add(size - 1)}
	if nb.end > reqEnd {
		nb.end = reqEnd
	}
	return nb, true
}

func (bat blockBatch) next(reqEnd primitives.Slot, size uint64) (blockBatch, bool) {
	if bat.error() != nil {
		return bat, false
	}
	if bat.nonLinear() {
		return blockBatch{}, false
	}
	return newBlockBatch(bat.end.Add(1), reqEnd, size)
}

// blocks returns the list of linear, canonical blocks read from the db.
func (bb blockBatch) canonical() []blocks.ROBlock {
	return bb.lin
}

// nonLinear is used to determine if there was a break in the chain of canonical blocks as read from the db.
// If true, code using the blockBatch should stop serving additional batches of blocks.
func (bb blockBatch) nonLinear() bool {
	return len(bb.nonlin) > 0
}

func (bb blockBatch) error() error {
	return bb.err
}

type canonicalChecker func(context.Context, [32]byte) (bool, error)

type canonicalFilter struct {
	prevRoot  [32]byte
	canonical canonicalChecker
}

// filters all the provided blocks to ensure they are canonical and strictly linear.
func (cf *canonicalFilter) filter(ctx context.Context, blks []blocks.ROBlock) ([]blocks.ROBlock, []blocks.ROBlock, error) {
	blks = sortedUniqueBlocks(blks)
	seq := make([]blocks.ROBlock, 0, len(blks))
	nseq := make([]blocks.ROBlock, 0)
	for i, b := range blks {
		cb, err := cf.canonical(ctx, b.Root())
		if err != nil {
			return nil, nil, err
		}
		if !cb {
			continue
		}
		// prevRoot will be the zero value until we find the first canonical block in the stream seen by an instance
		// of canonicalFilter. filter is called in batches; prevRoot can be the last root from the previous batch.
		first := cf.prevRoot == [32]byte{}
		// We assume blocks are processed in order, so the previous canonical root should be the parent of the next.
		if !first && cf.prevRoot != b.Block().ParentRoot() {
			// If the current block isn't descended from the last, something is wrong. Append everything remaining
			// to the list of non-linear blocks, and stop building the canonical list.
			nseq = append(nseq, blks[i:]...)
			break
		}
		seq = append(seq, blks[i])
		// Set the previous root as the
		// newly added block's root
		cf.prevRoot = b.Root()
	}
	return seq, nseq, nil
}

// returns a copy of the []ROBlock list in sorted order with duplicates removed
func sortedUniqueBlocks(blks []blocks.ROBlock) []blocks.ROBlock {
	// Remove duplicate blocks received
	sort.Sort(blocks.ROBlockSlice(blks))
	if len(blks) < 2 {
		return blks
	}
	u := 0
	for i := 1; i < len(blks); i++ {
		if blks[i].Root() != blks[u].Root() {
			u += 1
			if u != i {
				blks[u] = blks[i]
			}
		}
	}
	return blks[:u+1]
}
