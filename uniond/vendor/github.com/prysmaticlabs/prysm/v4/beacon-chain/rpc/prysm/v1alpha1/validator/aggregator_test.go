package validator

import (
	"context"
	"reflect"
	"testing"
	"time"

	"github.com/prysmaticlabs/go-bitfield"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/attestations"
	mockp2p "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/core"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	mockSync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync/initial-sync/testing"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation"
	attaggregation "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation/aggregation/attestations"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestSubmitAggregateAndProof_Syncing(t *testing.T) {
	ctx := context.Background()

	s, err := state_native.InitializeFromProtoUnsafePhase0(&ethpb.BeaconState{})
	require.NoError(t, err)

	aggregatorServer := &Server{
		HeadFetcher: &mock.ChainService{State: s},
		SyncChecker: &mockSync.Sync{IsSyncing: true},
	}

	req := &ethpb.AggregateSelectionRequest{CommitteeIndex: 1}
	wanted := "Syncing to latest head, not ready to respond"
	_, err = aggregatorServer.SubmitAggregateSelectionProof(ctx, req)
	assert.ErrorContains(t, wanted, err)
}

func TestSubmitAggregateAndProof_CantFindValidatorIndex(t *testing.T) {
	ctx := context.Background()

	s, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(t, err)

	server := &Server{
		HeadFetcher: &mock.ChainService{State: s},
		SyncChecker: &mockSync.Sync{IsSyncing: false},
		TimeFetcher: &mock.ChainService{Genesis: time.Now()},
	}

	priv, err := bls.RandKey()
	require.NoError(t, err)
	sig := priv.Sign([]byte{'A'})
	req := &ethpb.AggregateSelectionRequest{CommitteeIndex: 1, SlotSignature: sig.Marshal(), PublicKey: pubKey(3)}
	wanted := "Could not locate validator index in DB"
	_, err = server.SubmitAggregateSelectionProof(ctx, req)
	assert.ErrorContains(t, wanted, err)
}

func TestSubmitAggregateAndProof_IsAggregatorAndNoAtts(t *testing.T) {
	ctx := context.Background()

	s, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
		Validators: []*ethpb.Validator{
			{PublicKey: pubKey(0), ExitEpoch: params.BeaconConfig().FarFutureEpoch},
			{PublicKey: pubKey(1), ExitEpoch: params.BeaconConfig().FarFutureEpoch},
		},
	})
	require.NoError(t, err)

	server := &Server{
		HeadFetcher: &mock.ChainService{State: s},
		SyncChecker: &mockSync.Sync{IsSyncing: false},
		AttPool:     attestations.NewPool(),
		TimeFetcher: &mock.ChainService{Genesis: time.Now()},
	}

	priv, err := bls.RandKey()
	require.NoError(t, err)
	sig := priv.Sign([]byte{'A'})
	v, err := s.ValidatorAtIndex(1)
	require.NoError(t, err)
	pubKey := v.PublicKey
	req := &ethpb.AggregateSelectionRequest{CommitteeIndex: 1, SlotSignature: sig.Marshal(), PublicKey: pubKey}

	_, err = server.SubmitAggregateSelectionProof(ctx, req)
	assert.ErrorContains(t, "Could not find attestation for slot and committee in pool", err)
}

func TestSubmitAggregateAndProof_UnaggregateOk(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	c := params.MinimalSpecConfig().Copy()
	c.TargetAggregatorsPerCommittee = 16
	params.OverrideBeaconConfig(c)

	ctx := context.Background()

	beaconState, privKeys := util.DeterministicGenesisState(t, 32)
	att0, err := generateUnaggregatedAtt(beaconState, 0, privKeys)
	require.NoError(t, err)
	err = beaconState.SetSlot(beaconState.Slot() + params.BeaconConfig().MinAttestationInclusionDelay)
	require.NoError(t, err)

	aggregatorServer := &Server{
		HeadFetcher: &mock.ChainService{State: beaconState},
		SyncChecker: &mockSync.Sync{IsSyncing: false},
		AttPool:     attestations.NewPool(),
		P2P:         &mockp2p.MockBroadcaster{},
		TimeFetcher: &mock.ChainService{Genesis: time.Now()},
	}

	priv, err := bls.RandKey()
	require.NoError(t, err)
	sig := priv.Sign([]byte{'B'})
	v, err := beaconState.ValidatorAtIndex(1)
	require.NoError(t, err)
	pubKey := v.PublicKey
	req := &ethpb.AggregateSelectionRequest{CommitteeIndex: 1, SlotSignature: sig.Marshal(), PublicKey: pubKey}

	require.NoError(t, aggregatorServer.AttPool.SaveUnaggregatedAttestation(att0))
	_, err = aggregatorServer.SubmitAggregateSelectionProof(ctx, req)
	require.NoError(t, err)
}

