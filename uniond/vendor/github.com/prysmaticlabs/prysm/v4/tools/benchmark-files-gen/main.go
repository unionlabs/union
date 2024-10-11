package main

import (
	"context"
	"flag"
	"os"
	"path"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	state_native "github.com/prysmaticlabs/prysm/v4/beacon-chain/state/state-native"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/io/file"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/interop"
	"github.com/prysmaticlabs/prysm/v4/testing/benchmark"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	log "github.com/sirupsen/logrus"
)

var (
	outputDir = flag.String("output-dir", "", "Directory to write SSZ files to")
	overwrite = flag.Bool("overwrite", false, "If SSZ files exist in the output directory, they will be overwritten")
)

func main() {
	flag.Parse()
	if *outputDir == "" {
		log.Fatal("Please specify --output-dir to write SSZ files to")
	}

	if !*overwrite {
		if _, err := os.Stat(path.Join(*outputDir, benchmark.BState1EpochFileName)); err == nil {
			log.Fatal("The file exists. Use a different file name or the --overwrite flag")
		}
		if _, err := os.Stat(path.Join(*outputDir, benchmark.BstateEpochFileName)); err == nil {
			log.Fatal("The file exists. Use a different file name or the --overwrite flag")
		}
		if _, err := os.Stat(path.Join(*outputDir, benchmark.FullBlockFileName)); err == nil {
			log.Fatal("The file exists. Use a different file name or the --overwrite flag")
		}
	}

	if err := file.MkdirAll(*outputDir); err != nil {
		log.Fatal(err)
	}

	undo, err := benchmark.SetBenchmarkConfig()
	if err != nil {
		log.Fatal(err)
	}
	defer undo()
	log.Printf("Output dir is: %s", *outputDir)
	log.Println("Generating genesis state")
	// Generating this for the 2 following states.
	if err := generateGenesisBeaconState(); err != nil {
		log.WithError(err).Fatal("Could not generate genesis state")
	}
	log.Println("Generating full block and state after 1 skipped epoch")
	if err := generateMarshalledFullStateAndBlock(); err != nil {
		log.WithError(err).Fatal("Could not generate full state and block")
	}
	log.Println("Generating state after 2 fully attested epochs")
	if err := generate2FullEpochState(); err != nil {
		log.WithError(err).Fatal("Could not generate 2 full epoch state")
	}
	// Removing the genesis state SSZ since its 10MB large and no longer needed.
	if err := os.Remove(path.Join(*outputDir, benchmark.GenesisFileName)); err != nil {
		log.Fatal(err)
	}
}

func generateGenesisBeaconState() error {
	genesisState, _, err := interop.GenerateGenesisState(context.Background(), 0, benchmark.ValidatorCount)
	if err != nil {
		return err
	}
	beaconBytes, err := genesisState.MarshalSSZ()
	if err != nil {
		return err
	}
	return file.WriteFile(path.Join(*outputDir, benchmark.GenesisFileName), beaconBytes)
}

