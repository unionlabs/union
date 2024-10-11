package helpers

import (
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
)

// Seed returns the randao seed used for shuffling of a given epoch.
//
// Spec pseudocode definition:
//
//	def get_seed(state: BeaconState, epoch: Epoch, domain_type: DomainType) -> Bytes32:
//	  """
//	  Return the seed at ``epoch``.
//	  """
//	  mix = get_randao_mix(state, Epoch(epoch + EPOCHS_PER_HISTORICAL_VECTOR - MIN_SEED_LOOKAHEAD - 1))  # Avoid underflow
//	  return hash(domain_type + uint_to_bytes(epoch) + mix)
func Seed(state state.ReadOnlyBeaconState, epoch primitives.Epoch, domain [bls.DomainByteLength]byte) ([32]byte, error) {
	// See https://github.com/ethereum/consensus-specs/pull/1296 for
	// rationale on why offset has to look down by 1.
	lookAheadEpoch := epoch + params.BeaconConfig().EpochsPerHistoricalVector -
		params.BeaconConfig().MinSeedLookahead - 1

	randaoMix, err := RandaoMix(state, lookAheadEpoch)
	if err != nil {
		return [32]byte{}, err
	}
	seed := append(domain[:], bytesutil.Bytes8(uint64(epoch))...)
	seed = append(seed, randaoMix...)

	seed32 := hash.Hash(seed)

	return seed32, nil
}

// RandaoMix returns the randao mix (xor'ed seed)
// of a given slot. It is used to shuffle validators.
//
// Spec pseudocode definition:
//
//	def get_randao_mix(state: BeaconState, epoch: Epoch) -> Bytes32:
//	 """
//	 Return the randao mix at a recent ``epoch``.
//	 """
//	 return state.randao_mixes[epoch % EPOCHS_PER_HISTORICAL_VECTOR]
func RandaoMix(state state.ReadOnlyBeaconState, epoch primitives.Epoch) ([]byte, error) {
	return state.RandaoMixAtIndex(uint64(epoch % params.BeaconConfig().EpochsPerHistoricalVector))
}
