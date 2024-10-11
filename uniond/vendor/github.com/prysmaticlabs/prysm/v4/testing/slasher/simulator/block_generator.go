package simulator

import (
	"context"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

func (s *Simulator) generateBlockHeadersForSlot(
	ctx context.Context, slot primitives.Slot,
) ([]*ethpb.SignedBeaconBlockHeader, []*ethpb.ProposerSlashing, error) {
	blocks := make([]*ethpb.SignedBeaconBlockHeader, 0)
	slashings := make([]*ethpb.ProposerSlashing, 0)
	proposer := rand.NewGenerator().Uint64() % s.srvConfig.Params.NumValidators

	var parentRoot [32]byte
	beaconState, err := s.srvConfig.StateGen.StateByRoot(ctx, parentRoot)
	if err != nil {
		return nil, nil, err
	}
	block := &ethpb.SignedBeaconBlockHeader{
		Header: &ethpb.BeaconBlockHeader{
			Slot:          slot,
			ProposerIndex: primitives.ValidatorIndex(proposer),
			ParentRoot:    bytesutil.PadTo([]byte{}, 32),
			StateRoot:     bytesutil.PadTo([]byte{}, 32),
			BodyRoot:      bytesutil.PadTo([]byte("good block"), 32),
		},
	}
	sig, err := s.signBlockHeader(beaconState, block)
	if err != nil {
		return nil, nil, err
	}
	block.Signature = sig.Marshal()

	blocks = append(blocks, block)
	if rand.NewGenerator().Float64() < s.srvConfig.Params.ProposerSlashingProbab {
		log.WithField("proposerIndex", proposer).Infof("Slashable block made")
		slashableBlock := &ethpb.SignedBeaconBlockHeader{
			Header: &ethpb.BeaconBlockHeader{
				Slot:          slot,
				ProposerIndex: primitives.ValidatorIndex(proposer),
				ParentRoot:    bytesutil.PadTo([]byte{}, 32),
				StateRoot:     bytesutil.PadTo([]byte{}, 32),
				BodyRoot:      bytesutil.PadTo([]byte("bad block"), 32),
			},
			Signature: sig.Marshal(),
		}
		sig, err = s.signBlockHeader(beaconState, slashableBlock)
		if err != nil {
			return nil, nil, err
		}
		slashableBlock.Signature = sig.Marshal()

		blocks = append(blocks, slashableBlock)
		slashings = append(slashings, &ethpb.ProposerSlashing{
			Header_1: block,
			Header_2: slashableBlock,
		})
	}
	return blocks, slashings, nil
}

func (s *Simulator) signBlockHeader(
	beaconState state.BeaconState,
	header *ethpb.SignedBeaconBlockHeader,
) (bls.Signature, error) {
	domain, err := signing.Domain(
		beaconState.Fork(),
		0,
		params.BeaconConfig().DomainBeaconProposer,
		beaconState.GenesisValidatorsRoot(),
	)
	if err != nil {
		return nil, err
	}
	htr, err := header.Header.HashTreeRoot()
	if err != nil {
		return nil, err
	}
	container := &ethpb.SigningData{
		ObjectRoot: htr[:],
		Domain:     domain,
	}
	signingRoot, err := container.HashTreeRoot()
	if err != nil {
		return nil, err
	}
	validatorPrivKey := s.srvConfig.PrivateKeysByValidatorIndex[header.Header.ProposerIndex]
	return validatorPrivKey.Sign(signingRoot[:]), nil
}
