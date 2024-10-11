package blocks_test

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation/aggregation"
	attaggregation "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/attestation/aggregation/attestations"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
)

func TestProcessAggregatedAttestation_OverlappingBits(t *testing.T) {
	beaconState, privKeys := util.DeterministicGenesisState(t, 100)
	data := util.HydrateAttestationData(&ethpb.AttestationData{
		Source: &ethpb.Checkpoint{Epoch: 0, Root: bytesutil.PadTo([]byte("hello-world"), 32)},
		Target: &ethpb.Checkpoint{Epoch: 0, Root: bytesutil.PadTo([]byte("hello-world"), 32)},
	})
	aggBits1 := bitfield.NewBitlist(3)
	aggBits1.SetBitAt(0, true)
	aggBits1.SetBitAt(1, true)
	att1 := &ethpb.Attestation{
		Data:            data,
		AggregationBits: aggBits1,
	}

	cfc := beaconState.CurrentJustifiedCheckpoint()
	cfc.Root = bytesutil.PadTo([]byte("hello-world"), 32)
	require.NoError(t, beaconState.SetCurrentJustifiedCheckpoint(cfc))
	require.NoError(t, beaconState.AppendCurrentEpochAttestations(&ethpb.PendingAttestation{}))

	committee, err := helpers.BeaconCommitteeFromState(context.Background(), beaconState, att1.Data.Slot, att1.Data.CommitteeIndex)
	require.NoError(t, err)
	attestingIndices1, err := attestation.AttestingIndices(att1.AggregationBits, committee)
	require.NoError(t, err)
	sigs := make([]bls.Signature, len(attestingIndices1))
	for i, indice := range attestingIndices1 {
		sb, err := signing.ComputeDomainAndSign(beaconState, 0, att1.Data, params.BeaconConfig().DomainBeaconAttester, privKeys[indice])
		require.NoError(t, err)
		sig, err := bls.SignatureFromBytes(sb)
		require.NoError(t, err)
		sigs[i] = sig
	}
	att1.Signature = bls.AggregateSignatures(sigs).Marshal()

	aggBits2 := bitfield.NewBitlist(3)
	aggBits2.SetBitAt(1, true)
	aggBits2.SetBitAt(2, true)
	att2 := &ethpb.Attestation{
		Data:            data,
		AggregationBits: aggBits2,
	}

	committee, err = helpers.BeaconCommitteeFromState(context.Background(), beaconState, att2.Data.Slot, att2.Data.CommitteeIndex)
	require.NoError(t, err)
	attestingIndices2, err := attestation.AttestingIndices(att2.AggregationBits, committee)
	require.NoError(t, err)
	sigs = make([]bls.Signature, len(attestingIndices2))
	for i, indice := range attestingIndices2 {
		sb, err := signing.ComputeDomainAndSign(beaconState, 0, att2.Data, params.BeaconConfig().DomainBeaconAttester, privKeys[indice])
		require.NoError(t, err)
		sig, err := bls.SignatureFromBytes(sb)
		require.NoError(t, err)
		sigs[i] = sig
	}
	att2.Signature = bls.AggregateSignatures(sigs).Marshal()

	_, err = attaggregation.AggregatePair(att1, att2)
	assert.ErrorContains(t, aggregation.ErrBitsOverlap.Error(), err)
}

func TestVerifyAttestationNoVerifySignature_IncorrectSlotTargetEpoch(t *testing.T) {
	beaconState, _ := util.DeterministicGenesisState(t, 1)

	att := util.HydrateAttestation(&ethpb.Attestation{
		Data: &ethpb.AttestationData{
			Slot:   params.BeaconConfig().SlotsPerEpoch,
			Target: &ethpb.Checkpoint{Root: make([]byte, 32)},
		},
	})
	wanted := "slot 32 does not match target epoch 0"
	err := blocks.VerifyAttestationNoVerifySignature(context.TODO(), beaconState, att)
	assert.ErrorContains(t, wanted, err)
}

