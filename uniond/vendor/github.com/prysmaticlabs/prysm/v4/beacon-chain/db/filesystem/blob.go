package filesystem

import (
	"fmt"
	"os"
	"path"
	"strconv"
	"strings"
	"time"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/verification"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/logging"
	log "github.com/sirupsen/logrus"
	"github.com/spf13/afero"
)

var (
	errIndexOutOfBounds = errors.New("blob index in file name >= MaxBlobsPerBlock")
)

const (
	sszExt  = "ssz"
	partExt = "part"

	directoryPermissions = 0700
)

// BlobStorageOption is a functional option for configuring a BlobStorage.
type BlobStorageOption func(*BlobStorage) error

// WithBlobRetentionEpochs is an option that changes the number of epochs blobs will be persisted.
func WithBlobRetentionEpochs(e primitives.Epoch) BlobStorageOption {
	return func(b *BlobStorage) error {
		pruner, err := newBlobPruner(b.fs, e)
		if err != nil {
			return err
		}
		b.pruner = pruner
		return nil
	}
}

// NewBlobStorage creates a new instance of the BlobStorage object. Note that the implementation of BlobStorage may
// attempt to hold a file lock to guarantee exclusive control of the blob storage directory, so this should only be
// initialized once per beacon node.
func NewBlobStorage(base string, opts ...BlobStorageOption) (*BlobStorage, error) {
	base = path.Clean(base)
	if err := file.MkdirAll(base); err != nil {
		return nil, fmt.Errorf("failed to create blob storage at %s: %w", base, err)
	}
	fs := afero.NewBasePathFs(afero.NewOsFs(), base)
	b := &BlobStorage{
		fs: fs,
	}
	for _, o := range opts {
		if err := o(b); err != nil {
			return nil, fmt.Errorf("failed to create blob storage at %s: %w", base, err)
		}
	}
	if b.pruner == nil {
		log.Warn("Initializing blob filesystem storage with pruning disabled")
	}
	return b, nil
}

// BlobStorage is the concrete implementation of the filesystem backend for saving and retrieving BlobSidecars.
type BlobStorage struct {
	fs     afero.Fs
	pruner *blobPruner
}

// WarmCache runs the prune routine with an expiration of slot of 0, so nothing will be pruned, but the pruner's cache
// will be populated at node startup, avoiding a costly cold prune (~4s in syscalls) during syncing.
func (bs *BlobStorage) WarmCache() {
	if bs.pruner == nil {
		return
	}
	go func() {
		if err := bs.pruner.prune(0); err != nil {
			log.WithError(err).Error("Error encountered while warming up blob pruner cache.")
		}
	}()
}

// Save saves blobs given a list of sidecars.
func (bs *BlobStorage) Save(sidecar blocks.VerifiedROBlob) error {
	startTime := time.Now()
	fname := namerForSidecar(sidecar)
	sszPath := fname.path()
	exists, err := afero.Exists(bs.fs, sszPath)
	if err != nil {
		return err
	}
	if exists {
		log.WithFields(logging.BlobFields(sidecar.ROBlob)).Debug("ignoring a duplicate blob sidecar Save attempt")
		return nil
	}
	if bs.pruner != nil {
		bs.pruner.notify(sidecar.BlockRoot(), sidecar.Slot())
	}

	// Serialize the ethpb.BlobSidecar to binary data using SSZ.
	sidecarData, err := sidecar.MarshalSSZ()
	if err != nil {
		return errors.Wrap(err, "failed to serialize sidecar data")
	}
	if err := bs.fs.MkdirAll(fname.dir(), directoryPermissions); err != nil {
		return err
	}
	partPath := fname.partPath()

	partialMoved := false
	// Ensure the partial file is deleted.
	defer func() {
		if partialMoved {
			return
		}
		// It's expected to error if the save is successful.
		err = bs.fs.Remove(partPath)
		if err == nil {
			log.WithFields(log.Fields{
				"partPath": partPath,
			}).Debugf("removed partial file")
		}
	}()

	// Create a partial file and write the serialized data to it.
	partialFile, err := bs.fs.Create(partPath)
	if err != nil {
		return errors.Wrap(err, "failed to create partial file")
	}

	_, err = partialFile.Write(sidecarData)
	if err != nil {
		closeErr := partialFile.Close()
		if closeErr != nil {
			return closeErr
		}
		return errors.Wrap(err, "failed to write to partial file")
	}
	err = partialFile.Close()
	if err != nil {
		return err
	}

	// Atomically rename the partial file to its final name.
	err = bs.fs.Rename(partPath, sszPath)
	if err != nil {
		return errors.Wrap(err, "failed to rename partial file to final name")
	}
	partialMoved = true
	blobsWrittenCounter.Inc()
	blobSaveLatency.Observe(float64(time.Since(startTime).Milliseconds()))
	return nil
}

