package altair_test

import (
	"context"
	"math"
	"testing"

	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	p2pType "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func TestProcessSyncCommittee_PerfectParticipation(t *testing.T) {
	beaconState, privKeys := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().MaxValidatorsPerCommittee)
	require.NoError(t, beaconState.SetSlot(1))
	committee, err := altair.NextSyncCommittee(context.Background(), beaconState)
	require.NoError(t, err)
	require.NoError(t, beaconState.SetCurrentSyncCommittee(committee))

	syncBits := bitfield.NewBitvector512()
	for i := range syncBits {
		syncBits[i] = 0xff
	}
	indices, err := altair.NextSyncCommitteeIndices(context.Background(), beaconState)
	require.NoError(t, err)
	ps := slots.PrevSlot(beaconState.Slot())
	pbr, err := helpers.BlockRootAtSlot(beaconState, ps)
	require.NoError(t, err)
	sigs := make([]bls.Signature, len(indices))
	for i, indice := range indices {
		b := p2pType.SSZBytes(pbr)
		sb, err := signing.ComputeDomainAndSign(beaconState, time.CurrentEpoch(beaconState), &b, params.BeaconConfig().DomainSyncCommittee, privKeys[indice])
		require.NoError(t, err)
		sig, err := bls.SignatureFromBytes(sb)
		require.NoError(t, err)
		sigs[i] = sig
	}
	aggregatedSig := bls.AggregateSignatures(sigs).Marshal()
	syncAggregate := &ethpb.SyncAggregate{
		SyncCommitteeBits:      syncBits,
		SyncCommitteeSignature: aggregatedSig,
	}

	var reward uint64
	beaconState, reward, err = altair.ProcessSyncAggregate(context.Background(), beaconState, syncAggregate)
	require.NoError(t, err)
	assert.Equal(t, uint64(72192), reward)

	// Use a non-sync committee index to compare profitability.
	syncCommittee := make(map[primitives.ValidatorIndex]bool)
	for _, index := range indices {
		syncCommittee[index] = true
	}
	nonSyncIndex := primitives.ValidatorIndex(params.BeaconConfig().MaxValidatorsPerCommittee + 1)
	for i := primitives.ValidatorIndex(0); uint64(i) < params.BeaconConfig().MaxValidatorsPerCommittee; i++ {
		if !syncCommittee[i] {
			nonSyncIndex = i
			break
		}
	}

	// Sync committee should be more profitable than non sync committee
	balances := beaconState.Balances()
	require.Equal(t, true, balances[indices[0]] > balances[nonSyncIndex])

	// Proposer should be more profitable than rest of the sync committee
	proposerIndex, err := helpers.BeaconProposerIndex(context.Background(), beaconState)
	require.NoError(t, err)
	require.Equal(t, true, balances[proposerIndex] > balances[indices[0]])

	// Sync committee should have the same profits, except you are a proposer
	for i := 1; i < len(indices); i++ {
		if proposerIndex == indices[i-1] || proposerIndex == indices[i] {
			continue
		}
		require.Equal(t, balances[indices[i-1]], balances[indices[i]])
	}

	// Increased balance validator count should equal to sync committee count
	increased := uint64(0)
	for _, balance := range balances {
		if balance > params.BeaconConfig().MaxEffectiveBalance {
			increased++
		}
	}
	require.Equal(t, params.BeaconConfig().SyncCommitteeSize, increased)
}

func TestProcessSyncCommittee_MixParticipation_BadSignature(t *testing.T) {
	beaconState, privKeys := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().MaxValidatorsPerCommittee)
	require.NoError(t, beaconState.SetSlot(1))
	committee, err := altair.NextSyncCommittee(context.Background(), beaconState)
	require.NoError(t, err)
	require.NoError(t, beaconState.SetCurrentSyncCommittee(committee))

	syncBits := bitfield.NewBitvector512()
	for i := range syncBits {
		syncBits[i] = 0xAA
	}
	indices, err := altair.NextSyncCommitteeIndices(context.Background(), beaconState)
	require.NoError(t, err)
	ps := slots.PrevSlot(beaconState.Slot())
	pbr, err := helpers.BlockRootAtSlot(beaconState, ps)
	require.NoError(t, err)
	sigs := make([]bls.Signature, len(indices))
	for i, indice := range indices {
		b := p2pType.SSZBytes(pbr)
		sb, err := signing.ComputeDomainAndSign(beaconState, time.CurrentEpoch(beaconState), &b, params.BeaconConfig().DomainSyncCommittee, privKeys[indice])
		require.NoError(t, err)
		sig, err := bls.SignatureFromBytes(sb)
		require.NoError(t, err)
		sigs[i] = sig
	}
	aggregatedSig := bls.AggregateSignatures(sigs).Marshal()
	syncAggregate := &ethpb.SyncAggregate{
		SyncCommitteeBits:      syncBits,
		SyncCommitteeSignature: aggregatedSig,
	}

	_, _, err = altair.ProcessSyncAggregate(context.Background(), beaconState, syncAggregate)
	require.ErrorContains(t, "invalid sync committee signature", err)
}

