package helpers

import (
	"encoding/binary"
	"testing"

	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestRandaoMix_OK(t *testing.T) {
	randaoMixes := make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector)
	for i := 0; i < len(randaoMixes); i++ {
		intInBytes := make([]byte, 32)
		binary.LittleEndian.PutUint64(intInBytes, uint64(i))
		randaoMixes[i] = intInBytes
	}
	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{RandaoMixes: randaoMixes})
	require.NoError(t, err)
	tests := []struct {
		epoch     primitives.Epoch
		randaoMix []byte
	}{
		{
			epoch:     10,
			randaoMix: randaoMixes[10],
		},
		{
			epoch:     2344,
			randaoMix: randaoMixes[2344],
		},
		{
			epoch:     99999,
			randaoMix: randaoMixes[99999%params.BeaconConfig().EpochsPerHistoricalVector],
		},
	}
	for _, test := range tests {
		ClearCache()

		require.NoError(t, state.SetSlot(params.BeaconConfig().SlotsPerEpoch.Mul(uint64(test.epoch+1))))
		mix, err := RandaoMix(state, test.epoch)
		require.NoError(t, err)
		assert.DeepEqual(t, test.randaoMix, mix, "Incorrect randao mix")
	}
}

func TestRandaoMix_CopyOK(t *testing.T) {
	randaoMixes := make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector)
	for i := 0; i < len(randaoMixes); i++ {
		intInBytes := make([]byte, 32)
		binary.LittleEndian.PutUint64(intInBytes, uint64(i))
		randaoMixes[i] = intInBytes
	}
	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{RandaoMixes: randaoMixes})
	require.NoError(t, err)
	tests := []struct {
		epoch     primitives.Epoch
		randaoMix []byte
	}{
		{
			epoch:     10,
			randaoMix: randaoMixes[10],
		},
		{
			epoch:     2344,
			randaoMix: randaoMixes[2344],
		},
		{
			epoch:     99999,
			randaoMix: randaoMixes[99999%params.BeaconConfig().EpochsPerHistoricalVector],
		},
	}
	for _, test := range tests {
		ClearCache()

		require.NoError(t, state.SetSlot(params.BeaconConfig().SlotsPerEpoch.Mul(uint64(test.epoch+1))))
		mix, err := RandaoMix(state, test.epoch)
		require.NoError(t, err)
		uniqueNumber := uint64(params.BeaconConfig().EpochsPerHistoricalVector.Add(1000))
		binary.LittleEndian.PutUint64(mix, uniqueNumber)

		for _, mx := range randaoMixes {
			mxNum := bytesutil.FromBytes8(mx)
			assert.NotEqual(t, uniqueNumber, mxNum, "two distinct slices which have different representations in memory still contain the same value: %d", mxNum)
		}
	}
}

func TestGenerateSeed_OK(t *testing.T) {
	ClearCache()

	randaoMixes := make([][]byte, params.BeaconConfig().EpochsPerHistoricalVector)
	for i := 0; i < len(randaoMixes); i++ {
		intInBytes := make([]byte, 32)
		binary.LittleEndian.PutUint64(intInBytes, uint64(i))
		randaoMixes[i] = intInBytes
	}
	slot := params.BeaconConfig().SlotsPerEpoch.Mul(uint64(params.BeaconConfig().MinSeedLookahead * 10))
	state, err := state_native.InitializeFromProtoPhase0(&ethpb.BeaconState{
		RandaoMixes: randaoMixes,
		Slot:        slot,
	})
	require.NoError(t, err)

	got, err := Seed(state, 10, params.BeaconConfig().DomainBeaconAttester)
	require.NoError(t, err)

	wanted := [32]byte{102, 82, 23, 40, 226, 79, 171, 11, 203, 23, 175, 7, 88, 202, 80,
		103, 68, 126, 195, 143, 190, 249, 210, 85, 138, 196, 158, 208, 11, 18, 136, 23}
	assert.Equal(t, wanted, got, "Incorrect generated seeds")
}