func TestProcessAttestationsNoVerify_OK(t *testing.T) {
	// Attestation with an empty signature

	beaconState, _ := util.DeterministicGenesisState(t, 100)

	aggBits := bitfield.NewBitlist(3)
	aggBits.SetBitAt(1, true)
	var mockRoot [32]byte
	copy(mockRoot[:], "hello-world")
	att := &ethpb.Attestation{
		Data: &ethpb.AttestationData{
			Source: &ethpb.Checkpoint{Epoch: 0, Root: mockRoot[:]},
			Target: &ethpb.Checkpoint{Epoch: 0, Root: make([]byte, 32)},
		},
		AggregationBits: aggBits,
	}

	var zeroSig [fieldparams.BLSSignatureLength]byte
	att.Signature = zeroSig[:]

	err := beaconState.SetSlot(beaconState.Slot() + params.BeaconConfig().MinAttestationInclusionDelay)
	require.NoError(t, err)
	ckp := beaconState.CurrentJustifiedCheckpoint()
	copy(ckp.Root, "hello-world")
	require.NoError(t, beaconState.SetCurrentJustifiedCheckpoint(ckp))
	require.NoError(t, beaconState.AppendCurrentEpochAttestations(&ethpb.PendingAttestation{}))

	_, err = blocks.ProcessAttestationNoVerifySignature(context.TODO(), beaconState, att)
	assert.NoError(t, err)
}

func TestProcessAttestationsNoVerify_OlderThanSlotsPerEpoch(t *testing.T) {
	aggBits := bitfield.NewBitlist(3)
	aggBits.SetBitAt(1, true)
	att := &ethpb.Attestation{
		Data: &ethpb.AttestationData{
			Source: &ethpb.Checkpoint{Epoch: 0, Root: make([]byte, 32)},
			Target: &ethpb.Checkpoint{Epoch: 0, Root: make([]byte, 32)},
		},
		AggregationBits: aggBits,
	}
	ctx := context.Background()

	t.Run("attestation older than slots per epoch", func(t *testing.T) {
		beaconState, _ := util.DeterministicGenesisState(t, 100)

		err := beaconState.SetSlot(beaconState.Slot() + params.BeaconConfig().SlotsPerEpoch + 1)
		require.NoError(t, err)
		ckp := beaconState.CurrentJustifiedCheckpoint()
		copy(ckp.Root, "hello-world")
		require.NoError(t, beaconState.SetCurrentJustifiedCheckpoint(ckp))
		require.NoError(t, beaconState.AppendCurrentEpochAttestations(&ethpb.PendingAttestation{}))

		require.ErrorContains(t, "state slot 33 > attestation slot 0 + SLOTS_PER_EPOCH 32", blocks.VerifyAttestationNoVerifySignature(ctx, beaconState, att))
	})

	t.Run("attestation older than slots per epoch in deneb", func(t *testing.T) {
		beaconState, _ := util.DeterministicGenesisStateDeneb(t, 100)

		err := beaconState.SetSlot(beaconState.Slot() + params.BeaconConfig().SlotsPerEpoch + 1)
		require.NoError(t, err)
		ckp := beaconState.CurrentJustifiedCheckpoint()
		copy(ckp.Root, "hello-world")
		require.NoError(t, beaconState.SetCurrentJustifiedCheckpoint(ckp))

		require.NoError(t, blocks.VerifyAttestationNoVerifySignature(ctx, beaconState, att))
	})
}

func TestVerifyAttestationNoVerifySignature_OK(t *testing.T) {
	// Attestation with an empty signature

	beaconState, _ := util.DeterministicGenesisState(t, 100)

	aggBits := bitfield.NewBitlist(3)
	aggBits.SetBitAt(1, true)
	var mockRoot [32]byte
	copy(mockRoot[:], "hello-world")
	att := &ethpb.Attestation{
		Data: &ethpb.AttestationData{
			Source: &ethpb.Checkpoint{Epoch: 0, Root: mockRoot[:]},
			Target: &ethpb.Checkpoint{Epoch: 0, Root: make([]byte, 32)},
		},
		AggregationBits: aggBits,
	}

	var zeroSig [fieldparams.BLSSignatureLength]byte
	att.Signature = zeroSig[:]

	err := beaconState.SetSlot(beaconState.Slot() + params.BeaconConfig().MinAttestationInclusionDelay)
	require.NoError(t, err)
	ckp := beaconState.CurrentJustifiedCheckpoint()
	copy(ckp.Root, "hello-world")
	require.NoError(t, beaconState.SetCurrentJustifiedCheckpoint(ckp))
	require.NoError(t, beaconState.AppendCurrentEpochAttestations(&ethpb.PendingAttestation{}))

	err = blocks.VerifyAttestationNoVerifySignature(context.TODO(), beaconState, att)
	assert.NoError(t, err)
}

