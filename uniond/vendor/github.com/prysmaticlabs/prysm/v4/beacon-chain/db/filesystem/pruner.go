package filesystem

import (
	"encoding/binary"
	"io"
	"path"
	"path/filepath"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"github.com/pkg/errors"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	log "github.com/sirupsen/logrus"
	"github.com/spf13/afero"
)

const retentionBuffer primitives.Epoch = 2

var (
	errPruningFailures = errors.New("blobs could not be pruned for some roots")
)

type blobPruner struct {
	sync.Mutex
	prunedBefore atomic.Uint64
	windowSize   primitives.Slot
	slotMap      *slotForRoot
	fs           afero.Fs
}

func newBlobPruner(fs afero.Fs, retain primitives.Epoch) (*blobPruner, error) {
	r, err := slots.EpochStart(retain + retentionBuffer)
	if err != nil {
		return nil, errors.Wrap(err, "could not set retentionSlots")
	}
	return &blobPruner{fs: fs, windowSize: r, slotMap: newSlotForRoot()}, nil
}

// notify updates the pruner's view of root->blob mappings. This allows the pruner to build a cache
// of root->slot mappings and decide when to evict old blobs based on the age of present blobs.
func (p *blobPruner) notify(root [32]byte, latest primitives.Slot) {
	p.slotMap.ensure(rootString(root), latest)
	pruned := uint64(windowMin(latest, p.windowSize))
	if p.prunedBefore.Swap(pruned) == pruned {
		return
	}
	go func() {
		if err := p.prune(primitives.Slot(pruned)); err != nil {
			log.WithError(err).Errorf("Failed to prune blobs from slot %d", latest)
		}
	}()
}

func windowMin(latest primitives.Slot, offset primitives.Slot) primitives.Slot {
	// Safely compute the first slot in the epoch for the latest slot
	latest = latest - latest%params.BeaconConfig().SlotsPerEpoch
	if latest < offset {
		return 0
	}
	return latest - offset
}

// Prune prunes blobs in the base directory based on the retention epoch.
// It deletes blobs older than currentEpoch - (retentionEpochs+bufferEpochs).
// This is so that we keep a slight buffer and blobs are deleted after n+2 epochs.
func (p *blobPruner) prune(pruneBefore primitives.Slot) error {
	p.Lock()
	defer p.Unlock()
	start := time.Now()
	totalPruned, totalErr := 0, 0
	// Customize logging/metrics behavior for the initial cache warmup when slot=0.
	// We'll never see a prune request for slot 0, unless this is the initial call to warm up the cache.
	if pruneBefore == 0 {
		defer func() {
			log.WithField("duration", time.Since(start).String()).Debug("Warmed up pruner cache")
		}()
	} else {
		defer func() {
			log.WithFields(log.Fields{
				"upToEpoch":    slots.ToEpoch(pruneBefore),
				"duration":     time.Since(start).String(),
				"filesRemoved": totalPruned,
			}).Debug("Pruned old blobs")
			blobsPrunedCounter.Add(float64(totalPruned))
		}()
	}

	entries, err := listDir(p.fs, ".")
	if err != nil {
		return errors.Wrap(err, "unable to list root blobs directory")
	}
	dirs := filter(entries, filterRoot)
	for _, dir := range dirs {
		pruned, err := p.tryPruneDir(dir, pruneBefore)
		if err != nil {
			totalErr += 1
			log.WithError(err).WithField("directory", dir).Error("Unable to prune directory")
		}
		totalPruned += pruned
	}

	if totalErr > 0 {
		return errors.Wrapf(errPruningFailures, "pruning failed for %d root directories", totalErr)
	}
	return nil
}

func shouldRetain(slot, pruneBefore primitives.Slot) bool {
	return slot >= pruneBefore
}

