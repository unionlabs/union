package state_native

import (
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
)

// CurrentSyncCommittee of the current sync committee in beacon chain state.
func (b *BeaconState) CurrentSyncCommittee() (*ethpb.SyncCommittee, error) {
	b.lock.RLock()
	defer b.lock.RUnlock()

	if b.version == version.Phase0 {
		return nil, errNotSupported("CurrentSyncCommittee", b.version)
	}

	if b.currentSyncCommittee == nil {
		return nil, nil
	}

	return b.currentSyncCommitteeVal(), nil
}

// currentSyncCommitteeVal of the current sync committee in beacon chain state.
// This assumes that a lock is already held on BeaconState.
func (b *BeaconState) currentSyncCommitteeVal() *ethpb.SyncCommittee {
	return copySyncCommittee(b.currentSyncCommittee)
}

// NextSyncCommittee of the next sync committee in beacon chain state.
func (b *BeaconState) NextSyncCommittee() (*ethpb.SyncCommittee, error) {
	b.lock.RLock()
	defer b.lock.RUnlock()

	if b.version == version.Phase0 {
		return nil, errNotSupported("NextSyncCommittee", b.version)
	}

	if b.nextSyncCommittee == nil {
		return nil, nil
	}

	return b.nextSyncCommitteeVal(), nil
}

// nextSyncCommitteeVal of the next sync committee in beacon chain state.
// This assumes that a lock is already held on BeaconState.
func (b *BeaconState) nextSyncCommitteeVal() *ethpb.SyncCommittee {
	return copySyncCommittee(b.nextSyncCommittee)
}

// copySyncCommittee copies the provided sync committee object.
func copySyncCommittee(data *ethpb.SyncCommittee) *ethpb.SyncCommittee {
	if data == nil {
		return nil
	}
	return &ethpb.SyncCommittee{
		Pubkeys:         bytesutil.SafeCopy2dBytes(data.Pubkeys),
		AggregatePubkey: bytesutil.SafeCopyBytes(data.AggregatePubkey),
	}
}
