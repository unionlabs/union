package client

import (
	"context"
	"encoding/hex"
	"testing"

	"github.com/golang/mock/gomock"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/go-bitfield"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	logTest "github.com/sirupsen/logrus/hooks/test"
	"google.golang.org/protobuf/types/known/emptypb"
)

func TestSubmitSyncCommitteeMessage_ValidatorDutiesRequestFailure(t *testing.T) {
	hook := logTest.NewGlobal()
	validator, m, validatorKey, finish := setup(t)
	validator.duties = &ethpb.DutiesResponse{CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{}}
	defer finish()

	m.validatorClient.EXPECT().GetSyncMessageBlockRoot(
		gomock.Any(), // ctx
		&emptypb.Empty{},
	).Return(&ethpb.SyncMessageBlockRootResponse{
		Root: bytesutil.PadTo([]byte{}, 32),
	}, nil)

	var pubKey [fieldparams.BLSPubkeyLength]byte
	copy(pubKey[:], validatorKey.PublicKey().Marshal())
	validator.SubmitSyncCommitteeMessage(context.Background(), 1, pubKey)
	require.LogsContain(t, hook, "Could not fetch validator assignment")
}

func TestSubmitSyncCommitteeMessage_BadDomainData(t *testing.T) {
	validator, m, validatorKey, finish := setup(t)
	defer finish()
	hook := logTest.NewGlobal()
	validatorIndex := primitives.ValidatorIndex(7)
	committee := []primitives.ValidatorIndex{0, 3, 4, 2, validatorIndex, 6, 8, 9, 10}
	validator.duties = &ethpb.DutiesResponse{CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
		{
			PublicKey:      validatorKey.PublicKey().Marshal(),
			Committee:      committee,
			ValidatorIndex: validatorIndex,
		},
	}}

	r := []byte{'a'}
	m.validatorClient.EXPECT().GetSyncMessageBlockRoot(
		gomock.Any(), // ctx
		&emptypb.Empty{},
	).Return(&ethpb.SyncMessageBlockRootResponse{
		Root: bytesutil.PadTo(r, 32),
	}, nil)

	m.validatorClient.EXPECT().
		DomainData(gomock.Any(), gomock.Any()).
		Return(nil, errors.New("uh oh"))

	var pubKey [fieldparams.BLSPubkeyLength]byte
	copy(pubKey[:], validatorKey.PublicKey().Marshal())
	validator.SubmitSyncCommitteeMessage(context.Background(), 1, pubKey)
	require.LogsContain(t, hook, "Could not get sync committee domain data")
}

func TestSubmitSyncCommitteeMessage_CouldNotSubmit(t *testing.T) {
	validator, m, validatorKey, finish := setup(t)
	defer finish()
	hook := logTest.NewGlobal()
	validatorIndex := primitives.ValidatorIndex(7)
	committee := []primitives.ValidatorIndex{0, 3, 4, 2, validatorIndex, 6, 8, 9, 10}
	validator.duties = &ethpb.DutiesResponse{CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
		{
			PublicKey:      validatorKey.PublicKey().Marshal(),
			Committee:      committee,
			ValidatorIndex: validatorIndex,
		},
	}}

	r := []byte{'a'}
	m.validatorClient.EXPECT().GetSyncMessageBlockRoot(
		gomock.Any(), // ctx
		&emptypb.Empty{},
	).Return(&ethpb.SyncMessageBlockRootResponse{
		Root: bytesutil.PadTo(r, 32),
	}, nil)

	m.validatorClient.EXPECT().
		DomainData(gomock.Any(), // ctx
			gomock.Any()). // epoch
		Return(&ethpb.DomainResponse{
			SignatureDomain: make([]byte, 32),
		}, nil)

	m.validatorClient.EXPECT().SubmitSyncMessage(
		gomock.Any(), // ctx
		gomock.AssignableToTypeOf(&ethpb.SyncCommitteeMessage{}),
	).Return(&emptypb.Empty{}, errors.New("uh oh") /* error */)

	var pubKey [fieldparams.BLSPubkeyLength]byte
	copy(pubKey[:], validatorKey.PublicKey().Marshal())
	validator.SubmitSyncCommitteeMessage(context.Background(), 1, pubKey)

	require.LogsContain(t, hook, "Could not submit sync committee message")
}

