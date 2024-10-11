// Package benchmark contains useful helpers
// for pregenerating filled data structures such as blocks/states for benchmarks.
package benchmark

import (
	"fmt"
	"os"

	"github.com/bazelbuild/rules_go/go/tools/bazel"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

// ValidatorCount is for declaring how many validators the benchmarks will be
// performed with. Default is 16384 or 524K ETH staked.
var ValidatorCount = uint64(16384)

// AttestationsPerEpoch represents the requested amount attestations in an epoch.
// This affects the amount of attestations in a fully attested for block and the amount
// of attestations in the state per epoch, so a full 2 epochs should result in twice
// this amount of attestations in the state. Default is 128.
var AttestationsPerEpoch = uint64(128)

// GenesisFileName is the generated genesis beacon state file name.
var GenesisFileName = fmt.Sprintf("bStateGenesis-%dAtts-%dVals.ssz", AttestationsPerEpoch, ValidatorCount)

// BState1EpochFileName is the generated beacon state after 1 skipped epoch file name.
var BState1EpochFileName = fmt.Sprintf("bState1Epoch-%dAtts-%dVals.ssz", AttestationsPerEpoch, ValidatorCount)

// BstateEpochFileName is the generated beacon state after 2 full epochs file name.
var BstateEpochFileName = fmt.Sprintf("bState2Epochs-%dAtts-%dVals.ssz", AttestationsPerEpoch, ValidatorCount)

// FullBlockFileName is the generated full block file name.
var FullBlockFileName = fmt.Sprintf("fullBlock-%dAtts-%dVals.ssz", AttestationsPerEpoch, ValidatorCount)

func filePath(path string) string {
	return fmt.Sprintf("testing/benchmark/benchmark_files/%s", path)
}

// PreGenState1Epoch unmarshals the pre-generated beacon state after 1 epoch of block processing and returns it.
func PreGenState1Epoch() (state.BeaconState, error) {
	path, err := bazel.Runfile(filePath(BState1EpochFileName))
	if err != nil {
		return nil, err
	}
	beaconBytes, err := os.ReadFile(path) // #nosec G304
	if err != nil {
		return nil, err
	}
	beaconState := &ethpb.BeaconState{}
	if err := beaconState.UnmarshalSSZ(beaconBytes); err != nil {
		return nil, err
	}
	return state_native.InitializeFromProtoPhase0(beaconState)
}

// PreGenstateFullEpochs unmarshals the pre-generated beacon state after 2 epoch of full block processing and returns it.
func PreGenstateFullEpochs() (state.BeaconState, error) {
	path, err := bazel.Runfile(filePath(BstateEpochFileName))
	if err != nil {
		return nil, err
	}
	beaconBytes, err := os.ReadFile(path) // #nosec G304
	if err != nil {
		return nil, err
	}
	beaconState := &ethpb.BeaconState{}
	if err := beaconState.UnmarshalSSZ(beaconBytes); err != nil {
		return nil, err
	}
	return state_native.InitializeFromProtoPhase0(beaconState)
}

// PreGenFullBlock unmarshals the pre-generated signed beacon block containing an epochs worth of attestations and returns it.
func PreGenFullBlock() (*ethpb.SignedBeaconBlock, error) {
	path, err := bazel.Runfile(filePath(FullBlockFileName))
	if err != nil {
		return nil, err
	}
	blockBytes, err := os.ReadFile(path) // #nosec G304
	if err != nil {
		return nil, err
	}
	beaconBlock := &ethpb.SignedBeaconBlock{}
	if err := beaconBlock.UnmarshalSSZ(blockBytes); err != nil {
		return nil, err
	}
	return beaconBlock, nil
}

// SetBenchmarkConfig changes the beacon config to match the requested amount of
// attestations set to AttestationsPerEpoch.
func SetBenchmarkConfig() (func(), error) {
	maxAtts := AttestationsPerEpoch
	slotsPerEpoch := uint64(params.BeaconConfig().SlotsPerEpoch)
	committeeSize := (ValidatorCount / slotsPerEpoch) / (maxAtts / slotsPerEpoch)
	c := params.BeaconConfig().Copy()
	c.ShardCommitteePeriod = 0
	c.MinValidatorWithdrawabilityDelay = 0
	c.TargetCommitteeSize = committeeSize
	c.MaxAttestations = maxAtts
	undo, err := params.SetActiveWithUndo(c)
	return func() {
		if err := undo(); err != nil {
			panic(err)
		}
	}, err
}
