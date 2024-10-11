package evaluators

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/altair"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/debug"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/httputil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	e2eparams "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/policies"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/types/known/emptypb"
)

var expectedParticipation = 0.99

var expectedMulticlientParticipation = 0.95

var expectedSyncParticipation = 0.99

// ValidatorsAreActive ensures the expected amount of validators are active.
var ValidatorsAreActive = types.Evaluator{
	Name:       "validators_active_epoch_%d",
	Policy:     policies.AllEpochs,
	Evaluation: validatorsAreActive,
}

// ValidatorsParticipatingAtEpoch ensures the expected amount of validators are participating.
var ValidatorsParticipatingAtEpoch = func(epoch primitives.Epoch) types.Evaluator {
	return types.Evaluator{
		Name:       "validators_participating_epoch_%d",
		Policy:     policies.AfterNthEpoch(epoch),
		Evaluation: validatorsParticipating,
	}
}

// ValidatorSyncParticipation ensures the expected amount of sync committee participants
// are active.
var ValidatorSyncParticipation = types.Evaluator{
	Name: "validator_sync_participation_%d",
	Policy: func(e primitives.Epoch) bool {
		fEpoch := params.BeaconConfig().AltairForkEpoch
		return policies.OnwardsNthEpoch(fEpoch)(e)
	},
	Evaluation: validatorsSyncParticipation,
}

func validatorsAreActive(ec *types.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	client := ethpb.NewBeaconChainClient(conn)
	// Balances actually fluctuate but we just want to check initial balance.
	validatorRequest := &ethpb.ListValidatorsRequest{
		PageSize: int32(params.BeaconConfig().MinGenesisActiveValidatorCount),
		Active:   true,
	}
	validators, err := client.ListValidators(context.Background(), validatorRequest)
	if err != nil {
		return errors.Wrap(err, "failed to get validators")
	}

	expectedCount := params.BeaconConfig().MinGenesisActiveValidatorCount
	receivedCount := uint64(len(validators.ValidatorList))
	if expectedCount != receivedCount {
		return fmt.Errorf("expected validator count to be %d, received %d", expectedCount, receivedCount)
	}

	effBalanceLowCount := 0
	exitEpochWrongCount := 0
	withdrawEpochWrongCount := 0
	for _, item := range validators.ValidatorList {
		if ec.ExitedVals[bytesutil.ToBytes48(item.Validator.PublicKey)] {
			continue
		}
		if item.Validator.EffectiveBalance < params.BeaconConfig().MaxEffectiveBalance {
			effBalanceLowCount++
		}
		if item.Validator.ExitEpoch != params.BeaconConfig().FarFutureEpoch {
			exitEpochWrongCount++
		}
		if item.Validator.WithdrawableEpoch != params.BeaconConfig().FarFutureEpoch {
			withdrawEpochWrongCount++
		}
	}

	if effBalanceLowCount > 0 {
		return fmt.Errorf(
			"%d validators did not have genesis validator effective balance of %d",
			effBalanceLowCount,
			params.BeaconConfig().MaxEffectiveBalance,
		)
	} else if exitEpochWrongCount > 0 {
		return fmt.Errorf("%d validators did not have genesis validator exit epoch of far future epoch", exitEpochWrongCount)
	} else if withdrawEpochWrongCount > 0 {
		return fmt.Errorf("%d validators did not have genesis validator withdrawable epoch of far future epoch", withdrawEpochWrongCount)
	}

	return nil
}

