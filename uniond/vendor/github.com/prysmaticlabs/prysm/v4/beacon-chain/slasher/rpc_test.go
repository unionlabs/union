package slasher

import (
	"context"
	"testing"
	"time"

	dbtest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	slashertypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/slasher/types"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestIsSlashableBlock(t *testing.T) {
	ctx := context.Background()
	slasherDB := dbtest.SetupSlasherDB(t)
	s := &Service{
		serviceCfg: &ServiceConfig{
			Database: slasherDB,
		},
		params:    DefaultParams(),
		blksQueue: newBlocksQueue(),
	}
	err := slasherDB.SaveBlockProposals(ctx, []*slashertypes.SignedBlockHeaderWrapper{
		createProposalWrapper(t, 2, 3, []byte{1}),
		createProposalWrapper(t, 3, 3, []byte{1}),
	})
	require.NoError(t, err)
	tests := []struct {
		name              string
		blockToCheck      *slashertypes.SignedBlockHeaderWrapper
		shouldBeSlashable bool
	}{
		{
			name:              "should not detect if same signing root",
			blockToCheck:      createProposalWrapper(t, 2, 3, []byte{1}),
			shouldBeSlashable: false,
		},
		{
			name:              "should not detect if different slot",
			blockToCheck:      createProposalWrapper(t, 1, 3, []byte{2}),
			shouldBeSlashable: false,
		},
		{
			name:              "should not detect if different validator index",
			blockToCheck:      createProposalWrapper(t, 2, 4, []byte{2}),
			shouldBeSlashable: false,
		},
		{
			name:              "detects differing signing root",
			blockToCheck:      createProposalWrapper(t, 2, 3, []byte{2}),
			shouldBeSlashable: true,
		},
		{
			name:              "should detect another slot",
			blockToCheck:      createProposalWrapper(t, 3, 3, []byte{2}),
			shouldBeSlashable: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			proposerSlashing, err := s.IsSlashableBlock(ctx, tt.blockToCheck.SignedBeaconBlockHeader)
			require.NoError(t, err)
			assert.Equal(t, tt.shouldBeSlashable, proposerSlashing != nil)
		})
	}
}

func TestIsSlashableAttestation(t *testing.T) {
	ctx := context.Background()
	slasherDB := dbtest.SetupSlasherDB(t)

	currentEpoch := primitives.Epoch(3)
	currentTime := time.Now()
	totalSlots := uint64(currentEpoch) * uint64(params.BeaconConfig().SlotsPerEpoch)
	secondsSinceGenesis := time.Duration(totalSlots * params.BeaconConfig().SecondsPerSlot)
	genesisTime := currentTime.Add(-secondsSinceGenesis * time.Second)

	s := &Service{
		serviceCfg: &ServiceConfig{
			Database: slasherDB,
		},
		params:                         DefaultParams(),
		blksQueue:                      newBlocksQueue(),
		genesisTime:                    genesisTime,
		latestEpochWrittenForValidator: map[primitives.ValidatorIndex]primitives.Epoch{},
	}
	prevAtts := []*slashertypes.IndexedAttestationWrapper{
		createAttestationWrapper(t, 2, 3, []uint64{0}, []byte{1}),
		createAttestationWrapper(t, 2, 3, []uint64{1}, []byte{1}),
	}
	err := slasherDB.SaveAttestationRecordsForValidators(ctx, prevAtts)
	require.NoError(t, err)
	attesterSlashings, err := s.checkSlashableAttestations(ctx, currentEpoch, prevAtts)
	require.NoError(t, err)
	require.Equal(t, 0, len(attesterSlashings))

	tests := []struct {
		name         string
		attToCheck   *slashertypes.IndexedAttestationWrapper
		amtSlashable uint64
	}{
		{
			name:         "should not detect if same attestation data",
			attToCheck:   createAttestationWrapper(t, 2, 3, []uint64{1}, []byte{1}),
			amtSlashable: 0,
		},
		{
			name:         "should not detect if different index",
			attToCheck:   createAttestationWrapper(t, 0, 3, []uint64{2}, []byte{2}),
			amtSlashable: 0,
		},
		{
			name:         "should detect double if same index",
			attToCheck:   createAttestationWrapper(t, 0, 3, []uint64{0}, []byte{2}),
			amtSlashable: 1,
		},
		{
			name:         "should detect multiple double if multiple same indices",
			attToCheck:   createAttestationWrapper(t, 0, 3, []uint64{0, 1}, []byte{2}),
			amtSlashable: 2,
		},
		{
			name:         "should detect multiple surround if multiple same indices",
			attToCheck:   createAttestationWrapper(t, 1, 4, []uint64{0, 1}, []byte{2}),
			amtSlashable: 4,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			attesterSlashings, err = s.IsSlashableAttestation(ctx, tt.attToCheck.IndexedAttestation)
			require.NoError(t, err)
			assert.Equal(t, tt.amtSlashable, uint64(len(attesterSlashings)))
		})
	}
}

