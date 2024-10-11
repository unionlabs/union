//go:build fuzz

package sync

import (
	"context"
	"time"

	pubsub "github.com/libp2p/go-libp2p-pubsub"
	"github.com/libp2p/go-libp2p/core/peer"
	gcache "github.com/patrickmn/go-cache"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"google.golang.org/protobuf/proto"
)

// NewRegularSyncFuzz service without registering handlers.
func NewRegularSyncFuzz(opts ...Option) *Service {
	ctx, cancel := context.WithCancel(context.Background())
	r := &Service{
		cfg:                  &config{},
		ctx:                  ctx,
		cancel:               cancel,
		slotToPendingBlocks:  gcache.New(time.Second, 2*time.Second),
		seenPendingBlocks:    make(map[[32]byte]bool),
		blkRootToPendingAtts: make(map[[32]byte][]*ethpb.SignedAggregateAttestationAndProof),
	}
	r.rateLimiter = newRateLimiter(r.cfg.p2p)

	return r
}

// FuzzValidateBeaconBlockPubSub exports private method validateBeaconBlockPubSub for fuzz testing.
func (s *Service) FuzzValidateBeaconBlockPubSub(ctx context.Context, pid peer.ID, msg *pubsub.Message) pubsub.ValidationResult {
	res, err := s.validateBeaconBlockPubSub(ctx, pid, msg)
	_ = err
	return res
}

// FuzzBeaconBlockSubscriber exports private method beaconBlockSubscriber for fuzz testing.
func (s *Service) FuzzBeaconBlockSubscriber(ctx context.Context, msg proto.Message) error {
	return s.beaconBlockSubscriber(ctx, msg)
}

func (s *Service) InitCaches() {
	s.initCaches()
}
