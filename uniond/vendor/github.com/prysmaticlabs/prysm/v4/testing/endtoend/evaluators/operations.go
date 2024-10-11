package evaluators

import (
	"bytes"
	"context"
	"fmt"
	"math"
	"strings"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/api/client/beacon"
	corehelpers "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/rpc/eth/shared"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz/detect"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/helpers"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/policies"
	e2etypes "github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"golang.org/x/exp/rand"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/types/known/emptypb"
)

var depositValCount = e2e.DepositCount
var numOfExits = 2

// Deposits should be processed in twice the length of the epochs per eth1 voting period.
var depositsInBlockStart = params.E2ETestConfig().EpochsPerEth1VotingPeriod * 2

// deposits included + finalization + MaxSeedLookahead for activation.
var depositActivationStartEpoch = depositsInBlockStart + 2 + params.E2ETestConfig().MaxSeedLookahead
var depositEndEpoch = depositActivationStartEpoch + primitives.Epoch(math.Ceil(float64(depositValCount)/float64(params.E2ETestConfig().MinPerEpochChurnLimit)))
var exitSubmissionEpoch = primitives.Epoch(7)

// ProcessesDepositsInBlocks ensures the expected amount of deposits are accepted into blocks.
var ProcessesDepositsInBlocks = e2etypes.Evaluator{
	Name:       "processes_deposits_in_blocks_epoch_%d",
	Policy:     policies.OnEpoch(depositsInBlockStart), // We expect all deposits to enter in one epoch.
	Evaluation: processesDepositsInBlocks,
}

// VerifyBlockGraffiti ensures the block graffiti is one of the random list.
var VerifyBlockGraffiti = e2etypes.Evaluator{
	Name:       "verify_graffiti_in_blocks_epoch_%d",
	Policy:     policies.AfterNthEpoch(0),
	Evaluation: verifyGraffitiInBlocks,
}

// ActivatesDepositedValidators ensures the expected amount of validator deposits are activated into the state.
var ActivatesDepositedValidators = e2etypes.Evaluator{
	Name:       "processes_deposit_validators_epoch_%d",
	Policy:     policies.BetweenEpochs(depositActivationStartEpoch, depositEndEpoch),
	Evaluation: activatesDepositedValidators,
}

// DepositedValidatorsAreActive ensures the expected amount of validators are active after their deposits are processed.
var DepositedValidatorsAreActive = e2etypes.Evaluator{
	Name:       "deposited_validators_are_active_epoch_%d",
	Policy:     policies.AfterNthEpoch(depositEndEpoch),
	Evaluation: depositedValidatorsAreActive,
}

// ProposeVoluntaryExit sends a voluntary exit from randomly selected validator in the genesis set.
var ProposeVoluntaryExit = e2etypes.Evaluator{
	Name:       "propose_voluntary_exit_epoch_%d",
	Policy:     policies.OnEpoch(exitSubmissionEpoch),
	Evaluation: proposeVoluntaryExit,
}

// ValidatorsHaveExited checks the beacon state for the exited validator and ensures its marked as exited.
var ValidatorsHaveExited = e2etypes.Evaluator{
	Name:       "voluntary_has_exited_%d",
	Policy:     policies.OnEpoch(8),
	Evaluation: validatorsHaveExited,
}

// SubmitWithdrawal sends a withdrawal from a previously exited validator.
var SubmitWithdrawal = e2etypes.Evaluator{
	Name: "submit_withdrawal_epoch_%d",
	Policy: func(currentEpoch primitives.Epoch) bool {
		fEpoch := params.BeaconConfig().CapellaForkEpoch
		return policies.BetweenEpochs(fEpoch-2, fEpoch+1)(currentEpoch)
	},
	Evaluation: submitWithdrawal,
}

// ValidatorsHaveWithdrawn checks the beacon state for the withdrawn validator and ensures it has been withdrawn.
var ValidatorsHaveWithdrawn = e2etypes.Evaluator{
	Name: "validator_has_withdrawn_%d",
	Policy: func(currentEpoch primitives.Epoch) bool {
		// TODO: Fix this for mainnet configs.
		if params.BeaconConfig().ConfigName != params.EndToEndName {
			return false
		}
		// Only run this for minimal setups after capella
		validWithdrawnEpoch := params.BeaconConfig().CapellaForkEpoch + 1

		requiredPolicy := policies.OnEpoch(validWithdrawnEpoch)
		return requiredPolicy(currentEpoch)
	},
	Evaluation: validatorsAreWithdrawn,
}

