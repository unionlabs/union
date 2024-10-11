package blocks

import (
	"fmt"
	"math/big"

	"github.com/pkg/errors"
	ssz "github.com/prysmaticlabs/fastssz"
	field_params "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	consensus_types "github.com/prysmaticlabs/prysm/v4/consensus-types"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/math"
	enginev1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	validatorpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/validator-client"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	log "github.com/sirupsen/logrus"
)

// BeaconBlockIsNil checks if any composite field of input signed beacon block is nil.
// Access to these nil fields will result in run time panic,
// it is recommended to run these checks as first line of defense.
func BeaconBlockIsNil(b interfaces.ReadOnlySignedBeaconBlock) error {
	if b == nil || b.IsNil() {
		return ErrNilSignedBeaconBlock
	}
	return nil
}

// Signature returns the respective block signature.
func (b *SignedBeaconBlock) Signature() [field_params.BLSSignatureLength]byte {
	return b.signature
}

// Block returns the underlying beacon block object.
func (b *SignedBeaconBlock) Block() interfaces.ReadOnlyBeaconBlock {
	return b.block
}

// IsNil checks if the underlying beacon block is nil.
func (b *SignedBeaconBlock) IsNil() bool {
	return b == nil || b.block.IsNil()
}

// Copy performs a deep copy of the signed beacon block object.
func (b *SignedBeaconBlock) Copy() (interfaces.SignedBeaconBlock, error) {
	if b == nil {
		return nil, nil
	}

	pb, err := b.Proto()
	if err != nil {
		return nil, err
	}
	switch b.version {
	case version.Phase0:
		cp := eth.CopySignedBeaconBlock(pb.(*eth.SignedBeaconBlock))
		return initSignedBlockFromProtoPhase0(cp)
	case version.Altair:
		cp := eth.CopySignedBeaconBlockAltair(pb.(*eth.SignedBeaconBlockAltair))
		return initSignedBlockFromProtoAltair(cp)
	case version.Bellatrix:
		if b.IsBlinded() {
			cp := eth.CopySignedBlindedBeaconBlockBellatrix(pb.(*eth.SignedBlindedBeaconBlockBellatrix))
			return initBlindedSignedBlockFromProtoBellatrix(cp)
		}
		cp := eth.CopySignedBeaconBlockBellatrix(pb.(*eth.SignedBeaconBlockBellatrix))
		return initSignedBlockFromProtoBellatrix(cp)
	case version.Capella:
		if b.IsBlinded() {
			cp := eth.CopySignedBlindedBeaconBlockCapella(pb.(*eth.SignedBlindedBeaconBlockCapella))
			return initBlindedSignedBlockFromProtoCapella(cp)
		}
		cp := eth.CopySignedBeaconBlockCapella(pb.(*eth.SignedBeaconBlockCapella))
		return initSignedBlockFromProtoCapella(cp)
	case version.Deneb:
		if b.IsBlinded() {
			cp := eth.CopySignedBlindedBeaconBlockDeneb(pb.(*eth.SignedBlindedBeaconBlockDeneb))
			return initBlindedSignedBlockFromProtoDeneb(cp)
		}
		cp := eth.CopySignedBeaconBlockDeneb(pb.(*eth.SignedBeaconBlockDeneb))
		return initSignedBlockFromProtoDeneb(cp)
	default:
		return nil, errIncorrectBlockVersion
	}
}

// PbGenericBlock returns a generic signed beacon block.
func (b *SignedBeaconBlock) PbGenericBlock() (*eth.GenericSignedBeaconBlock, error) {
	pb, err := b.Proto()
	if err != nil {
		return nil, err
	}
	switch b.version {
	case version.Phase0:
		return &eth.GenericSignedBeaconBlock{
			Block: &eth.GenericSignedBeaconBlock_Phase0{Phase0: pb.(*eth.SignedBeaconBlock)},
		}, nil
	case version.Altair:
		return &eth.GenericSignedBeaconBlock{
			Block: &eth.GenericSignedBeaconBlock_Altair{Altair: pb.(*eth.SignedBeaconBlockAltair)},
		}, nil
	case version.Bellatrix:
		if b.IsBlinded() {
			return &eth.GenericSignedBeaconBlock{
				Block: &eth.GenericSignedBeaconBlock_BlindedBellatrix{BlindedBellatrix: pb.(*eth.SignedBlindedBeaconBlockBellatrix)},
			}, nil
		}
		return &eth.GenericSignedBeaconBlock{
			Block: &eth.GenericSignedBeaconBlock_Bellatrix{Bellatrix: pb.(*eth.SignedBeaconBlockBellatrix)},
		}, nil
	case version.Capella:
		if b.IsBlinded() {
			return &eth.GenericSignedBeaconBlock{
				Block: &eth.GenericSignedBeaconBlock_BlindedCapella{BlindedCapella: pb.(*eth.SignedBlindedBeaconBlockCapella)},
			}, nil
		}
		return &eth.GenericSignedBeaconBlock{
			Block: &eth.GenericSignedBeaconBlock_Capella{Capella: pb.(*eth.SignedBeaconBlockCapella)},
		}, nil
	case version.Deneb:
		if b.IsBlinded() {
			return &eth.GenericSignedBeaconBlock{
				Block: &eth.GenericSignedBeaconBlock_BlindedDeneb{BlindedDeneb: pb.(*eth.SignedBlindedBeaconBlockDeneb)},
			}, nil
		}
		return &eth.GenericSignedBeaconBlock{
			Block: &eth.GenericSignedBeaconBlock_Deneb{Deneb: pb.(*eth.SignedBeaconBlockContentsDeneb)},
		}, nil
	default:
		return nil, errIncorrectBlockVersion
	}
}