func generateMarshalledFullStateAndBlock() error {
	beaconState, err := genesisBeaconState()
	if err != nil {
		return err
	}

	privs, _, err := interop.DeterministicallyGenerateKeys(0, benchmark.ValidatorCount)
	if err != nil {
		return err
	}

	conf := &util.BlockGenConfig{}
	slotsPerEpoch := params.BeaconConfig().SlotsPerEpoch
	// Small offset for the beacon state so we don't process a block on an epoch.
	slotOffset := primitives.Slot(2)
	block, err := util.GenerateFullBlock(beaconState, privs, conf, slotsPerEpoch+slotOffset)
	if err != nil {
		return err
	}
	wsb, err := blocks.NewSignedBeaconBlock(block)
	if err != nil {
		return err
	}
	beaconState, err = transition.ExecuteStateTransition(context.Background(), beaconState, wsb)
	if err != nil {
		return err
	}

	attConfig := &util.BlockGenConfig{
		NumAttestations: benchmark.AttestationsPerEpoch / uint64(slotsPerEpoch),
	}

	var atts []*ethpb.Attestation
	for i := slotOffset + 1; i < slotsPerEpoch+slotOffset; i++ {
		attsForSlot, err := util.GenerateAttestations(beaconState, privs, attConfig.NumAttestations, i, false)
		if err != nil {
			return err
		}
		atts = append(atts, attsForSlot...)
	}

	block, err = util.GenerateFullBlock(beaconState, privs, attConfig, beaconState.Slot())
	if err != nil {
		return errors.Wrap(err, "could not generate full block")
	}
	block.Block.Body.Attestations = append(atts, block.Block.Body.Attestations...)

	wsb, err = blocks.NewSignedBeaconBlock(block)
	if err != nil {
		return err
	}
	s, err := transition.CalculateStateRoot(context.Background(), beaconState, wsb)
	if err != nil {
		return errors.Wrap(err, "could not calculate state root")
	}
	block.Block.StateRoot = s[:]
	// Temporarily incrementing the beacon state slot here since BeaconProposerIndex is a
	// function deterministic on beacon state slot.
	if err := beaconState.SetSlot(beaconState.Slot() + 1); err != nil {
		return err
	}
	proposerIdx, err := helpers.BeaconProposerIndex(context.Background(), beaconState)
	if err != nil {
		return err
	}
	block.Signature, err = signing.ComputeDomainAndSign(beaconState, time.CurrentEpoch(beaconState), block.Block, params.BeaconConfig().DomainBeaconProposer, privs[proposerIdx])
	if err != nil {
		return err
	}
	if err := beaconState.SetSlot(beaconState.Slot() - 1); err != nil {
		return err
	}

	beaconBytes, err := beaconState.MarshalSSZ()
	if err != nil {
		return err
	}
	if err := file.WriteFile(path.Join(*outputDir, benchmark.BState1EpochFileName), beaconBytes); err != nil {
		return err
	}

	// Running a single state transition to make sure the generated files aren't broken.
	wsb, err = blocks.NewSignedBeaconBlock(block)
	if err != nil {
		return err
	}
	_, err = transition.ExecuteStateTransition(context.Background(), beaconState, wsb)
	if err != nil {
		return err
	}

	blockBytes, err := block.MarshalSSZ()
	if err != nil {
		return err
	}

	return file.WriteFile(path.Join(*outputDir, benchmark.FullBlockFileName), blockBytes)
}

func generate2FullEpochState() error {
	beaconState, err := genesisBeaconState()
	if err != nil {
		return err
	}

	privs, _, err := interop.DeterministicallyGenerateKeys(0, benchmark.ValidatorCount)
	if err != nil {
		return err
	}

	attConfig := &util.BlockGenConfig{
		NumAttestations: benchmark.AttestationsPerEpoch / uint64(params.BeaconConfig().SlotsPerEpoch),
	}

	for i := primitives.Slot(0); i < params.BeaconConfig().SlotsPerEpoch*2-1; i++ {
		block, err := util.GenerateFullBlock(beaconState, privs, attConfig, beaconState.Slot())
		if err != nil {
			return err
		}
		wsb, err := blocks.NewSignedBeaconBlock(block)
		if err != nil {
			return err
		}
		beaconState, err = transition.ExecuteStateTransition(context.Background(), beaconState, wsb)
		if err != nil {
			return err
		}
	}

	beaconBytes, err := beaconState.MarshalSSZ()
	if err != nil {
		return err
	}

	return file.WriteFile(path.Join(*outputDir, benchmark.BstateEpochFileName), beaconBytes)
}

func genesisBeaconState() (state.BeaconState, error) {
	beaconBytes, err := os.ReadFile(path.Join(*outputDir, benchmark.GenesisFileName))
	if err != nil {
		return nil, errors.Wrap(err, "cannot read genesis state file")
	}
	genesisState := &ethpb.BeaconState{}
	if err := genesisState.UnmarshalSSZ(beaconBytes); err != nil {
		return nil, errors.Wrap(err, "cannot unmarshal genesis state file")
	}
	return state_native.InitializeFromProtoUnsafePhase0(genesisState)
}
