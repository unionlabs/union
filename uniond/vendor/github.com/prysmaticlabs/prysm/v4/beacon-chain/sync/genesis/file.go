package genesis

import (
	"context"
	"fmt"
	"os"

	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/io/file"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
)

// Initializer describes a type that is able to obtain the checkpoint sync data (BeaconState and SignedBeaconBlock)
// in some way and perform database setup to prepare the beacon node for syncing from the given checkpoint.
// See FileInitializer and APIInitializer.
type Initializer interface {
	Initialize(ctx context.Context, d db.Database) error
}

// NewFileInitializer validates the given path information and creates an Initializer which will
// use the provided state and block files to prepare the node for checkpoint sync.
func NewFileInitializer(statePath string) (*FileInitializer, error) {
	var err error
	if err = existsAndIsFile(statePath); err != nil {
		return nil, err
	}
	// stat just to make sure it actually exists and is a file
	return &FileInitializer{statePath: statePath}, nil
}

// FileInitializer initializes a beacon-node database genesis state and block
// using ssz-encoded state data stored in files on the local filesystem.
type FileInitializer struct {
	statePath string
}

// Initialize is called in the BeaconNode db startup code if an Initializer is present.
// Initialize prepares the beacondb using the provided genesis state.
func (fi *FileInitializer) Initialize(ctx context.Context, d db.Database) error {
	serState, err := file.ReadFileAsBytes(fi.statePath)
	if err != nil {
		return errors.Wrapf(err, "error reading state file %s for checkpoint sync init", fi.statePath)
	}
	log.WithField(
		"hash", fmt.Sprintf("%#x", hash.FastSum256(serState)),
	).Info("Loading genesis state from disk.")
	return d.LoadGenesis(ctx, serState)
}

var _ Initializer = &FileInitializer{}

func existsAndIsFile(path string) error {
	info, err := os.Stat(path)
	if err != nil {
		return errors.Wrapf(err, "error checking existence of ssz-encoded file %s for genesis state init", path)
	}
	if info.IsDir() {
		return fmt.Errorf("%s is a directory, please specify full path to file", path)
	}
	return nil
}