// ValidatorsVoteWithTheMajority verifies whether validator vote for eth1data using the majority algorithm.
var ValidatorsVoteWithTheMajority = e2etypes.Evaluator{
	Name:       "validators_vote_with_the_majority_%d",
	Policy:     policies.AfterNthEpoch(0),
	Evaluation: validatorsVoteWithTheMajority,
}

type mismatch struct {
	k [48]byte
	e uint64
	o uint64
}

func (m mismatch) String() string {
	return fmt.Sprintf("(%#x:%d:%d)", m.k, m.e, m.o)
}

func processesDepositsInBlocks(ec *e2etypes.EvaluationContext, conns ...*grpc.ClientConn) error {
	expected := ec.Balances(e2etypes.PostGenesisDepositBatch)
	conn := conns[0]
	client := ethpb.NewBeaconChainClient(conn)
	chainHead, err := client.GetChainHead(context.Background(), &emptypb.Empty{})
	if err != nil {
		return errors.Wrap(err, "failed to get chain head")
	}

	req := &ethpb.ListBlocksRequest{QueryFilter: &ethpb.ListBlocksRequest_Epoch{Epoch: chainHead.HeadEpoch - 1}}
	blks, err := client.ListBeaconBlocks(context.Background(), req)
	if err != nil {
		return errors.Wrap(err, "failed to get blocks from beacon-chain")
	}
	observed := make(map[[48]byte]uint64)
	for _, blk := range blks.BlockContainers {
		sb, err := blocks.BeaconBlockContainerToSignedBeaconBlock(blk)
		if err != nil {
			return errors.Wrap(err, "failed to convert api response type to SignedBeaconBlock interface")
		}
		b := sb.Block()
		deposits := b.Body().Deposits()
		for _, d := range deposits {
			k := bytesutil.ToBytes48(d.Data.PublicKey)
			v := observed[k]
			observed[k] = v + d.Data.Amount
		}
	}
	var mismatches []string
	for k, ev := range expected {
		ov := observed[k]
		if ev != ov {
			mismatches = append(mismatches, mismatch{k: k, e: ev, o: ov}.String())
		}
	}
	if len(mismatches) != 0 {
		return fmt.Errorf("not all expected deposits observed on chain, len(expected)=%d, len(observed)=%d, mismatches=%d; details(key:expected:observed): %s", len(expected), len(observed), len(mismatches), strings.Join(mismatches, ","))
	}
	return nil
}

func verifyGraffitiInBlocks(_ *e2etypes.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	client := ethpb.NewBeaconChainClient(conn)
	chainHead, err := client.GetChainHead(context.Background(), &emptypb.Empty{})
	if err != nil {
		return errors.Wrap(err, "failed to get chain head")
	}
	begin := chainHead.HeadEpoch
	// Prevent underflow when this runs at epoch 0.
	if begin > 0 {
		begin = begin.Sub(1)
	}
	req := &ethpb.ListBlocksRequest{QueryFilter: &ethpb.ListBlocksRequest_Epoch{Epoch: begin}}
	blks, err := client.ListBeaconBlocks(context.Background(), req)
	if err != nil {
		return errors.Wrap(err, "failed to get blocks from beacon-chain")
	}
	for _, ctr := range blks.BlockContainers {
		blk, err := blocks.BeaconBlockContainerToSignedBeaconBlock(ctr)
		if err != nil {
			return err
		}
		var e bool
		slot := blk.Block().Slot()
		graffitiInBlock := blk.Block().Body().Graffiti()
		for _, graffiti := range helpers.Graffiti {
			if bytes.Equal(bytesutil.PadTo([]byte(graffiti), 32), graffitiInBlock[:]) {
				e = true
				break
			}
		}
		if !e && slot != 0 {
			return errors.New("could not get graffiti from the list")
		}
	}

	return nil
}