// PbPhase0Block returns the underlying protobuf object.
func (b *SignedBeaconBlock) PbPhase0Block() (*eth.SignedBeaconBlock, error) {
	if b.version != version.Phase0 {
		return nil, consensus_types.ErrNotSupported("PbPhase0Block", b.version)
	}
	pb, err := b.Proto()
	if err != nil {
		return nil, err
	}
	return pb.(*eth.SignedBeaconBlock), nil
}

// PbAltairBlock returns the underlying protobuf object.
func (b *SignedBeaconBlock) PbAltairBlock() (*eth.SignedBeaconBlockAltair, error) {
	if b.version != version.Altair {
		return nil, consensus_types.ErrNotSupported("PbAltairBlock", b.version)
	}
	pb, err := b.Proto()
	if err != nil {
		return nil, err
	}
	return pb.(*eth.SignedBeaconBlockAltair), nil
}

// PbBellatrixBlock returns the underlying protobuf object.
func (b *SignedBeaconBlock) PbBellatrixBlock() (*eth.SignedBeaconBlockBellatrix, error) {
	if b.version != version.Bellatrix || b.IsBlinded() {
		return nil, consensus_types.ErrNotSupported("PbBellatrixBlock", b.version)
	}
	pb, err := b.Proto()
	if err != nil {
		return nil, err
	}
	return pb.(*eth.SignedBeaconBlockBellatrix), nil
}

// PbBlindedBellatrixBlock returns the underlying protobuf object.
func (b *SignedBeaconBlock) PbBlindedBellatrixBlock() (*eth.SignedBlindedBeaconBlockBellatrix, error) {
	if b.version != version.Bellatrix || !b.IsBlinded() {
		return nil, consensus_types.ErrNotSupported("PbBlindedBellatrixBlock", b.version)
	}
	pb, err := b.Proto()
	if err != nil {
		return nil, err
	}
	return pb.(*eth.SignedBlindedBeaconBlockBellatrix), nil
}

// PbCapellaBlock returns the underlying protobuf object.
func (b *SignedBeaconBlock) PbCapellaBlock() (*eth.SignedBeaconBlockCapella, error) {
	if b.version != version.Capella || b.IsBlinded() {
		return nil, consensus_types.ErrNotSupported("PbCapellaBlock", b.version)
	}
	pb, err := b.Proto()
	if err != nil {
		return nil, err
	}
	return pb.(*eth.SignedBeaconBlockCapella), nil
}

// PbBlindedCapellaBlock returns the underlying protobuf object.
func (b *SignedBeaconBlock) PbBlindedCapellaBlock() (*eth.SignedBlindedBeaconBlockCapella, error) {
	if b.version != version.Capella || !b.IsBlinded() {
		return nil, consensus_types.ErrNotSupported("PbBlindedCapellaBlock", b.version)
	}
	pb, err := b.Proto()
	if err != nil {
		return nil, err
	}
	return pb.(*eth.SignedBlindedBeaconBlockCapella), nil
}

// PbDenebBlock returns the underlying protobuf object.
func (b *SignedBeaconBlock) PbDenebBlock() (*eth.SignedBeaconBlockDeneb, error) {
	if b.version != version.Deneb || b.IsBlinded() {
		return nil, consensus_types.ErrNotSupported("PbDenebBlock", b.version)
	}
	pb, err := b.Proto()
	if err != nil {
		return nil, err
	}
	return pb.(*eth.SignedBeaconBlockDeneb), nil
}

// PbBlindedDenebBlock returns the underlying protobuf object.
func (b *SignedBeaconBlock) PbBlindedDenebBlock() (*eth.SignedBlindedBeaconBlockDeneb, error) {
	if b.version != version.Deneb || !b.IsBlinded() {
		return nil, consensus_types.ErrNotSupported("PbBlindedDenebBlock", b.version)
	}
	pb, err := b.Proto()
	if err != nil {
		return nil, err
	}
	return pb.(*eth.SignedBlindedBeaconBlockDeneb), nil
}

