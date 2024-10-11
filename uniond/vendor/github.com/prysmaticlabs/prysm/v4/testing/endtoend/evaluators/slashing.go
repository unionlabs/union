package evaluators

import (
	"context"
	"fmt"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/container/slice"
	"github.com/prysmaticlabs/prysm/v4/crypto/bls"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/policies"
	e2eTypes "github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/types/known/emptypb"
)

// InjectDoubleVoteOnEpoch broadcasts a double vote into the beacon node pool for the slasher to detect.
var InjectDoubleVoteOnEpoch = func(n primitives.Epoch) e2eTypes.Evaluator {
	return e2eTypes.Evaluator{
		Name:       "inject_double_vote_%d",
		Policy:     policies.OnEpoch(n),
		Evaluation: insertDoubleAttestationIntoPool,
	}
}

// InjectDoubleBlockOnEpoch proposes a double block to the beacon node for the slasher to detect.
var InjectDoubleBlockOnEpoch = func(n primitives.Epoch) e2eTypes.Evaluator {
	return e2eTypes.Evaluator{
		Name:       "inject_double_block_%d",
		Policy:     policies.OnEpoch(n),
		Evaluation: proposeDoubleBlock,
	}
}

// ValidatorsSlashedAfterEpoch ensures the expected amount of validators are slashed.
var ValidatorsSlashedAfterEpoch = func(n primitives.Epoch) e2eTypes.Evaluator {
	return e2eTypes.Evaluator{
		Name:       "validators_slashed_epoch_%d",
		Policy:     policies.AfterNthEpoch(n),
		Evaluation: validatorsSlashed,
	}
}

// SlashedValidatorsLoseBalanceAfterEpoch checks if the validators slashed lose the right balance.
var SlashedValidatorsLoseBalanceAfterEpoch = func(n primitives.Epoch) e2eTypes.Evaluator {
	return e2eTypes.Evaluator{
		Name:       "slashed_validators_lose_valance_epoch_%d",
		Policy:     policies.AfterNthEpoch(n),
		Evaluation: validatorsLoseBalance,
	}
}

var slashedIndices []uint64

func validatorsSlashed(_ *e2eTypes.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	ctx := context.Background()
	client := eth.NewBeaconChainClient(conn)

	actualSlashedIndices := 0

	for _, slashedIndex := range slashedIndices {
		req := &eth.GetValidatorRequest{
			QueryFilter: &eth.GetValidatorRequest_Index{
				Index: primitives.ValidatorIndex(slashedIndex),
			},
		}
		valResp, err := client.GetValidator(ctx, req)
		if err != nil {
			return err
		}

		if valResp.Slashed {
			actualSlashedIndices++
		}
	}

	if actualSlashedIndices != len(slashedIndices) {
		return fmt.Errorf("expected %d indices to be slashed, received %d", len(slashedIndices), actualSlashedIndices)
	}
	return nil
}

func validatorsLoseBalance(_ *e2eTypes.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	ctx := context.Background()
	client := eth.NewBeaconChainClient(conn)

	for i, slashedIndex := range slashedIndices {
		req := &eth.GetValidatorRequest{
			QueryFilter: &eth.GetValidatorRequest_Index{
				Index: primitives.ValidatorIndex(slashedIndex),
			},
		}
		valResp, err := client.GetValidator(ctx, req)
		if err != nil {
			return err
		}

		slashedPenalty := params.BeaconConfig().MaxEffectiveBalance / params.BeaconConfig().MinSlashingPenaltyQuotient
		slashedBal := params.BeaconConfig().MaxEffectiveBalance - slashedPenalty + params.BeaconConfig().EffectiveBalanceIncrement/10
		if valResp.EffectiveBalance >= slashedBal {
			return fmt.Errorf(
				"expected slashed validator %d to balance less than %d, received %d",
				i,
				slashedBal,
				valResp.EffectiveBalance,
			)
		}
	}
	return nil
}

func insertDoubleAttestationIntoPool(_ *e2eTypes.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	valClient := eth.NewBeaconNodeValidatorClient(conn)
	beaconClient := eth.NewBeaconChainClient(conn)

	ctx := context.Background()

	h := doubleAttestationHelper{
		valClient:    valClient,
		beaconClient: beaconClient,
	}
	err := h.setup(ctx)
	if err != nil {
		return errors.Wrap(err, "could not setup doubleAttestationHelper")
	}

	valsToSlash := uint64(2)
	for i := uint64(0); i < valsToSlash; i++ {
		valIdx := h.validatorIndexAtCommitteeIndex(i)

		if len(slice.IntersectionUint64(slashedIndices, []uint64{uint64(valIdx)})) > 0 {
			valsToSlash++
			continue
		}

		// Need to send proposal to both beacon nodes to avoid flakiness.
		// See: https://github.com/prysmaticlabs/prysm/issues/12415#issuecomment-1874643269
		c := eth.NewBeaconNodeValidatorClient(conns[0])
		att, err := h.getSlashableAttestation(i)
		if err != nil {
			return err
		}
		if _, err := c.ProposeAttestation(ctx, att); err != nil {
			return errors.Wrap(err, "could not propose attestation")
		}

		c1 := eth.NewBeaconNodeValidatorClient(conns[1])
		att1, err := h.getSlashableAttestation(i)
		if err != nil {
			return err
		}
		if _, err := c1.ProposeAttestation(ctx, att1); err != nil {
			return errors.Wrap(err, "could not propose attestation")
		}

		slashedIndices = append(slashedIndices, uint64(valIdx))
	}
	return nil
}

