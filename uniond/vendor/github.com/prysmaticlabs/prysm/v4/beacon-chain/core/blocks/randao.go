package blocks

import (
	"context"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/crypto/hash"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// ProcessRandao checks the block proposer's
// randao commitment and generates a new randao mix to update
// in the beacon state's latest randao mixes slice.
//
// Spec pseudocode definition:
//
//	def process_randao(state: BeaconState, body: ReadOnlyBeaconBlockBody) -> None:
//	 epoch = get_current_epoch(state)
//	 # Verify RANDAO reveal
//	 proposer = state.validators[get_beacon_proposer_index(state)]
//	 signing_root = compute_signing_root(epoch, get_domain(state, DOMAIN_RANDAO))
//	 assert bls.Verify(proposer.pubkey, signing_root, body.randao_reveal)
//	 # Mix in RANDAO reveal
//	 mix = xor(get_randao_mix(state, epoch), hash(body.randao_reveal))
//	 state.randao_mixes[epoch % EPOCHS_PER_HISTORICAL_VECTOR] = mix
func ProcessRandao(
	ctx context.Context,
	beaconState state.BeaconState,
	b interfaces.ReadOnlySignedBeaconBlock,
) (state.BeaconState, error) {
	if err := blocks.BeaconBlockIsNil(b); err != nil {
		return nil, err
	}
	body := b.Block().Body()
	buf, proposerPub, domain, err := randaoSigningData(ctx, beaconState)
	if err != nil {
		return nil, err
	}

	randaoReveal := body.RandaoReveal()
	if err := verifySignature(buf, proposerPub, randaoReveal[:], domain); err != nil {
		return nil, errors.Wrap(err, "could not verify block randao")
	}

	beaconState, err = ProcessRandaoNoVerify(beaconState, randaoReveal[:])
	if err != nil {
		return nil, errors.Wrap(err, "could not process randao")
	}
	return beaconState, nil
}

// ProcessRandaoNoVerify generates a new randao mix to update
// in the beacon state's latest randao mixes slice.
//
// Spec pseudocode definition:
//
//	# Mix it in
//	state.latest_randao_mixes[get_current_epoch(state) % LATEST_RANDAO_MIXES_LENGTH] = (
//	    xor(get_randao_mix(state, get_current_epoch(state)),
//	        hash(body.randao_reveal))
//	)
func ProcessRandaoNoVerify(
	beaconState state.BeaconState,
	randaoReveal []byte,
) (state.BeaconState, error) {
	currentEpoch := slots.ToEpoch(beaconState.Slot())
	// If block randao passed verification, we XOR the state's latest randao mix with the block's
	// randao and update the state's corresponding latest randao mix value.
	latestMixesLength := params.BeaconConfig().EpochsPerHistoricalVector
	latestMixSlice, err := beaconState.RandaoMixAtIndex(uint64(currentEpoch % latestMixesLength))
	if err != nil {
		return nil, err
	}
	blockRandaoReveal := hash.Hash(randaoReveal)
	if len(blockRandaoReveal) != len(latestMixSlice) {
		return nil, errors.New("blockRandaoReveal length doesn't match latestMixSlice length")
	}
	for i, x := range blockRandaoReveal {
		latestMixSlice[i] ^= x
	}
	if err := beaconState.UpdateRandaoMixesAtIndex(uint64(currentEpoch%latestMixesLength), [32]byte(latestMixSlice)); err != nil {
		return nil, err
	}
	return beaconState, nil
}