func activatesDepositedValidators(ec *e2etypes.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	client := ethpb.NewBeaconChainClient(conn)

	chainHead, err := client.GetChainHead(context.Background(), &emptypb.Empty{})
	if err != nil {
		return errors.Wrap(err, "failed to get chain head")
	}
	epoch := chainHead.HeadEpoch

	validators, err := getAllValidators(client)
	if err != nil {
		return errors.Wrap(err, "failed to get validators")
	}
	expected := ec.Balances(e2etypes.PostGenesisDepositBatch)

	var deposits, lowBalance, wrongExit, wrongWithdraw int
	for _, v := range validators {
		key := bytesutil.ToBytes48(v.PublicKey)
		if _, ok := expected[key]; !ok {
			continue
		}
		delete(expected, key)
		if v.ActivationEpoch != epoch {
			continue
		}
		deposits++
		if v.EffectiveBalance < params.BeaconConfig().MaxEffectiveBalance {
			lowBalance++
		}
		if v.ExitEpoch != params.BeaconConfig().FarFutureEpoch {
			wrongExit++
		}
		if v.WithdrawableEpoch != params.BeaconConfig().FarFutureEpoch {
			wrongWithdraw++
		}
	}

	// Make sure every post-genesis deposit has been proecssed, resulting in a validator.
	if len(expected) > 0 {
		return fmt.Errorf("missing %d validators for post-genesis deposits", len(expected))
	}

	if uint64(deposits) != params.BeaconConfig().MinPerEpochChurnLimit {
		return fmt.Errorf("expected %d deposits to be processed in epoch %d, received %d", params.BeaconConfig().MinPerEpochChurnLimit, epoch, deposits)
	}

	if lowBalance > 0 {
		return fmt.Errorf(
			"%d validators did not have genesis validator effective balance of %d",
			lowBalance,
			params.BeaconConfig().MaxEffectiveBalance,
		)
	} else if wrongExit > 0 {
		return fmt.Errorf("%d validators did not have an exit epoch of far future epoch", wrongExit)
	} else if wrongWithdraw > 0 {
		return fmt.Errorf("%d validators did not have a withdrawable epoch of far future epoch", wrongWithdraw)
	}
	return nil
}

func getAllValidators(c ethpb.BeaconChainClient) ([]*ethpb.Validator, error) {
	vals := make([]*ethpb.Validator, 0)
	pageToken := "0"
	for pageToken != "" {
		validatorRequest := &ethpb.ListValidatorsRequest{
			PageSize:  100,
			PageToken: pageToken,
		}
		validators, err := c.ListValidators(context.Background(), validatorRequest)
		if err != nil {
			return nil, errors.Wrap(err, "failed to get validators")
		}
		for _, v := range validators.ValidatorList {
			vals = append(vals, v.Validator)
		}
		pageToken = validators.NextPageToken
	}
	return vals, nil
}

func depositedValidatorsAreActive(ec *e2etypes.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	client := ethpb.NewBeaconChainClient(conn)

	chainHead, err := client.GetChainHead(context.Background(), &emptypb.Empty{})
	if err != nil {
		return errors.Wrap(err, "failed to get chain head")
	}

	vals, err := getAllValidators(client)
	if err != nil {
		return errors.Wrap(err, "error retrieving validator list from API")
	}
	inactive := 0
	lowBalance := 0
	nexits := 0
	expected := ec.Balances(e2etypes.PostGenesisDepositBatch)
	nexpected := len(expected)
	for _, v := range vals {
		key := bytesutil.ToBytes48(v.PublicKey)
		if _, ok := expected[key]; !ok {
			continue // we aren't checking for this validator
		}
		// ignore voluntary exits when checking balance and active status
		exited := ec.ExitedVals[key]
		if exited {
			nexits++
			delete(expected, key)
			continue
		}
		if !corehelpers.IsActiveValidator(v, chainHead.HeadEpoch) {
			inactive++
		}
		if v.EffectiveBalance < params.BeaconConfig().MaxEffectiveBalance {
			lowBalance++
		}
		delete(expected, key)
	}
	if len(expected) > 0 {
		mk := make([]string, 0)
		for k := range expected {
			mk = append(mk, fmt.Sprintf("%#x", k))
		}
		return fmt.Errorf("API response missing %d validators, based on deposits; keys=%s", len(expected), strings.Join(mk, ","))
	}
	if inactive != 0 || lowBalance != 0 {
		return fmt.Errorf("active validator set does not match %d total deposited. %d exited, %d inactive, %d low balance", nexpected, nexits, inactive, lowBalance)
	}

	return nil
}