func proposeDoubleBlock(_ *e2eTypes.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	valClient := eth.NewBeaconNodeValidatorClient(conn)
	beaconClient := eth.NewBeaconChainClient(conn)

	ctx := context.Background()
	chainHead, err := beaconClient.GetChainHead(ctx, &emptypb.Empty{})
	if err != nil {
		return errors.Wrap(err, "could not get chain head")
	}
	_, privKeys, err := util.DeterministicDepositsAndKeys(params.BeaconConfig().MinGenesisActiveValidatorCount)
	if err != nil {
		return err
	}
	pubKeys := make([][]byte, len(privKeys))
	for i, priv := range privKeys {
		pubKeys[i] = priv.PublicKey().Marshal()
	}
	duties, err := valClient.GetDuties(ctx, &eth.DutiesRequest{
		Epoch:      chainHead.HeadEpoch,
		PublicKeys: pubKeys,
	})
	if err != nil {
		return errors.Wrap(err, "could not get duties")
	}

	var proposerIndex primitives.ValidatorIndex
	for i, duty := range duties.CurrentEpochDuties {
		if slice.IsInSlots(chainHead.HeadSlot-1, duty.ProposerSlots) {
			proposerIndex = primitives.ValidatorIndex(i)
			break
		}
	}

	validatorNum := int(params.BeaconConfig().MinGenesisActiveValidatorCount)
	beaconNodeNum := e2e.TestParams.BeaconNodeCount
	if validatorNum%beaconNodeNum != 0 {
		return errors.New("validator count is not easily divisible by beacon node count")
	}
	validatorsPerNode := validatorNum / beaconNodeNum

	// If the proposer index is in the second validator client, we connect to
	// the corresponding beacon node instead.
	if proposerIndex >= primitives.ValidatorIndex(uint64(validatorsPerNode)) {
		valClient = eth.NewBeaconNodeValidatorClient(conns[1])
	}

	b, err := generateSignedBeaconBlock(chainHead, proposerIndex, valClient, privKeys, "bad state root")
	if err != nil {
		return err
	}
	if _, err = valClient.ProposeBeaconBlock(ctx, b); err == nil {
		return errors.New("expected block to fail processing")
	}

	b, err = generateSignedBeaconBlock(chainHead, proposerIndex, valClient, privKeys, "bad state root 2")
	if err != nil {
		return err
	}
	if _, err = valClient.ProposeBeaconBlock(ctx, b); err == nil {
		return errors.New("expected block to fail processing")
	}

	slashedIndices = append(slashedIndices, uint64(proposerIndex))
	return nil
}

func generateSignedBeaconBlock(
	chainHead *eth.ChainHead,
	proposerIndex primitives.ValidatorIndex,
	valClient eth.BeaconNodeValidatorClient,
	privKeys []bls.SecretKey,
	stateRoot string,
) (*eth.GenericSignedBeaconBlock, error) {
	ctx := context.Background()

	hashLen := 32
	blk := &eth.BeaconBlock{
		Slot:          chainHead.HeadSlot - 1,
		ParentRoot:    chainHead.HeadBlockRoot,
		StateRoot:     bytesutil.PadTo([]byte(stateRoot), hashLen),
		ProposerIndex: proposerIndex,
		Body: &eth.BeaconBlockBody{
			Eth1Data: &eth.Eth1Data{
				BlockHash:    bytesutil.PadTo([]byte("bad block hash"), hashLen),
				DepositRoot:  bytesutil.PadTo([]byte("bad deposit root"), hashLen),
				DepositCount: 1,
			},
			RandaoReveal:      bytesutil.PadTo([]byte("bad randao"), fieldparams.BLSSignatureLength),
			Graffiti:          bytesutil.PadTo([]byte("teehee"), hashLen),
			ProposerSlashings: []*eth.ProposerSlashing{},
			AttesterSlashings: []*eth.AttesterSlashing{},
			Attestations:      []*eth.Attestation{},
			Deposits:          []*eth.Deposit{},
			VoluntaryExits:    []*eth.SignedVoluntaryExit{},
		},
	}

	req := &eth.DomainRequest{
		Epoch:  chainHead.HeadEpoch,
		Domain: params.BeaconConfig().DomainBeaconProposer[:],
	}
	resp, err := valClient.DomainData(ctx, req)
	if err != nil {
		return nil, errors.Wrap(err, "could not get domain data")
	}
	signingRoot, err := signing.ComputeSigningRoot(blk, resp.SignatureDomain)
	if err != nil {
		return nil, errors.Wrap(err, "could not compute signing root")
	}
	sig := privKeys[proposerIndex].Sign(signingRoot[:]).Marshal()
	signedBlk := &eth.SignedBeaconBlock{
		Block:     blk,
		Signature: sig,
	}

	// We only broadcast to conns[0] here since we can trust that at least 1 node will be online.
	// Only broadcasting the attestation to one node also helps test slashing propagation.
	wb, err := blocks.NewSignedBeaconBlock(signedBlk)
	if err != nil {
		return nil, err
	}
	return wb.PbGenericBlock()
}