func TestVerifyAttestationNoVerifySignature_BadAttIdx(t *testing.T) {
	beaconState, _ := util.DeterministicGenesisState(t, 100)
	aggBits := bitfield.NewBitlist(3)
	aggBits.SetBitAt(1, true)
	var mockRoot [32]byte
	copy(mockRoot[:], "hello-world")
	att := &ethpb.Attestation{
		Data: &ethpb.AttestationData{
			CommitteeIndex: 100,
			Source:         &ethpb.Checkpoint{Epoch: 0, Root: mockRoot[:]},
			Target:         &ethpb.Checkpoint{Epoch: 0, Root: make([]byte, 32)},
		},
		AggregationBits: aggBits,
	}
	var zeroSig [fieldparams.BLSSignatureLength]byte
	att.Signature = zeroSig[:]
	require.NoError(t, beaconState.SetSlot(beaconState.Slot()+params.BeaconConfig().MinAttestationInclusionDelay))
	ckp := beaconState.CurrentJustifiedCheckpoint()
	copy(ckp.Root, "hello-world")
	require.NoError(t, beaconState.SetCurrentJustifiedCheckpoint(ckp))
	require.NoError(t, beaconState.AppendCurrentEpochAttestations(&ethpb.PendingAttestation{}))
	err := blocks.VerifyAttestationNoVerifySignature(context.TODO(), beaconState, att)
	require.ErrorContains(t, "committee index 100 >= committee count 1", err)
}

func TestConvertToIndexed_OK(t *testing.T) {
	helpers.ClearCache()
	validators := make([]*ethpb.Validator, 2*params.BeaconConfig().SlotsPerEpoch)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch: params.BeaconConfig().FarFutureEpoch,
		}
	}

	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Slot:        5,
		Validators:  validators,
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(t, err)
	tests := []struct {
		aggregationBitfield    bitfield.Bitlist
		wantedAttestingIndices []uint64
	}{
		{
			aggregationBitfield:    bitfield.Bitlist{0x07},
			wantedAttestingIndices: []uint64{43, 47},
		},
		{
			aggregationBitfield:    bitfield.Bitlist{0x05},
			wantedAttestingIndices: []uint64{47},
		},
		{
			aggregationBitfield:    bitfield.Bitlist{0x04},
			wantedAttestingIndices: []uint64{},
		},
	}

	var sig [fieldparams.BLSSignatureLength]byte
	copy(sig[:], "signed")
	att := util.HydrateAttestation(&ethpb.Attestation{
		Signature: sig[:],
	})
	for _, tt := range tests {
		att.AggregationBits = tt.aggregationBitfield
		wanted := &ethpb.IndexedAttestation{
			AttestingIndices: tt.wantedAttestingIndices,
			Data:             att.Data,
			Signature:        att.Signature,
		}

		committee, err := helpers.BeaconCommitteeFromState(context.Background(), state, att.Data.Slot, att.Data.CommitteeIndex)
		require.NoError(t, err)
		ia, err := attestation.ConvertToIndexed(context.Background(), att, committee)
		require.NoError(t, err)
		assert.DeepEqual(t, wanted, ia, "Convert attestation to indexed attestation didn't result as wanted")
	}
}

