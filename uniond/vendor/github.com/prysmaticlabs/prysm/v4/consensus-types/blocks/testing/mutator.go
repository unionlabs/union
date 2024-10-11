package testing

import (
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
)

type blockMutator struct {
	Phase0    func(beaconBlock *eth.SignedBeaconBlock)
	Altair    func(beaconBlock *eth.SignedBeaconBlockAltair)
	Bellatrix func(beaconBlock *eth.SignedBeaconBlockBellatrix)
	Capella   func(beaconBlock *eth.SignedBeaconBlockCapella)
}

func (m blockMutator) apply(b interfaces.SignedBeaconBlock) (interfaces.SignedBeaconBlock, error) {
	switch b.Version() {
	case version.Phase0:
		bb, err := b.PbPhase0Block()
		if err != nil {
			return nil, err
		}
		m.Phase0(bb)
		return blocks.NewSignedBeaconBlock(bb)
	case version.Altair:
		bb, err := b.PbAltairBlock()
		if err != nil {
			return nil, err
		}
		m.Altair(bb)
		return blocks.NewSignedBeaconBlock(bb)
	case version.Bellatrix:
		bb, err := b.PbBellatrixBlock()
		if err != nil {
			return nil, err
		}
		m.Bellatrix(bb)
		return blocks.NewSignedBeaconBlock(bb)
	case version.Capella:
		bb, err := b.PbCapellaBlock()
		if err != nil {
			return nil, err
		}
		m.Capella(bb)
		return blocks.NewSignedBeaconBlock(bb)
	default:
		return nil, blocks.ErrUnsupportedSignedBeaconBlock
	}
}

// SetBlockStateRoot modifies the block's state root.
func SetBlockStateRoot(b interfaces.SignedBeaconBlock, sr [32]byte) (interfaces.SignedBeaconBlock, error) {
	return blockMutator{
		Phase0:    func(bb *eth.SignedBeaconBlock) { bb.Block.StateRoot = sr[:] },
		Altair:    func(bb *eth.SignedBeaconBlockAltair) { bb.Block.StateRoot = sr[:] },
		Bellatrix: func(bb *eth.SignedBeaconBlockBellatrix) { bb.Block.StateRoot = sr[:] },
		Capella:   func(bb *eth.SignedBeaconBlockCapella) { bb.Block.StateRoot = sr[:] },
	}.apply(b)
}

// SetBlockParentRoot modifies the block's parent root.
func SetBlockParentRoot(b interfaces.SignedBeaconBlock, pr [32]byte) (interfaces.SignedBeaconBlock, error) {
	return blockMutator{
		Phase0:    func(bb *eth.SignedBeaconBlock) { bb.Block.ParentRoot = pr[:] },
		Altair:    func(bb *eth.SignedBeaconBlockAltair) { bb.Block.ParentRoot = pr[:] },
		Bellatrix: func(bb *eth.SignedBeaconBlockBellatrix) { bb.Block.ParentRoot = pr[:] },
		Capella:   func(bb *eth.SignedBeaconBlockCapella) { bb.Block.ParentRoot = pr[:] },
	}.apply(b)
}

// SetBlockSlot modifies the block's slot.
func SetBlockSlot(b interfaces.SignedBeaconBlock, s primitives.Slot) (interfaces.SignedBeaconBlock, error) {
	return blockMutator{
		Phase0:    func(bb *eth.SignedBeaconBlock) { bb.Block.Slot = s },
		Altair:    func(bb *eth.SignedBeaconBlockAltair) { bb.Block.Slot = s },
		Bellatrix: func(bb *eth.SignedBeaconBlockBellatrix) { bb.Block.Slot = s },
		Capella:   func(bb *eth.SignedBeaconBlockCapella) { bb.Block.Slot = s },
	}.apply(b)
}

// SetProposerIndex modifies the block's proposer index.
func SetProposerIndex(b interfaces.SignedBeaconBlock, idx primitives.ValidatorIndex) (interfaces.SignedBeaconBlock, error) {
	return blockMutator{
		Phase0:    func(bb *eth.SignedBeaconBlock) { bb.Block.ProposerIndex = idx },
		Altair:    func(bb *eth.SignedBeaconBlockAltair) { bb.Block.ProposerIndex = idx },
		Bellatrix: func(bb *eth.SignedBeaconBlockBellatrix) { bb.Block.ProposerIndex = idx },
		Capella:   func(bb *eth.SignedBeaconBlockCapella) { bb.Block.ProposerIndex = idx },
	}.apply(b)
}
