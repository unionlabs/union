package pruning

import (
	"container/list"
	"encoding/binary"
	"fmt"
	"sync"

	dbm "github.com/cometbft/cometbft-db"
	"github.com/cometbft/cometbft/libs/log"

	"github.com/cosmos/cosmos-sdk/store/pruning/types"
)

// Manager is an abstraction to handle the logic needed for
// determining when to prune old heights of the store
// based on the strategy described by the pruning options.
type Manager struct {
	db               dbm.DB
	logger           log.Logger
	opts             types.PruningOptions
	snapshotInterval uint64
	// Although pruneHeights happen in the same goroutine with the normal execution,
	// we sync access to them to avoid soundness issues in the future if concurrency pattern changes.
	pruneHeightsMx sync.Mutex
	pruneHeights   []int64
	// Snapshots are taken in a separate goroutine from the regular execution
	// and can be delivered asynchrounously via HandleHeightSnapshot.
	// Therefore, we sync access to pruneSnapshotHeights with this mutex.
	pruneSnapshotHeightsMx sync.Mutex
	// These are the heights that are multiples of snapshotInterval and kept for state sync snapshots.
	// The heights are added to this list to be pruned when a snapshot is complete.
	pruneSnapshotHeights *list.List
}

// NegativeHeightsError is returned when a negative height is provided to the manager.
type NegativeHeightsError struct {
	Height int64
}

var _ error = &NegativeHeightsError{}

func (e *NegativeHeightsError) Error() string {
	return fmt.Sprintf("failed to get pruned heights: %d", e.Height)
}

var (
	pruneHeightsKey         = []byte("s/pruneheights")
	pruneSnapshotHeightsKey = []byte("s/prunesnapshotheights")
)

// NewManager returns a new Manager with the given db and logger.
// The retuned manager uses a pruning strategy of "nothing" which
// keeps all heights. Users of the Manager may change the strategy
// by calling SetOptions.
func NewManager(db dbm.DB, logger log.Logger) *Manager {
	return &Manager{
		db:                   db,
		logger:               logger,
		opts:                 types.NewPruningOptions(types.PruningNothing),
		pruneHeights:         []int64{},
		pruneSnapshotHeights: list.New(),
	}
}

// SetOptions sets the pruning strategy on the manager.
func (m *Manager) SetOptions(opts types.PruningOptions) {
	m.opts = opts
}

// GetOptions fetches the pruning strategy from the manager.
func (m *Manager) GetOptions() types.PruningOptions {
	return m.opts
}

// GetFlushAndResetPruningHeights returns all heights to be pruned during the next call to Prune().
// It also flushes and resets the pruning heights.
func (m *Manager) GetFlushAndResetPruningHeights() ([]int64, error) {
	if m.opts.GetPruningStrategy() == types.PruningNothing {
		return []int64{}, nil
	}
	m.pruneHeightsMx.Lock()
	defer m.pruneHeightsMx.Unlock()

	// flush the updates to disk so that it is not lost if crash happens.
	if err := m.db.SetSync(pruneHeightsKey, int64SliceToBytes(m.pruneHeights)); err != nil {
		return nil, err
	}

	// Return a copy to prevent data races.
	pruningHeights := make([]int64, len(m.pruneHeights))
	copy(pruningHeights, m.pruneHeights)
	m.pruneHeights = m.pruneHeights[:0]

	return pruningHeights, nil
}

// HandleHeight determines if previousHeight height needs to be kept for pruning at the right interval prescribed by
// the pruning strategy. Returns previousHeight, if it was kept to be pruned at the next call to Prune(), 0 otherwise.
// previousHeight must be greater than 0 for the handling to take effect since valid heights start at 1 and 0 represents
// the latest height. The latest height cannot be pruned. As a result, if previousHeight is less than or equal to 0, 0 is returned.
func (m *Manager) HandleHeight(previousHeight int64) int64 {
	if m.opts.GetPruningStrategy() == types.PruningNothing || previousHeight <= 0 {
		return 0
	}

	defer func() {
		m.pruneHeightsMx.Lock()
		defer m.pruneHeightsMx.Unlock()

		m.pruneSnapshotHeightsMx.Lock()
		defer m.pruneSnapshotHeightsMx.Unlock()

		// move persisted snapshot heights to pruneHeights which
		// represent the heights to be pruned at the next pruning interval.
		var next *list.Element
		for e := m.pruneSnapshotHeights.Front(); e != nil; e = next {
			snHeight := e.Value.(int64)
			if snHeight < previousHeight-int64(m.opts.KeepRecent) {
				m.pruneHeights = append(m.pruneHeights, snHeight)

				// We must get next before removing to be able to continue iterating.
				next = e.Next()
				m.pruneSnapshotHeights.Remove(e)
			} else {
				next = e.Next()
			}
		}

		// flush the updates to disk so that they are not lost if crash happens.
		if err := m.db.SetSync(pruneHeightsKey, int64SliceToBytes(m.pruneHeights)); err != nil {
			panic(err)
		}
	}()

	if int64(m.opts.KeepRecent) < previousHeight {
		pruneHeight := previousHeight - int64(m.opts.KeepRecent)
		// We consider this height to be pruned iff:
		//
		// - snapshotInterval is zero as that means that all heights should be pruned.
		// - snapshotInterval % (height - KeepRecent) != 0 as that means the height is not
		// a 'snapshot' height.
		if m.snapshotInterval == 0 || pruneHeight%int64(m.snapshotInterval) != 0 {
			m.pruneHeightsMx.Lock()
			defer m.pruneHeightsMx.Unlock()

			m.pruneHeights = append(m.pruneHeights, pruneHeight)
			return pruneHeight
		}
	}
	return 0
}