func TestVerifyIndexedAttestation_OK(t *testing.T) {
	numOfValidators := uint64(params.BeaconConfig().SlotsPerEpoch.Mul(4))
	validators := make([]*ethpb.Validator, numOfValidators)
	_, keys, err := util.DeterministicDepositsAndKeys(numOfValidators)
	require.NoError(t, err)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch:             params.BeaconConfig().FarFutureEpoch,
			PublicKey:             keys[i].PublicKey().Marshal(),
			WithdrawalCredentials: make([]byte, 32),
		}
	}

	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		Slot:       5,
		Validators: validators,
		Fork: &ethpb.Fork{
			Epoch:           0,
			CurrentVersion:  params.BeaconConfig().GenesisForkVersion,
			PreviousVersion: params.BeaconConfig().GenesisForkVersion,
		},
		RandaoMixes: make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector),
	})
	require.NoError(t, err)
	tests := []struct {
		attestation *ethpb.IndexedAttestation
	}{
		{attestation: &ethpb.IndexedAttestation{
			Data: util.HydrateAttestationData(&ethpb.AttestationData{
				Target: &ethpb.Checkpoint{
					Epoch: 2,
				},
				Source: &ethpb.Checkpoint{},
			}),
			AttestingIndices: []uint64{1},
			Signature:        make([]byte, fieldparams.BLSSignatureLength),
		}},
		{attestation: &ethpb.IndexedAttestation{
			Data: util.HydrateAttestationData(&ethpb.AttestationData{
				Target: &ethpb.Checkpoint{
					Epoch: 1,
				},
			}),
			AttestingIndices: []uint64{47, 99, 101},
			Signature:        make([]byte, fieldparams.BLSSignatureLength),
		}},
		{attestation: &ethpb.IndexedAttestation{
			Data: util.HydrateAttestationData(&ethpb.AttestationData{
				Target: &ethpb.Checkpoint{
					Epoch: 4,
				},
			}),
			AttestingIndices: []uint64{21, 72},
			Signature:        make([]byte, fieldparams.BLSSignatureLength),
		}},
		{attestation: &ethpb.IndexedAttestation{
			Data: util.HydrateAttestationData(&ethpb.AttestationData{
				Target: &ethpb.Checkpoint{
					Epoch: 7,
				},
			}),
			AttestingIndices: []uint64{100, 121, 122},
			Signature:        make([]byte, fieldparams.BLSSignatureLength),
		}},
	}

	for _, tt := range tests {
		var sig []bls.Signature
		for _, idx := range tt.attestation.AttestingIndices {
			sb, err := signing.ComputeDomainAndSign(state, tt.attestation.Data.Target.Epoch, tt.attestation.Data, params.BeaconConfig().DomainBeaconAttester, keys[idx])
			require.NoError(t, err)
			validatorSig, err := bls.SignatureFromBytes(sb)
			require.NoError(t, err)
			sig = append(sig, validatorSig)
		}
		aggSig := bls.AggregateSignatures(sig)
		marshalledSig := aggSig.Marshal()

		tt.attestation.Signature = marshalledSig

		err = blocks.VerifyIndexedAttestation(context.Background(), state, tt.attestation)
		assert.NoError(t, err, "Failed to verify indexed attestation")
	}
}

func TestValidateIndexedAttestation_AboveMaxLength(t *testing.T) {
	indexedAtt1 := &ethpb.IndexedAttestation{
		AttestingIndices: make([]uint64, params.BeaconConfig().MaxValidatorsPerCommittee+5),
	}

	for i := uint64(0); i < params.BeaconConfig().MaxValidatorsPerCommittee+5; i++ {
		indexedAtt1.AttestingIndices[i] = i
		indexedAtt1.Data = &ethpb.AttestationData{
			Target: &ethpb.Checkpoint{
				Epoch: primitives.Epoch(i),
			},
		}
	}

	want := "validator indices count exceeds MAX_VALIDATORS_PER_COMMITTEE"
	st, err := state_native.InitializeFromProtoUnsafePhase0(&ethpb.BeaconState{})
	require.NoError(t, err)
	err = blocks.VerifyIndexedAttestation(context.Background(), st, indexedAtt1)
	assert.ErrorContains(t, want, err)
}

func TestValidateIndexedAttestation_BadAttestationsSignatureSet(t *testing.T) {
	beaconState, keys := util.DeterministicGenesisState(t, 128)

	sig := keys[0].Sign([]byte{'t', 'e', 's', 't'})
	list := bitfield.Bitlist{0b11111}
	var atts []*ethpb.Attestation
	for i := uint64(0); i < 1000; i++ {
		atts = append(atts, &ethpb.Attestation{
			Data: &ethpb.AttestationData{
				CommitteeIndex: 1,
				Slot:           1,
			},
			Signature:       sig.Marshal(),
			AggregationBits: list,
		})
	}

	want := "nil or missing indexed attestation data"
	_, err := blocks.AttestationSignatureBatch(context.Background(), beaconState, atts)
	assert.ErrorContains(t, want, err)

	atts = []*ethpb.Attestation{}
	list = bitfield.Bitlist{0b10000}
	for i := uint64(0); i < 1000; i++ {
		atts = append(atts, &ethpb.Attestation{
			Data: &ethpb.AttestationData{
				CommitteeIndex: 1,
				Slot:           1,
				Target: &ethpb.Checkpoint{
					Root: []byte{},
				},
			},
			Signature:       sig.Marshal(),
			AggregationBits: list,
		})
	}

	want = "expected non-empty attesting indices"
	_, err = blocks.AttestationSignatureBatch(context.Background(), beaconState, atts)
	assert.ErrorContains(t, want, err)
}

