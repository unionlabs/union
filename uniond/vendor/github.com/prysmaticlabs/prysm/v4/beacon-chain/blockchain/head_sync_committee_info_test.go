package blockchain

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	dbTest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

func TestService_HeadSyncCommitteeIndices(t *testing.T) {
	s, _ := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().TargetCommitteeSize)
	c := &Service{cfg: &config{BeaconDB: dbTest.SetupDB(t)}}
	c.head = &head{state: s}

	// Current period
	slot := 2*uint64(params.BeaconConfig().EpochsPerSyncCommitteePeriod)*uint64(params.BeaconConfig().SlotsPerEpoch) + 1
	a, err := c.HeadSyncCommitteeIndices(context.Background(), 0, primitives.Slot(slot))
	require.NoError(t, err)

	// Current period where slot-2 across EPOCHS_PER_SYNC_COMMITTEE_PERIOD
	slot = 3*uint64(params.BeaconConfig().EpochsPerSyncCommitteePeriod)*uint64(params.BeaconConfig().SlotsPerEpoch) - 2
	b, err := c.HeadSyncCommitteeIndices(context.Background(), 0, primitives.Slot(slot))
	require.NoError(t, err)
	require.DeepEqual(t, a, b)

	// Next period where slot-1 across EPOCHS_PER_SYNC_COMMITTEE_PERIOD
	slot = 3*uint64(params.BeaconConfig().EpochsPerSyncCommitteePeriod)*uint64(params.BeaconConfig().SlotsPerEpoch) - 1
	b, err = c.HeadSyncCommitteeIndices(context.Background(), 0, primitives.Slot(slot))
	require.NoError(t, err)
	require.DeepNotEqual(t, a, b)
}

func TestService_headCurrentSyncCommitteeIndices(t *testing.T) {
	s, _ := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().TargetCommitteeSize)
	c := &Service{cfg: &config{BeaconDB: dbTest.SetupDB(t)}}
	c.head = &head{state: s}

	// Process slot up to `EpochsPerSyncCommitteePeriod` so it can `ProcessSyncCommitteeUpdates`.
	slot := uint64(params.BeaconConfig().EpochsPerSyncCommitteePeriod)*uint64(params.BeaconConfig().SlotsPerEpoch) + 1
	indices, err := c.headCurrentSyncCommitteeIndices(context.Background(), 0, primitives.Slot(slot))
	require.NoError(t, err)

	// NextSyncCommittee becomes CurrentSyncCommittee so it should be empty by default.
	require.Equal(t, 0, len(indices))
}

func TestService_headNextSyncCommitteeIndices(t *testing.T) {
	s, _ := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().TargetCommitteeSize)
	c := &Service{}
	c.head = &head{state: s}

	// Process slot up to `EpochsPerSyncCommitteePeriod` so it can `ProcessSyncCommitteeUpdates`.
	slot := uint64(params.BeaconConfig().EpochsPerSyncCommitteePeriod)*uint64(params.BeaconConfig().SlotsPerEpoch) + 1
	indices, err := c.headNextSyncCommitteeIndices(context.Background(), 0, primitives.Slot(slot))
	require.NoError(t, err)

	// NextSyncCommittee should be be empty after `ProcessSyncCommitteeUpdates`. Validator should get indices.
	require.NotEqual(t, 0, len(indices))
}

func TestService_HeadSyncCommitteePubKeys(t *testing.T) {
	s, _ := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().TargetCommitteeSize)
	c := &Service{cfg: &config{BeaconDB: dbTest.SetupDB(t)}}
	c.head = &head{state: s}

	// Process slot up to 2 * `EpochsPerSyncCommitteePeriod` so it can run `ProcessSyncCommitteeUpdates` twice.
	slot := uint64(2*params.BeaconConfig().EpochsPerSyncCommitteePeriod)*uint64(params.BeaconConfig().SlotsPerEpoch) + 1
	pubkeys, err := c.HeadSyncCommitteePubKeys(context.Background(), primitives.Slot(slot), 0)
	require.NoError(t, err)

	// Any subcommittee should match the subcommittee size.
	subCommitteeSize := params.BeaconConfig().SyncCommitteeSize / params.BeaconConfig().SyncCommitteeSubnetCount
	require.Equal(t, int(subCommitteeSize), len(pubkeys))
}

func TestService_HeadSyncCommitteeDomain(t *testing.T) {
	s, _ := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().TargetCommitteeSize)
	c := &Service{cfg: &config{BeaconDB: dbTest.SetupDB(t)}}
	c.head = &head{state: s}

	wanted, err := signing.Domain(s.Fork(), slots.ToEpoch(s.Slot()), params.BeaconConfig().DomainSyncCommittee, s.GenesisValidatorsRoot())
	require.NoError(t, err)

	d, err := c.HeadSyncCommitteeDomain(context.Background(), 0)
	require.NoError(t, err)

	require.DeepEqual(t, wanted, d)
}

func TestService_HeadSyncContributionProofDomain(t *testing.T) {
	s, _ := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().TargetCommitteeSize)
	c := &Service{}
	c.head = &head{state: s}

	wanted, err := signing.Domain(s.Fork(), slots.ToEpoch(s.Slot()), params.BeaconConfig().DomainContributionAndProof, s.GenesisValidatorsRoot())
	require.NoError(t, err)

	d, err := c.HeadSyncContributionProofDomain(context.Background(), 0)
	require.NoError(t, err)

	require.DeepEqual(t, wanted, d)
}

func TestService_HeadSyncSelectionProofDomain(t *testing.T) {
	s, _ := util.DeterministicGenesisStateAltair(t, params.BeaconConfig().TargetCommitteeSize)
	c := &Service{}
	c.head = &head{state: s}

	wanted, err := signing.Domain(s.Fork(), slots.ToEpoch(s.Slot()), params.BeaconConfig().DomainSyncCommitteeSelectionProof, s.GenesisValidatorsRoot())
	require.NoError(t, err)

	d, err := c.HeadSyncSelectionProofDomain(context.Background(), 0)
	require.NoError(t, err)

	require.DeepEqual(t, wanted, d)
}

func TestSyncCommitteeHeadStateCache_RoundTrip(t *testing.T) {
	c := syncCommitteeHeadStateCache
	t.Cleanup(func() {
		syncCommitteeHeadStateCache = cache.NewSyncCommitteeHeadState()
	})
	beaconState, _ := util.DeterministicGenesisStateAltair(t, 100)
	require.NoError(t, beaconState.SetSlot(100))
	cachedState, err := c.Get(101)
	require.ErrorContains(t, cache.ErrNotFound.Error(), err)
	require.Equal(t, nil, cachedState)
	require.NoError(t, c.Put(101, beaconState))
	cachedState, err = c.Get(101)
	require.NoError(t, err)
	require.DeepEqual(t, beaconState, cachedState)
}