func TestSubmitAggregateAndProof_AggregateOk(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	c := params.MinimalSpecConfig().Copy()
	c.TargetAggregatorsPerCommittee = 16
	params.OverrideBeaconConfig(c)

	ctx := context.Background()

	beaconState, privKeys := util.DeterministicGenesisState(t, 32)
	att0, err := generateAtt(beaconState, 0, privKeys)
	require.NoError(t, err)
	att1, err := generateAtt(beaconState, 2, privKeys)
	require.NoError(t, err)

	err = beaconState.SetSlot(beaconState.Slot() + params.BeaconConfig().MinAttestationInclusionDelay)
	require.NoError(t, err)

	aggregatorServer := &Server{
		HeadFetcher: &mock.ChainService{State: beaconState},
		SyncChecker: &mockSync.Sync{IsSyncing: false},
		AttPool:     attestations.NewPool(),
		P2P:         &mockp2p.MockBroadcaster{},
		TimeFetcher: &mock.ChainService{Genesis: time.Now()},
	}

	priv, err := bls.RandKey()
	require.NoError(t, err)
	sig := priv.Sign([]byte{'B'})
	v, err := beaconState.ValidatorAtIndex(1)
	require.NoError(t, err)
	pubKey := v.PublicKey
	req := &ethpb.AggregateSelectionRequest{CommitteeIndex: 1, SlotSignature: sig.Marshal(), PublicKey: pubKey}

	require.NoError(t, aggregatorServer.AttPool.SaveAggregatedAttestation(att0))
	require.NoError(t, aggregatorServer.AttPool.SaveAggregatedAttestation(att1))
	_, err = aggregatorServer.SubmitAggregateSelectionProof(ctx, req)
	require.NoError(t, err)

	aggregatedAtts := aggregatorServer.AttPool.AggregatedAttestations()
	wanted, err := attaggregation.AggregatePair(att0, att1)
	require.NoError(t, err)
	if reflect.DeepEqual(aggregatedAtts, wanted) {
		t.Error("Did not receive wanted attestation")
	}
}

func TestSubmitAggregateAndProof_AggregateNotOk(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	c := params.MinimalSpecConfig().Copy()
	c.TargetAggregatorsPerCommittee = 16
	params.OverrideBeaconConfig(c)

	ctx := context.Background()

	beaconState, _ := util.DeterministicGenesisState(t, 32)
	require.NoError(t, beaconState.SetSlot(beaconState.Slot()+params.BeaconConfig().MinAttestationInclusionDelay))

	aggregatorServer := &Server{
		HeadFetcher: &mock.ChainService{State: beaconState},
		SyncChecker: &mockSync.Sync{IsSyncing: false},
		AttPool:     attestations.NewPool(),
		P2P:         &mockp2p.MockBroadcaster{},
		TimeFetcher: &mock.ChainService{Genesis: time.Now()},
	}

	priv, err := bls.RandKey()
	require.NoError(t, err)
	sig := priv.Sign([]byte{'B'})
	v, err := beaconState.ValidatorAtIndex(1)
	require.NoError(t, err)
	pubKey := v.PublicKey
	req := &ethpb.AggregateSelectionRequest{CommitteeIndex: 1, SlotSignature: sig.Marshal(), PublicKey: pubKey}

	_, err = aggregatorServer.SubmitAggregateSelectionProof(ctx, req)
	assert.ErrorContains(t, "Could not find attestation for slot and committee in pool", err)

	aggregatedAtts := aggregatorServer.AttPool.AggregatedAttestations()
	assert.Equal(t, 0, len(aggregatedAtts), "Wanted aggregated attestation")
}

func generateAtt(state state.ReadOnlyBeaconState, index uint64, privKeys []bls.SecretKey) (*ethpb.Attestation, error) {
	aggBits := bitfield.NewBitlist(4)
	aggBits.SetBitAt(index, true)
	aggBits.SetBitAt(index+1, true)
	att := util.HydrateAttestation(&ethpb.Attestation{
		Data:            &ethpb.AttestationData{CommitteeIndex: 1},
		AggregationBits: aggBits,
	})
	committee, err := helpers.BeaconCommitteeFromState(context.Background(), state, att.Data.Slot, att.Data.CommitteeIndex)
	if err != nil {
		return nil, err
	}
	attestingIndices, err := attestation.AttestingIndices(att.AggregationBits, committee)
	if err != nil {
		return nil, err
	}

	sigs := make([]bls.Signature, len(attestingIndices))
	var zeroSig [96]byte
	att.Signature = zeroSig[:]

	for i, indice := range attestingIndices {
		sb, err := signing.ComputeDomainAndSign(state, 0, att.Data, params.BeaconConfig().DomainBeaconAttester, privKeys[indice])
		if err != nil {
			return nil, err
		}
		sig, err := bls.SignatureFromBytes(sb)
		if err != nil {
			return nil, err
		}
		sigs[i] = sig
	}

	att.Signature = bls.AggregateSignatures(sigs).Marshal()

	return att, nil
}