// Get retrieves a single BlobSidecar by its root and index.
// Since BlobStorage only writes blobs that have undergone full verification, the return
// value is always a VerifiedROBlob.
func (bs *BlobStorage) Get(root [32]byte, idx uint64) (blocks.VerifiedROBlob, error) {
	startTime := time.Now()
	expected := blobNamer{root: root, index: idx}
	encoded, err := afero.ReadFile(bs.fs, expected.path())
	var v blocks.VerifiedROBlob
	if err != nil {
		return v, err
	}
	s := &ethpb.BlobSidecar{}
	if err := s.UnmarshalSSZ(encoded); err != nil {
		return v, err
	}
	ro, err := blocks.NewROBlobWithRoot(s, root)
	if err != nil {
		return blocks.VerifiedROBlob{}, err
	}
	defer func() {
		blobFetchLatency.Observe(float64(time.Since(startTime).Milliseconds()))
	}()
	return verification.BlobSidecarNoop(ro)
}

// Remove removes all blobs for a given root.
func (bs *BlobStorage) Remove(root [32]byte) error {
	rootDir := blobNamer{root: root}.dir()
	return bs.fs.RemoveAll(rootDir)
}

// Indices generates a bitmap representing which BlobSidecar.Index values are present on disk for a given root.
// This value can be compared to the commitments observed in a block to determine which indices need to be found
// on the network to confirm data availability.
func (bs *BlobStorage) Indices(root [32]byte) ([fieldparams.MaxBlobsPerBlock]bool, error) {
	var mask [fieldparams.MaxBlobsPerBlock]bool
	rootDir := blobNamer{root: root}.dir()
	entries, err := afero.ReadDir(bs.fs, rootDir)
	if err != nil {
		if os.IsNotExist(err) {
			return mask, nil
		}
		return mask, err
	}
	for i := range entries {
		if entries[i].IsDir() {
			continue
		}
		name := entries[i].Name()
		if !strings.HasSuffix(name, sszExt) {
			continue
		}
		parts := strings.Split(name, ".")
		if len(parts) != 2 {
			continue
		}
		u, err := strconv.ParseUint(parts[0], 10, 64)
		if err != nil {
			return mask, errors.Wrapf(err, "unexpected directory entry breaks listing, %s", parts[0])
		}
		if u >= fieldparams.MaxBlobsPerBlock {
			return mask, errIndexOutOfBounds
		}
		mask[u] = true
	}
	return mask, nil
}

type blobNamer struct {
	root  [32]byte
	index uint64
}

func namerForSidecar(sc blocks.VerifiedROBlob) blobNamer {
	return blobNamer{root: sc.BlockRoot(), index: sc.Index}
}

func (p blobNamer) dir() string {
	return rootString(p.root)
}

func (p blobNamer) fname(ext string) string {
	return path.Join(p.dir(), fmt.Sprintf("%d.%s", p.index, ext))
}

func (p blobNamer) partPath() string {
	return p.fname(partExt)
}

func (p blobNamer) path() string {
	return p.fname(sszExt)
}

func rootString(root [32]byte) string {
	return fmt.Sprintf("%#x", root)
}