// ToBlinded converts a non-blinded block to its blinded equivalent.
func (b *SignedBeaconBlock) ToBlinded() (interfaces.ReadOnlySignedBeaconBlock, error) {
	if b.version < version.Bellatrix {
		return nil, ErrUnsupportedVersion
	}
	if b.IsBlinded() {
		return b, nil
	}
	if b.block.IsNil() {
		return nil, errors.New("cannot convert nil block to blinded format")
	}
	payload, err := b.block.Body().Execution()
	if err != nil {
		return nil, err
	}

	switch p := payload.Proto().(type) {
	case *enginev1.ExecutionPayload:
		header, err := PayloadToHeader(payload)
		if err != nil {
			return nil, err
		}
		return initBlindedSignedBlockFromProtoBellatrix(
			&eth.SignedBlindedBeaconBlockBellatrix{
				Block: &eth.BlindedBeaconBlockBellatrix{
					Slot:          b.block.slot,
					ProposerIndex: b.block.proposerIndex,
					ParentRoot:    b.block.parentRoot[:],
					StateRoot:     b.block.stateRoot[:],
					Body: &eth.BlindedBeaconBlockBodyBellatrix{
						RandaoReveal:           b.block.body.randaoReveal[:],
						Eth1Data:               b.block.body.eth1Data,
						Graffiti:               b.block.body.graffiti[:],
						ProposerSlashings:      b.block.body.proposerSlashings,
						AttesterSlashings:      b.block.body.attesterSlashings,
						Attestations:           b.block.body.attestations,
						Deposits:               b.block.body.deposits,
						VoluntaryExits:         b.block.body.voluntaryExits,
						SyncAggregate:          b.block.body.syncAggregate,
						ExecutionPayloadHeader: header,
					},
				},
				Signature: b.signature[:],
			})
	case *enginev1.ExecutionPayloadCapella:
		header, err := PayloadToHeaderCapella(payload)
		if err != nil {
			return nil, err
		}
		return initBlindedSignedBlockFromProtoCapella(
			&eth.SignedBlindedBeaconBlockCapella{
				Block: &eth.BlindedBeaconBlockCapella{
					Slot:          b.block.slot,
					ProposerIndex: b.block.proposerIndex,
					ParentRoot:    b.block.parentRoot[:],
					StateRoot:     b.block.stateRoot[:],
					Body: &eth.BlindedBeaconBlockBodyCapella{
						RandaoReveal:           b.block.body.randaoReveal[:],
						Eth1Data:               b.block.body.eth1Data,
						Graffiti:               b.block.body.graffiti[:],
						ProposerSlashings:      b.block.body.proposerSlashings,
						AttesterSlashings:      b.block.body.attesterSlashings,
						Attestations:           b.block.body.attestations,
						Deposits:               b.block.body.deposits,
						VoluntaryExits:         b.block.body.voluntaryExits,
						SyncAggregate:          b.block.body.syncAggregate,
						ExecutionPayloadHeader: header,
						BlsToExecutionChanges:  b.block.body.blsToExecutionChanges,
					},
				},
				Signature: b.signature[:],
			})
	case *enginev1.ExecutionPayloadDeneb:
		header, err := PayloadToHeaderDeneb(payload)
		if err != nil {
			return nil, err
		}
		return initBlindedSignedBlockFromProtoDeneb(
			&eth.SignedBlindedBeaconBlockDeneb{
				Message: &eth.BlindedBeaconBlockDeneb{
					Slot:          b.block.slot,
					ProposerIndex: b.block.proposerIndex,
					ParentRoot:    b.block.parentRoot[:],
					StateRoot:     b.block.stateRoot[:],
					Body: &eth.BlindedBeaconBlockBodyDeneb{
						RandaoReveal:           b.block.body.randaoReveal[:],
						Eth1Data:               b.block.body.eth1Data,
						Graffiti:               b.block.body.graffiti[:],
						ProposerSlashings:      b.block.body.proposerSlashings,
						AttesterSlashings:      b.block.body.attesterSlashings,
						Attestations:           b.block.body.attestations,
						Deposits:               b.block.body.deposits,
						VoluntaryExits:         b.block.body.voluntaryExits,
						SyncAggregate:          b.block.body.syncAggregate,
						ExecutionPayloadHeader: header,
						BlsToExecutionChanges:  b.block.body.blsToExecutionChanges,
						BlobKzgCommitments:     b.block.body.blobKzgCommitments,
					},
				},
				Signature: b.signature[:],
			})
	default:
		return nil, fmt.Errorf("%T is not an execution payload header", p)
	}
}

func (b *SignedBeaconBlock) Unblind(e interfaces.ExecutionData) error {
	if e.IsNil() {
		return errors.New("cannot unblind with nil execution data")
	}
	if !b.IsBlinded() {
		return errors.New("cannot unblind if the block is already unblinded")
	}
	payloadRoot, err := e.HashTreeRoot()
	if err != nil {
		return err
	}
	header, err := b.Block().Body().Execution()
	if err != nil {
		return err
	}
	headerRoot, err := header.HashTreeRoot()
	if err != nil {
		return err
	}
	if payloadRoot != headerRoot {
		return errors.New("cannot unblind with different execution data")
	}
	if err := b.SetExecution(e); err != nil {
		return err
	}
	return nil
}

// Version of the underlying protobuf object.
func (b *SignedBeaconBlock) Version() int {
	return b.version
}

// IsBlinded metadata on whether a block is blinded
func (b *SignedBeaconBlock) IsBlinded() bool {
	return b.version >= version.Bellatrix && b.block.body.executionPayload == nil
}

// ValueInWei metadata on the payload value returned by the builder.
func (b *SignedBeaconBlock) ValueInWei() math.Wei {
	exec, err := b.block.body.Execution()
	if err != nil {
		if !errors.Is(err, consensus_types.ErrUnsupportedField) {
			log.WithError(err).Warn("failed to retrieve execution payload")
		}
		return big.NewInt(0)
	}
	val, err := exec.ValueInWei()
	if err != nil {
		if !errors.Is(err, consensus_types.ErrUnsupportedField) {
			log.WithError(err).Warn("failed to retrieve execution payload")
		}
		return big.NewInt(0)
	}
	return val
}