func TestService_HighestAttestations(t *testing.T) {
	ctx := context.Background()
	slasherDB := dbtest.SetupSlasherDB(t)

	currentEpoch := primitives.Epoch(3)
	currentTime := time.Now()
	totalSlots := uint64(currentEpoch) * uint64(params.BeaconConfig().SlotsPerEpoch)
	secondsSinceGenesis := time.Duration(totalSlots * params.BeaconConfig().SecondsPerSlot)
	genesisTime := currentTime.Add(-secondsSinceGenesis * time.Second)

	s := &Service{
		serviceCfg: &ServiceConfig{
			Database: slasherDB,
		},
		params:      DefaultParams(),
		blksQueue:   newBlocksQueue(),
		genesisTime: genesisTime,
	}
	prevAtts := []*slashertypes.IndexedAttestationWrapper{
		createAttestationWrapper(t, 0, 1, []uint64{1}, []byte{1}),
		createAttestationWrapper(t, 2, 3, []uint64{2}, []byte{1}),
	}
	err := slasherDB.SaveAttestationRecordsForValidators(ctx, prevAtts)
	require.NoError(t, err)
	t.Run("single index not found", func(t *testing.T) {
		atts, err := s.HighestAttestations(ctx, []primitives.ValidatorIndex{0})
		require.NoError(t, err)
		require.Equal(t, 0, len(atts))
	})
	t.Run("single index case 1", func(t *testing.T) {
		atts, err := s.HighestAttestations(ctx, []primitives.ValidatorIndex{1})
		require.NoError(t, err)
		require.Equal(t, 1, len(atts))
		require.DeepEqual(t, &ethpb.HighestAttestation{ValidatorIndex: 1, HighestSourceEpoch: 0, HighestTargetEpoch: 1}, atts[0])
	})
	t.Run("single index case 2", func(t *testing.T) {
		atts, err := s.HighestAttestations(ctx, []primitives.ValidatorIndex{2})
		require.NoError(t, err)
		require.Equal(t, 1, len(atts))
		require.DeepEqual(t, &ethpb.HighestAttestation{ValidatorIndex: 2, HighestSourceEpoch: 2, HighestTargetEpoch: 3}, atts[0])
	})
	t.Run("multiple indices all found", func(t *testing.T) {
		atts, err := s.HighestAttestations(ctx, []primitives.ValidatorIndex{1, 2})
		require.NoError(t, err)
		require.Equal(t, 2, len(atts))
		require.DeepEqual(t, &ethpb.HighestAttestation{ValidatorIndex: 1, HighestSourceEpoch: 0, HighestTargetEpoch: 1}, atts[0])
		require.DeepEqual(t, &ethpb.HighestAttestation{ValidatorIndex: 2, HighestSourceEpoch: 2, HighestTargetEpoch: 3}, atts[1])
	})
	t.Run("multiple indices all not found", func(t *testing.T) {
		atts, err := s.HighestAttestations(ctx, []primitives.ValidatorIndex{3, 4})
		require.NoError(t, err)
		require.Equal(t, 0, len(atts))
	})
	t.Run("multiple indices some not found", func(t *testing.T) {
		atts, err := s.HighestAttestations(ctx, []primitives.ValidatorIndex{1, 4})
		require.NoError(t, err)
		require.Equal(t, 1, len(atts))
		require.DeepEqual(t, &ethpb.HighestAttestation{ValidatorIndex: 1, HighestSourceEpoch: 0, HighestTargetEpoch: 1}, atts[0])
	})
}