func TestProcessSyncCommittee_MixParticipation_GoodSignature(t *testing.T) {
	beaconState, privKeys := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().MaxValidatorsPerCommittee)
	require.NoError(t, beaconState.SetSlot(1))
	committee, err := altair.NextSyncCommittee(context.Background(), beaconState)
	require.NoError(t, err)
	require.NoError(t, beaconState.SetCurrentSyncCommittee(committee))

	syncBits := bitfield.NewBitvector512()
	for i := range syncBits {
		syncBits[i] = 0xAA
	}
	indices, err := altair.NextSyncCommitteeIndices(context.Background(), beaconState)
	require.NoError(t, err)
	ps := slots.PrevSlot(beaconState.Slot())
	pbr, err := helpers.BlockRootAtSlot(beaconState, ps)
	require.NoError(t, err)
	sigs := make([]bls.Signature, 0, len(indices))
	for i, indice := range indices {
		if syncBits.BitAt(uint64(i)) {
			b := p2pType.SSZBytes(pbr)
			sb, err := signing.ComputeDomainAndSign(beaconState, time.CurrentEpoch(beaconState), &b, params.BeaconConfig().DomainSyncCommittee, privKeys[indice])
			require.NoError(t, err)
			sig, err := bls.SignatureFromBytes(sb)
			require.NoError(t, err)
			sigs = append(sigs, sig)
		}
	}
	aggregatedSig := bls.AggregateSignatures(sigs).Marshal()
	syncAggregate := &ethpb.SyncAggregate{
		SyncCommitteeBits:      syncBits,
		SyncCommitteeSignature: aggregatedSig,
	}

	_, _, err = altair.ProcessSyncAggregate(context.Background(), beaconState, syncAggregate)
	require.NoError(t, err)
}

// This is a regression test #11696
func TestProcessSyncCommittee_DontPrecompute(t *testing.T) {
	beaconState, _ := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().MaxValidatorsPerCommittee)
	require.NoError(t, beaconState.SetSlot(1))
	committee, err := altair.NextSyncCommittee(context.Background(), beaconState)
	require.NoError(t, err)
	committeeKeys := committee.Pubkeys
	committeeKeys[1] = committeeKeys[0]
	require.NoError(t, beaconState.SetCurrentSyncCommittee(committee))
	idx, ok := beaconState.ValidatorIndexByPubkey(bytesutil.ToBytes48(committeeKeys[0]))
	require.Equal(t, true, ok)

	syncBits := bitfield.NewBitvector512()
	for i := range syncBits {
		syncBits[i] = 0xFF
	}
	syncBits.SetBitAt(0, false)
	syncAggregate := &ethpb.SyncAggregate{
		SyncCommitteeBits: syncBits,
	}
	require.NoError(t, beaconState.UpdateBalancesAtIndex(idx, 0))
	st, votedKeys, _, err := altair.ProcessSyncAggregateEported(context.Background(), beaconState, syncAggregate)
	require.NoError(t, err)
	require.Equal(t, 511, len(votedKeys))
	require.DeepEqual(t, committeeKeys[0], votedKeys[0].Marshal())
	balances := st.Balances()
	require.Equal(t, uint64(988), balances[idx])
}

func TestProcessSyncCommittee_processSyncAggregate(t *testing.T) {
	beaconState, _ := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().MaxValidatorsPerCommittee)
	require.NoError(t, beaconState.SetSlot(1))
	committee, err := altair.NextSyncCommittee(context.Background(), beaconState)
	require.NoError(t, err)
	require.NoError(t, beaconState.SetCurrentSyncCommittee(committee))

	syncBits := bitfield.NewBitvector512()
	for i := range syncBits {
		syncBits[i] = 0xAA
	}
	syncAggregate := &ethpb.SyncAggregate{
		SyncCommitteeBits: syncBits,
	}

	st, votedKeys, _, err := altair.ProcessSyncAggregateEported(context.Background(), beaconState, syncAggregate)
	require.NoError(t, err)
	votedMap := make(map[[fieldparams.BLSPubkeyLength]byte]bool)
	for _, key := range votedKeys {
		votedMap[bytesutil.ToBytes48(key.Marshal())] = true
	}
	require.Equal(t, int(syncBits.Len()/2), len(votedKeys))

	currentSyncCommittee, err := st.CurrentSyncCommittee()
	require.NoError(t, err)
	committeeKeys := currentSyncCommittee.Pubkeys
	balances := st.Balances()

	proposerIndex, err := helpers.BeaconProposerIndex(context.Background(), beaconState)
	require.NoError(t, err)

	for i := 0; i < len(syncBits); i++ {
		if syncBits.BitAt(uint64(i)) {
			pk := bytesutil.ToBytes48(committeeKeys[i])
			require.DeepEqual(t, true, votedMap[pk])
			idx, ok := st.ValidatorIndexByPubkey(pk)
			require.Equal(t, true, ok)
			require.Equal(t, uint64(32000000988), balances[idx])
		} else {
			pk := bytesutil.ToBytes48(committeeKeys[i])
			require.DeepEqual(t, false, votedMap[pk])
			idx, ok := st.ValidatorIndexByPubkey(pk)
			require.Equal(t, true, ok)
			if idx != proposerIndex {
				require.Equal(t, uint64(31999999012), balances[idx])
			}
		}
	}
	require.Equal(t, uint64(32000035108), balances[proposerIndex])
}