func proposeVoluntaryExit(ec *e2etypes.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	valClient := ethpb.NewBeaconNodeValidatorClient(conn)
	beaconClient := ethpb.NewBeaconChainClient(conn)
	debugClient := ethpb.NewDebugClient(conn)

	ctx := context.Background()
	chainHead, err := beaconClient.GetChainHead(ctx, &emptypb.Empty{})
	if err != nil {
		return errors.Wrap(err, "could not get chain head")
	}
	stObj, err := debugClient.GetBeaconState(ctx, &ethpb.BeaconStateRequest{QueryFilter: &ethpb.BeaconStateRequest_Slot{Slot: chainHead.HeadSlot}})
	if err != nil {
		return errors.Wrap(err, "could not get state object")
	}
	versionedMarshaler, err := detect.FromState(stObj.Encoded)
	if err != nil {
		return errors.Wrap(err, "could not get state marshaler")
	}
	st, err := versionedMarshaler.UnmarshalBeaconState(stObj.Encoded)
	if err != nil {
		return errors.Wrap(err, "could not get state")
	}
	var execIndices []int
	err = st.ReadFromEveryValidator(func(idx int, val state.ReadOnlyValidator) error {
		if val.WithdrawalCredentials()[0] == params.BeaconConfig().ETH1AddressWithdrawalPrefixByte {
			execIndices = append(execIndices, idx)
		}
		return nil
	})
	if err != nil {
		return err
	}
	if len(execIndices) > numOfExits {
		execIndices = execIndices[:numOfExits]
	}

	deposits, privKeys, err := util.DeterministicDepositsAndKeys(params.BeaconConfig().MinGenesisActiveValidatorCount)
	if err != nil {
		return err
	}

	var sendExit = func(exitedIndex primitives.ValidatorIndex) error {
		voluntaryExit := &ethpb.VoluntaryExit{
			Epoch:          chainHead.HeadEpoch,
			ValidatorIndex: exitedIndex,
		}
		req := &ethpb.DomainRequest{
			Epoch:  chainHead.HeadEpoch,
			Domain: params.BeaconConfig().DomainVoluntaryExit[:],
		}
		domain, err := valClient.DomainData(ctx, req)
		if err != nil {
			return err
		}
		signingData, err := signing.ComputeSigningRoot(voluntaryExit, domain.SignatureDomain)
		if err != nil {
			return err
		}
		signature := privKeys[exitedIndex].Sign(signingData[:])
		signedExit := &ethpb.SignedVoluntaryExit{
			Exit:      voluntaryExit,
			Signature: signature.Marshal(),
		}

		if _, err = valClient.ProposeExit(ctx, signedExit); err != nil {
			return errors.Wrap(err, "could not propose exit")
		}
		pubk := bytesutil.ToBytes48(deposits[exitedIndex].Data.PublicKey)
		ec.ExitedVals[pubk] = true
		return nil
	}

	// Send exits for keys which already contain execution credentials.
	for _, idx := range execIndices {
		if err := sendExit(primitives.ValidatorIndex(idx)); err != nil {
			return err
		}
	}

	// Send an exit for a non-exited validator.
	for i := 0; i < numOfExits; {
		randIndex := primitives.ValidatorIndex(rand.Uint64() % params.BeaconConfig().MinGenesisActiveValidatorCount)
		if ec.ExitedVals[bytesutil.ToBytes48(privKeys[randIndex].PublicKey().Marshal())] {
			continue
		}
		if err := sendExit(randIndex); err != nil {
			return err
		}
		i++
	}

	return nil
}

func validatorsHaveExited(ec *e2etypes.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	client := ethpb.NewBeaconChainClient(conn)
	for k := range ec.ExitedVals {
		validatorRequest := &ethpb.GetValidatorRequest{
			QueryFilter: &ethpb.GetValidatorRequest_PublicKey{
				PublicKey: k[:],
			},
		}
		validator, err := client.GetValidator(context.Background(), validatorRequest)
		if err != nil {
			return errors.Wrap(err, "failed to get validators")
		}
		if validator.ExitEpoch == params.BeaconConfig().FarFutureEpoch {
			return fmt.Errorf("expected validator %#x to be submitted for exit", k)
		}
	}
	return nil
}