// ValueInGwei metadata on the payload value returned by the builder.
func (b *SignedBeaconBlock) ValueInGwei() uint64 {
	exec, err := b.block.body.Execution()
	if err != nil {
		if !errors.Is(err, consensus_types.ErrUnsupportedField) {
			log.WithError(err).Warn("failed to retrieve execution payload")
		}
		return 0
	}
	val, err := exec.ValueInGwei()
	if err != nil {
		if !errors.Is(err, consensus_types.ErrUnsupportedField) {
			log.WithError(err).Warn("failed to retrieve execution payload")
		}
		return 0
	}
	return val
}

// Header converts the underlying protobuf object from blinded block to header format.
func (b *SignedBeaconBlock) Header() (*eth.SignedBeaconBlockHeader, error) {
	if b.IsNil() {
		return nil, errNilBlock
	}
	root, err := b.block.body.HashTreeRoot()
	if err != nil {
		return nil, errors.Wrapf(err, "could not hash block body")
	}

	return &eth.SignedBeaconBlockHeader{
		Header: &eth.BeaconBlockHeader{
			Slot:          b.block.slot,
			ProposerIndex: b.block.proposerIndex,
			ParentRoot:    b.block.parentRoot[:],
			StateRoot:     b.block.stateRoot[:],
			BodyRoot:      root[:],
		},
		Signature: b.signature[:],
	}, nil
}

// MarshalSSZ marshals the signed beacon block to its relevant ssz form.
func (b *SignedBeaconBlock) MarshalSSZ() ([]byte, error) {
	pb, err := b.Proto()
	if err != nil {
		return []byte{}, err
	}
	switch b.version {
	case version.Phase0:
		return pb.(*eth.SignedBeaconBlock).MarshalSSZ()
	case version.Altair:
		return pb.(*eth.SignedBeaconBlockAltair).MarshalSSZ()
	case version.Bellatrix:
		if b.IsBlinded() {
			return pb.(*eth.SignedBlindedBeaconBlockBellatrix).MarshalSSZ()
		}
		return pb.(*eth.SignedBeaconBlockBellatrix).MarshalSSZ()
	case version.Capella:
		if b.IsBlinded() {
			return pb.(*eth.SignedBlindedBeaconBlockCapella).MarshalSSZ()
		}
		return pb.(*eth.SignedBeaconBlockCapella).MarshalSSZ()
	case version.Deneb:
		if b.IsBlinded() {
			return pb.(*eth.SignedBlindedBeaconBlockDeneb).MarshalSSZ()
		}
		return pb.(*eth.SignedBeaconBlockDeneb).MarshalSSZ()
	default:
		return []byte{}, errIncorrectBlockVersion
	}
}

// MarshalSSZTo marshals the signed beacon block's ssz
// form to the provided byte buffer.
func (b *SignedBeaconBlock) MarshalSSZTo(dst []byte) ([]byte, error) {
	pb, err := b.Proto()
	if err != nil {
		return []byte{}, err
	}
	switch b.version {
	case version.Phase0:
		return pb.(*eth.SignedBeaconBlock).MarshalSSZTo(dst)
	case version.Altair:
		return pb.(*eth.SignedBeaconBlockAltair).MarshalSSZTo(dst)
	case version.Bellatrix:
		if b.IsBlinded() {
			return pb.(*eth.SignedBlindedBeaconBlockBellatrix).MarshalSSZTo(dst)
		}
		return pb.(*eth.SignedBeaconBlockBellatrix).MarshalSSZTo(dst)
	case version.Capella:
		if b.IsBlinded() {
			return pb.(*eth.SignedBlindedBeaconBlockCapella).MarshalSSZTo(dst)
		}
		return pb.(*eth.SignedBeaconBlockCapella).MarshalSSZTo(dst)
	case version.Deneb:
		if b.IsBlinded() {
			return pb.(*eth.SignedBlindedBeaconBlockDeneb).MarshalSSZTo(dst)
		}
		return pb.(*eth.SignedBeaconBlockDeneb).MarshalSSZTo(dst)
	default:
		return []byte{}, errIncorrectBlockVersion
	}
}

// SizeSSZ returns the size of the serialized signed block
//
// WARNING: This function panics. It is required to change the signature
// of fastssz's SizeSSZ() interface function to avoid panicking.
// Changing the signature causes very problematic issues with wealdtech deps.
// For the time being panicking is preferable.
func (b *SignedBeaconBlock) SizeSSZ() int {
	pb, err := b.Proto()
	if err != nil {
		panic(err)
	}
	switch b.version {
	case version.Phase0:
		return pb.(*eth.SignedBeaconBlock).SizeSSZ()
	case version.Altair:
		return pb.(*eth.SignedBeaconBlockAltair).SizeSSZ()
	case version.Bellatrix:
		if b.IsBlinded() {
			return pb.(*eth.SignedBlindedBeaconBlockBellatrix).SizeSSZ()
		}
		return pb.(*eth.SignedBeaconBlockBellatrix).SizeSSZ()
	case version.Capella:
		if b.IsBlinded() {
			return pb.(*eth.SignedBlindedBeaconBlockCapella).SizeSSZ()
		}
		return pb.(*eth.SignedBeaconBlockCapella).SizeSSZ()
	case version.Deneb:
		if b.IsBlinded() {
			return pb.(*eth.SignedBlindedBeaconBlockDeneb).SizeSSZ()
		}
		return pb.(*eth.SignedBeaconBlockDeneb).SizeSSZ()
	default:
		panic(incorrectBlockVersion)
	}
}

