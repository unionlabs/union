package p2p

import (
	"context"
	"encoding/hex"
	"strings"
	"time"

	pubsub "github.com/libp2p/go-libp2p-pubsub"
	pubsubpb "github.com/libp2p/go-libp2p-pubsub/pb"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	pbrpc "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
)

const (
	// overlay parameters
	gossipSubD   = 8  // topic stable mesh target count
	gossipSubDlo = 6  // topic stable mesh low watermark
	gossipSubDhi = 12 // topic stable mesh high watermark

	// gossip parameters
	gossipSubMcacheLen    = 6   // number of windows to retain full messages in cache for `IWANT` responses
	gossipSubMcacheGossip = 3   // number of windows to gossip about
	gossipSubSeenTTL      = 550 // number of heartbeat intervals to retain message IDs

	// fanout ttl
	gossipSubFanoutTTL = 60000000000 // TTL for fanout maps for topics we are not subscribed to but have published to, in nano seconds

	// heartbeat interval
	gossipSubHeartbeatInterval = 700 * time.Millisecond // frequency of heartbeat, milliseconds

	// misc
	rSubD = 8 // random gossip target
)

var errInvalidTopic = errors.New("invalid topic format")

// Specifies the fixed size context length.
const digestLength = 4

// Specifies the prefix for any pubsub topic.
const gossipTopicPrefix = "/eth2/"

// JoinTopic will join PubSub topic, if not already joined.
func (s *Service) JoinTopic(topic string, opts ...pubsub.TopicOpt) (*pubsub.Topic, error) {
	s.joinedTopicsLock.Lock()
	defer s.joinedTopicsLock.Unlock()

	if _, ok := s.joinedTopics[topic]; !ok {
		topicHandle, err := s.pubsub.Join(topic, opts...)
		if err != nil {
			return nil, err
		}
		s.joinedTopics[topic] = topicHandle
	}

	return s.joinedTopics[topic], nil
}

// LeaveTopic closes topic and removes corresponding handler from list of joined topics.
// This method will return error if there are outstanding event handlers or subscriptions.
func (s *Service) LeaveTopic(topic string) error {
	s.joinedTopicsLock.Lock()
	defer s.joinedTopicsLock.Unlock()

	if t, ok := s.joinedTopics[topic]; ok {
		if err := t.Close(); err != nil {
			return err
		}
		delete(s.joinedTopics, topic)
	}
	return nil
}

// PublishToTopic joins (if necessary) and publishes a message to a PubSub topic.
func (s *Service) PublishToTopic(ctx context.Context, topic string, data []byte, opts ...pubsub.PubOpt) error {
	topicHandle, err := s.JoinTopic(topic)
	if err != nil {
		return err
	}

	// Wait for at least 1 peer to be available to receive the published message.
	for {
		if len(topicHandle.ListPeers()) > 0 || flags.Get().MinimumSyncPeers == 0 {
			return topicHandle.Publish(ctx, data, opts...)
		}
		select {
		case <-ctx.Done():
			return errors.Wrapf(ctx.Err(), "unable to find requisite number of peers for topic %s, 0 peers found to publish to", topic)
		default:
			time.Sleep(100 * time.Millisecond)
		}
	}
}

// SubscribeToTopic joins (if necessary) and subscribes to PubSub topic.
func (s *Service) SubscribeToTopic(topic string, opts ...pubsub.SubOpt) (*pubsub.Subscription, error) {
	s.awaitStateInitialized() // Genesis time and genesis validators root are required to subscribe.

	topicHandle, err := s.JoinTopic(topic)
	if err != nil {
		return nil, err
	}
	scoringParams, err := s.topicScoreParams(topic)
	if err != nil {
		return nil, err
	}

	if scoringParams != nil {
		if err := topicHandle.SetScoreParams(scoringParams); err != nil {
			return nil, err
		}
		logGossipParameters(topic, scoringParams)
	}
	return topicHandle.Subscribe(opts...)
}

