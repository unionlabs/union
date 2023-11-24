package app

import (
	"encoding/json"
	"errors"
	"fmt"
	"math"
	"os"
	"path/filepath"
	"sync"

	abci "github.com/cometbft/cometbft/abci/types"
)

const (
	snapshotChunkSize = 1e6

	// Keep only the most recent 10 snapshots. Older snapshots are pruned
	maxSnapshotCount = 10
)

// SnapshotStore stores state sync snapshots. Snapshots are stored simply as
// JSON files, and chunks are generated on-the-fly by splitting the JSON data
// into fixed-size chunks.
type SnapshotStore struct {
	sync.RWMutex
	dir      string
	metadata []abci.Snapshot
}

// NewSnapshotStore creates a new snapshot store.
func NewSnapshotStore(dir string) (*SnapshotStore, error) {
	store := &SnapshotStore{dir: dir}
	if err := os.MkdirAll(dir, 0o755); err != nil {
		return nil, err
	}
	if err := store.loadMetadata(); err != nil {
		return nil, err
	}
	return store, nil
}

// loadMetadata loads snapshot metadata. Does not take out locks, since it's
// called internally on construction.
func (s *SnapshotStore) loadMetadata() error {
	file := filepath.Join(s.dir, "metadata.json")
	metadata := []abci.Snapshot{}

	bz, err := os.ReadFile(file)
	switch {
	case errors.Is(err, os.ErrNotExist):
	case err != nil:
		return fmt.Errorf("failed to load snapshot metadata from %q: %w", file, err)
	}
	if len(bz) != 0 {
		err = json.Unmarshal(bz, &metadata)
		if err != nil {
			return fmt.Errorf("invalid snapshot data in %q: %w", file, err)
		}
	}
	s.metadata = metadata
	return nil
}

// saveMetadata saves snapshot metadata. Does not take out locks, since it's
// called internally from e.g. Create().
func (s *SnapshotStore) saveMetadata() error {
	bz, err := json.Marshal(s.metadata)
	if err != nil {
		return err
	}

	// save the file to a new file and move it to make saving atomic.
	newFile := filepath.Join(s.dir, "metadata.json.new")
	file := filepath.Join(s.dir, "metadata.json")
	err = os.WriteFile(newFile, bz, 0o644) //nolint: gosec
	if err != nil {
		return err
	}
	return os.Rename(newFile, file)
}

// Create creates a snapshot of the given application state's key/value pairs.
func (s *SnapshotStore) Create(state *State) (abci.Snapshot, error) {
	s.Lock()
	defer s.Unlock()
	bz, height, stateHash, err := state.Export()
	if err != nil {
		return abci.Snapshot{}, err
	}
	snapshot := abci.Snapshot{
		Height: height,
		Format: 1,
		Hash:   stateHash,
		Chunks: byteChunks(bz),
	}
	err = os.WriteFile(filepath.Join(s.dir, fmt.Sprintf("%v.json", height)), bz, 0o644) //nolint:gosec
	if err != nil {
		return abci.Snapshot{}, err
	}
	s.metadata = append(s.metadata, snapshot)
	err = s.saveMetadata()
	if err != nil {
		return abci.Snapshot{}, err
	}
	return snapshot, nil
}

// Prune removes old snapshots ensuring only the most recent n snapshots remain
func (s *SnapshotStore) Prune(n int) error {
	s.Lock()
	defer s.Unlock()
	// snapshots are appended to the metadata struct, hence pruning removes from
	// the front of the array
	i := 0
	for ; i < len(s.metadata)-n; i++ {
		h := s.metadata[i].Height
		if err := os.Remove(filepath.Join(s.dir, fmt.Sprintf("%v.json", h))); err != nil {
			return err
		}
	}

	// update metadata by removing the deleted snapshots
	pruned := make([]abci.Snapshot, len(s.metadata[i:]))
	copy(pruned, s.metadata[i:])
	s.metadata = pruned
	return nil
}

// List lists available snapshots.
func (s *SnapshotStore) List() ([]*abci.Snapshot, error) {
	s.RLock()
	defer s.RUnlock()
	snapshots := make([]*abci.Snapshot, len(s.metadata))
	for idx := range s.metadata {
		snapshots[idx] = &s.metadata[idx]
	}
	return snapshots, nil
}

// LoadChunk loads a snapshot chunk.
func (s *SnapshotStore) LoadChunk(height uint64, format uint32, chunk uint32) ([]byte, error) {
	s.RLock()
	defer s.RUnlock()
	for _, snapshot := range s.metadata {
		if snapshot.Height == height && snapshot.Format == format {
			bz, err := os.ReadFile(filepath.Join(s.dir, fmt.Sprintf("%v.json", height)))
			if err != nil {
				return nil, err
			}
			return byteChunk(bz, chunk), nil
		}
	}
	return nil, nil
}

// byteChunk returns the chunk at a given index from the full byte slice.
func byteChunk(bz []byte, index uint32) []byte {
	start := int(index * snapshotChunkSize)
	end := int((index + 1) * snapshotChunkSize)
	switch {
	case start >= len(bz):
		return nil
	case end >= len(bz):
		return bz[start:]
	default:
		return bz[start:end]
	}
}

// byteChunks calculates the number of chunks in the byte slice.
func byteChunks(bz []byte) uint32 {
	return uint32(math.Ceil(float64(len(bz)) / snapshotChunkSize))
}