// UnmarshalSSZ unmarshals the signed beacon block from its relevant ssz form.
func (b *SignedBeaconBlock) UnmarshalSSZ(buf []byte) error {
	var newBlock *SignedBeaconBlock
	switch b.version {
	case version.Phase0:
		pb := &eth.SignedBeaconBlock{}
		if err := pb.UnmarshalSSZ(buf); err != nil {
			return err
		}
		var err error
		newBlock, err = initSignedBlockFromProtoPhase0(pb)
		if err != nil {
			return err
		}
	case version.Altair:
		pb := &eth.SignedBeaconBlockAltair{}
		if err := pb.UnmarshalSSZ(buf); err != nil {
			return err
		}
		var err error
		newBlock, err = initSignedBlockFromProtoAltair(pb)
		if err != nil {
			return err
		}
	case version.Bellatrix:
		if b.IsBlinded() {
			pb := &eth.SignedBlindedBeaconBlockBellatrix{}
			if err := pb.UnmarshalSSZ(buf); err != nil {
				return err
			}
			var err error
			newBlock, err = initBlindedSignedBlockFromProtoBellatrix(pb)
			if err != nil {
				return err
			}
		} else {
			pb := &eth.SignedBeaconBlockBellatrix{}
			if err := pb.UnmarshalSSZ(buf); err != nil {
				return err
			}
			var err error
			newBlock, err = initSignedBlockFromProtoBellatrix(pb)
			if err != nil {
				return err
			}
		}
	case version.Capella:
		if b.IsBlinded() {
			pb := &eth.SignedBlindedBeaconBlockCapella{}
			if err := pb.UnmarshalSSZ(buf); err != nil {
				return err
			}
			var err error
			newBlock, err = initBlindedSignedBlockFromProtoCapella(pb)
			if err != nil {
				return err
			}
		} else {
			pb := &eth.SignedBeaconBlockCapella{}
			if err := pb.UnmarshalSSZ(buf); err != nil {
				return err
			}
			var err error
			newBlock, err = initSignedBlockFromProtoCapella(pb)
			if err != nil {
				return err
			}
		}
	case version.Deneb:
		if b.IsBlinded() {
			pb := &eth.SignedBlindedBeaconBlockDeneb{}
			if err := pb.UnmarshalSSZ(buf); err != nil {
				return err
			}
			var err error
			newBlock, err = initBlindedSignedBlockFromProtoDeneb(pb)
			if err != nil {
				return err
			}
		} else {
			pb := &eth.SignedBeaconBlockDeneb{}
			if err := pb.UnmarshalSSZ(buf); err != nil {
				return err
			}
			var err error
			newBlock, err = initSignedBlockFromProtoDeneb(pb)
			if err != nil {
				return err
			}
		}
	default:
		return errIncorrectBlockVersion
	}
	*b = *newBlock
	return nil
}

// Slot returns the respective slot of the block.
func (b *BeaconBlock) Slot() primitives.Slot {
	return b.slot
}

// ProposerIndex returns the proposer index of the beacon block.
func (b *BeaconBlock) ProposerIndex() primitives.ValidatorIndex {
	return b.proposerIndex
}

// ParentRoot returns the parent root of beacon block.
func (b *BeaconBlock) ParentRoot() [field_params.RootLength]byte {
	return b.parentRoot
}

// StateRoot returns the state root of the beacon block.
func (b *BeaconBlock) StateRoot() [field_params.RootLength]byte {
	return b.stateRoot
}

// Body returns the underlying block body.
func (b *BeaconBlock) Body() interfaces.ReadOnlyBeaconBlockBody {
	return b.body
}

// IsNil checks if the beacon block is nil.
func (b *BeaconBlock) IsNil() bool {
	return b == nil || b.Body().IsNil()
}

// IsBlinded checks if the beacon block is a blinded block.
func (b *BeaconBlock) IsBlinded() bool {
	return b.version >= version.Bellatrix && b.body.executionPayload == nil
}

// Version of the underlying protobuf object.
func (b *BeaconBlock) Version() int {
	return b.version
}

