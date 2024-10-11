package p2p

import (
	pubsub "github.com/libp2p/go-libp2p-pubsub"
	"github.com/libp2p/go-libp2p/core/host"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/libp2p/go-libp2p/core/protocol"
	"github.com/prometheus/client_golang/prometheus"
)

var _ = pubsub.RawTracer(gossipTracer{})

// Initializes the values for the pubsub rpc action.
type action int

const (
	recv action = iota
	send
	drop
)

// This tracer is used to implement metrics collection for messages received
// and broadcasted through gossipsub.
type gossipTracer struct {
	host host.Host
}

// AddPeer .
func (g gossipTracer) AddPeer(p peer.ID, proto protocol.ID) {
	// no-op
}

// RemovePeer .
func (g gossipTracer) RemovePeer(p peer.ID) {
	// no-op
}

// Join .
func (g gossipTracer) Join(topic string) {
	pubsubTopicsActive.WithLabelValues(topic).Set(1)
}

// Leave .
func (g gossipTracer) Leave(topic string) {
	pubsubTopicsActive.WithLabelValues(topic).Set(0)
}

// Graft .
func (g gossipTracer) Graft(p peer.ID, topic string) {
	pubsubTopicsGraft.WithLabelValues(topic).Inc()
}

// Prune .
func (g gossipTracer) Prune(p peer.ID, topic string) {
	pubsubTopicsPrune.WithLabelValues(topic).Inc()
}

// ValidateMessage .
func (g gossipTracer) ValidateMessage(msg *pubsub.Message) {
	pubsubMessageValidate.WithLabelValues(*msg.Topic).Inc()
}

// DeliverMessage .
func (g gossipTracer) DeliverMessage(msg *pubsub.Message) {
	pubsubMessageDeliver.WithLabelValues(*msg.Topic).Inc()
}

// RejectMessage .
func (g gossipTracer) RejectMessage(msg *pubsub.Message, reason string) {
	pubsubMessageReject.WithLabelValues(*msg.Topic, reason).Inc()
}

// DuplicateMessage .
func (g gossipTracer) DuplicateMessage(msg *pubsub.Message) {
	pubsubMessageDuplicate.WithLabelValues(*msg.Topic).Inc()
}

// UndeliverableMessage .
func (g gossipTracer) UndeliverableMessage(msg *pubsub.Message) {
	pubsubMessageUndeliverable.WithLabelValues(*msg.Topic).Inc()
}

// ThrottlePeer .
func (g gossipTracer) ThrottlePeer(p peer.ID) {
	agent := agentFromPid(p, g.host.Peerstore())
	pubsubPeerThrottle.WithLabelValues(agent).Inc()
}

// RecvRPC .
func (g gossipTracer) RecvRPC(rpc *pubsub.RPC) {
	g.setMetricFromRPC(recv, pubsubRPCSubRecv, pubsubRPCPubRecv, pubsubRPCRecv, rpc)
}

// SendRPC .
func (g gossipTracer) SendRPC(rpc *pubsub.RPC, p peer.ID) {
	g.setMetricFromRPC(send, pubsubRPCSubSent, pubsubRPCPubSent, pubsubRPCSent, rpc)
}

// DropRPC .
func (g gossipTracer) DropRPC(rpc *pubsub.RPC, p peer.ID) {
	g.setMetricFromRPC(drop, pubsubRPCSubDrop, pubsubRPCPubDrop, pubsubRPCDrop, rpc)
}

func (g gossipTracer) setMetricFromRPC(act action, subCtr prometheus.Counter, pubCtr, ctrlCtr *prometheus.CounterVec, rpc *pubsub.RPC) {
	subCtr.Add(float64(len(rpc.Subscriptions)))
	if rpc.Control != nil {
		ctrlCtr.WithLabelValues("graft").Add(float64(len(rpc.Control.Graft)))
		ctrlCtr.WithLabelValues("prune").Add(float64(len(rpc.Control.Prune)))
		ctrlCtr.WithLabelValues("ihave").Add(float64(len(rpc.Control.Ihave)))
		ctrlCtr.WithLabelValues("iwant").Add(float64(len(rpc.Control.Iwant)))
	}
	for _, msg := range rpc.Publish {
		// For incoming messages from pubsub, we do not record metrics for them as these values
		// could be junk.
		if act == recv {
			continue
		}
		pubCtr.WithLabelValues(*msg.Topic).Inc()
	}
}