func generateUnaggregatedAtt(state state.ReadOnlyBeaconState, index uint64, privKeys []bls.SecretKey) (*ethpb.Attestation, error) {
	aggBits := bitfield.NewBitlist(4)
	aggBits.SetBitAt(index, true)
	att := util.HydrateAttestation(&ethpb.Attestation{
		Data: &ethpb.AttestationData{
			CommitteeIndex: 1,
		},
		AggregationBits: aggBits,
	})
	committee, err := helpers.BeaconCommitteeFromState(context.Background(), state, att.Data.Slot, att.Data.CommitteeIndex)
	if err != nil {
		return nil, err
	}
	attestingIndices, err := attestation.AttestingIndices(att.AggregationBits, committee)
	if err != nil {
		return nil, err
	}
	domain, err := signing.Domain(state.Fork(), 0, params.BeaconConfig().DomainBeaconAttester, params.BeaconConfig().ZeroHash[:])
	if err != nil {
		return nil, err
	}

	sigs := make([]bls.Signature, len(attestingIndices))
	var zeroSig [96]byte
	att.Signature = zeroSig[:]

	for i, indice := range attestingIndices {
		hashTreeRoot, err := signing.ComputeSigningRoot(att.Data, domain)
		if err != nil {
			return nil, err
		}
		sig := privKeys[indice].Sign(hashTreeRoot[:])
		sigs[i] = sig
	}

	att.Signature = bls.AggregateSignatures(sigs).Marshal()

	return att, nil
}

func TestSubmitAggregateAndProof_PreferOwnAttestation(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	c := params.MinimalSpecConfig().Copy()
	c.TargetAggregatorsPerCommittee = 16
	params.OverrideBeaconConfig(c)

	ctx := context.Background()

	// This test creates 3 attestations. 0 and 2 have the same attestation data and can be
	// aggregated. 1 has the validator's signature making this request and that is the expected
	// attestation to sign, even though the aggregated 0&2 would have more aggregated bits.
	beaconState, privKeys := util.DeterministicGenesisState(t, 32)
	att0, err := generateAtt(beaconState, 0, privKeys)
	require.NoError(t, err)
	att0.Data.BeaconBlockRoot = bytesutil.PadTo([]byte("foo"), fieldparams.RootLength)
	att0.AggregationBits = bitfield.Bitlist{0b11100}
	att1, err := generateAtt(beaconState, 0, privKeys)
	require.NoError(t, err)
	att1.Data.BeaconBlockRoot = bytesutil.PadTo([]byte("bar"), fieldparams.RootLength)
	att1.AggregationBits = bitfield.Bitlist{0b11001}
	att2, err := generateAtt(beaconState, 2, privKeys)
	require.NoError(t, err)
	att2.Data.BeaconBlockRoot = bytesutil.PadTo([]byte("foo"), fieldparams.RootLength)
	att2.AggregationBits = bitfield.Bitlist{0b11110}

	err = beaconState.SetSlot(beaconState.Slot() + params.BeaconConfig().MinAttestationInclusionDelay)
	require.NoError(t, err)

	aggregatorServer := &Server{
		HeadFetcher: &mock.ChainService{State: beaconState},
		SyncChecker: &mockSync.Sync{IsSyncing: false},
		AttPool:     attestations.NewPool(),
		P2P:         &mockp2p.MockBroadcaster{},
		TimeFetcher: &mock.ChainService{Genesis: time.Now()},
	}

	priv, err := bls.RandKey()
	require.NoError(t, err)
	sig := priv.Sign([]byte{'B'})
	v, err := beaconState.ValidatorAtIndex(1)
	require.NoError(t, err)
	pubKey := v.PublicKey
	req := &ethpb.AggregateSelectionRequest{CommitteeIndex: 1, SlotSignature: sig.Marshal(), PublicKey: pubKey}

	err = aggregatorServer.AttPool.SaveAggregatedAttestations([]*ethpb.Attestation{
		att0,
		att1,
		att2,
	})
	require.NoError(t, err)

	res, err := aggregatorServer.SubmitAggregateSelectionProof(ctx, req)
	require.NoError(t, err)
	assert.DeepSSZEqual(t, att1, res.AggregateAndProof.Aggregate, "Did not receive wanted attestation")
}