// HashTreeRoot returns the ssz root of the block.
func (b *BeaconBlock) HashTreeRoot() ([field_params.RootLength]byte, error) {
	pb, err := b.Proto()
	if err != nil {
		return [field_params.RootLength]byte{}, err
	}
	switch b.version {
	case version.Phase0:
		return pb.(*eth.BeaconBlock).HashTreeRoot()
	case version.Altair:
		return pb.(*eth.BeaconBlockAltair).HashTreeRoot()
	case version.Bellatrix:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockBellatrix).HashTreeRoot()
		}
		return pb.(*eth.BeaconBlockBellatrix).HashTreeRoot()
	case version.Capella:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockCapella).HashTreeRoot()
		}
		return pb.(*eth.BeaconBlockCapella).HashTreeRoot()
	case version.Deneb:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockDeneb).HashTreeRoot()
		}
		return pb.(*eth.BeaconBlockDeneb).HashTreeRoot()
	default:
		return [field_params.RootLength]byte{}, errIncorrectBlockVersion
	}
}

// HashTreeRootWith ssz hashes the BeaconBlock object with a hasher.
func (b *BeaconBlock) HashTreeRootWith(h *ssz.Hasher) error {
	pb, err := b.Proto()
	if err != nil {
		return err
	}
	switch b.version {
	case version.Phase0:
		return pb.(*eth.BeaconBlock).HashTreeRootWith(h)
	case version.Altair:
		return pb.(*eth.BeaconBlockAltair).HashTreeRootWith(h)
	case version.Bellatrix:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockBellatrix).HashTreeRootWith(h)
		}
		return pb.(*eth.BeaconBlockBellatrix).HashTreeRootWith(h)
	case version.Capella:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockCapella).HashTreeRootWith(h)
		}
		return pb.(*eth.BeaconBlockCapella).HashTreeRootWith(h)
	case version.Deneb:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockDeneb).HashTreeRootWith(h)
		}
		return pb.(*eth.BeaconBlockDeneb).HashTreeRootWith(h)
	default:
		return errIncorrectBlockVersion
	}
}

// MarshalSSZ marshals the block into its respective
// ssz form.
func (b *BeaconBlock) MarshalSSZ() ([]byte, error) {
	pb, err := b.Proto()
	if err != nil {
		return []byte{}, err
	}
	switch b.version {
	case version.Phase0:
		return pb.(*eth.BeaconBlock).MarshalSSZ()
	case version.Altair:
		return pb.(*eth.BeaconBlockAltair).MarshalSSZ()
	case version.Bellatrix:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockBellatrix).MarshalSSZ()
		}
		return pb.(*eth.BeaconBlockBellatrix).MarshalSSZ()
	case version.Capella:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockCapella).MarshalSSZ()
		}
		return pb.(*eth.BeaconBlockCapella).MarshalSSZ()
	case version.Deneb:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockDeneb).MarshalSSZ()
		}
		return pb.(*eth.BeaconBlockDeneb).MarshalSSZ()
	default:
		return []byte{}, errIncorrectBlockVersion
	}
}

// MarshalSSZTo marshals the beacon block's ssz
// form to the provided byte buffer.
func (b *BeaconBlock) MarshalSSZTo(dst []byte) ([]byte, error) {
	pb, err := b.Proto()
	if err != nil {
		return []byte{}, err
	}
	switch b.version {
	case version.Phase0:
		return pb.(*eth.BeaconBlock).MarshalSSZTo(dst)
	case version.Altair:
		return pb.(*eth.BeaconBlockAltair).MarshalSSZTo(dst)
	case version.Bellatrix:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockBellatrix).MarshalSSZTo(dst)
		}
		return pb.(*eth.BeaconBlockBellatrix).MarshalSSZTo(dst)
	case version.Capella:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockCapella).MarshalSSZTo(dst)
		}
		return pb.(*eth.BeaconBlockCapella).MarshalSSZTo(dst)
	case version.Deneb:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockDeneb).MarshalSSZTo(dst)
		}
		return pb.(*eth.BeaconBlockDeneb).MarshalSSZTo(dst)
	default:
		return []byte{}, errIncorrectBlockVersion
	}
}

// SizeSSZ returns the size of the serialized block.
//
// WARNING: This function panics. It is required to change the signature
// of fastssz's SizeSSZ() interface function to avoid panicking.
// Changing the signature causes very problematic issues with wealdtech deps.
// For the time being panicking is preferable.
func (b *BeaconBlock) SizeSSZ() int {
	pb, err := b.Proto()
	if err != nil {
		panic(err)
	}
	switch b.version {
	case version.Phase0:
		return pb.(*eth.BeaconBlock).SizeSSZ()
	case version.Altair:
		return pb.(*eth.BeaconBlockAltair).SizeSSZ()
	case version.Bellatrix:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockBellatrix).SizeSSZ()
		}
		return pb.(*eth.BeaconBlockBellatrix).SizeSSZ()
	case version.Capella:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockCapella).SizeSSZ()
		}
		return pb.(*eth.BeaconBlockCapella).SizeSSZ()
	case version.Deneb:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockDeneb).SizeSSZ()
		}
		return pb.(*eth.BeaconBlockDeneb).SizeSSZ()
	default:
		panic(incorrectBodyVersion)
	}
}

