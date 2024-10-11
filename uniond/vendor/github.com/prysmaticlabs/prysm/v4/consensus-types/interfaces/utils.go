package interfaces

import (
	"github.com/pkg/errors"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// SignedBeaconBlockHeaderFromBlock function to retrieve signed block header from block.
func SignedBeaconBlockHeaderFromBlock(block *ethpb.SignedBeaconBlock) (*ethpb.SignedBeaconBlockHeader, error) {
	if block.Block == nil || block.Block.Body == nil {
		return nil, errors.New("nil block")
	}

	bodyRoot, err := block.Block.Body.HashTreeRoot()
	if err != nil {
		return nil, errors.Wrap(err, "failed to get body root of block")
	}
	return &ethpb.SignedBeaconBlockHeader{
		Header: &ethpb.BeaconBlockHeader{
			Slot:          block.Block.Slot,
			ProposerIndex: block.Block.ProposerIndex,
			ParentRoot:    block.Block.ParentRoot,
			StateRoot:     block.Block.StateRoot,
			BodyRoot:      bodyRoot[:],
		},
		Signature: block.Signature,
	}, nil
}

// SignedBeaconBlockHeaderFromBlockInterface function to retrieve signed block header from block.
func SignedBeaconBlockHeaderFromBlockInterface(sb ReadOnlySignedBeaconBlock) (*ethpb.SignedBeaconBlockHeader, error) {
	b := sb.Block()
	if b.IsNil() || b.Body().IsNil() {
		return nil, errors.New("nil block")
	}

	h, err := BeaconBlockHeaderFromBlockInterface(b)
	if err != nil {
		return nil, errors.Wrap(err, "failed to get block header of block")
	}
	sig := sb.Signature()
	return &ethpb.SignedBeaconBlockHeader{
		Header:    h,
		Signature: sig[:],
	}, nil
}

// BeaconBlockHeaderFromBlock function to retrieve block header from block.
func BeaconBlockHeaderFromBlock(block *ethpb.BeaconBlock) (*ethpb.BeaconBlockHeader, error) {
	if block.Body == nil {
		return nil, errors.New("nil block body")
	}

	bodyRoot, err := block.Body.HashTreeRoot()
	if err != nil {
		return nil, errors.Wrap(err, "failed to get body root of block")
	}
	return &ethpb.BeaconBlockHeader{
		Slot:          block.Slot,
		ProposerIndex: block.ProposerIndex,
		ParentRoot:    block.ParentRoot,
		StateRoot:     block.StateRoot,
		BodyRoot:      bodyRoot[:],
	}, nil
}

// BeaconBlockHeaderFromBlockInterface function to retrieve block header from block.
func BeaconBlockHeaderFromBlockInterface(block ReadOnlyBeaconBlock) (*ethpb.BeaconBlockHeader, error) {
	if block.Body().IsNil() {
		return nil, errors.New("nil block body")
	}

	bodyRoot, err := block.Body().HashTreeRoot()
	if err != nil {
		return nil, errors.Wrap(err, "failed to get body root of block")
	}
	parentRoot := block.ParentRoot()
	stateRoot := block.StateRoot()
	return &ethpb.BeaconBlockHeader{
		Slot:          block.Slot(),
		ProposerIndex: block.ProposerIndex(),
		ParentRoot:    parentRoot[:],
		StateRoot:     stateRoot[:],
		BodyRoot:      bodyRoot[:],
	}, nil
}
