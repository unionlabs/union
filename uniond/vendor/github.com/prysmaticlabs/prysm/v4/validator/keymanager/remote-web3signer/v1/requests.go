package v1

import (
	"fmt"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	validatorpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/validator-client"
)

// GetBlockSignRequest maps the request for signing type BLOCK.
func GetBlockSignRequest(request *validatorpb.SignRequest, genesisValidatorsRoot []byte) (*BlockSignRequest, error) {
	beaconBlock, ok := request.Object.(*validatorpb.SignRequest_Block)
	if !ok {
		return nil, errors.New("failed to cast request object to block")
	}
	if beaconBlock == nil {
		return nil, errors.New("invalid sign request: ReadOnlyBeaconBlock is nil")
	}
	fork, err := MapForkInfo(request.SigningSlot, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	beaconBlockBody, err := MapBeaconBlockBody(beaconBlock.Block.Body)
	if err != nil {
		return nil, err
	}
	return &BlockSignRequest{
		Type:        "BLOCK",
		ForkInfo:    fork,
		SigningRoot: request.SigningRoot,
		Block: &BeaconBlock{
			Slot:          fmt.Sprint(beaconBlock.Block.Slot),
			ProposerIndex: fmt.Sprint(beaconBlock.Block.ProposerIndex),
			ParentRoot:    beaconBlock.Block.ParentRoot,
			StateRoot:     beaconBlock.Block.StateRoot,
			Body:          beaconBlockBody,
		},
	}, nil
}

// GetAggregationSlotSignRequest maps the request for signing type AGGREGATION_SLOT.
func GetAggregationSlotSignRequest(request *validatorpb.SignRequest, genesisValidatorsRoot []byte) (*AggregationSlotSignRequest, error) {
	aggregationSlot, ok := request.Object.(*validatorpb.SignRequest_Slot)
	if !ok {
		return nil, errors.New("failed to cast request object to aggregation slot")
	}
	if aggregationSlot == nil {
		return nil, errors.New("invalid sign request: Slot is nil")
	}
	fork, err := MapForkInfo(request.SigningSlot, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	return &AggregationSlotSignRequest{
		Type:        "AGGREGATION_SLOT",
		ForkInfo:    fork,
		SigningRoot: request.SigningRoot,
		AggregationSlot: &AggregationSlot{
			Slot: fmt.Sprint(aggregationSlot.Slot),
		},
	}, nil
}

// GetAggregateAndProofSignRequest maps the request for signing type AGGREGATE_AND_PROOF.
func GetAggregateAndProofSignRequest(request *validatorpb.SignRequest, genesisValidatorsRoot []byte) (*AggregateAndProofSignRequest, error) {
	aggregateAttestationAndProof, ok := request.Object.(*validatorpb.SignRequest_AggregateAttestationAndProof)
	if !ok {
		return nil, errors.New("failed to cast request object to aggregate attestation and proof")
	}
	if aggregateAttestationAndProof == nil {
		return nil, errors.New("invalid sign request: AggregateAndProof is nil")
	}
	fork, err := MapForkInfo(request.SigningSlot, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	aggregateAndProof, err := MapAggregateAndProof(aggregateAttestationAndProof.AggregateAttestationAndProof)
	if err != nil {
		return nil, err
	}
	return &AggregateAndProofSignRequest{
		Type:              "AGGREGATE_AND_PROOF",
		ForkInfo:          fork,
		SigningRoot:       request.SigningRoot,
		AggregateAndProof: aggregateAndProof,
	}, nil
}

// GetAttestationSignRequest maps the request for signing type ATTESTATION.
func GetAttestationSignRequest(request *validatorpb.SignRequest, genesisValidatorsRoot []byte) (*AttestationSignRequest, error) {
	attestation, ok := request.Object.(*validatorpb.SignRequest_AttestationData)
	if !ok {
		return nil, errors.New("failed to cast request object to attestation")
	}
	if attestation == nil {
		return nil, errors.New("invalid sign request: Attestation is nil")
	}
	fork, err := MapForkInfo(request.SigningSlot, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	attestationData, err := MapAttestationData(attestation.AttestationData)
	if err != nil {
		return nil, err
	}
	return &AttestationSignRequest{
		Type:        "ATTESTATION",
		ForkInfo:    fork,
		SigningRoot: request.SigningRoot,
		Attestation: attestationData,
	}, nil
}

// GetBlockAltairSignRequest maps the request for signing type BLOCK_V2.
func GetBlockAltairSignRequest(request *validatorpb.SignRequest, genesisValidatorsRoot []byte) (*BlockAltairSignRequest, error) {
	beaconBlockAltair, ok := request.Object.(*validatorpb.SignRequest_BlockAltair)
	if !ok {
		return nil, errors.New("failed to cast request object to block altair")
	}
	if beaconBlockAltair == nil {
		return nil, errors.New("invalid sign request: ReadOnlyBeaconBlock is nil")
	}
	fork, err := MapForkInfo(request.SigningSlot, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	blockAltair, err := MapBeaconBlockAltair(beaconBlockAltair.BlockAltair)
	if err != nil {
		return nil, err
	}
	return &BlockAltairSignRequest{
		Type:        "BLOCK_V2",
		ForkInfo:    fork,
		SigningRoot: request.SigningRoot,
		BeaconBlock: &BeaconBlockAltairBlockV2{
			Version: "ALTAIR",
			Block:   blockAltair,
		},
	}, nil
}

// GetRandaoRevealSignRequest maps the request for signing type RANDAO_REVEAL.
func GetRandaoRevealSignRequest(request *validatorpb.SignRequest, genesisValidatorsRoot []byte) (*RandaoRevealSignRequest, error) {
	randaoReveal, ok := request.Object.(*validatorpb.SignRequest_Epoch)
	if !ok {
		return nil, errors.New("failed to cast request object to randao reveal")
	}
	if randaoReveal == nil {
		return nil, errors.New("invalid sign request: Epoch is nil")
	}
	fork, err := MapForkInfo(request.SigningSlot, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	return &RandaoRevealSignRequest{
		Type:        "RANDAO_REVEAL",
		ForkInfo:    fork,
		SigningRoot: request.SigningRoot,
		RandaoReveal: &RandaoReveal{
			Epoch: fmt.Sprint(randaoReveal.Epoch),
		},
	}, nil
}

// GetVoluntaryExitSignRequest maps the request for signing type VOLUNTARY_EXIT.
func GetVoluntaryExitSignRequest(request *validatorpb.SignRequest, genesisValidatorsRoot []byte) (*VoluntaryExitSignRequest, error) {
	voluntaryExit, ok := request.Object.(*validatorpb.SignRequest_Exit)
	if !ok {
		return nil, errors.New("failed to cast request object to voluntary exit")
	}
	if voluntaryExit == nil || voluntaryExit.Exit == nil {
		return nil, errors.New("invalid sign request: Exit is nil")
	}
	fork, err := MapForkInfo(request.SigningSlot, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	return &VoluntaryExitSignRequest{
		Type:        "VOLUNTARY_EXIT",
		ForkInfo:    fork,
		SigningRoot: request.SigningRoot,
		VoluntaryExit: &VoluntaryExit{
			ValidatorIndex: fmt.Sprint(voluntaryExit.Exit.ValidatorIndex),
			Epoch:          fmt.Sprint(voluntaryExit.Exit.Epoch),
		},
	}, nil
}

// GetSyncCommitteeMessageSignRequest maps the request for signing type SYNC_COMMITTEE_MESSAGE.
func GetSyncCommitteeMessageSignRequest(request *validatorpb.SignRequest, genesisValidatorsRoot []byte) (*SyncCommitteeMessageSignRequest, error) {
	if request == nil {
		return nil, errors.New("nil sign request provided")
	}
	syncCommitteeMessage, ok := request.Object.(*validatorpb.SignRequest_SyncMessageBlockRoot)
	if !ok {
		return nil, errors.New("failed to cast request object to sync committee message")
	}
	if syncCommitteeMessage == nil || syncCommitteeMessage.SyncMessageBlockRoot == nil {
		return nil, errors.New("invalid sign request: SyncCommitteeMessage is nil")
	}
	fork, err := MapForkInfo(request.SigningSlot, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	return &SyncCommitteeMessageSignRequest{
		Type:        "SYNC_COMMITTEE_MESSAGE",
		ForkInfo:    fork,
		SigningRoot: request.SigningRoot,
		SyncCommitteeMessage: &SyncCommitteeMessage{
			BeaconBlockRoot: syncCommitteeMessage.SyncMessageBlockRoot,
			Slot:            fmt.Sprint(request.SigningSlot),
		},
	}, nil
}

// GetSyncCommitteeSelectionProofSignRequest maps the request for signing type SYNC_COMMITTEE_SELECTION_PROOF.
func GetSyncCommitteeSelectionProofSignRequest(request *validatorpb.SignRequest, genesisValidatorsRoot []byte) (*SyncCommitteeSelectionProofSignRequest, error) {
	if request == nil {
		return nil, errors.New("nil sign request provided")
	}
	syncCommitteeSelectionProof, ok := request.Object.(*validatorpb.SignRequest_SyncAggregatorSelectionData)
	if !ok {
		return nil, errors.New("failed to cast request object to sync committee selection proof")
	}
	if syncCommitteeSelectionProof == nil {
		return nil, errors.New("invalid sign request: SyncCommitteeSelectionProof is nil")
	}
	fork, err := MapForkInfo(request.SigningSlot, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	aggregatorSelectionData, err := MapSyncAggregatorSelectionData(syncCommitteeSelectionProof.SyncAggregatorSelectionData)
	if err != nil {
		return nil, err
	}
	return &SyncCommitteeSelectionProofSignRequest{
		Type:                        "SYNC_COMMITTEE_SELECTION_PROOF",
		ForkInfo:                    fork,
		SigningRoot:                 request.SigningRoot,
		SyncAggregatorSelectionData: aggregatorSelectionData,
	}, nil
}

// GetSyncCommitteeContributionAndProofSignRequest maps the request for signing type SYNC_COMMITTEE_CONTRIBUTION_AND_PROOF.
func GetSyncCommitteeContributionAndProofSignRequest(request *validatorpb.SignRequest, genesisValidatorsRoot []byte) (*SyncCommitteeContributionAndProofSignRequest, error) {
	if request == nil {
		return nil, errors.New("nil sign request provided")
	}
	syncCommitteeContributionAndProof, ok := request.Object.(*validatorpb.SignRequest_ContributionAndProof)
	if !ok {
		return nil, errors.New("failed to cast request object to sync committee contribution and proof")
	}
	if syncCommitteeContributionAndProof == nil {
		return nil, errors.New("invalid sign request: SyncCommitteeContributionAndProof is nil")
	}
	fork, err := MapForkInfo(request.SigningSlot, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	contribution, err := MapContributionAndProof(syncCommitteeContributionAndProof.ContributionAndProof)
	if err != nil {
		return nil, err
	}
	return &SyncCommitteeContributionAndProofSignRequest{
		Type:                 "SYNC_COMMITTEE_CONTRIBUTION_AND_PROOF",
		ForkInfo:             fork,
		SigningRoot:          request.SigningRoot,
		ContributionAndProof: contribution,
	}, nil
}

// GetBlockV2BlindedSignRequest maps the request for signing types (GetBlockV2 id defined by the remote signer interface and not the beacon APIs)
// Supports Bellatrix, Capella, Deneb
func GetBlockV2BlindedSignRequest(request *validatorpb.SignRequest, genesisValidatorsRoot []byte) (*BlockV2BlindedSignRequest, error) {
	if request == nil {
		return nil, errors.New("nil sign request provided")
	}
	var b interfaces.ReadOnlyBeaconBlock
	var version string
	switch request.Object.(type) {
	case *validatorpb.SignRequest_BlindedBlockBellatrix:
		version = "BELLATRIX"
		blindedBlockBellatrix, ok := request.Object.(*validatorpb.SignRequest_BlindedBlockBellatrix)
		if !ok {
			return nil, errors.New("failed to cast request object to blinded block bellatrix")
		}
		if blindedBlockBellatrix == nil {
			return nil, errors.New("invalid sign request - blinded bellatrix block is nil")
		}
		beaconBlock, err := blocks.NewBeaconBlock(blindedBlockBellatrix.BlindedBlockBellatrix)
		if err != nil {
			return nil, err
		}
		b = beaconBlock
	case *validatorpb.SignRequest_BlockBellatrix:
		version = "BELLATRIX"
		blockBellatrix, ok := request.Object.(*validatorpb.SignRequest_BlockBellatrix)
		if !ok {
			return nil, errors.New("failed to cast request object to bellatrix block")
		}

		if blockBellatrix == nil {
			return nil, errors.New("invalid sign request: bellatrix block is nil")
		}
		beaconBlock, err := blocks.NewBeaconBlock(blockBellatrix.BlockBellatrix)
		if err != nil {
			return nil, err
		}
		b = beaconBlock
	case *validatorpb.SignRequest_BlockCapella:
		version = "CAPELLA"
		blockCapella, ok := request.Object.(*validatorpb.SignRequest_BlockCapella)
		if !ok {
			return nil, errors.New("failed to cast request object to capella block")
		}
		if blockCapella == nil {
			return nil, errors.New("invalid sign request: capella block is nil")
		}
		beaconBlock, err := blocks.NewBeaconBlock(blockCapella.BlockCapella)
		if err != nil {
			return nil, err
		}
		b = beaconBlock
	case *validatorpb.SignRequest_BlindedBlockCapella:
		version = "CAPELLA"
		blindedBlockCapella, ok := request.Object.(*validatorpb.SignRequest_BlindedBlockCapella)
		if !ok {
			return nil, errors.New("failed to cast request object to blinded capella block")
		}
		if blindedBlockCapella == nil {
			return nil, errors.New("invalid sign request: blinded capella block is nil")
		}
		beaconBlock, err := blocks.NewBeaconBlock(blindedBlockCapella.BlindedBlockCapella)
		if err != nil {
			return nil, err
		}
		b = beaconBlock
	case *validatorpb.SignRequest_BlockDeneb:
		version = "DENEB"
		blockDeneb, ok := request.Object.(*validatorpb.SignRequest_BlockDeneb)
		if !ok {
			return nil, errors.New("failed to cast request object to deneb block")
		}
		if blockDeneb == nil {
			return nil, errors.New("invalid sign request: deneb block is nil")
		}
		beaconBlock, err := blocks.NewBeaconBlock(blockDeneb.BlockDeneb)
		if err != nil {
			return nil, err
		}
		b = beaconBlock
	case *validatorpb.SignRequest_BlindedBlockDeneb:
		version = "DENEB"
		blindedBlockDeneb, ok := request.Object.(*validatorpb.SignRequest_BlindedBlockDeneb)
		if !ok {
			return nil, errors.New("failed to cast request object to blinded deneb block")
		}
		if blindedBlockDeneb == nil {
			return nil, errors.New("invalid sign request: blinded deneb block is nil")
		}
		beaconBlock, err := blocks.NewBeaconBlock(blindedBlockDeneb.BlindedBlockDeneb)
		if err != nil {
			return nil, err
		}
		b = beaconBlock
	default:
		return nil, errors.New("invalid sign request - invalid object type")
	}
	fork, err := MapForkInfo(request.SigningSlot, genesisValidatorsRoot)
	if err != nil {
		return nil, err
	}
	beaconBlockHeader, err := interfaces.BeaconBlockHeaderFromBlockInterface(b)
	if err != nil {
		return nil, err
	}
	return &BlockV2BlindedSignRequest{
		Type:        "BLOCK_V2",
		ForkInfo:    fork,
		SigningRoot: request.SigningRoot,
		BeaconBlock: &BeaconBlockV2Blinded{
			Version: version,
			BlockHeader: &BeaconBlockHeader{
				Slot:          fmt.Sprint(beaconBlockHeader.Slot),
				ProposerIndex: fmt.Sprint(beaconBlockHeader.ProposerIndex),
				ParentRoot:    beaconBlockHeader.ParentRoot,
				StateRoot:     beaconBlockHeader.StateRoot,
				BodyRoot:      beaconBlockHeader.BodyRoot,
			},
		},
	}, nil
}

// GetValidatorRegistrationSignRequest maps the request for signing type VALIDATOR_REGISTRATION.
func GetValidatorRegistrationSignRequest(request *validatorpb.SignRequest) (*ValidatorRegistrationSignRequest, error) {
	if request == nil {
		return nil, errors.New("nil sign request provided")
	}
	validatorRegistrationRequest, ok := request.Object.(*validatorpb.SignRequest_Registration)
	if !ok {
		return nil, errors.New("failed to cast request object to validator registration")
	}
	registration := validatorRegistrationRequest.Registration
	return &ValidatorRegistrationSignRequest{
		Type:        "VALIDATOR_REGISTRATION",
		SigningRoot: request.SigningRoot,
		ValidatorRegistration: &ValidatorRegistration{
			FeeRecipient: registration.FeeRecipient,
			GasLimit:     fmt.Sprint(registration.GasLimit),
			Timestamp:    fmt.Sprint(registration.Timestamp),
			Pubkey:       registration.Pubkey,
		},
	}, nil
}