// HandleHeightSnapshot persists the snapshot height to be pruned at the next appropriate
// height defined by the pruning strategy. Flushes the update to disk and panics if the flush fails
// The input height must be greater than 0 and pruning strategy any but pruning nothing.
// If one of these conditions is not met, this function does nothing.
func (m *Manager) HandleHeightSnapshot(height int64) {
	if m.opts.GetPruningStrategy() == types.PruningNothing || height <= 0 {
		return
	}

	m.pruneSnapshotHeightsMx.Lock()
	defer m.pruneSnapshotHeightsMx.Unlock()

	m.logger.Debug("HandleHeightSnapshot", "height", height)
	m.pruneSnapshotHeights.PushBack(height)

	// flush the updates to disk so that they are not lost if crash happens.
	if err := m.db.SetSync(pruneSnapshotHeightsKey, listToBytes(m.pruneSnapshotHeights)); err != nil {
		panic(err)
	}
}

// SetSnapshotInterval sets the interval at which the snapshots are taken.
func (m *Manager) SetSnapshotInterval(snapshotInterval uint64) {
	m.snapshotInterval = snapshotInterval
}

// ShouldPruneAtHeight return true if the given height should be pruned, false otherwise
func (m *Manager) ShouldPruneAtHeight(height int64) bool {
	return m.opts.Interval > 0 && m.opts.GetPruningStrategy() != types.PruningNothing && height%int64(m.opts.Interval) == 0
}

// LoadPruningHeights loads the pruning heights from the database as a crash recovery.
func (m *Manager) LoadPruningHeights(db dbm.DB) error {
	if m.opts.GetPruningStrategy() == types.PruningNothing {
		return nil
	}
	loadedPruneHeights, err := loadPruningHeights(db)
	if err != nil {
		return err
	}

	if len(loadedPruneHeights) > 0 {
		m.pruneHeightsMx.Lock()
		defer m.pruneHeightsMx.Unlock()
		m.pruneHeights = loadedPruneHeights
	}

	loadedPruneSnapshotHeights, err := loadPruningSnapshotHeights(db)
	if err != nil {
		return err
	}

	if loadedPruneSnapshotHeights.Len() > 0 {
		m.pruneSnapshotHeightsMx.Lock()
		defer m.pruneSnapshotHeightsMx.Unlock()
		m.pruneSnapshotHeights = loadedPruneSnapshotHeights
	}

	return nil
}

func loadPruningHeights(db dbm.DB) ([]int64, error) {
	bz, err := db.Get(pruneHeightsKey)
	if err != nil {
		return nil, fmt.Errorf("failed to get pruned heights: %w", err)
	}
	if len(bz) == 0 {
		return []int64{}, nil
	}

	prunedHeights := make([]int64, len(bz)/8)
	i, offset := 0, 0
	for offset < len(bz) {
		h := int64(binary.BigEndian.Uint64(bz[offset : offset+8]))
		if h < 0 {
			return []int64{}, &NegativeHeightsError{Height: h}
		}

		prunedHeights[i] = h
		i++
		offset += 8
	}

	return prunedHeights, nil
}

func loadPruningSnapshotHeights(db dbm.DB) (*list.List, error) {
	bz, err := db.Get(pruneSnapshotHeightsKey)
	if err != nil {
		return nil, fmt.Errorf("failed to get post-snapshot pruned heights: %w", err)
	}
	pruneSnapshotHeights := list.New()
	if len(bz) == 0 {
		return pruneSnapshotHeights, nil
	}

	i, offset := 0, 0
	for offset < len(bz) {
		h := int64(binary.BigEndian.Uint64(bz[offset : offset+8]))
		if h < 0 {
			return nil, &NegativeHeightsError{Height: h}
		}
		pruneSnapshotHeights.PushBack(h)
		i++
		offset += 8
	}

	return pruneSnapshotHeights, nil
}

func int64SliceToBytes(slice []int64) []byte {
	bz := make([]byte, 0, len(slice)*8)
	for _, ph := range slice {
		buf := make([]byte, 8)
		binary.BigEndian.PutUint64(buf, uint64(ph))
		bz = append(bz, buf...)
	}
	return bz
}

func listToBytes(list *list.List) []byte {
	bz := make([]byte, 0, list.Len()*8)
	for e := list.Front(); e != nil; e = e.Next() {
		buf := make([]byte, 8)
		binary.BigEndian.PutUint64(buf, uint64(e.Value.(int64)))
		bz = append(bz, buf...)
	}
	return bz
}