func TestSubmitSyncCommitteeMessage_OK(t *testing.T) {
	validator, m, validatorKey, finish := setup(t)
	defer finish()
	hook := logTest.NewGlobal()
	validatorIndex := primitives.ValidatorIndex(7)
	committee := []primitives.ValidatorIndex{0, 3, 4, 2, validatorIndex, 6, 8, 9, 10}
	validator.duties = &ethpb.DutiesResponse{CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
		{
			PublicKey:      validatorKey.PublicKey().Marshal(),
			Committee:      committee,
			ValidatorIndex: validatorIndex,
		},
	}}

	r := []byte{'a'}
	m.validatorClient.EXPECT().GetSyncMessageBlockRoot(
		gomock.Any(), // ctx
		&emptypb.Empty{},
	).Return(&ethpb.SyncMessageBlockRootResponse{
		Root: bytesutil.PadTo(r, 32),
	}, nil)

	m.validatorClient.EXPECT().
		DomainData(gomock.Any(), // ctx
			gomock.Any()). // epoch
		Return(&ethpb.DomainResponse{
			SignatureDomain: make([]byte, 32),
		}, nil)

	var generatedMsg *ethpb.SyncCommitteeMessage
	m.validatorClient.EXPECT().SubmitSyncMessage(
		gomock.Any(), // ctx
		gomock.AssignableToTypeOf(&ethpb.SyncCommitteeMessage{}),
	).Do(func(_ context.Context, msg *ethpb.SyncCommitteeMessage) {
		generatedMsg = msg
	}).Return(&emptypb.Empty{}, nil /* error */)

	var pubKey [fieldparams.BLSPubkeyLength]byte
	copy(pubKey[:], validatorKey.PublicKey().Marshal())
	validator.SubmitSyncCommitteeMessage(context.Background(), 1, pubKey)

	require.LogsDoNotContain(t, hook, "Could not")
	require.Equal(t, primitives.Slot(1), generatedMsg.Slot)
	require.Equal(t, validatorIndex, generatedMsg.ValidatorIndex)
	require.DeepEqual(t, bytesutil.PadTo(r, 32), generatedMsg.BlockRoot)
}

func TestSubmitSignedContributionAndProof_ValidatorDutiesRequestFailure(t *testing.T) {
	hook := logTest.NewGlobal()
	validator, _, validatorKey, finish := setup(t)
	validator.duties = &ethpb.DutiesResponse{CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{}}
	defer finish()

	var pubKey [fieldparams.BLSPubkeyLength]byte
	copy(pubKey[:], validatorKey.PublicKey().Marshal())
	validator.SubmitSignedContributionAndProof(context.Background(), 1, pubKey)
	require.LogsContain(t, hook, "Could not fetch validator assignment")
}

func TestSubmitSignedContributionAndProof_GetSyncSubcommitteeIndexFailure(t *testing.T) {
	hook := logTest.NewGlobal()
	validator, m, validatorKey, finish := setup(t)
	validatorIndex := primitives.ValidatorIndex(7)
	committee := []primitives.ValidatorIndex{0, 3, 4, 2, validatorIndex, 6, 8, 9, 10}
	validator.duties = &ethpb.DutiesResponse{CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
		{
			PublicKey:      validatorKey.PublicKey().Marshal(),
			Committee:      committee,
			ValidatorIndex: validatorIndex,
		},
	}}
	defer finish()

	var pubKey [fieldparams.BLSPubkeyLength]byte
	copy(pubKey[:], validatorKey.PublicKey().Marshal())
	m.validatorClient.EXPECT().GetSyncSubcommitteeIndex(
		gomock.Any(), // ctx
		&ethpb.SyncSubcommitteeIndexRequest{
			Slot:      1,
			PublicKey: pubKey[:],
		},
	).Return(&ethpb.SyncSubcommitteeIndexResponse{}, errors.New("Bad index"))

	validator.SubmitSignedContributionAndProof(context.Background(), 1, pubKey)
	require.LogsContain(t, hook, "Could not get sync subcommittee index")
}

