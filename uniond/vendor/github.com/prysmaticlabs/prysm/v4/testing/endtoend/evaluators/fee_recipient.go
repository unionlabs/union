package evaluators

import (
	"bytes"
	"context"
	"fmt"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/ethclient"
	"github.com/ethereum/go-ethereum/rpc"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/runtime/interop"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/components"
	e2e "github.com/prysmaticlabs/prysm/v4/testing/endtoend/params"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/policies"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	log "github.com/sirupsen/logrus"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/types/known/emptypb"
)

var FeeRecipientIsPresent = types.Evaluator{
	Name: "fee_recipient_is_present_%d",
	Policy: func(e primitives.Epoch) bool {
		fEpoch := params.BeaconConfig().BellatrixForkEpoch
		return policies.AfterNthEpoch(fEpoch)(e)
	},
	Evaluation: feeRecipientIsPresent,
}

func lhKeyMap() (map[string]bool, error) {
	if e2e.TestParams.LighthouseBeaconNodeCount == 0 {
		return nil, nil
	}
	pry, lh := e2e.TestParams.BeaconNodeCount, e2e.TestParams.LighthouseBeaconNodeCount
	valPerNode := int(params.BeaconConfig().MinGenesisActiveValidatorCount) / (pry + lh)
	lhOff := valPerNode * pry
	_, keys, err := interop.DeterministicallyGenerateKeys(uint64(lhOff), uint64(valPerNode*lh))
	if err != nil {
		return nil, err
	}

	km := make(map[string]bool)
	for _, k := range keys {
		km[hexutil.Encode(k.Marshal())] = true
	}
	return km, nil
}

func valKeyMap() (map[string]bool, error) {
	nvals := params.BeaconConfig().MinGenesisActiveValidatorCount
	// matches validator start in validator component + validators used for deposits
	_, pubs, err := interop.DeterministicallyGenerateKeys(0, nvals+e2e.DepositCount)
	if err != nil {
		return nil, err
	}
	km := make(map[string]bool)
	for _, k := range pubs {
		km[hexutil.Encode(k.Marshal())] = true
	}
	return km, nil
}

func feeRecipientIsPresent(_ *types.EvaluationContext, conns ...*grpc.ClientConn) error {
	conn := conns[0]
	client := ethpb.NewBeaconChainClient(conn)
	chainHead, err := client.GetChainHead(context.Background(), &emptypb.Empty{})
	if err != nil {
		return errors.Wrap(err, "failed to get chain head")
	}
	req := &ethpb.ListBlocksRequest{QueryFilter: &ethpb.ListBlocksRequest_Epoch{Epoch: chainHead.HeadEpoch.Sub(1)}}
	blks, err := client.ListBeaconBlocks(context.Background(), req)
	if err != nil {
		return errors.Wrap(err, "failed to list blocks")
	}

	rpcclient, err := rpc.DialHTTP(fmt.Sprintf("http://127.0.0.1:%d", e2e.TestParams.Ports.Eth1RPCPort))
	if err != nil {
		return err
	}
	defer rpcclient.Close()

	valkeys, err := valKeyMap()
	if err != nil {
		return err
	}
	lhkeys, err := lhKeyMap()
	if err != nil {
		return err
	}

	for _, ctr := range blks.BlockContainers {
		if ctr.GetBellatrixBlock() != nil {
			bb := ctr.GetBellatrixBlock().Block
			payload := bb.Body.ExecutionPayload
			// If the beacon chain has transitioned to Bellatrix, but the EL hasn't hit TTD, we could see a few slots
			// of blocks with empty payloads.
			if bytes.Equal(payload.BlockHash, make([]byte, 32)) {
				continue
			}
			if len(payload.FeeRecipient) == 0 || hexutil.Encode(payload.FeeRecipient) == params.BeaconConfig().EthBurnAddressHex {
				log.WithField("proposer_index", bb.ProposerIndex).WithField("slot", bb.Slot).Error("fee recipient eval bug")
				return errors.New("fee recipient is not set")
			}

			fr := common.BytesToAddress(payload.FeeRecipient)
			gvr := &ethpb.GetValidatorRequest{
				QueryFilter: &ethpb.GetValidatorRequest_Index{
					Index: ctr.GetBellatrixBlock().Block.ProposerIndex,
				},
			}
			validator, err := client.GetValidator(context.Background(), gvr)
			if err != nil {
				return errors.Wrap(err, "failed to get validators")
			}
			pk := hexutil.Encode(validator.GetPublicKey())

			if _, ok := lhkeys[pk]; ok {
				// Don't check lighthouse keys.
				continue
			}

			// In e2e we generate deterministic keys by validator index, and then use a slice of their public key bytes
			// as the fee recipient, so that this will also be deterministic, so this test can statelessly verify it.
			// These should be the only keys we see.
			// Otherwise something has changed in e2e and this test needs to be updated.
			_, knownKey := valkeys[pk]
			if !knownKey {
				log.WithField("pubkey", pk).
					WithField("slot", bb.Slot).
					WithField("proposer_index", bb.ProposerIndex).
					WithField("fee_recipient", fr.Hex()).
					Warn("unknown key observed, not a deterministically generated key")
				return errors.New("unknown key observed, not a deterministically generated key")
			}

			if components.FeeRecipientFromPubkey(pk) != fr.Hex() {
				return fmt.Errorf("publickey %s, fee recipient %s does not match the proposer settings fee recipient %s",
					pk, fr.Hex(), components.FeeRecipientFromPubkey(pk))
			}

			if err := checkRecipientBalance(rpcclient, common.BytesToHash(payload.BlockHash), common.BytesToHash(payload.ParentHash), fr); err != nil {
				return err
			}
		}
	}

	return nil
}

func checkRecipientBalance(c *rpc.Client, block, parent common.Hash, account common.Address) error {
	web3 := ethclient.NewClient(c)
	ctx := context.Background()
	b, err := web3.BlockByHash(ctx, block)
	if err != nil {
		return err
	}

	bal, err := web3.BalanceAt(ctx, account, b.Number())
	if err != nil {
		return err
	}
	pBlock, err := web3.BlockByHash(ctx, parent)
	if err != nil {
		return err
	}
	pBal, err := web3.BalanceAt(ctx, account, pBlock.Number())
	if err != nil {
		return err
	}
	if b.GasUsed() > 0 && bal.Uint64() <= pBal.Uint64() {
		return errors.Errorf("account balance didn't change after applying fee recipient for account: %s", account.Hex())
	}

	return nil
}