// validatorsParticipating ensures the validators have an acceptable participation rate.
func validatorsParticipating(_ *types.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	client := ethpb.NewBeaconChainClient(conn)
	validatorRequest := &ethpb.GetValidatorParticipationRequest{}
	participation, err := client.GetValidatorParticipation(context.Background(), validatorRequest)
	if err != nil {
		return errors.Wrap(err, "failed to get validator participation")
	}

	partRate := participation.Participation.GlobalParticipationRate
	expected := float32(expectedParticipation)
	if e2eparams.TestParams.LighthouseBeaconNodeCount != 0 {
		expected = float32(expectedMulticlientParticipation)
	}
	if participation.Epoch > 0 && participation.Epoch.Sub(1) == params.BeaconConfig().BellatrixForkEpoch {
		// Reduce Participation requirement to 95% to account for longer EE calls for
		// the merge block. Target and head will likely be missed for a few validators at
		// slot 0.
		expected = 0.95
	}
	if partRate < expected {
		path := fmt.Sprintf("http://localhost:%d/eth/v2/debug/beacon/states/head", e2eparams.TestParams.Ports.PrysmBeaconNodeGatewayPort)
		resp := debug.GetBeaconStateV2Response{}
		httpResp, err := http.Get(path) // #nosec G107 -- path can't be constant because it depends on port param
		if err != nil {
			return err
		}
		if httpResp.StatusCode != http.StatusOK {
			e := httputil.DefaultJsonError{}
			if err = json.NewDecoder(httpResp.Body).Decode(&e); err != nil {
				return err
			}
			return fmt.Errorf("%s (status code %d)", e.Message, e.Code)
		}
		if err = json.NewDecoder(httpResp.Body).Decode(&resp); err != nil {
			return err
		}

		var respPrevEpochParticipation []string
		switch resp.Version {
		case version.String(version.Phase0):
		// Do Nothing
		case version.String(version.Altair):
			st := &shared.BeaconStateAltair{}
			if err = json.Unmarshal(resp.Data, st); err != nil {
				return err
			}
			respPrevEpochParticipation = st.PreviousEpochParticipation
		case version.String(version.Bellatrix):
			st := &shared.BeaconStateBellatrix{}
			if err = json.Unmarshal(resp.Data, st); err != nil {
				return err
			}
			respPrevEpochParticipation = st.PreviousEpochParticipation
		case version.String(version.Capella):
			st := &shared.BeaconStateCapella{}
			if err = json.Unmarshal(resp.Data, st); err != nil {
				return err
			}
			respPrevEpochParticipation = st.PreviousEpochParticipation
		default:
			return fmt.Errorf("unrecognized version %s", resp.Version)
		}

		prevEpochParticipation := make([]byte, len(respPrevEpochParticipation))
		for i, p := range respPrevEpochParticipation {
			n, err := strconv.ParseUint(p, 10, 64)
			if err != nil {
				return err
			}
			prevEpochParticipation[i] = byte(n)
		}
		missSrcVals, missTgtVals, missHeadVals, err := findMissingValidators(prevEpochParticipation)
		if err != nil {
			return errors.Wrap(err, "failed to get missing validators")
		}

		return fmt.Errorf(
			"validator participation was below for epoch %d, expected %f, received: %f."+
				" Missing Source,Target and Head validators are %v, %v, %v",
			participation.Epoch,
			expected,
			partRate,
			missSrcVals,
			missTgtVals,
			missHeadVals,
		)
	}
	return nil
}

// validatorsSyncParticipation ensures the validators have an acceptable participation rate for
// sync committee assignments.
func validatorsSyncParticipation(_ *types.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	client := ethpb.NewNodeClient(conn)
	altairClient := ethpb.NewBeaconChainClient(conn)
	genesis, err := client.GetGenesis(context.Background(), &emptypb.Empty{})
	if err != nil {
		return errors.Wrap(err, "failed to get genesis data")
	}
	currSlot := slots.CurrentSlot(uint64(genesis.GenesisTime.AsTime().Unix()))
	currEpoch := slots.ToEpoch(currSlot)
	lowestBound := primitives.Epoch(0)
	if currEpoch >= 1 {
		lowestBound = currEpoch - 1
	}

	if lowestBound < params.BeaconConfig().AltairForkEpoch {
		lowestBound = params.BeaconConfig().AltairForkEpoch
	}
	blockCtrs, err := altairClient.ListBeaconBlocks(context.Background(), &ethpb.ListBlocksRequest{QueryFilter: &ethpb.ListBlocksRequest_Epoch{Epoch: lowestBound}})
	if err != nil {
		return errors.Wrap(err, "failed to get validator participation")
	}
	for _, ctr := range blockCtrs.BlockContainers {
		b, err := syncCompatibleBlockFromCtr(ctr)
		if err != nil {
			return errors.Wrapf(err, "block type doesn't exist for block at epoch %d", lowestBound)
		}

		if b.IsNil() {
			return errors.New("nil block provided")
		}
		forkStartSlot, err := slots.EpochStart(params.BeaconConfig().AltairForkEpoch)
		if err != nil {
			return err
		}
		if forkStartSlot == b.Block().Slot() {
			// Skip fork slot.
			continue
		}
		expectedParticipation := expectedSyncParticipation
		switch slots.ToEpoch(b.Block().Slot()) {
		case params.BeaconConfig().AltairForkEpoch:
			// Drop expected sync participation figure.
			expectedParticipation = 0.90
		default:
			// no-op
		}
		syncAgg, err := b.Block().Body().SyncAggregate()
		if err != nil {
			return err
		}
		threshold := uint64(float64(syncAgg.SyncCommitteeBits.Len()) * expectedParticipation)
		if syncAgg.SyncCommitteeBits.Count() < threshold {
			return errors.Errorf("In block of slot %d ,the aggregate bitvector with length of %d only got a count of %d", b.Block().Slot(), threshold, syncAgg.SyncCommitteeBits.Count())
		}
	}
	if lowestBound == currEpoch {
		return nil
	}
	blockCtrs, err = altairClient.ListBeaconBlocks(context.Background(), &ethpb.ListBlocksRequest{QueryFilter: &ethpb.ListBlocksRequest_Epoch{Epoch: currEpoch}})
	if err != nil {
		return errors.Wrap(err, "failed to get validator participation")
	}
	for _, ctr := range blockCtrs.BlockContainers {
		b, err := syncCompatibleBlockFromCtr(ctr)
		if err != nil {
			return errors.Wrapf(err, "block type doesn't exist for block at epoch %d", lowestBound)
		}

		if b.IsNil() {
			return errors.New("nil block provided")
		}
		forkSlot, err := slots.EpochStart(params.BeaconConfig().AltairForkEpoch)
		if err != nil {
			return err
		}
		nexForkSlot, err := slots.EpochStart(params.BeaconConfig().BellatrixForkEpoch)
		if err != nil {
			return err
		}
		switch b.Block().Slot() {
		case forkSlot, forkSlot + 1, nexForkSlot:
			// Skip evaluation of the slot.
			continue
		default:
			// no-op
		}
		syncAgg, err := b.Block().Body().SyncAggregate()
		if err != nil {
			return err
		}
		threshold := uint64(float64(syncAgg.SyncCommitteeBits.Len()) * expectedSyncParticipation)
		if syncAgg.SyncCommitteeBits.Count() < threshold {
			return errors.Errorf("In block of slot %d ,the aggregate bitvector with length of %d only got a count of %d", b.Block().Slot(), threshold, syncAgg.SyncCommitteeBits.Count())
		}
	}
	return nil
}