func TestVerifyAttestations_HandlesPlannedFork(t *testing.T) {
	// In this test, att1 is from the prior fork and att2 is from the new fork.
	numOfValidators := uint64(params.BeaconConfig().SlotsPerEpoch.Mul(4))
	validators := make([]*ethpb.Validator, numOfValidators)
	_, keys, err := util.DeterministicDepositsAndKeys(numOfValidators)
	require.NoError(t, err)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch:             params.BeaconConfig().FarFutureEpoch,
			PublicKey:             keys[i].PublicKey().Marshal(),
			WithdrawalCredentials: make([]byte, 32),
		}
	}

	st, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, st.SetSlot(35))
	require.NoError(t, st.SetValidators(validators))
	require.NoError(t, st.SetFork(&ethpb.Fork{
		Epoch:           1,
		CurrentVersion:  []byte{0, 1, 2, 3},
		PreviousVersion: params.BeaconConfig().GenesisForkVersion,
	}))

	comm1, err := helpers.BeaconCommitteeFromState(context.Background(), st, 1 /*slot*/, 0 /*committeeIndex*/)
	require.NoError(t, err)
	att1 := util.HydrateAttestation(&ethpb.Attestation{
		AggregationBits: bitfield.NewBitlist(uint64(len(comm1))),
		Data: &ethpb.AttestationData{
			Slot: 1,
		},
	})
	prevDomain, err := signing.Domain(st.Fork(), st.Fork().Epoch-1, params.BeaconConfig().DomainBeaconAttester, st.GenesisValidatorsRoot())
	require.NoError(t, err)
	root, err := signing.ComputeSigningRoot(att1.Data, prevDomain)
	require.NoError(t, err)
	var sigs []bls.Signature
	for i, u := range comm1 {
		att1.AggregationBits.SetBitAt(uint64(i), true)
		sigs = append(sigs, keys[u].Sign(root[:]))
	}
	att1.Signature = bls.AggregateSignatures(sigs).Marshal()

	comm2, err := helpers.BeaconCommitteeFromState(context.Background(), st, 1*params.BeaconConfig().SlotsPerEpoch+1 /*slot*/, 1 /*committeeIndex*/)
	require.NoError(t, err)
	att2 := util.HydrateAttestation(&ethpb.Attestation{
		AggregationBits: bitfield.NewBitlist(uint64(len(comm2))),
		Data: &ethpb.AttestationData{
			Slot:           1*params.BeaconConfig().SlotsPerEpoch + 1,
			CommitteeIndex: 1,
		},
	})
	currDomain, err := signing.Domain(st.Fork(), st.Fork().Epoch, params.BeaconConfig().DomainBeaconAttester, st.GenesisValidatorsRoot())
	require.NoError(t, err)
	root, err = signing.ComputeSigningRoot(att2.Data, currDomain)
	require.NoError(t, err)
	sigs = nil
	for i, u := range comm2 {
		att2.AggregationBits.SetBitAt(uint64(i), true)
		sigs = append(sigs, keys[u].Sign(root[:]))
	}
	att2.Signature = bls.AggregateSignatures(sigs).Marshal()
}