func validatorsVoteWithTheMajority(ec *e2etypes.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	client := ethpb.NewBeaconChainClient(conn)
	chainHead, err := client.GetChainHead(context.Background(), &emptypb.Empty{})
	if err != nil {
		return errors.Wrap(err, "failed to get chain head")
	}

	begin := chainHead.HeadEpoch
	// Prevent underflow when this runs at epoch 0.
	if begin > 0 {
		begin = begin.Sub(1)
	}
	req := &ethpb.ListBlocksRequest{QueryFilter: &ethpb.ListBlocksRequest_Epoch{Epoch: begin}}
	blks, err := client.ListBeaconBlocks(context.Background(), req)
	if err != nil {
		return errors.Wrap(err, "failed to get blocks from beacon-chain")
	}

	slotsPerVotingPeriod := params.E2ETestConfig().SlotsPerEpoch.Mul(uint64(params.E2ETestConfig().EpochsPerEth1VotingPeriod))
	for _, blk := range blks.BlockContainers {
		var slot primitives.Slot
		var vote []byte
		switch blk.Block.(type) {
		case *ethpb.BeaconBlockContainer_Phase0Block:
			b := blk.GetPhase0Block().Block
			slot = b.Slot
			vote = b.Body.Eth1Data.BlockHash
		case *ethpb.BeaconBlockContainer_AltairBlock:
			b := blk.GetAltairBlock().Block
			slot = b.Slot
			vote = b.Body.Eth1Data.BlockHash
		case *ethpb.BeaconBlockContainer_BellatrixBlock:
			b := blk.GetBellatrixBlock().Block
			slot = b.Slot
			vote = b.Body.Eth1Data.BlockHash
		case *ethpb.BeaconBlockContainer_BlindedBellatrixBlock:
			b := blk.GetBlindedBellatrixBlock().Block
			slot = b.Slot
			vote = b.Body.Eth1Data.BlockHash
		case *ethpb.BeaconBlockContainer_CapellaBlock:
			b := blk.GetCapellaBlock().Block
			slot = b.Slot
			vote = b.Body.Eth1Data.BlockHash
		case *ethpb.BeaconBlockContainer_BlindedCapellaBlock:
			b := blk.GetBlindedCapellaBlock().Block
			slot = b.Slot
			vote = b.Body.Eth1Data.BlockHash
		case *ethpb.BeaconBlockContainer_DenebBlock:
			b := blk.GetDenebBlock().Block
			slot = b.Slot
			vote = b.Body.Eth1Data.BlockHash
		case *ethpb.BeaconBlockContainer_BlindedDenebBlock:
			b := blk.GetBlindedDenebBlock().Message
			slot = b.Slot
			vote = b.Body.Eth1Data.BlockHash
		default:
			return errors.New("block neither phase0,altair or bellatrix")
		}
		ec.SeenVotes[slot] = vote

		// We treat epoch 1 differently from other epoch for two reasons:
		// - this evaluator is not executed for epoch 0 so we have to calculate the first slot differently
		// - for some reason the vote for the first slot in epoch 1 is 0x000... so we skip this slot
		var isFirstSlotInVotingPeriod bool
		if chainHead.HeadEpoch == 1 && slot%params.BeaconConfig().SlotsPerEpoch == 0 {
			continue
		}
		// We skipped the first slot so we treat the second slot as the starting slot of epoch 1.
		if chainHead.HeadEpoch == 1 {
			isFirstSlotInVotingPeriod = slot%params.BeaconConfig().SlotsPerEpoch == 1
		} else {
			isFirstSlotInVotingPeriod = slot%slotsPerVotingPeriod == 0
		}
		if isFirstSlotInVotingPeriod {
			ec.ExpectedEth1DataVote = vote
			return nil
		}

		if !bytes.Equal(vote, ec.ExpectedEth1DataVote) {
			for i := primitives.Slot(0); i < slot; i++ {
				v, ok := ec.SeenVotes[i]
				if ok {
					fmt.Printf("vote at slot=%d = %#x\n", i, v)
				} else {
					fmt.Printf("did not see slot=%d\n", i)
				}
			}
			return fmt.Errorf("incorrect eth1data vote for slot %d; expected: %#x vs voted: %#x",
				slot, ec.ExpectedEth1DataVote, vote)
		}
	}
	return nil
}

