package testing

import (
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// NewSignedBeaconBlockFromGeneric creates a signed beacon block
// from a protobuf generic signed beacon block.
func NewSignedBeaconBlockFromGeneric(gb *eth.GenericSignedBeaconBlock) (interfaces.ReadOnlySignedBeaconBlock, error) {
	if gb == nil {
		return nil, blocks.ErrNilObject
	}
	switch bb := gb.Block.(type) {
	case *eth.GenericSignedBeaconBlock_Phase0:
		return blocks.NewSignedBeaconBlock(bb.Phase0)
	case *eth.GenericSignedBeaconBlock_Altair:
		return blocks.NewSignedBeaconBlock(bb.Altair)
	case *eth.GenericSignedBeaconBlock_Bellatrix:
		return blocks.NewSignedBeaconBlock(bb.Bellatrix)
	case *eth.GenericSignedBeaconBlock_BlindedBellatrix:
		return blocks.NewSignedBeaconBlock(bb.BlindedBellatrix)
	case *eth.GenericSignedBeaconBlock_Capella:
		return blocks.NewSignedBeaconBlock(bb.Capella)
	case *eth.GenericSignedBeaconBlock_BlindedCapella:
		return blocks.NewSignedBeaconBlock(bb.BlindedCapella)
	case *eth.GenericSignedBeaconBlock_Deneb:
		return blocks.NewSignedBeaconBlock(bb.Deneb.Block)
	case *eth.GenericSignedBeaconBlock_BlindedDeneb:
		return blocks.NewSignedBeaconBlock(bb.BlindedDeneb)
	// Generic Signed Beacon Block Deneb can't be used here as it is not a block, but block content with blobs
	default:
		return nil, errors.Wrapf(blocks.ErrUnsupportedSignedBeaconBlock, "unable to create block from type %T", gb)
	}
}
