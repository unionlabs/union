package sync

import (
	"context"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	opfeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/operation"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"google.golang.org/protobuf/proto"
)

func (s *Service) blsToExecutionChangeSubscriber(_ context.Context, msg proto.Message) error {
	blsMsg, ok := msg.(*ethpb.SignedBLSToExecutionChange)
	if !ok {
		return errors.Errorf("incorrect type of message received, wanted %T but got %T", &ethpb.SignedBLSToExecutionChange{}, msg)
	}
	s.cfg.operationNotifier.OperationFeed().Send(&feed.Event{
		Type: opfeed.BLSToExecutionChangeReceived,
		Data: &opfeed.BLSToExecutionChangeReceivedData{
			Change: blsMsg,
		},
	})
	s.cfg.blsToExecPool.InsertBLSToExecChange(blsMsg)
	return nil
}