func Test_VerifySyncCommitteeSig(t *testing.T) {
	beaconState, privKeys := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().MaxValidatorsPerCommittee)
	require.NoError(t, beaconState.SetSlot(1))
	committee, err := altair.NextSyncCommittee(context.Background(), beaconState)
	require.NoError(t, err)
	require.NoError(t, beaconState.SetCurrentSyncCommittee(committee))

	syncBits := bitfield.NewBitvector512()
	for i := range syncBits {
		syncBits[i] = 0xff
	}
	indices, err := altair.NextSyncCommitteeIndices(context.Background(), beaconState)
	require.NoError(t, err)
	ps := slots.PrevSlot(beaconState.Slot())
	pbr, err := helpers.BlockRootAtSlot(beaconState, ps)
	require.NoError(t, err)
	sigs := make([]bls.Signature, len(indices))
	pks := make([]bls.PublicKey, len(indices))
	for i, indice := range indices {
		b := p2pType.SSZBytes(pbr)
		sb, err := signing.ComputeDomainAndSign(beaconState, time.CurrentEpoch(beaconState), &b, params.BeaconConfig().DomainSyncCommittee, privKeys[indice])
		require.NoError(t, err)
		sig, err := bls.SignatureFromBytes(sb)
		require.NoError(t, err)
		sigs[i] = sig
		pks[i] = privKeys[indice].PublicKey()
	}
	aggregatedSig := bls.AggregateSignatures(sigs).Marshal()

	blsKey, err := bls.RandKey()
	require.NoError(t, err)
	require.ErrorContains(t, "invalid sync committee signature", altair.VerifySyncCommitteeSig(beaconState, pks, blsKey.Sign([]byte{'m', 'e', 'o', 'w'}).Marshal()))

	require.NoError(t, altair.VerifySyncCommitteeSig(beaconState, pks, aggregatedSig))
}

func Test_SyncRewards(t *testing.T) {
	tests := []struct {
		name                  string
		activeBalance         uint64
		wantProposerReward    uint64
		wantParticipantReward uint64
		errString             string
	}{
		{
			name:                  "active balance is 0",
			activeBalance:         0,
			wantProposerReward:    0,
			wantParticipantReward: 0,
			errString:             "active balance can't be 0",
		},
		{
			name:                  "active balance is 1",
			activeBalance:         1,
			wantProposerReward:    0,
			wantParticipantReward: 0,
			errString:             "",
		},
		{
			name:                  "active balance is 1eth",
			activeBalance:         params.BeaconConfig().EffectiveBalanceIncrement,
			wantProposerReward:    0,
			wantParticipantReward: 3,
			errString:             "",
		},
		{
			name:                  "active balance is 32eth",
			activeBalance:         params.BeaconConfig().MaxEffectiveBalance,
			wantProposerReward:    3,
			wantParticipantReward: 21,
			errString:             "",
		},
		{
			name:                  "active balance is 32eth * 1m validators",
			activeBalance:         params.BeaconConfig().MaxEffectiveBalance * 1e9,
			wantProposerReward:    62780,
			wantParticipantReward: 439463,
			errString:             "",
		},
		{
			name:                  "active balance is max uint64",
			activeBalance:         math.MaxUint64,
			wantProposerReward:    70368,
			wantParticipantReward: 492581,
			errString:             "",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			proposerReward, participantReward, err := altair.SyncRewards(tt.activeBalance)
			if (err != nil) && (tt.errString != "") {
				require.ErrorContains(t, tt.errString, err)
				return
			}
			require.Equal(t, tt.wantProposerReward, proposerReward)
			require.Equal(t, tt.wantParticipantReward, participantReward)
		})
	}
}