func submitWithdrawal(ec *e2etypes.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	beaconClient := ethpb.NewBeaconChainClient(conn)
	debugClient := ethpb.NewDebugClient(conn)

	ctx := context.Background()
	chainHead, err := beaconClient.GetChainHead(ctx, &emptypb.Empty{})
	if err != nil {
		return errors.Wrap(err, "could not get chain head")
	}
	stObj, err := debugClient.GetBeaconState(ctx, &ethpb.BeaconStateRequest{QueryFilter: &ethpb.BeaconStateRequest_Slot{Slot: chainHead.HeadSlot}})
	if err != nil {
		return errors.Wrap(err, "could not get state object")
	}
	versionedMarshaler, err := detect.FromState(stObj.Encoded)
	if err != nil {
		return errors.Wrap(err, "could not get state marshaler")
	}
	st, err := versionedMarshaler.UnmarshalBeaconState(stObj.Encoded)
	if err != nil {
		return errors.Wrap(err, "could not get state")
	}
	exitedIndices := make([]primitives.ValidatorIndex, 0)

	for key := range ec.ExitedVals {
		valIdx, ok := st.ValidatorIndexByPubkey(key)
		if !ok {
			return errors.Errorf("pubkey %#x does not exist in our state", key)
		}
		exitedIndices = append(exitedIndices, valIdx)
	}

	_, privKeys, err := util.DeterministicDepositsAndKeys(params.BeaconConfig().MinGenesisActiveValidatorCount)
	if err != nil {
		return err
	}
	changes := make([]*shared.SignedBLSToExecutionChange, 0)
	// Only send half the number of changes each time, to allow us to test
	// at the fork boundary.
	wantedChanges := numOfExits / 2
	for _, idx := range exitedIndices {
		// Exit sending more change messages.
		if len(changes) >= wantedChanges {
			break
		}
		val, err := st.ValidatorAtIndex(idx)
		if err != nil {
			return err
		}
		if val.WithdrawalCredentials[0] == params.BeaconConfig().ETH1AddressWithdrawalPrefixByte {
			continue
		}
		if !bytes.Equal(val.PublicKey, privKeys[idx].PublicKey().Marshal()) {
			return errors.Errorf("pubkey is not equal, wanted %#x but received %#x", val.PublicKey, privKeys[idx].PublicKey().Marshal())
		}
		message := &ethpb.BLSToExecutionChange{
			ValidatorIndex:     idx,
			FromBlsPubkey:      privKeys[idx].PublicKey().Marshal(),
			ToExecutionAddress: bytesutil.ToBytes(uint64(idx), 20),
		}
		domain, err := signing.ComputeDomain(params.BeaconConfig().DomainBLSToExecutionChange, params.BeaconConfig().GenesisForkVersion, st.GenesisValidatorsRoot())
		if err != nil {
			return err
		}
		sigRoot, err := signing.ComputeSigningRoot(message, domain)
		if err != nil {
			return err
		}
		signature := privKeys[idx].Sign(sigRoot[:]).Marshal()

		changes = append(changes, &shared.SignedBLSToExecutionChange{
			Message:   shared.BLSChangeFromConsensus(message),
			Signature: hexutil.Encode(signature),
		})
	}

	beaconAPIClient, err := beacon.NewClient(fmt.Sprintf("http://localhost:%d/eth/v1", e2e.TestParams.Ports.PrysmBeaconNodeGatewayPort)) // only uses the first node so no updates to port
	if err != nil {
		return err
	}

	return beaconAPIClient.SubmitChangeBLStoExecution(ctx, changes)
}

func validatorsAreWithdrawn(ec *e2etypes.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	beaconClient := ethpb.NewBeaconChainClient(conn)
	debugClient := ethpb.NewDebugClient(conn)

	ctx := context.Background()
	chainHead, err := beaconClient.GetChainHead(ctx, &emptypb.Empty{})
	if err != nil {
		return errors.Wrap(err, "could not get chain head")
	}
	stObj, err := debugClient.GetBeaconState(ctx, &ethpb.BeaconStateRequest{QueryFilter: &ethpb.BeaconStateRequest_Slot{Slot: chainHead.HeadSlot}})
	if err != nil {
		return errors.Wrap(err, "could not get state object")
	}
	versionedMarshaler, err := detect.FromState(stObj.Encoded)
	if err != nil {
		return errors.Wrap(err, "could not get state marshaler")
	}
	st, err := versionedMarshaler.UnmarshalBeaconState(stObj.Encoded)
	if err != nil {
		return errors.Wrap(err, "could not get state")
	}

	for key := range ec.ExitedVals {
		valIdx, ok := st.ValidatorIndexByPubkey(key)
		if !ok {
			return errors.Errorf("pubkey %#x does not exist in our state", key)
		}
		bal, err := st.BalanceAtIndex(valIdx)
		if err != nil {
			return err
		}
		// Only return an error if the validator has more than 1 eth
		// in its balance.
		if bal > 1*params.BeaconConfig().GweiPerEth {
			return errors.Errorf("Validator index %d with key %#x hasn't withdrawn. Their balance is %d.", valIdx, key, bal)
		}
	}
	return nil
}