func TestRetrieveAttestationSignatureSet_VerifiesMultipleAttestations(t *testing.T) {
	ctx := context.Background()
	numOfValidators := uint64(params.BeaconConfig().SlotsPerEpoch.Mul(4))
	validators := make([]*ethpb.Validator, numOfValidators)
	_, keys, err := util.DeterministicDepositsAndKeys(numOfValidators)
	require.NoError(t, err)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch:             params.BeaconConfig().FarFutureEpoch,
			PublicKey:             keys[i].PublicKey().Marshal(),
			WithdrawalCredentials: make([]byte, 32),
		}
	}

	st, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, st.SetSlot(5))
	require.NoError(t, st.SetValidators(validators))

	comm1, err := helpers.BeaconCommitteeFromState(context.Background(), st, 1 /*slot*/, 0 /*committeeIndex*/)
	require.NoError(t, err)
	att1 := util.HydrateAttestation(&ethpb.Attestation{
		AggregationBits: bitfield.NewBitlist(uint64(len(comm1))),
		Data: &ethpb.AttestationData{
			Slot: 1,
		},
	})
	domain, err := signing.Domain(st.Fork(), st.Fork().Epoch, params.BeaconConfig().DomainBeaconAttester, st.GenesisValidatorsRoot())
	require.NoError(t, err)
	root, err := signing.ComputeSigningRoot(att1.Data, domain)
	require.NoError(t, err)
	var sigs []bls.Signature
	for i, u := range comm1 {
		att1.AggregationBits.SetBitAt(uint64(i), true)
		sigs = append(sigs, keys[u].Sign(root[:]))
	}
	att1.Signature = bls.AggregateSignatures(sigs).Marshal()

	comm2, err := helpers.BeaconCommitteeFromState(context.Background(), st, 1 /*slot*/, 1 /*committeeIndex*/)
	require.NoError(t, err)
	att2 := util.HydrateAttestation(&ethpb.Attestation{
		AggregationBits: bitfield.NewBitlist(uint64(len(comm2))),
		Data: &ethpb.AttestationData{
			Slot:           1,
			CommitteeIndex: 1,
		},
	})
	root, err = signing.ComputeSigningRoot(att2.Data, domain)
	require.NoError(t, err)
	sigs = nil
	for i, u := range comm2 {
		att2.AggregationBits.SetBitAt(uint64(i), true)
		sigs = append(sigs, keys[u].Sign(root[:]))
	}
	att2.Signature = bls.AggregateSignatures(sigs).Marshal()

	set, err := blocks.AttestationSignatureBatch(ctx, st, []*ethpb.Attestation{att1, att2})
	require.NoError(t, err)
	verified, err := set.Verify()
	require.NoError(t, err)
	assert.Equal(t, true, verified, "Multiple signatures were unable to be verified.")
}

func TestRetrieveAttestationSignatureSet_AcrossFork(t *testing.T) {
	ctx := context.Background()
	numOfValidators := uint64(params.BeaconConfig().SlotsPerEpoch.Mul(4))
	validators := make([]*ethpb.Validator, numOfValidators)
	_, keys, err := util.DeterministicDepositsAndKeys(numOfValidators)
	require.NoError(t, err)
	for i := 0; i < len(validators); i++ {
		validators[i] = &ethpb.Validator{
			ExitEpoch:             params.BeaconConfig().FarFutureEpoch,
			PublicKey:             keys[i].PublicKey().Marshal(),
			WithdrawalCredentials: make([]byte, 32),
		}
	}

	st, err := util.NewBeaconState()
	require.NoError(t, err)
	require.NoError(t, st.SetSlot(5))
	require.NoError(t, st.SetValidators(validators))
	require.NoError(t, st.SetFork(&ethpb.Fork{Epoch: 1, CurrentVersion: []byte{0, 1, 2, 3}, PreviousVersion: []byte{0, 1, 1, 1}}))

	comm1, err := helpers.BeaconCommitteeFromState(ctx, st, 1 /*slot*/, 0 /*committeeIndex*/)
	require.NoError(t, err)
	att1 := util.HydrateAttestation(&ethpb.Attestation{
		AggregationBits: bitfield.NewBitlist(uint64(len(comm1))),
		Data: &ethpb.AttestationData{
			Slot: 1,
		},
	})
	domain, err := signing.Domain(st.Fork(), st.Fork().Epoch, params.BeaconConfig().DomainBeaconAttester, st.GenesisValidatorsRoot())
	require.NoError(t, err)
	root, err := signing.ComputeSigningRoot(att1.Data, domain)
	require.NoError(t, err)
	var sigs []bls.Signature
	for i, u := range comm1 {
		att1.AggregationBits.SetBitAt(uint64(i), true)
		sigs = append(sigs, keys[u].Sign(root[:]))
	}
	att1.Signature = bls.AggregateSignatures(sigs).Marshal()

	comm2, err := helpers.BeaconCommitteeFromState(ctx, st, 1 /*slot*/, 1 /*committeeIndex*/)
	require.NoError(t, err)
	att2 := util.HydrateAttestation(&ethpb.Attestation{
		AggregationBits: bitfield.NewBitlist(uint64(len(comm2))),
		Data: &ethpb.AttestationData{
			Slot:           1,
			CommitteeIndex: 1,
		},
	})
	root, err = signing.ComputeSigningRoot(att2.Data, domain)
	require.NoError(t, err)
	sigs = nil
	for i, u := range comm2 {
		att2.AggregationBits.SetBitAt(uint64(i), true)
		sigs = append(sigs, keys[u].Sign(root[:]))
	}
	att2.Signature = bls.AggregateSignatures(sigs).Marshal()

	_, err = blocks.AttestationSignatureBatch(ctx, st, []*ethpb.Attestation{att1, att2})
	require.NoError(t, err)
}