func (p *blobPruner) tryPruneDir(dir string, pruneBefore primitives.Slot) (int, error) {
	root := rootFromDir(dir)
	slot, slotCached := p.slotMap.slot(root)
	// Return early if the slot is cached and doesn't need pruning.
	if slotCached && shouldRetain(slot, pruneBefore) {
		return 0, nil
	}

	// entries will include things that aren't ssz files, like dangling .part files. We need these to
	// completely clean up the directory.
	entries, err := listDir(p.fs, dir)
	if err != nil {
		return 0, errors.Wrapf(err, "failed to list blobs in directory %s", dir)
	}
	// scFiles filters the dir listing down to the ssz encoded BlobSidecar files. This allows us to peek
	// at the first one in the list to figure out the slot.
	scFiles := filter(entries, filterSsz)
	if len(scFiles) == 0 {
		log.WithField("dir", dir).Warn("Pruner ignoring directory with no blob files")
		return 0, nil
	}
	if !slotCached {
		slot, err = slotFromFile(path.Join(dir, scFiles[0]), p.fs)
		if err != nil {
			return 0, errors.Wrapf(err, "slot could not be read from blob file %s", scFiles[0])
		}
		p.slotMap.ensure(root, slot)
		if shouldRetain(slot, pruneBefore) {
			return 0, nil
		}
	}

	removed := 0
	for _, fname := range entries {
		fullName := path.Join(dir, fname)
		if err := p.fs.Remove(fullName); err != nil {
			return removed, errors.Wrapf(err, "unable to remove %s", fullName)
		}
		// Don't count other files that happen to be in the dir, like dangling .part files.
		if filterSsz(fname) {
			removed += 1
		}
		// Log a warning whenever we clean up a .part file
		if filterPart(fullName) {
			log.WithField("file", fullName).Warn("Deleting abandoned blob .part file")
		}
	}
	if err := p.fs.Remove(dir); err != nil {
		return removed, errors.Wrapf(err, "unable to remove blob directory %s", dir)
	}

	p.slotMap.evict(rootFromDir(dir))
	return len(scFiles), nil
}

func rootFromDir(dir string) string {
	return filepath.Base(dir) // end of the path should be the blob directory, named by hex encoding of root
}

// Read slot from marshaled BlobSidecar data in the given file. See slotFromBlob for details.
func slotFromFile(file string, fs afero.Fs) (primitives.Slot, error) {
	f, err := fs.Open(file)
	if err != nil {
		return 0, err
	}
	defer func() {
		if err := f.Close(); err != nil {
			log.WithError(err).Errorf("Could not close blob file")
		}
	}()
	return slotFromBlob(f)
}

// slotFromBlob reads the ssz data of a file at the specified offset (8 + 131072 + 48 + 48 = 131176 bytes),
// which is calculated based on the size of the BlobSidecar struct and is based on the size of the fields
// preceding the slot information within SignedBeaconBlockHeader.
func slotFromBlob(at io.ReaderAt) (primitives.Slot, error) {
	b := make([]byte, 8)
	_, err := at.ReadAt(b, 131176)
	if err != nil {
		return 0, err
	}
	rawSlot := binary.LittleEndian.Uint64(b)
	return primitives.Slot(rawSlot), nil
}

func listDir(fs afero.Fs, dir string) ([]string, error) {
	top, err := fs.Open(dir)
	if err != nil {
		return nil, errors.Wrap(err, "failed to open directory descriptor")
	}
	defer func() {
		if err := top.Close(); err != nil {
			log.WithError(err).Errorf("Could not close file %s", dir)
		}
	}()
	// re the -1 param: "If n <= 0, Readdirnames returns all the names from the directory in a single slice"
	dirs, err := top.Readdirnames(-1)
	if err != nil {
		return nil, errors.Wrap(err, "failed to read directory listing")
	}
	return dirs, nil
}

func filter(entries []string, filt func(string) bool) []string {
	filtered := make([]string, 0, len(entries))
	for i := range entries {
		if filt(entries[i]) {
			filtered = append(filtered, entries[i])
		}
	}
	return filtered
}

func filterRoot(s string) bool {
	return strings.HasPrefix(s, "0x")
}

var dotSszExt = "." + sszExt
var dotPartExt = "." + partExt

func filterSsz(s string) bool {
	return filepath.Ext(s) == dotSszExt
}

func filterPart(s string) bool {
	return filepath.Ext(s) == dotPartExt
}

func newSlotForRoot() *slotForRoot {
	return &slotForRoot{
		cache: make(map[string]primitives.Slot, params.BeaconConfig().MinEpochsForBlobsSidecarsRequest*fieldparams.SlotsPerEpoch),
	}
}

type slotForRoot struct {
	sync.RWMutex
	cache map[string]primitives.Slot
}

func (s *slotForRoot) ensure(key string, slot primitives.Slot) {
	s.Lock()
	defer s.Unlock()
	s.cache[key] = slot
}

func (s *slotForRoot) slot(key string) (primitives.Slot, bool) {
	s.RLock()
	defer s.RUnlock()
	slot, ok := s.cache[key]
	return slot, ok
}

func (s *slotForRoot) evict(key string) {
	s.Lock()
	defer s.Unlock()
	delete(s.cache, key)
}
