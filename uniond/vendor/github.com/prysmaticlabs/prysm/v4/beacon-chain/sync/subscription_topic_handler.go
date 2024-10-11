package sync

import (
	"sync"

	pubsub "github.com/libp2p/go-libp2p-pubsub"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
)

// This is a subscription topic handler that is used to handle basic
// CRUD operations on the topic map. All operations are thread safe
// so they can be called from multiple routines.
type subTopicHandler struct {
	sync.RWMutex
	subTopics map[string]*pubsub.Subscription
	digestMap map[[4]byte]int
}

func newSubTopicHandler() *subTopicHandler {
	return &subTopicHandler{
		subTopics: map[string]*pubsub.Subscription{},
		digestMap: map[[4]byte]int{},
	}
}

func (s *subTopicHandler) addTopic(topic string, sub *pubsub.Subscription) {
	s.Lock()
	defer s.Unlock()
	s.subTopics[topic] = sub
	digest, err := p2p.ExtractGossipDigest(topic)
	if err != nil {
		log.WithError(err).Error("Could not retrieve digest")
		return
	}
	s.digestMap[digest] += 1
}

func (s *subTopicHandler) topicExists(topic string) bool {
	s.RLock()
	defer s.RUnlock()
	_, ok := s.subTopics[topic]
	return ok
}

func (s *subTopicHandler) removeTopic(topic string) {
	s.Lock()
	defer s.Unlock()
	delete(s.subTopics, topic)
	digest, err := p2p.ExtractGossipDigest(topic)
	if err != nil {
		log.WithError(err).Error("Could not retrieve digest")
		return
	}
	currAmt, ok := s.digestMap[digest]
	// Should never be possible, is a
	// defensive check.
	if !ok || currAmt <= 0 {
		delete(s.digestMap, digest)
		return
	}
	s.digestMap[digest] -= 1
	if s.digestMap[digest] == 0 {
		delete(s.digestMap, digest)
	}
}

func (s *subTopicHandler) digestExists(digest [4]byte) bool {
	s.RLock()
	defer s.RUnlock()

	count, ok := s.digestMap[digest]
	return ok && count > 0
}

func (s *subTopicHandler) allTopics() []string {
	s.RLock()
	defer s.RUnlock()
	var topics []string
	for t := range s.subTopics {
		copiedTopic := t
		topics = append(topics, copiedTopic)
	}
	return topics
}

func (s *subTopicHandler) subForTopic(topic string) *pubsub.Subscription {
	s.RLock()
	defer s.RUnlock()
	return s.subTopics[topic]
}
