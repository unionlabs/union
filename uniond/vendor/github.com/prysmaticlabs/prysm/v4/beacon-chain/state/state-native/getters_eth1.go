package state_native

import (
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// Eth1Data corresponding to the proof-of-work chain information stored in the beacon state.
func (b *BeaconState) Eth1Data() *ethpb.Eth1Data {
	if b.eth1Data == nil {
		return nil
	}

	b.lock.RLock()
	defer b.lock.RUnlock()

	return b.eth1DataVal()
}

// eth1DataVal corresponding to the proof-of-work chain information stored in the beacon state.
// This assumes that a lock is already held on BeaconState.
func (b *BeaconState) eth1DataVal() *ethpb.Eth1Data {
	if b.eth1Data == nil {
		return nil
	}

	return ethpb.CopyETH1Data(b.eth1Data)
}

// Eth1DataVotes corresponds to votes from Ethereum on the canonical proof-of-work chain
// data retrieved from eth1.
func (b *BeaconState) Eth1DataVotes() []*ethpb.Eth1Data {
	if b.eth1DataVotes == nil {
		return nil
	}

	b.lock.RLock()
	defer b.lock.RUnlock()

	return b.eth1DataVotesVal()
}

// eth1DataVotesVal corresponds to votes from Ethereum on the canonical proof-of-work chain
// data retrieved from eth1.
// This assumes that a lock is already held on BeaconState.
func (b *BeaconState) eth1DataVotesVal() []*ethpb.Eth1Data {
	if b.eth1DataVotes == nil {
		return nil
	}

	res := make([]*ethpb.Eth1Data, len(b.eth1DataVotes))
	for i := 0; i < len(res); i++ {
		res[i] = ethpb.CopyETH1Data(b.eth1DataVotes[i])
	}
	return res
}

// Eth1DepositIndex corresponds to the index of the deposit made to the
// validator deposit contract at the time of this state's eth1 data.
func (b *BeaconState) Eth1DepositIndex() uint64 {
	b.lock.RLock()
	defer b.lock.RUnlock()

	return b.eth1DepositIndex
}