func TestSubmitSignedContributionAndProof_NothingToDo(t *testing.T) {
	hook := logTest.NewGlobal()
	validator, m, validatorKey, finish := setup(t)
	validatorIndex := primitives.ValidatorIndex(7)
	committee := []primitives.ValidatorIndex{0, 3, 4, 2, validatorIndex, 6, 8, 9, 10}
	validator.duties = &ethpb.DutiesResponse{CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
		{
			PublicKey:      validatorKey.PublicKey().Marshal(),
			Committee:      committee,
			ValidatorIndex: validatorIndex,
		},
	}}
	defer finish()

	var pubKey [fieldparams.BLSPubkeyLength]byte
	copy(pubKey[:], validatorKey.PublicKey().Marshal())
	m.validatorClient.EXPECT().GetSyncSubcommitteeIndex(
		gomock.Any(), // ctx
		&ethpb.SyncSubcommitteeIndexRequest{
			Slot:      1,
			PublicKey: pubKey[:],
		},
	).Return(&ethpb.SyncSubcommitteeIndexResponse{Indices: []primitives.CommitteeIndex{}}, nil)

	validator.SubmitSignedContributionAndProof(context.Background(), 1, pubKey)
	require.LogsContain(t, hook, "Empty subcommittee index list, do nothing")
}

func TestSubmitSignedContributionAndProof_BadDomain(t *testing.T) {
	hook := logTest.NewGlobal()
	validator, m, validatorKey, finish := setup(t)
	validatorIndex := primitives.ValidatorIndex(7)
	committee := []primitives.ValidatorIndex{0, 3, 4, 2, validatorIndex, 6, 8, 9, 10}
	validator.duties = &ethpb.DutiesResponse{CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
		{
			PublicKey:      validatorKey.PublicKey().Marshal(),
			Committee:      committee,
			ValidatorIndex: validatorIndex,
		},
	}}
	defer finish()

	var pubKey [fieldparams.BLSPubkeyLength]byte
	copy(pubKey[:], validatorKey.PublicKey().Marshal())
	m.validatorClient.EXPECT().GetSyncSubcommitteeIndex(
		gomock.Any(), // ctx
		&ethpb.SyncSubcommitteeIndexRequest{
			Slot:      1,
			PublicKey: pubKey[:],
		},
	).Return(&ethpb.SyncSubcommitteeIndexResponse{Indices: []primitives.CommitteeIndex{1}}, nil)

	m.validatorClient.EXPECT().
		DomainData(gomock.Any(), // ctx
			gomock.Any()). // epoch
		Return(&ethpb.DomainResponse{
			SignatureDomain: make([]byte, 32),
		}, errors.New("bad domain response"))

	validator.SubmitSignedContributionAndProof(context.Background(), 1, pubKey)
	require.LogsContain(t, hook, "Could not get selection proofs")
	require.LogsContain(t, hook, "bad domain response")
}

