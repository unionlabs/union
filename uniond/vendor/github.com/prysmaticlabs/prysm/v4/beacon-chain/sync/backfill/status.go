package backfill

import (
	"context"
	"sync"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/proto/dbval"
)

var errBatchDisconnected = errors.New("Highest block root in backfill batch doesn't match next parent_root")

// NewUpdater correctly initializes a StatusUpdater value with the required database value.
func NewUpdater(ctx context.Context, store BeaconDB) (*Store, error) {
	s := &Store{
		store: store,
	}
	status, err := s.store.BackfillStatus(ctx)
	if err != nil {
		if errors.Is(err, db.ErrNotFound) {
			return s, s.recoverLegacy(ctx)
		}
		return nil, errors.Wrap(err, "db error while reading status of previous backfill")
	}
	s.swapStatus(status)
	return s, nil
}

// Store provides a way to update and query the status of a backfill process that may be necessary to track when
// a node was initialized via checkpoint sync. With checkpoint sync, there will be a gap in node history from genesis
// until the checkpoint sync origin block. Store provides the means to update the value keeping track of the lower
// end of the missing block range via the FillFwd() method, to check whether a Slot is missing from the database
// via the AvailableBlock() method, and to see the current StartGap() and EndGap().
type Store struct {
	sync.RWMutex
	store       BeaconDB
	genesisSync bool
	bs          *dbval.BackfillStatus
}

// AvailableBlock determines if the given slot is covered by the current chain history.
// If the slot is <= backfill low slot, or >= backfill high slot, the result is true.
// If the slot is between the backfill low and high slots, the result is false.
func (s *Store) AvailableBlock(sl primitives.Slot) bool {
	s.RLock()
	defer s.RUnlock()
	// short circuit if the node was synced from genesis
	if s.genesisSync || sl == 0 || s.bs.LowSlot <= uint64(sl) {
		return true
	}
	return false
}

// Status is a threadsafe method to access a copy of the BackfillStatus value.
func (s *Store) status() *dbval.BackfillStatus {
	s.RLock()
	defer s.RUnlock()
	return &dbval.BackfillStatus{
		LowSlot:       s.bs.LowSlot,
		LowRoot:       s.bs.LowRoot,
		LowParentRoot: s.bs.LowParentRoot,
		OriginSlot:    s.bs.OriginSlot,
		OriginRoot:    s.bs.OriginRoot,
	}
}

// fillBack saves the slice of blocks and updates the BackfillStatus LowSlot/Root/ParentRoot tracker to the values
// from the first block in the slice. This method assumes that the block slice has been fully validated and
// sorted in slot order by the calling function.
func (s *Store) fillBack(ctx context.Context, blocks []blocks.ROBlock) (*dbval.BackfillStatus, error) {
	status := s.status()
	if len(blocks) == 0 {
		return status, nil
	}

	highest := blocks[len(blocks)-1]
	// The root of the highest block needs to match the parent root of the previous status. The backfill service will do
	// the same check, but this is an extra defensive layer in front of the db index.
	if highest.Root() != bytesutil.ToBytes32(status.LowParentRoot) {
		return nil, errors.Wrapf(errBatchDisconnected, "prev parent_root=%#x, root=%#x, prev slot=%d, slot=%d",
			status.LowParentRoot, highest.Root(), status.LowSlot, highest.Block().Slot())
	}

	if err := s.store.SaveROBlocks(ctx, blocks, false); err != nil {
		return nil, errors.Wrapf(err, "error saving backfill blocks")
	}

	// Update finalized block index.
	if err := s.store.BackfillFinalizedIndex(ctx, blocks, bytesutil.ToBytes32(status.LowRoot)); err != nil {
		return nil, errors.Wrapf(err, "failed to update finalized index for batch, connecting root %#x to previously finalized block %#x",
			highest.Root(), status.LowRoot)
	}

	// Update backfill status based on the block with the lowest slot in the batch.
	lowest := blocks[0]
	pr := lowest.Block().ParentRoot()
	status.LowSlot = uint64(lowest.Block().Slot())
	status.LowRoot = lowest.RootSlice()
	status.LowParentRoot = pr[:]
	return status, s.saveStatus(ctx, status)
}

// recoverLegacy will check to see if the db is from a legacy checkpoint sync, and either build a new BackfillStatus
// or label the node as synced from genesis.
func (s *Store) recoverLegacy(ctx context.Context) error {
	cpr, err := s.store.OriginCheckpointBlockRoot(ctx)
	if errors.Is(err, db.ErrNotFoundOriginBlockRoot) {
		s.genesisSync = true
		return nil
	}

	cpb, err := s.store.Block(ctx, cpr)
	if err != nil {
		return errors.Wrapf(err, "error retrieving block for origin checkpoint root=%#x", cpr)
	}
	if err := blocks.BeaconBlockIsNil(cpb); err != nil {
		return errors.Wrapf(err, "nil block found for origin checkpoint root=%#x", cpr)
	}
	os := uint64(cpb.Block().Slot())
	lpr := cpb.Block().ParentRoot()
	bs := &dbval.BackfillStatus{
		LowSlot:       os,
		LowRoot:       cpr[:],
		LowParentRoot: lpr[:],
		OriginSlot:    os,
		OriginRoot:    cpr[:],
	}
	return s.saveStatus(ctx, bs)
}

func (s *Store) saveStatus(ctx context.Context, bs *dbval.BackfillStatus) error {
	if err := s.store.SaveBackfillStatus(ctx, bs); err != nil {
		return err
	}

	s.swapStatus(bs)
	return nil
}

func (s *Store) swapStatus(bs *dbval.BackfillStatus) {
	s.Lock()
	defer s.Unlock()
	s.bs = bs
}

func (s *Store) isGenesisSync() bool {
	s.RLock()
	defer s.RUnlock()
	return s.genesisSync
}

// originState looks up the state for the checkpoint sync origin. This is a hack, because StatusUpdater is the only
// thing that needs db access and it has the origin root handy, so it's convenient to look it up here. The state is
// needed by the verifier.
func (s *Store) originState(ctx context.Context) (state.BeaconState, error) {
	return s.store.StateOrError(ctx, bytesutil.ToBytes32(s.status().OriginRoot))
}

// BeaconDB describes the set of DB methods that the StatusUpdater type needs to function.
type BeaconDB interface {
	SaveBackfillStatus(context.Context, *dbval.BackfillStatus) error
	BackfillStatus(context.Context) (*dbval.BackfillStatus, error)
	BackfillFinalizedIndex(ctx context.Context, blocks []blocks.ROBlock, finalizedChildRoot [32]byte) error
	OriginCheckpointBlockRoot(context.Context) ([32]byte, error)
	Block(context.Context, [32]byte) (interfaces.ReadOnlySignedBeaconBlock, error)
	SaveROBlocks(ctx context.Context, blks []blocks.ROBlock, cache bool) error
	StateOrError(ctx context.Context, blockRoot [32]byte) (state.BeaconState, error)
}
