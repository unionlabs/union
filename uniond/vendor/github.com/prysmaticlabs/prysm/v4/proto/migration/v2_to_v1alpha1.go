package migration

import (
	"github.com/pkg/errors"
	ethpbv2 "github.com/prysmaticlabs/prysm/v4/proto/eth/v2"
	ethpbalpha "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"google.golang.org/protobuf/proto"
)

// AltairToV1Alpha1SignedBlock converts a v2 SignedBeaconBlockAltair proto to a v1alpha1 proto.
func AltairToV1Alpha1SignedBlock(altairBlk *ethpbv2.SignedBeaconBlockAltair) (*ethpbalpha.SignedBeaconBlockAltair, error) {
	marshaledBlk, err := proto.Marshal(altairBlk)
	if err != nil {
		return nil, errors.Wrap(err, "could not marshal block")
	}
	v1alpha1Block := &ethpbalpha.SignedBeaconBlockAltair{}
	if err := proto.Unmarshal(marshaledBlk, v1alpha1Block); err != nil {
		return nil, errors.Wrap(err, "could not unmarshal block")
	}
	return v1alpha1Block, nil
}

// BellatrixToV1Alpha1SignedBlock converts a v2 SignedBeaconBlockBellatrix proto to a v1alpha1 proto.
func BellatrixToV1Alpha1SignedBlock(bellatrixBlk *ethpbv2.SignedBeaconBlockBellatrix) (*ethpbalpha.SignedBeaconBlockBellatrix, error) {
	marshaledBlk, err := proto.Marshal(bellatrixBlk)
	if err != nil {
		return nil, errors.Wrap(err, "could not marshal block")
	}
	v1alpha1Block := &ethpbalpha.SignedBeaconBlockBellatrix{}
	if err := proto.Unmarshal(marshaledBlk, v1alpha1Block); err != nil {
		return nil, errors.Wrap(err, "could not unmarshal block")
	}
	return v1alpha1Block, nil
}

// CapellaToV1Alpha1SignedBlock converts a v2 SignedBeaconBlockCapella proto to a v1alpha1 proto.
func CapellaToV1Alpha1SignedBlock(capellaBlk *ethpbv2.SignedBeaconBlockCapella) (*ethpbalpha.SignedBeaconBlockCapella, error) {
	marshaledBlk, err := proto.Marshal(capellaBlk)
	if err != nil {
		return nil, errors.Wrap(err, "could not marshal block")
	}
	v1alpha1Block := &ethpbalpha.SignedBeaconBlockCapella{}
	if err := proto.Unmarshal(marshaledBlk, v1alpha1Block); err != nil {
		return nil, errors.Wrap(err, "could not unmarshal block")
	}
	return v1alpha1Block, nil
}

// DenebToV1Alpha1SignedBlock converts a v2 SignedBeaconBlockDeneb proto to a v1alpha1 proto.
func DenebToV1Alpha1SignedBlock(denebBlk *ethpbv2.SignedBeaconBlockDeneb) (*ethpbalpha.SignedBeaconBlockDeneb, error) {
	marshaledBlk, err := proto.Marshal(denebBlk)
	if err != nil {
		return nil, errors.Wrap(err, "could not marshal block")
	}
	v1alpha1Block := &ethpbalpha.SignedBeaconBlockDeneb{}
	if err := proto.Unmarshal(marshaledBlk, v1alpha1Block); err != nil {
		return nil, errors.Wrap(err, "could not unmarshal block")
	}
	return v1alpha1Block, nil
}

// V2BeaconBlockDenebToV1Alpha1 converts a v2 Deneb beacon block to a v1alpha1
// Deneb block.
func V2BeaconBlockDenebToV1Alpha1(v2block *ethpbv2.BeaconBlockDeneb) (*ethpbalpha.BeaconBlockDeneb, error) {
	marshaledBlk, err := proto.Marshal(v2block)
	if err != nil {
		return nil, errors.Wrap(err, "could not marshal block")
	}
	v1alpha1block := &ethpbalpha.BeaconBlockDeneb{}
	if err := proto.Unmarshal(marshaledBlk, v1alpha1block); err != nil {
		return nil, errors.Wrap(err, "could not unmarshal block")
	}
	return v1alpha1block, nil
}

// BlindedBellatrixToV1Alpha1SignedBlock converts a v2 SignedBlindedBeaconBlockBellatrix proto to a v1alpha1 proto.
func BlindedBellatrixToV1Alpha1SignedBlock(bellatrixBlk *ethpbv2.SignedBlindedBeaconBlockBellatrix) (*ethpbalpha.SignedBlindedBeaconBlockBellatrix, error) {
	marshaledBlk, err := proto.Marshal(bellatrixBlk)
	if err != nil {
		return nil, errors.Wrap(err, "could not marshal block")
	}
	v1alpha1Block := &ethpbalpha.SignedBlindedBeaconBlockBellatrix{}
	if err := proto.Unmarshal(marshaledBlk, v1alpha1Block); err != nil {
		return nil, errors.Wrap(err, "could not unmarshal block")
	}
	return v1alpha1Block, nil
}

// BlindedCapellaToV1Alpha1SignedBlock converts a v2 SignedBlindedBeaconBlockCapella proto to a v1alpha1 proto.
func BlindedCapellaToV1Alpha1SignedBlock(capellaBlk *ethpbv2.SignedBlindedBeaconBlockCapella) (*ethpbalpha.SignedBlindedBeaconBlockCapella, error) {
	marshaledBlk, err := proto.Marshal(capellaBlk)
	if err != nil {
		return nil, errors.Wrap(err, "could not marshal block")
	}
	v1alpha1Block := &ethpbalpha.SignedBlindedBeaconBlockCapella{}
	if err := proto.Unmarshal(marshaledBlk, v1alpha1Block); err != nil {
		return nil, errors.Wrap(err, "could not unmarshal block")
	}
	return v1alpha1Block, nil
}

// BlindedDenebToV1Alpha1SignedBlock converts a v2 SignedBlindedBeaconBlockDeneb proto to a v1alpha1 proto.
func BlindedDenebToV1Alpha1SignedBlock(denebBlk *ethpbv2.SignedBlindedBeaconBlockDeneb) (*ethpbalpha.SignedBlindedBeaconBlockDeneb, error) {
	marshaledBlk, err := proto.Marshal(denebBlk)
	if err != nil {
		return nil, errors.Wrap(err, "could not marshal block")
	}
	v1alpha1Block := &ethpbalpha.SignedBlindedBeaconBlockDeneb{}
	if err := proto.Unmarshal(marshaledBlk, v1alpha1Block); err != nil {
		return nil, errors.Wrap(err, "could not unmarshal block")
	}
	return v1alpha1Block, nil
}