// UnmarshalSSZ unmarshals the beacon block from its relevant ssz form.
func (b *BeaconBlock) UnmarshalSSZ(buf []byte) error {
	var newBlock *BeaconBlock
	switch b.version {
	case version.Phase0:
		pb := &eth.BeaconBlock{}
		if err := pb.UnmarshalSSZ(buf); err != nil {
			return err
		}
		var err error
		newBlock, err = initBlockFromProtoPhase0(pb)
		if err != nil {
			return err
		}
	case version.Altair:
		pb := &eth.BeaconBlockAltair{}
		if err := pb.UnmarshalSSZ(buf); err != nil {
			return err
		}
		var err error
		newBlock, err = initBlockFromProtoAltair(pb)
		if err != nil {
			return err
		}
	case version.Bellatrix:
		if b.IsBlinded() {
			pb := &eth.BlindedBeaconBlockBellatrix{}
			if err := pb.UnmarshalSSZ(buf); err != nil {
				return err
			}
			var err error
			newBlock, err = initBlindedBlockFromProtoBellatrix(pb)
			if err != nil {
				return err
			}
		} else {
			pb := &eth.BeaconBlockBellatrix{}
			if err := pb.UnmarshalSSZ(buf); err != nil {
				return err
			}
			var err error
			newBlock, err = initBlockFromProtoBellatrix(pb)
			if err != nil {
				return err
			}
		}
	case version.Capella:
		if b.IsBlinded() {
			pb := &eth.BlindedBeaconBlockCapella{}
			if err := pb.UnmarshalSSZ(buf); err != nil {
				return err
			}
			var err error
			newBlock, err = initBlindedBlockFromProtoCapella(pb)
			if err != nil {
				return err
			}
		} else {
			pb := &eth.BeaconBlockCapella{}
			if err := pb.UnmarshalSSZ(buf); err != nil {
				return err
			}
			var err error
			newBlock, err = initBlockFromProtoCapella(pb)
			if err != nil {
				return err
			}
		}
	case version.Deneb:
		if b.IsBlinded() {
			pb := &eth.BlindedBeaconBlockDeneb{}
			if err := pb.UnmarshalSSZ(buf); err != nil {
				return err
			}
			var err error
			newBlock, err = initBlindedBlockFromProtoDeneb(pb)
			if err != nil {
				return err
			}
		} else {
			pb := &eth.BeaconBlockDeneb{}
			if err := pb.UnmarshalSSZ(buf); err != nil {
				return err
			}
			var err error
			newBlock, err = initBlockFromProtoDeneb(pb)
			if err != nil {
				return err
			}
		}
	default:
		return errIncorrectBlockVersion
	}
	*b = *newBlock
	return nil
}

// AsSignRequestObject returns the underlying sign request object.
func (b *BeaconBlock) AsSignRequestObject() (validatorpb.SignRequestObject, error) {
	pb, err := b.Proto()
	if err != nil {
		return nil, err
	}
	switch b.version {
	case version.Phase0:
		return &validatorpb.SignRequest_Block{Block: pb.(*eth.BeaconBlock)}, nil
	case version.Altair:
		return &validatorpb.SignRequest_BlockAltair{BlockAltair: pb.(*eth.BeaconBlockAltair)}, nil
	case version.Bellatrix:
		if b.IsBlinded() {
			return &validatorpb.SignRequest_BlindedBlockBellatrix{BlindedBlockBellatrix: pb.(*eth.BlindedBeaconBlockBellatrix)}, nil
		}
		return &validatorpb.SignRequest_BlockBellatrix{BlockBellatrix: pb.(*eth.BeaconBlockBellatrix)}, nil
	case version.Capella:
		if b.IsBlinded() {
			return &validatorpb.SignRequest_BlindedBlockCapella{BlindedBlockCapella: pb.(*eth.BlindedBeaconBlockCapella)}, nil
		}
		return &validatorpb.SignRequest_BlockCapella{BlockCapella: pb.(*eth.BeaconBlockCapella)}, nil
	case version.Deneb:
		if b.IsBlinded() {
			return &validatorpb.SignRequest_BlindedBlockDeneb{BlindedBlockDeneb: pb.(*eth.BlindedBeaconBlockDeneb)}, nil
		}
		return &validatorpb.SignRequest_BlockDeneb{BlockDeneb: pb.(*eth.BeaconBlockDeneb)}, nil
	default:
		return nil, errIncorrectBlockVersion
	}
}

func (b *BeaconBlock) Copy() (interfaces.ReadOnlyBeaconBlock, error) {
	if b == nil {
		return nil, nil
	}

	pb, err := b.Proto()
	if err != nil {
		return nil, err
	}
	switch b.version {
	case version.Phase0:
		cp := eth.CopyBeaconBlock(pb.(*eth.BeaconBlock))
		return initBlockFromProtoPhase0(cp)
	case version.Altair:
		cp := eth.CopyBeaconBlockAltair(pb.(*eth.BeaconBlockAltair))
		return initBlockFromProtoAltair(cp)
	case version.Bellatrix:
		if b.IsBlinded() {
			cp := eth.CopyBlindedBeaconBlockBellatrix(pb.(*eth.BlindedBeaconBlockBellatrix))
			return initBlindedBlockFromProtoBellatrix(cp)
		}
		cp := eth.CopyBeaconBlockBellatrix(pb.(*eth.BeaconBlockBellatrix))
		return initBlockFromProtoBellatrix(cp)
	case version.Capella:
		if b.IsBlinded() {
			cp := eth.CopyBlindedBeaconBlockCapella(pb.(*eth.BlindedBeaconBlockCapella))
			return initBlindedBlockFromProtoCapella(cp)
		}
		cp := eth.CopyBeaconBlockCapella(pb.(*eth.BeaconBlockCapella))
		return initBlockFromProtoCapella(cp)
	case version.Deneb:
		if b.IsBlinded() {
			cp := eth.CopyBlindedBeaconBlockDeneb(pb.(*eth.BlindedBeaconBlockDeneb))
			return initBlindedBlockFromProtoDeneb(cp)
		}
		cp := eth.CopyBeaconBlockDeneb(pb.(*eth.BeaconBlockDeneb))
		return initBlockFromProtoDeneb(cp)
	default:
		return nil, errIncorrectBlockVersion
	}
}