func syncCompatibleBlockFromCtr(container *ethpb.BeaconBlockContainer) (interfaces.ReadOnlySignedBeaconBlock, error) {
	if container.GetPhase0Block() != nil {
		return nil, errors.New("block doesn't support sync committees")
	}
	if container.GetAltairBlock() != nil {
		return blocks.NewSignedBeaconBlock(container.GetAltairBlock())
	}
	if container.GetBellatrixBlock() != nil {
		return blocks.NewSignedBeaconBlock(container.GetBellatrixBlock())
	}
	if container.GetBlindedBellatrixBlock() != nil {
		return blocks.NewSignedBeaconBlock(container.GetBlindedBellatrixBlock())
	}
	if container.GetCapellaBlock() != nil {
		return blocks.NewSignedBeaconBlock(container.GetCapellaBlock())
	}
	if container.GetBlindedCapellaBlock() != nil {
		return blocks.NewSignedBeaconBlock(container.GetBlindedCapellaBlock())
	}
	if container.GetDenebBlock() != nil {
		return blocks.NewSignedBeaconBlock(container.GetDenebBlock())
	}
	if container.GetBlindedDenebBlock() != nil {
		return blocks.NewSignedBeaconBlock(container.GetBlindedDenebBlock())
	}
	return nil, errors.New("no supported block type in container")
}

func findMissingValidators(participation []byte) ([]uint64, []uint64, []uint64, error) {
	cfg := params.BeaconConfig()
	sourceFlagIndex := cfg.TimelySourceFlagIndex
	targetFlagIndex := cfg.TimelyTargetFlagIndex
	headFlagIndex := cfg.TimelyHeadFlagIndex
	var missingSourceValidators []uint64
	var missingHeadValidators []uint64
	var missingTargetValidators []uint64
	for i, b := range participation {
		hasSource, err := altair.HasValidatorFlag(b, sourceFlagIndex)
		if err != nil {
			return nil, nil, nil, err
		}
		if !hasSource {
			missingSourceValidators = append(missingSourceValidators, uint64(i))
		}
		hasTarget, err := altair.HasValidatorFlag(b, targetFlagIndex)
		if err != nil {
			return nil, nil, nil, err
		}
		if !hasTarget {
			missingTargetValidators = append(missingTargetValidators, uint64(i))
		}
		hasHead, err := altair.HasValidatorFlag(b, headFlagIndex)
		if err != nil {
			return nil, nil, nil, err
		}
		if !hasHead {
			missingHeadValidators = append(missingHeadValidators, uint64(i))
		}
	}
	return missingSourceValidators, missingTargetValidators, missingHeadValidators, nil
}
