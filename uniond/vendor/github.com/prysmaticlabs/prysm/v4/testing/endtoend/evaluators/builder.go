package evaluators

import (
	"context"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/ssz"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/policies"
	e2etypes "github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/types/known/emptypb"
)

// BuilderIsActive checks that the builder is indeed producing the respective payloads
var BuilderIsActive = e2etypes.Evaluator{
	Name: "builder_is_active_at_epoch_%d",
	Policy: func(e primitives.Epoch) bool {
		fEpoch := params.BeaconConfig().BellatrixForkEpoch
		return policies.OnwardsNthEpoch(fEpoch)(e)
	},
	Evaluation: builderActive,
}

func builderActive(_ *e2etypes.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	client := ethpb.NewNodeClient(conn)
	beaconClient := ethpb.NewBeaconChainClient(conn)
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

	if lowestBound < params.BeaconConfig().BellatrixForkEpoch {
		lowestBound = params.BeaconConfig().BellatrixForkEpoch
	}
	emptyRt, err := ssz.TransactionsRoot([][]byte{})
	if err != nil {
		return err
	}
	blockCtrs, err := beaconClient.ListBeaconBlocks(context.Background(), &ethpb.ListBlocksRequest{QueryFilter: &ethpb.ListBlocksRequest_Epoch{Epoch: lowestBound}})
	if err != nil {
		return errors.Wrap(err, "failed to get beacon blocks")
	}
	for _, ctr := range blockCtrs.BlockContainers {
		b, err := syncCompatibleBlockFromCtr(ctr)
		if err != nil {
			return errors.Wrapf(err, "block type doesn't exist for block at epoch %d", lowestBound)
		}

		if b.IsNil() {
			return errors.New("nil block provided")
		}
		forkStartSlot, err := slots.EpochStart(params.BeaconConfig().BellatrixForkEpoch)
		if err != nil {
			return err
		}
		if forkStartSlot == b.Block().Slot() || forkStartSlot+1 == b.Block().Slot() {
			// Skip fork slot and the next one, as we don't send FCUs yet.
			continue
		}
		execPayload, err := b.Block().Body().Execution()
		if err != nil {
			return err
		}
		txRoot, err := execPayload.TransactionsRoot()
		if err != nil {
			return err
		}
		if [32]byte(txRoot) == emptyRt && string(execPayload.ExtraData()) != "prysm-builder" {
			// If a local payload is built with 0 transactions, builder cannot build a payload with more transactions
			// since they both utilize the same EL.
			continue
		}
		if string(execPayload.ExtraData()) != "prysm-builder" {
			return errors.Errorf("block with slot %d was not built by the builder. It has an extra data of %s", b.Block().Slot(), string(execPayload.ExtraData()))
		}
		if execPayload.GasLimit() == 0 {
			return errors.Errorf("block with slot %d has a gas limit of 0, when it should be in the 30M range", b.Block().Slot())
		}
	}
	if lowestBound == currEpoch {
		return nil
	}
	blockCtrs, err = beaconClient.ListBeaconBlocks(context.Background(), &ethpb.ListBlocksRequest{QueryFilter: &ethpb.ListBlocksRequest_Epoch{Epoch: currEpoch}})
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
		forkStartSlot, err := slots.EpochStart(params.BeaconConfig().BellatrixForkEpoch)
		if err != nil {
			return err
		}
		if forkStartSlot == b.Block().Slot() || forkStartSlot+1 == b.Block().Slot() {
			// Skip fork slot and the next one, as we don't send FCUs yet.
			continue
		}
		execPayload, err := b.Block().Body().Execution()
		if err != nil {
			return err
		}
		txRoot, err := execPayload.TransactionsRoot()
		if err != nil {
			return err
		}
		if [32]byte(txRoot) == emptyRt && string(execPayload.ExtraData()) != "prysm-builder" {
			// If a local payload is built with 0 transactions, builder cannot build a payload with more transactions
			// since they both utilize the same EL.
			continue
		}
		if string(execPayload.ExtraData()) != "prysm-builder" {
			return errors.Errorf("block with slot %d was not built by the builder. It has an extra data of %s", b.Block().Slot(), string(execPayload.ExtraData()))
		}
		if execPayload.GasLimit() == 0 {
			return errors.Errorf("block with slot %d has a gas limit of 0, when it should be in the 30M range", b.Block().Slot())
		}
	}
	return nil
}