func TestSubmitSignedContributionAndProof_CouldNotGetContribution(t *testing.T) {
	hook := logTest.NewGlobal()
	// Hardcode secret key in order to have a valid aggregator signature.
	rawKey, err := hex.DecodeString("659e875e1b062c03f2f2a57332974d475b97df6cfc581d322e79642d39aca8fd")
	assert.NoError(t, err)
	validatorKey, err := bls.SecretKeyFromBytes(rawKey)
	assert.NoError(t, err)

	validator, m, validatorKey, finish := setupWithKey(t, validatorKey)
	validatorIndex := primitives.ValidatorIndex(7)
	committee := []primitives.ValidatorIndex{0, 3, 4, 2, validatorIndex, 6, 8, 9, 10}
	validator.duties = &ethpb.DutiesResponse{CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
		{
			PublicKey:      validatorKey.PublicKey().Marshal(),
			Committee:      committee,
			ValidatorIndex: validatorIndex,
		},
	}}
	defer finish()

	var pubKey [fieldparams.BLSPubkeyLength]byte
	copy(pubKey[:], validatorKey.PublicKey().Marshal())
	m.validatorClient.EXPECT().GetSyncSubcommitteeIndex(
		gomock.Any(), // ctx
		&ethpb.SyncSubcommitteeIndexRequest{
			Slot:      1,
			PublicKey: pubKey[:],
		},
	).Return(&ethpb.SyncSubcommitteeIndexResponse{Indices: []primitives.CommitteeIndex{1}}, nil)

	m.validatorClient.EXPECT().
		DomainData(gomock.Any(), // ctx
			gomock.Any()). // epoch
		Return(&ethpb.DomainResponse{
			SignatureDomain: make([]byte, 32),
		}, nil)

	m.validatorClient.EXPECT().GetSyncCommitteeContribution(
		gomock.Any(), // ctx
		&ethpb.SyncCommitteeContributionRequest{
			Slot:      1,
			PublicKey: pubKey[:],
			SubnetId:  0,
		},
	).Return(nil, errors.New("Bad contribution"))

	validator.SubmitSignedContributionAndProof(context.Background(), 1, pubKey)
	require.LogsContain(t, hook, "Could not get sync committee contribution")
}

func TestSubmitSignedContributionAndProof_CouldNotSubmitContribution(t *testing.T) {
	hook := logTest.NewGlobal()
	// Hardcode secret key in order to have a valid aggregator signature.
	rawKey, err := hex.DecodeString("659e875e1b062c03f2f2a57332974d475b97df6cfc581d322e79642d39aca8fd")
	assert.NoError(t, err)
	validatorKey, err := bls.SecretKeyFromBytes(rawKey)
	assert.NoError(t, err)

	validator, m, validatorKey, finish := setupWithKey(t, validatorKey)
	validatorIndex := primitives.ValidatorIndex(7)
	committee := []primitives.ValidatorIndex{0, 3, 4, 2, validatorIndex, 6, 8, 9, 10}
	validator.duties = &ethpb.DutiesResponse{CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
		{
			PublicKey:      validatorKey.PublicKey().Marshal(),
			Committee:      committee,
			ValidatorIndex: validatorIndex,
		},
	}}
	defer finish()

	var pubKey [fieldparams.BLSPubkeyLength]byte
	copy(pubKey[:], validatorKey.PublicKey().Marshal())
	m.validatorClient.EXPECT().GetSyncSubcommitteeIndex(
		gomock.Any(), // ctx
		&ethpb.SyncSubcommitteeIndexRequest{
			Slot:      1,
			PublicKey: pubKey[:],
		},
	).Return(&ethpb.SyncSubcommitteeIndexResponse{Indices: []primitives.CommitteeIndex{1}}, nil)

	m.validatorClient.EXPECT().
		DomainData(gomock.Any(), // ctx
			gomock.Any()). // epoch
		Return(&ethpb.DomainResponse{
			SignatureDomain: make([]byte, 32),
		}, nil)

	aggBits := bitfield.NewBitvector128()
	aggBits.SetBitAt(0, true)
	m.validatorClient.EXPECT().GetSyncCommitteeContribution(
		gomock.Any(), // ctx
		&ethpb.SyncCommitteeContributionRequest{
			Slot:      1,
			PublicKey: pubKey[:],
			SubnetId:  0,
		},
	).Return(&ethpb.SyncCommitteeContribution{
		BlockRoot:       make([]byte, fieldparams.RootLength),
		Signature:       make([]byte, 96),
		AggregationBits: aggBits,
	}, nil)

	m.validatorClient.EXPECT().
		DomainData(gomock.Any(), // ctx
			gomock.Any()). // epoch
		Return(&ethpb.DomainResponse{
			SignatureDomain: make([]byte, 32),
		}, nil)

	m.validatorClient.EXPECT().SubmitSignedContributionAndProof(
		gomock.Any(), // ctx
		gomock.AssignableToTypeOf(&ethpb.SignedContributionAndProof{
			Message: &ethpb.ContributionAndProof{
				AggregatorIndex: 7,
				Contribution: &ethpb.SyncCommitteeContribution{
					BlockRoot:         make([]byte, fieldparams.RootLength),
					Signature:         make([]byte, 96),
					AggregationBits:   bitfield.NewBitvector128(),
					Slot:              1,
					SubcommitteeIndex: 1,
				},
			},
		}),
	).Return(&emptypb.Empty{}, errors.New("Could not submit contribution"))

	validator.SubmitSignedContributionAndProof(context.Background(), 1, pubKey)
	require.LogsContain(t, hook, "Could not submit signed contribution and proof")
}

