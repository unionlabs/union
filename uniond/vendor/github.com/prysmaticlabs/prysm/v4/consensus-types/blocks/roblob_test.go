package blocks

import (
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
)

func TestNewROBlobWithRoot(t *testing.T) {
	sidecar := &ethpb.BlobSidecar{}
	root := [32]byte{}

	blob, err := NewROBlobWithRoot(sidecar, root)
	assert.NoError(t, err)
	assert.Equal(t, root, blob.BlockRoot())

	blob, err = NewROBlobWithRoot(nil, root)
	assert.Equal(t, errNilBlock, err)
}

// TestNewROBlob tests the NewROBlob function.
func TestNewROBlob(t *testing.T) {
	h := &ethpb.SignedBeaconBlockHeader{
		Header: &ethpb.BeaconBlockHeader{
			ParentRoot: make([]byte, fieldparams.RootLength),
			StateRoot:  make([]byte, fieldparams.RootLength),
			BodyRoot:   make([]byte, fieldparams.RootLength),
		},
		Signature: make([]byte, fieldparams.BLSSignatureLength),
	}
	sidecar := &ethpb.BlobSidecar{
		SignedBlockHeader: h,
	}

	blob, err := NewROBlob(sidecar)
	assert.NoError(t, err)
	assert.NotNil(t, blob)

	_, err = NewROBlob(nil)
	assert.Equal(t, errNilBlock, err)

	sidecar.SignedBlockHeader = nil
	_, err = NewROBlob(sidecar)
	assert.Equal(t, errNilBlockHeader, err)

	sidecar.SignedBlockHeader = &ethpb.SignedBeaconBlockHeader{}
	_, err = NewROBlob(sidecar)
	assert.Equal(t, errNilBlockHeader, err)
}

func TestBlockRoot(t *testing.T) {
	root := [32]byte{1}
	blob := &ROBlob{
		root: root,
	}
	assert.Equal(t, root, blob.BlockRoot())
}

func TestSlot(t *testing.T) {
	slot := primitives.Slot(1)
	blob := &ROBlob{
		BlobSidecar: &ethpb.BlobSidecar{
			SignedBlockHeader: &ethpb.SignedBeaconBlockHeader{
				Header: &ethpb.BeaconBlockHeader{
					Slot: slot,
				},
			},
		},
	}
	assert.Equal(t, slot, blob.Slot())
}

func TestParentRoot(t *testing.T) {
	root := [32]byte{1}
	blob := &ROBlob{
		BlobSidecar: &ethpb.BlobSidecar{
			SignedBlockHeader: &ethpb.SignedBeaconBlockHeader{
				Header: &ethpb.BeaconBlockHeader{
					ParentRoot: root[:],
				},
			},
		},
	}
	assert.Equal(t, root, blob.ParentRoot())
}

func TestBodyRoot(t *testing.T) {
	root := [32]byte{1}
	blob := &ROBlob{
		BlobSidecar: &ethpb.BlobSidecar{
			SignedBlockHeader: &ethpb.SignedBeaconBlockHeader{
				Header: &ethpb.BeaconBlockHeader{
					BodyRoot: root[:],
				},
			},
		},
	}
	assert.Equal(t, root, blob.BodyRoot())
}

func TestProposeIndex(t *testing.T) {
	index := primitives.ValidatorIndex(1)
	blob := &ROBlob{
		BlobSidecar: &ethpb.BlobSidecar{
			SignedBlockHeader: &ethpb.SignedBeaconBlockHeader{
				Header: &ethpb.BeaconBlockHeader{
					ProposerIndex: index,
				},
			},
		},
	}
	assert.Equal(t, index, blob.ProposerIndex())
}
