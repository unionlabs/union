package util

import (
	"context"
	"encoding/binary"
	"testing"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/rand"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// RandaoReveal returns a signature of the requested epoch using the beacon proposer private key.
func RandaoReveal(beaconState state.ReadOnlyBeaconState, epoch primitives.Epoch, privKeys []bls.SecretKey) ([]byte, error) {
	// We fetch the proposer's index as that is whom the RANDAO will be verified against.
	proposerIdx, err := helpers.BeaconProposerIndex(context.Background(), beaconState)
	if err != nil {
		return []byte{}, errors.Wrap(err, "could not get beacon proposer index")
	}
	buf := make([]byte, 32)
	binary.LittleEndian.PutUint64(buf, uint64(epoch))

	// We make the previous validator's index sign the message instead of the proposer.
	sszEpoch := primitives.SSZUint64(epoch)
	return signing.ComputeDomainAndSign(beaconState, epoch, &sszEpoch, params.BeaconConfig().DomainRandao, privKeys[proposerIdx])
}

// BlockSignature calculates the post-state root of the block and returns the signature.
func BlockSignature(
	bState state.BeaconState,
	block interface{},
	privKeys []bls.SecretKey,
) (bls.Signature, error) {
	var wsb interfaces.ReadOnlySignedBeaconBlock
	var err error
	// copy the state since we need to process slots
	bState = bState.Copy()
	switch b := block.(type) {
	case *ethpb.BeaconBlock:
		wsb, err = blocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlock{Block: b})
	case *ethpb.BeaconBlockAltair:
		wsb, err = blocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlockAltair{Block: b})
	case *ethpb.BeaconBlockBellatrix:
		wsb, err = blocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlockBellatrix{Block: b})
	case *ethpb.BeaconBlockCapella:
		wsb, err = blocks.NewSignedBeaconBlock(&ethpb.SignedBeaconBlockCapella{Block: b})
	default:
		return nil, errors.New("unsupported block type")
	}
	if err != nil {
		return nil, errors.Wrap(err, "could not wrap block")
	}
	s, err := transition.CalculateStateRoot(context.Background(), bState, wsb)
	if err != nil {
		return nil, errors.Wrap(err, "could not calculate state root")
	}

	switch b := block.(type) {
	case *ethpb.BeaconBlock:
		b.StateRoot = s[:]
	case *ethpb.BeaconBlockAltair:
		b.StateRoot = s[:]
	case *ethpb.BeaconBlockBellatrix:
		b.StateRoot = s[:]
	case *ethpb.BeaconBlockCapella:
		b.StateRoot = s[:]
	}

	// Temporarily increasing the beacon state slot here since BeaconProposerIndex is a
	// function deterministic on beacon state slot.
	var blockSlot primitives.Slot
	switch b := block.(type) {
	case *ethpb.BeaconBlock:
		blockSlot = b.Slot
	case *ethpb.BeaconBlockAltair:
		blockSlot = b.Slot
	case *ethpb.BeaconBlockBellatrix:
		blockSlot = b.Slot
	case *ethpb.BeaconBlockCapella:
		blockSlot = b.Slot
	}

	// process slots to get the right fork
	bState, err = transition.ProcessSlots(context.Background(), bState, blockSlot)
	if err != nil {
		return nil, err
	}

	domain, err := signing.Domain(bState.Fork(), time.CurrentEpoch(bState), params.BeaconConfig().DomainBeaconProposer, bState.GenesisValidatorsRoot())
	if err != nil {
		return nil, err
	}

	var blockRoot [32]byte
	switch b := block.(type) {
	case *ethpb.BeaconBlock:
		blockRoot, err = signing.ComputeSigningRoot(b, domain)
	case *ethpb.BeaconBlockAltair:
		blockRoot, err = signing.ComputeSigningRoot(b, domain)
	case *ethpb.BeaconBlockBellatrix:
		blockRoot, err = signing.ComputeSigningRoot(b, domain)
	case *ethpb.BeaconBlockCapella:
		blockRoot, err = signing.ComputeSigningRoot(b, domain)
	}
	if err != nil {
		return nil, err
	}

	proposerIdx, err := helpers.BeaconProposerIndex(context.Background(), bState)
	if err != nil {
		return nil, err
	}
	return privKeys[proposerIdx].Sign(blockRoot[:]), nil
}

// Random32Bytes generates a random 32 byte slice.
func Random32Bytes(t *testing.T) []byte {
	b := make([]byte, 32)
	_, err := rand.NewDeterministicGenerator().Read(b)
	if err != nil {
		t.Fatal(err)
	}
	return b
}