func TestSubmitAggregateAndProof_SelectsMostBitsWhenOwnAttestationNotPresent(t *testing.T) {
	params.SetupTestConfigCleanup(t)
	c := params.MinimalSpecConfig().Copy()
	c.TargetAggregatorsPerCommittee = 16
	params.OverrideBeaconConfig(c)

	ctx := context.Background()

	// This test creates two distinct attestations, neither of which contain the validator's index,
	// index 0. This test should choose the most bits attestation, att1.
	beaconState, privKeys := util.DeterministicGenesisState(t, fieldparams.RootLength)
	att0, err := generateAtt(beaconState, 0, privKeys)
	require.NoError(t, err)
	att0.Data.BeaconBlockRoot = bytesutil.PadTo([]byte("foo"), fieldparams.RootLength)
	att0.AggregationBits = bitfield.Bitlist{0b11100}
	att1, err := generateAtt(beaconState, 2, privKeys)
	require.NoError(t, err)
	att1.Data.BeaconBlockRoot = bytesutil.PadTo([]byte("bar"), fieldparams.RootLength)
	att1.AggregationBits = bitfield.Bitlist{0b11110}

	err = beaconState.SetSlot(beaconState.Slot() + params.BeaconConfig().MinAttestationInclusionDelay)
	require.NoError(t, err)

	aggregatorServer := &Server{
		HeadFetcher: &mock.ChainService{State: beaconState},
		SyncChecker: &mockSync.Sync{IsSyncing: false},
		AttPool:     attestations.NewPool(),
		P2P:         &mockp2p.MockBroadcaster{},
		TimeFetcher: &mock.ChainService{Genesis: time.Now()},
	}

	priv, err := bls.RandKey()
	require.NoError(t, err)
	sig := priv.Sign([]byte{'B'})
	v, err := beaconState.ValidatorAtIndex(1)
	require.NoError(t, err)
	pubKey := v.PublicKey
	req := &ethpb.AggregateSelectionRequest{CommitteeIndex: 1, SlotSignature: sig.Marshal(), PublicKey: pubKey}

	err = aggregatorServer.AttPool.SaveAggregatedAttestations([]*ethpb.Attestation{
		att0,
		att1,
	})
	require.NoError(t, err)

	res, err := aggregatorServer.SubmitAggregateSelectionProof(ctx, req)
	require.NoError(t, err)
	assert.DeepSSZEqual(t, att1, res.AggregateAndProof.Aggregate, "Did not receive wanted attestation")
}

func TestSubmitSignedAggregateSelectionProof_ZeroHashesSignatures(t *testing.T) {
	aggregatorServer := &Server{
		TimeFetcher: &mock.ChainService{Genesis: time.Now()},
	}
	req := &ethpb.SignedAggregateSubmitRequest{
		SignedAggregateAndProof: &ethpb.SignedAggregateAttestationAndProof{
			Signature: make([]byte, fieldparams.BLSSignatureLength),
			Message: &ethpb.AggregateAttestationAndProof{
				Aggregate: &ethpb.Attestation{
					Data: &ethpb.AttestationData{},
				},
			},
		},
	}
	_, err := aggregatorServer.SubmitSignedAggregateSelectionProof(context.Background(), req)
	require.ErrorContains(t, "signed signatures can't be zero hashes", err)

	req = &ethpb.SignedAggregateSubmitRequest{
		SignedAggregateAndProof: &ethpb.SignedAggregateAttestationAndProof{
			Signature: []byte{'a'},
			Message: &ethpb.AggregateAttestationAndProof{
				Aggregate: &ethpb.Attestation{
					Data: &ethpb.AttestationData{},
				},
				SelectionProof: make([]byte, fieldparams.BLSSignatureLength),
			},
		},
	}
	_, err = aggregatorServer.SubmitSignedAggregateSelectionProof(context.Background(), req)
	require.ErrorContains(t, "signed signatures can't be zero hashes", err)
}

func TestSubmitSignedAggregateSelectionProof_InvalidSlot(t *testing.T) {
	c := &mock.ChainService{Genesis: time.Now()}
	aggregatorServer := &Server{
		CoreService: &core.Service{
			GenesisTimeFetcher: c,
		},
	}
	req := &ethpb.SignedAggregateSubmitRequest{
		SignedAggregateAndProof: &ethpb.SignedAggregateAttestationAndProof{
			Signature: []byte{'a'},
			Message: &ethpb.AggregateAttestationAndProof{
				SelectionProof: []byte{'a'},
				Aggregate: &ethpb.Attestation{
					Data: &ethpb.AttestationData{Slot: 1000},
				},
			},
		},
	}
	_, err := aggregatorServer.SubmitSignedAggregateSelectionProof(context.Background(), req)
	require.ErrorContains(t, "attestation slot is no longer valid from current time", err)
}
