package p2p

import (
	"fmt"
	"strings"

	pubsub "github.com/libp2p/go-libp2p-pubsub"
	pubsubpb "github.com/libp2p/go-libp2p-pubsub/pb"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/encoder"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
)

var _ pubsub.SubscriptionFilter = (*Service)(nil)

// It is set at this limit to handle the possibility
// of double topic subscriptions at fork boundaries.
// -> 64 Attestation Subnets * 2.
// -> 4 Sync Committee Subnets * 2.
// -> Block,Aggregate,ProposerSlashing,AttesterSlashing,Exits,SyncContribution * 2.
const pubsubSubscriptionRequestLimit = 200

// CanSubscribe returns true if the topic is of interest and we could subscribe to it.
func (s *Service) CanSubscribe(topic string) bool {
	if !s.isInitialized() {
		return false
	}
	parts := strings.Split(topic, "/")
	if len(parts) != 5 {
		return false
	}
	// The topic must start with a slash, which means the first part will be empty.
	if parts[0] != "" {
		return false
	}
	if parts[1] != "eth2" {
		return false
	}
	phase0ForkDigest, err := s.currentForkDigest()
	if err != nil {
		log.WithError(err).Error("Could not determine fork digest")
		return false
	}
	altairForkDigest, err := forks.ForkDigestFromEpoch(params.BeaconConfig().AltairForkEpoch, s.genesisValidatorsRoot)
	if err != nil {
		log.WithError(err).Error("Could not determine altair fork digest")
		return false
	}
	bellatrixForkDigest, err := forks.ForkDigestFromEpoch(params.BeaconConfig().BellatrixForkEpoch, s.genesisValidatorsRoot)
	if err != nil {
		log.WithError(err).Error("Could not determine Bellatrix fork digest")
		return false
	}
	capellaForkDigest, err := forks.ForkDigestFromEpoch(params.BeaconConfig().CapellaForkEpoch, s.genesisValidatorsRoot)
	if err != nil {
		log.WithError(err).Error("Could not determine Capella fork digest")
		return false
	}
	denebForkDigest, err := forks.ForkDigestFromEpoch(params.BeaconConfig().DenebForkEpoch, s.genesisValidatorsRoot)
	if err != nil {
		log.WithError(err).Error("Could not determine Deneb fork digest")
		return false
	}
	switch parts[2] {
	case fmt.Sprintf("%x", phase0ForkDigest):
	case fmt.Sprintf("%x", altairForkDigest):
	case fmt.Sprintf("%x", bellatrixForkDigest):
	case fmt.Sprintf("%x", capellaForkDigest):
	case fmt.Sprintf("%x", denebForkDigest):
	default:
		return false
	}

	if parts[4] != encoder.ProtocolSuffixSSZSnappy {
		return false
	}

	// Check the incoming topic matches any topic mapping. This includes a check for part[3].
	for gt := range gossipTopicMappings {
		if _, err := scanfcheck(strings.Join(parts[0:4], "/"), gt); err == nil {
			return true
		}
	}

	return false
}

// FilterIncomingSubscriptions is invoked for all RPCs containing subscription notifications.
// This method returns only the topics of interest and may return an error if the subscription
// request contains too many topics.
func (s *Service) FilterIncomingSubscriptions(_ peer.ID, subs []*pubsubpb.RPC_SubOpts) ([]*pubsubpb.RPC_SubOpts, error) {
	if len(subs) > pubsubSubscriptionRequestLimit {
		return nil, pubsub.ErrTooManySubscriptions
	}

	return pubsub.FilterSubscriptions(subs, s.CanSubscribe), nil
}

// scanfcheck uses fmt.Sscanf to check that a given string matches expected format. This method
// returns the number of formatting substitutions matched and error if the string does not match
// the expected format. Note: this method only accepts integer compatible formatting substitutions
// such as %d or %x.
func scanfcheck(input, format string) (int, error) {
	var t int
	// Sscanf requires argument pointers with the appropriate type to load the value from the input.
	// This method only checks that the input conforms to the format, the arguments are not used and
	// therefore we can reuse the same integer pointer.
	var cnt = strings.Count(format, "%")
	var args []interface{}
	for i := 0; i < cnt; i++ {
		args = append(args, &t)
	}
	return fmt.Sscanf(input, format, args...)
}
