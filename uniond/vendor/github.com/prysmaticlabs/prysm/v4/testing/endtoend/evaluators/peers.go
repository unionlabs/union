package evaluators

import (
	"context"

	"github.com/pkg/errors"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/policies"
	"github.com/prysmaticlabs/prysm/v4/testing/endtoend/types"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/types/known/emptypb"
)

// PeersCheck performs a check on peer data to ensure that any connected peers
// are not publishing invalid data.
var PeersCheck = types.Evaluator{
	Name:       "peers_check_epoch_%d",
	Policy:     policies.AfterNthEpoch(0),
	Evaluation: peersTest,
}

func peersTest(_ *types.EvaluationContext, conns ...*grpc.ClientConn) error {
	debugClient := eth.NewDebugClient(conns[0])

	peerResponses, err := debugClient.ListPeers(context.Background(), &emptypb.Empty{})
	if err != nil {
		return err
	}
	baseErr := error(nil)
	for _, res := range peerResponses.Responses {
		if res.ScoreInfo.GossipScore < 0 {
			baseErr = wrapError(baseErr, "Gossip score for peer %s is %f and negative.", res.PeerId, res.ScoreInfo.GossipScore)
		}
		if res.ScoreInfo.BehaviourPenalty > 0 {
			baseErr = wrapError(baseErr, "Behaviour penalty for peer %s is %f and larger than zero.", res.PeerId, res.ScoreInfo.BehaviourPenalty)
		}
		if res.ScoreInfo.BlockProviderScore < 0 {
			baseErr = wrapError(baseErr, "Block provider score for peer %s is %f and negative.", res.PeerId, res.ScoreInfo.BlockProviderScore)
		}
		if res.ScoreInfo.OverallScore < 0 {
			baseErr = wrapError(baseErr, "Overall score for peer %s is %f and negative.", res.PeerId, res.ScoreInfo.OverallScore)
		}
		if res.ScoreInfo.ValidationError != "" {
			baseErr = wrapError(baseErr, "Peer %s has a validation error: %s", res.PeerId, res.ScoreInfo.ValidationError)
		}
		if res.PeerInfo != nil && res.PeerInfo.FaultCount > 0 {
			baseErr = wrapError(baseErr, "Peer %s has a non zero fault count: %d", res.PeerId, res.PeerInfo.FaultCount)
		}
		for topic, snap := range res.ScoreInfo.TopicScores {
			if snap.InvalidMessageDeliveries > 0 {
				baseErr = wrapError(baseErr, "Peer %s in Topic %s has sent invalid deliveries: %f", res.PeerId, topic, snap.InvalidMessageDeliveries)
			}
		}
	}
	return baseErr
}

func wrapError(err error, format string, args ...interface{}) error {
	if err == nil {
		err = errors.New("")
	}
	return errors.Wrapf(err, format, args...)
}
