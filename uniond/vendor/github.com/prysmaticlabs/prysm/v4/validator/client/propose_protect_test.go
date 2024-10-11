package client

import (
	"context"
	"testing"

	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func Test_slashableProposalCheck_PreventsLowerThanMinProposal(t *testing.T) {
	ctx := context.Background()
	validator, _, validatorKey, finish := setup(t)
	defer finish()
	lowestSignedSlot := primitives.Slot(10)
	var pubKeyBytes [fieldparams.BLSPubkeyLength]byte
	copy(pubKeyBytes[:], validatorKey.PublicKey().Marshal())

	// We save a proposal at the lowest signed slot in the DB.
	err := validator.db.SaveProposalHistoryForSlot(ctx, pubKeyBytes, lowestSignedSlot, []byte{1})
	require.NoError(t, err)
	require.NoError(t, err)

	// We expect the same block with a slot lower than the lowest
	// signed slot to fail validation.
	blk := &ethpb.SignedBeaconBlock{
		Block: &ethpb.BeaconBlock{
			Slot:          lowestSignedSlot - 1,
			ProposerIndex: 0,
			Body:          &ethpb.BeaconBlockBody{},
		},
		Signature: params.BeaconConfig().EmptySignature[:],
	}
	wsb, err := blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	err = validator.slashableProposalCheck(context.Background(), pubKeyBytes, wsb, [32]byte{4})
	require.ErrorContains(t, "could not sign block with slot < lowest signed", err)

	// We expect the same block with a slot equal to the lowest
	// signed slot to pass validation if signing roots are equal.
	blk = &ethpb.SignedBeaconBlock{
		Block: &ethpb.BeaconBlock{
			Slot:          lowestSignedSlot,
			ProposerIndex: 0,
			Body:          &ethpb.BeaconBlockBody{},
		},
		Signature: params.BeaconConfig().EmptySignature[:],
	}
	wsb, err = blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	err = validator.slashableProposalCheck(context.Background(), pubKeyBytes, wsb, [32]byte{1})
	require.NoError(t, err)

	// We expect the same block with a slot equal to the lowest
	// signed slot to fail validation if signing roots are different.
	wsb, err = blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	err = validator.slashableProposalCheck(context.Background(), pubKeyBytes, wsb, [32]byte{4})
	require.ErrorContains(t, "could not sign block with slot == lowest signed", err)

	// We expect the same block with a slot > than the lowest
	// signed slot to pass validation.
	blk = &ethpb.SignedBeaconBlock{
		Block: &ethpb.BeaconBlock{
			Slot:          lowestSignedSlot + 1,
			ProposerIndex: 0,
			Body:          &ethpb.BeaconBlockBody{},
		},
		Signature: params.BeaconConfig().EmptySignature[:],
	}

	wsb, err = blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	err = validator.slashableProposalCheck(context.Background(), pubKeyBytes, wsb, [32]byte{3})
	require.NoError(t, err)
}

func Test_slashableProposalCheck(t *testing.T) {
	ctx := context.Background()
	validator, _, validatorKey, finish := setup(t)
	defer finish()

	blk := util.HydrateSignedBeaconBlock(&ethpb.SignedBeaconBlock{
		Block: &ethpb.BeaconBlock{
			Slot:          10,
			ProposerIndex: 0,
			Body:          &ethpb.BeaconBlockBody{},
		},
		Signature: params.BeaconConfig().EmptySignature[:],
	})

	var pubKeyBytes [fieldparams.BLSPubkeyLength]byte
	copy(pubKeyBytes[:], validatorKey.PublicKey().Marshal())

	// We save a proposal at slot 1 as our lowest proposal.
	err := validator.db.SaveProposalHistoryForSlot(ctx, pubKeyBytes, 1, []byte{1})
	require.NoError(t, err)

	// We save a proposal at slot 10 with a dummy signing root.
	dummySigningRoot := [32]byte{1}
	err = validator.db.SaveProposalHistoryForSlot(ctx, pubKeyBytes, 10, dummySigningRoot[:])
	require.NoError(t, err)
	var pubKey [fieldparams.BLSPubkeyLength]byte
	copy(pubKey[:], validatorKey.PublicKey().Marshal())
	sBlock, err := blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)

	// We expect the same block sent out with the same root should not be slasahble.
	err = validator.slashableProposalCheck(context.Background(), pubKey, sBlock, dummySigningRoot)
	require.NoError(t, err)

	// We expect the same block sent out with a different signing root should be slasahble.
	err = validator.slashableProposalCheck(context.Background(), pubKey, sBlock, [32]byte{2})
	require.ErrorContains(t, failedBlockSignLocalErr, err)

	// We save a proposal at slot 11 with a nil signing root.
	blk.Block.Slot = 11
	sBlock, err = blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	err = validator.db.SaveProposalHistoryForSlot(ctx, pubKeyBytes, blk.Block.Slot, nil)
	require.NoError(t, err)

	// We expect the same block sent out should return slashable error even
	// if we had a nil signing root stored in the database.
	err = validator.slashableProposalCheck(context.Background(), pubKey, sBlock, [32]byte{2})
	require.ErrorContains(t, failedBlockSignLocalErr, err)

	// A block with a different slot for which we do not have a proposing history
	// should not be failing validation.
	blk.Block.Slot = 9
	sBlock, err = blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)
	err = validator.slashableProposalCheck(context.Background(), pubKey, sBlock, [32]byte{3})
	require.NoError(t, err, "Expected allowed block not to throw error")
}

func Test_slashableProposalCheck_RemoteProtection(t *testing.T) {
	validator, _, validatorKey, finish := setup(t)
	defer finish()
	var pubKey [fieldparams.BLSPubkeyLength]byte
	copy(pubKey[:], validatorKey.PublicKey().Marshal())

	blk := util.NewBeaconBlock()
	blk.Block.Slot = 10
	sBlock, err := blocks.NewSignedBeaconBlock(blk)
	require.NoError(t, err)

	err = validator.slashableProposalCheck(context.Background(), pubKey, sBlock, [32]byte{2})
	require.NoError(t, err, "Expected allowed block not to throw error")
}