func TestSubmitSignedContributionAndProof_Ok(t *testing.T) {
	// Hardcode secret key in order to have a valid aggregator signature.
	rawKey, err := hex.DecodeString("659e875e1b062c03f2f2a57332974d475b97df6cfc581d322e79642d39aca8fd")
	assert.NoError(t, err)
	validatorKey, err := bls.SecretKeyFromBytes(rawKey)
	assert.NoError(t, err)

	validator, m, validatorKey, finish := setupWithKey(t, validatorKey)
	validatorIndex := primitives.ValidatorIndex(7)
	committee := []primitives.ValidatorIndex{0, 3, 4, 2, validatorIndex, 6, 8, 9, 10}
	validator.duties = &ethpb.DutiesResponse{CurrentEpochDuties: []*ethpb.DutiesResponse_Duty{
		{
			PublicKey:      validatorKey.PublicKey().Marshal(),
			Committee:      committee,
			ValidatorIndex: validatorIndex,
		},
	}}
	defer finish()

	var pubKey [fieldparams.BLSPubkeyLength]byte
	copy(pubKey[:], validatorKey.PublicKey().Marshal())
	m.validatorClient.EXPECT().GetSyncSubcommitteeIndex(
		gomock.Any(), // ctx
		&ethpb.SyncSubcommitteeIndexRequest{
			Slot:      1,
			PublicKey: pubKey[:],
		},
	).Return(&ethpb.SyncSubcommitteeIndexResponse{Indices: []primitives.CommitteeIndex{1}}, nil)

	m.validatorClient.EXPECT().
		DomainData(gomock.Any(), // ctx
			gomock.Any()). // epoch
		Return(&ethpb.DomainResponse{
			SignatureDomain: make([]byte, 32),
		}, nil)

	aggBits := bitfield.NewBitvector128()
	aggBits.SetBitAt(0, true)
	m.validatorClient.EXPECT().GetSyncCommitteeContribution(
		gomock.Any(), // ctx
		&ethpb.SyncCommitteeContributionRequest{
			Slot:      1,
			PublicKey: pubKey[:],
			SubnetId:  0,
		},
	).Return(&ethpb.SyncCommitteeContribution{
		BlockRoot:       make([]byte, fieldparams.RootLength),
		Signature:       make([]byte, 96),
		AggregationBits: aggBits,
	}, nil)

	m.validatorClient.EXPECT().
		DomainData(gomock.Any(), // ctx
			gomock.Any()). // epoch
		Return(&ethpb.DomainResponse{
			SignatureDomain: make([]byte, 32),
		}, nil)

	m.validatorClient.EXPECT().SubmitSignedContributionAndProof(
		gomock.Any(), // ctx
		gomock.AssignableToTypeOf(&ethpb.SignedContributionAndProof{
			Message: &ethpb.ContributionAndProof{
				AggregatorIndex: 7,
				Contribution: &ethpb.SyncCommitteeContribution{
					BlockRoot:         make([]byte, 32),
					Signature:         make([]byte, 96),
					AggregationBits:   bitfield.NewBitvector128(),
					Slot:              1,
					SubcommitteeIndex: 1,
				},
			},
		}),
	).Return(&emptypb.Empty{}, nil)

	validator.SubmitSignedContributionAndProof(context.Background(), 1, pubKey)
}