// IsNil checks if the block body is nil.
func (b *BeaconBlockBody) IsNil() bool {
	return b == nil
}

// RandaoReveal returns the randao reveal from the block body.
func (b *BeaconBlockBody) RandaoReveal() [field_params.BLSSignatureLength]byte {
	return b.randaoReveal
}

// Eth1Data returns the eth1 data in the block.
func (b *BeaconBlockBody) Eth1Data() *eth.Eth1Data {
	return b.eth1Data
}

// Graffiti returns the graffiti in the block.
func (b *BeaconBlockBody) Graffiti() [field_params.RootLength]byte {
	return b.graffiti
}

// ProposerSlashings returns the proposer slashings in the block.
func (b *BeaconBlockBody) ProposerSlashings() []*eth.ProposerSlashing {
	return b.proposerSlashings
}

// AttesterSlashings returns the attester slashings in the block.
func (b *BeaconBlockBody) AttesterSlashings() []*eth.AttesterSlashing {
	return b.attesterSlashings
}

// Attestations returns the stored attestations in the block.
func (b *BeaconBlockBody) Attestations() []*eth.Attestation {
	return b.attestations
}

// Deposits returns the stored deposits in the block.
func (b *BeaconBlockBody) Deposits() []*eth.Deposit {
	return b.deposits
}

// VoluntaryExits returns the voluntary exits in the block.
func (b *BeaconBlockBody) VoluntaryExits() []*eth.SignedVoluntaryExit {
	return b.voluntaryExits
}

// SyncAggregate returns the sync aggregate in the block.
func (b *BeaconBlockBody) SyncAggregate() (*eth.SyncAggregate, error) {
	if b.version == version.Phase0 {
		return nil, consensus_types.ErrNotSupported("SyncAggregate", b.version)
	}
	return b.syncAggregate, nil
}

// Execution returns the execution payload of the block body.
func (b *BeaconBlockBody) Execution() (interfaces.ExecutionData, error) {
	switch b.version {
	case version.Phase0, version.Altair:
		return nil, consensus_types.ErrNotSupported("Execution", b.version)
	default:
		if b.IsBlinded() {
			return b.executionPayloadHeader, nil
		}
		return b.executionPayload, nil
	}
}

func (b *BeaconBlockBody) BLSToExecutionChanges() ([]*eth.SignedBLSToExecutionChange, error) {
	if b.version < version.Capella {
		return nil, consensus_types.ErrNotSupported("BLSToExecutionChanges", b.version)
	}
	return b.blsToExecutionChanges, nil
}

// BlobKzgCommitments returns the blob kzg commitments in the block.
func (b *BeaconBlockBody) BlobKzgCommitments() ([][]byte, error) {
	switch b.version {
	case version.Phase0, version.Altair, version.Bellatrix, version.Capella:
		return nil, consensus_types.ErrNotSupported("BlobKzgCommitments", b.version)
	case version.Deneb:
		return b.blobKzgCommitments, nil
	default:
		return nil, errIncorrectBlockVersion
	}
}

// Version returns the version of the beacon block body
func (b *BeaconBlockBody) Version() int {
	return b.version
}

// HashTreeRoot returns the ssz root of the block body.
func (b *BeaconBlockBody) HashTreeRoot() ([field_params.RootLength]byte, error) {
	pb, err := b.Proto()
	if err != nil {
		return [field_params.RootLength]byte{}, err
	}
	switch b.version {
	case version.Phase0:
		return pb.(*eth.BeaconBlockBody).HashTreeRoot()
	case version.Altair:
		return pb.(*eth.BeaconBlockBodyAltair).HashTreeRoot()
	case version.Bellatrix:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockBodyBellatrix).HashTreeRoot()
		}
		return pb.(*eth.BeaconBlockBodyBellatrix).HashTreeRoot()
	case version.Capella:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockBodyCapella).HashTreeRoot()
		}
		return pb.(*eth.BeaconBlockBodyCapella).HashTreeRoot()
	case version.Deneb:
		if b.IsBlinded() {
			return pb.(*eth.BlindedBeaconBlockBodyDeneb).HashTreeRoot()
		}
		return pb.(*eth.BeaconBlockBodyDeneb).HashTreeRoot()
	default:
		return [field_params.RootLength]byte{}, errIncorrectBodyVersion
	}
}

// IsBlinded checks if the beacon block body is a blinded block body.
func (b *BeaconBlockBody) IsBlinded() bool {
	return b.version >= version.Bellatrix && b.executionPayload == nil
}
