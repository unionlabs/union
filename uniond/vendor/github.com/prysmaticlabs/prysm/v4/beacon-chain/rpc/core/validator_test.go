package core

import (
	"encoding/binary"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/validator"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

func TestRegisterSyncSubnetProto(t *testing.T) {
	k := pubKey(3)
	committee := make([][]byte, 0)

	for i := 0; i < 100; i++ {
		committee = append(committee, pubKey(uint64(i)))
	}
	sCommittee := &ethpb.SyncCommittee{
		Pubkeys: committee,
	}
	registerSyncSubnetProto(0, 0, k, sCommittee, ethpb.ValidatorStatus_ACTIVE)
	coms, _, ok, exp := cache.SyncSubnetIDs.GetSyncCommitteeSubnets(k, 0)
	require.Equal(t, true, ok, "No cache entry found for validator")
	assert.Equal(t, uint64(1), uint64(len(coms)))
	epochDuration := time.Duration(params.BeaconConfig().SlotsPerEpoch.Mul(params.BeaconConfig().SecondsPerSlot))
	totalTime := time.Duration(params.BeaconConfig().EpochsPerSyncCommitteePeriod) * epochDuration * time.Second
	receivedTime := time.Until(exp.Round(time.Second)).Round(time.Second)
	if receivedTime < totalTime {
		t.Fatalf("Expiration time of %f was less than expected duration of %f ", receivedTime.Seconds(), totalTime.Seconds())
	}
}

func TestRegisterSyncSubnet(t *testing.T) {
	k := pubKey(3)
	committee := make([][]byte, 0)

	for i := 0; i < 100; i++ {
		committee = append(committee, pubKey(uint64(i)))
	}
	sCommittee := &ethpb.SyncCommittee{
		Pubkeys: committee,
	}
	registerSyncSubnet(0, 0, k, sCommittee, validator.Active)
	coms, _, ok, exp := cache.SyncSubnetIDs.GetSyncCommitteeSubnets(k, 0)
	require.Equal(t, true, ok, "No cache entry found for validator")
	assert.Equal(t, uint64(1), uint64(len(coms)))
	epochDuration := time.Duration(params.BeaconConfig().SlotsPerEpoch.Mul(params.BeaconConfig().SecondsPerSlot))
	totalTime := time.Duration(params.BeaconConfig().EpochsPerSyncCommitteePeriod) * epochDuration * time.Second
	receivedTime := time.Until(exp.Round(time.Second)).Round(time.Second)
	if receivedTime < totalTime {
		t.Fatalf("Expiration time of %f was less than expected duration of %f ", receivedTime.Seconds(), totalTime.Seconds())
	}
}

// pubKey is a helper to generate a well-formed public key.
func pubKey(i uint64) []byte {
	pubKey := make([]byte, params.BeaconConfig().BLSPubkeyLength)
	binary.LittleEndian.PutUint64(pubKey, i)
	return pubKey
}