// peerInspector will scrape all the relevant scoring data and add it to our
// peer handler.
func (s *Service) peerInspector(peerMap map[peer.ID]*pubsub.PeerScoreSnapshot) {
	// Iterate through all the connected peers and through any of their
	// relevant topics.
	for pid, snap := range peerMap {
		s.peers.Scorers().GossipScorer().SetGossipData(pid, snap.Score,
			snap.BehaviourPenalty, convertTopicScores(snap.Topics))
	}
}

// Creates a list of pubsub options to configure out router with.
func (s *Service) pubsubOptions() []pubsub.Option {
	psOpts := []pubsub.Option{
		pubsub.WithMessageSignaturePolicy(pubsub.StrictNoSign),
		pubsub.WithNoAuthor(),
		pubsub.WithMessageIdFn(func(pmsg *pubsubpb.Message) string {
			return MsgID(s.genesisValidatorsRoot, pmsg)
		}),
		pubsub.WithSubscriptionFilter(s),
		pubsub.WithPeerOutboundQueueSize(int(s.cfg.QueueSize)),
		pubsub.WithMaxMessageSize(int(params.BeaconConfig().GossipMaxSize)),
		pubsub.WithValidateQueueSize(int(s.cfg.QueueSize)),
		pubsub.WithPeerScore(peerScoringParams()),
		pubsub.WithPeerScoreInspect(s.peerInspector, time.Minute),
		pubsub.WithGossipSubParams(pubsubGossipParam()),
		pubsub.WithRawTracer(gossipTracer{host: s.host}),
	}
	return psOpts
}

// creates a custom gossipsub parameter set.
func pubsubGossipParam() pubsub.GossipSubParams {
	gParams := pubsub.DefaultGossipSubParams()
	gParams.Dlo = gossipSubDlo
	gParams.D = gossipSubD
	gParams.HeartbeatInterval = gossipSubHeartbeatInterval
	gParams.HistoryLength = gossipSubMcacheLen
	gParams.HistoryGossip = gossipSubMcacheGossip
	return gParams
}

// We have to unfortunately set this globally in order
// to configure our message id time-cache rather than instantiating
// it with a router instance.
func setPubSubParameters() {
	pubsub.TimeCacheDuration = 550 * gossipSubHeartbeatInterval
}

// convert from libp2p's internal schema to a compatible prysm protobuf format.
func convertTopicScores(topicMap map[string]*pubsub.TopicScoreSnapshot) map[string]*pbrpc.TopicScoreSnapshot {
	newMap := make(map[string]*pbrpc.TopicScoreSnapshot, len(topicMap))
	for t, s := range topicMap {
		newMap[t] = &pbrpc.TopicScoreSnapshot{
			TimeInMesh:               uint64(s.TimeInMesh.Milliseconds()),
			FirstMessageDeliveries:   float32(s.FirstMessageDeliveries),
			MeshMessageDeliveries:    float32(s.MeshMessageDeliveries),
			InvalidMessageDeliveries: float32(s.InvalidMessageDeliveries),
		}
	}
	return newMap
}

// ExtractGossipDigest extracts the relevant fork digest from the gossip topic.
// Topics are in the form of /eth2/{fork-digest}/{topic} and this method extracts the
// fork digest from the topic string to a 4 byte array.
func ExtractGossipDigest(topic string) ([4]byte, error) {
	// Ensure the topic prefix is correct.
	if len(topic) < len(gossipTopicPrefix)+1 || topic[:len(gossipTopicPrefix)] != gossipTopicPrefix {
		return [4]byte{}, errInvalidTopic
	}
	start := len(gossipTopicPrefix)
	end := strings.Index(topic[start:], "/")
	if end == -1 { // Ensure a topic suffix exists.
		return [4]byte{}, errInvalidTopic
	}
	end += start
	strDigest := topic[start:end]
	digest, err := hex.DecodeString(strDigest)
	if err != nil {
		return [4]byte{}, err
	}
	if len(digest) != digestLength {
		return [4]byte{}, errors.Errorf("invalid digest length wanted %d but got %d", digestLength, len(digest))
	}
	return bytesutil.ToBytes4(digest), nil
}
