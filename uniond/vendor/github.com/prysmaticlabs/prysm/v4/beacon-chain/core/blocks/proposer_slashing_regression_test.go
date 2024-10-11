package blocks_test

import (
	"os"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/blocks"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

// Beaconfuzz discovered an issue where a proposer slashing could be produced which would pass
// validation where we use the slashing's slot instead of the current epoch of our state for validation.
// This would lead to us accepting an invalid slashing by marking the respective validator as 'slashable'
// when it was not in actuality.
// See: https://github.com/sigp/beacon-fuzz/issues/91
func TestVerifyProposerSlashing_BeaconFuzzIssue91(t *testing.T) {
	file, err := os.ReadFile("testdata/beaconfuzz_91_beacon.ssz")
	require.NoError(t, err)
	rawState := &ethpb.BeaconState{}
	err = rawState.UnmarshalSSZ(file)
	require.NoError(t, err)

	st, err := state_native.InitializeFromProtoUnsafePhase0(rawState)
	require.NoError(t, err)

	file, err = os.ReadFile("testdata/beaconfuzz_91_proposer_slashing.ssz")
	require.NoError(t, err)
	slashing := &ethpb.ProposerSlashing{}
	err = slashing.UnmarshalSSZ(file)
	require.NoError(t, err)

	err = blocks.VerifyProposerSlashing(st, slashing)
	require.ErrorContains(t, "validator with key 0x97f1d3a73197d7942695638c4fa9ac0fc3688c4f9774b905a14e3a3f171bac586c55e83ff97a1aeffb3af00adb22c6bb is not slashable", err)
}
