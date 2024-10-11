package slasher

import (
	"context"
	"testing"

	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	dbtest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	doublylinkedtree "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/doubly-linked-tree"
	slashingsmock "github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/slashings/mock"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestService_processAttesterSlashings(t *testing.T) {
	ctx := context.Background()
	slasherDB := dbtest.SetupSlasherDB(t)
	beaconDB := dbtest.SetupDB(t)

	beaconState, err := util.NewBeaconState()
	require.NoError(t, err)

	privKey, err := bls.RandKey()
	require.NoError(t, err)
	validators := make([]*ethpb.Validator, 1)
	validators[0] = &ethpb.Validator{
		PublicKey:             privKey.PublicKey().Marshal(),
		WithdrawalCredentials: make([]byte, 32),
		EffectiveBalance:      params.BeaconConfig().MaxEffectiveBalance,
	}
	err = beaconState.SetValidators(validators)
	require.NoError(t, err)

	mockChain := &mock.ChainService{
		State: beaconState,
	}
	s := &Service{
		serviceCfg: &ServiceConfig{
			Database:                slasherDB,
			AttestationStateFetcher: mockChain,
			StateGen:                stategen.New(beaconDB, doublylinkedtree.New()),
			SlashingPoolInserter:    &slashingsmock.PoolMock{},
			HeadStateFetcher:        mockChain,
		},
	}

	firstAtt := util.HydrateIndexedAttestation(&ethpb.IndexedAttestation{
		AttestingIndices: []uint64{0},
	})
	secondAtt := util.HydrateIndexedAttestation(&ethpb.IndexedAttestation{
		AttestingIndices: []uint64{0},
	})

	domain, err := signing.Domain(
		beaconState.Fork(),
		0,
		params.BeaconConfig().DomainBeaconAttester,
		beaconState.GenesisValidatorsRoot(),
	)
	require.NoError(t, err)
	signingRoot, err := signing.ComputeSigningRoot(firstAtt.Data, domain)
	require.NoError(t, err)

	t.Run("first_att_valid_sig_second_invalid", func(tt *testing.T) {
		hook := logTest.NewGlobal()
		// Use valid signature for the first att, but bad one for the second.
		signature := privKey.Sign(signingRoot[:])
		firstAtt.Signature = signature.Marshal()
		secondAtt.Signature = make([]byte, 96)

		slashings := []*ethpb.AttesterSlashing{
			{
				Attestation_1: firstAtt,
				Attestation_2: secondAtt,
			},
		}

		err = s.processAttesterSlashings(ctx, slashings)
		require.NoError(tt, err)
		require.LogsContain(tt, hook, "Invalid signature")
	})

	t.Run("first_att_invalid_sig_second_valid", func(tt *testing.T) {
		hook := logTest.NewGlobal()
		// Use invalid signature for the first att, but valid for the second.
		signature := privKey.Sign(signingRoot[:])
		firstAtt.Signature = make([]byte, 96)
		secondAtt.Signature = signature.Marshal()

		slashings := []*ethpb.AttesterSlashing{
			{
				Attestation_1: firstAtt,
				Attestation_2: secondAtt,
			},
		}

		err = s.processAttesterSlashings(ctx, slashings)
		require.NoError(tt, err)
		require.LogsContain(tt, hook, "Invalid signature")
	})

	t.Run("both_valid_att_signatures", func(tt *testing.T) {
		hook := logTest.NewGlobal()
		// Use valid signatures.
		signature := privKey.Sign(signingRoot[:])
		firstAtt.Signature = signature.Marshal()
		secondAtt.Signature = signature.Marshal()

		slashings := []*ethpb.AttesterSlashing{
			{
				Attestation_1: firstAtt,
				Attestation_2: secondAtt,
			},
		}

		err = s.processAttesterSlashings(ctx, slashings)
		require.NoError(tt, err)
		require.LogsDoNotContain(tt, hook, "Invalid signature")
	})
}

func TestService_processProposerSlashings(t *testing.T) {
	ctx := context.Background()
	slasherDB := dbtest.SetupSlasherDB(t)
	beaconDB := dbtest.SetupDB(t)

	beaconState, err := util.NewBeaconState()
	require.NoError(t, err)

	privKey, err := bls.RandKey()
	require.NoError(t, err)
	validators := make([]*ethpb.Validator, 1)
	validators[0] = &ethpb.Validator{
		PublicKey:             privKey.PublicKey().Marshal(),
		WithdrawalCredentials: make([]byte, 32),
		EffectiveBalance:      params.BeaconConfig().MaxEffectiveBalance,
	}
	err = beaconState.SetValidators(validators)
	require.NoError(t, err)

	mockChain := &mock.ChainService{
		State: beaconState,
	}
	s := &Service{
		serviceCfg: &ServiceConfig{
			Database:                slasherDB,
			AttestationStateFetcher: mockChain,
			StateGen:                stategen.New(beaconDB, doublylinkedtree.New()),
			SlashingPoolInserter:    &slashingsmock.PoolMock{},
			HeadStateFetcher:        mockChain,
		},
	}

	parentRoot := bytesutil.ToBytes32([]byte("parent"))
	err = s.serviceCfg.StateGen.SaveState(ctx, parentRoot, beaconState)
	require.NoError(t, err)

	firstBlockHeader := util.HydrateSignedBeaconHeader(&ethpb.SignedBeaconBlockHeader{
		Header: &ethpb.BeaconBlockHeader{
			Slot:          0,
			ProposerIndex: 0,
			ParentRoot:    parentRoot[:],
		},
	})
	secondBlockHeader := util.HydrateSignedBeaconHeader(&ethpb.SignedBeaconBlockHeader{
		Header: &ethpb.BeaconBlockHeader{
			Slot:          0,
			ProposerIndex: 0,
			ParentRoot:    parentRoot[:],
		},
	})

	domain, err := signing.Domain(
		beaconState.Fork(),
		0,
		params.BeaconConfig().DomainBeaconProposer,
		beaconState.GenesisValidatorsRoot(),
	)
	require.NoError(t, err)
	htr, err := firstBlockHeader.Header.HashTreeRoot()
	require.NoError(t, err)
	container := &ethpb.SigningData{
		ObjectRoot: htr[:],
		Domain:     domain,
	}
	require.NoError(t, err)
	signingRoot, err := container.HashTreeRoot()
	require.NoError(t, err)

	t.Run("first_header_valid_sig_second_invalid", func(tt *testing.T) {
		hook := logTest.NewGlobal()
		// Use valid signature for the first header, but bad one for the second.
		signature := privKey.Sign(signingRoot[:])
		firstBlockHeader.Signature = signature.Marshal()
		secondBlockHeader.Signature = make([]byte, 96)

		slashings := []*ethpb.ProposerSlashing{
			{
				Header_1: firstBlockHeader,
				Header_2: secondBlockHeader,
			},
		}

		err = s.processProposerSlashings(ctx, slashings)
		require.NoError(tt, err)
		require.LogsContain(tt, hook, "Invalid signature")
	})

	t.Run("first_header_invalid_sig_second_valid", func(tt *testing.T) {
		hook := logTest.NewGlobal()
		// Use invalid signature for the first header, but valid for the second.
		signature := privKey.Sign(signingRoot[:])
		firstBlockHeader.Signature = make([]byte, 96)
		secondBlockHeader.Signature = signature.Marshal()

		slashings := []*ethpb.ProposerSlashing{
			{
				Header_1: firstBlockHeader,
				Header_2: secondBlockHeader,
			},
		}

		err = s.processProposerSlashings(ctx, slashings)
		require.NoError(tt, err)
		require.LogsContain(tt, hook, "Invalid signature")
	})

	t.Run("both_valid_header_signatures", func(tt *testing.T) {
		hook := logTest.NewGlobal()
		// Use valid signatures.
		signature := privKey.Sign(signingRoot[:])
		firstBlockHeader.Signature = signature.Marshal()
		secondBlockHeader.Signature = signature.Marshal()

		slashings := []*ethpb.ProposerSlashing{
			{
				Header_1: firstBlockHeader,
				Header_2: secondBlockHeader,
			},
		}

		err = s.processProposerSlashings(ctx, slashings)
		require.NoError(tt, err)
		require.LogsDoNotContain(tt, hook, "Invalid signature")
	})
}
