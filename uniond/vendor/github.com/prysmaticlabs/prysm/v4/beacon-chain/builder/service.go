package builder

import (
	"context"
	"reflect"
	"time"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/api/client/builder"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	v1 "github.com/prysmaticlabs/prysm/v4/proto/engine/v1"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	log "github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
)

// ErrNoBuilder is used when builder endpoint is not configured.
var ErrNoBuilder = errors.New("builder endpoint not configured")

// BlockBuilder defines the interface for interacting with the block builder
type BlockBuilder interface {
	SubmitBlindedBlock(ctx context.Context, block interfaces.ReadOnlySignedBeaconBlock) (interfaces.ExecutionData, *v1.BlobsBundle, error)
	GetHeader(ctx context.Context, slot primitives.Slot, parentHash [32]byte, pubKey [48]byte) (builder.SignedBid, error)
	RegisterValidator(ctx context.Context, reg []*ethpb.SignedValidatorRegistrationV1) error
	RegistrationByValidatorID(ctx context.Context, id primitives.ValidatorIndex) (*ethpb.ValidatorRegistrationV1, error)
	Configured() bool
}

// config defines a config struct for dependencies into the service.
type config struct {
	builderClient builder.BuilderClient
	beaconDB      db.HeadAccessDatabase
	headFetcher   blockchain.HeadFetcher
}

// Service defines a service that provides a client for interacting with the beacon chain and MEV relay network.
type Service struct {
	cfg               *config
	c                 builder.BuilderClient
	ctx               context.Context
	cancel            context.CancelFunc
	registrationCache *cache.RegistrationCache
}

// NewService instantiates a new service.
func NewService(ctx context.Context, opts ...Option) (*Service, error) {
	ctx, cancel := context.WithCancel(ctx)
	s := &Service{
		ctx:    ctx,
		cancel: cancel,
		cfg:    &config{},
	}
	for _, opt := range opts {
		if err := opt(s); err != nil {
			return nil, err
		}
	}
	if s.cfg.builderClient != nil && !reflect.ValueOf(s.cfg.builderClient).IsNil() {
		s.c = s.cfg.builderClient

		// Is the builder up?
		if err := s.c.Status(ctx); err != nil {
			log.WithError(err).Error("Failed to check builder status")
		} else {
			log.WithField("endpoint", s.c.NodeURL()).Info("Builder has been configured")
			log.Warn("Outsourcing block construction to external builders adds non-trivial delay to block propagation time.  " +
				"Builder-constructed blocks or fallback blocks may get orphaned. Use at your own risk!")
		}
	}
	return s, nil
}

// Start initializes the service.
func (s *Service) Start() {
	go s.pollRelayerStatus(s.ctx)
}

// Stop halts the service.
func (s *Service) Stop() error {
	s.cancel()
	return nil
}

// SubmitBlindedBlock submits a blinded block to the builder relay network.
func (s *Service) SubmitBlindedBlock(ctx context.Context, b interfaces.ReadOnlySignedBeaconBlock) (interfaces.ExecutionData, *v1.BlobsBundle, error) {
	ctx, span := trace.StartSpan(ctx, "builder.SubmitBlindedBlock")
	defer span.End()
	start := time.Now()
	defer func() {
		submitBlindedBlockLatency.Observe(float64(time.Since(start).Milliseconds()))
	}()
	if s.c == nil {
		return nil, nil, ErrNoBuilder
	}

	return s.c.SubmitBlindedBlock(ctx, b)
}

// GetHeader retrieves the header for a given slot and parent hash from the builder relay network.
func (s *Service) GetHeader(ctx context.Context, slot primitives.Slot, parentHash [32]byte, pubKey [48]byte) (builder.SignedBid, error) {
	ctx, span := trace.StartSpan(ctx, "builder.GetHeader")
	defer span.End()
	start := time.Now()
	defer func() {
		getHeaderLatency.Observe(float64(time.Since(start).Milliseconds()))
	}()
	if s.c == nil {
		tracing.AnnotateError(span, ErrNoBuilder)
		return nil, ErrNoBuilder
	}

	h, err := s.c.GetHeader(ctx, slot, parentHash, pubKey)
	tracing.AnnotateError(span, err)
	return h, err
}

// Status retrieves the status of the builder relay network.
func (s *Service) Status() error {
	// Return early if builder isn't initialized in service.
	if s.c == nil {
		return nil
	}

	return nil
}

// RegisterValidator registers a validator with the builder relay network.
// It also saves the registration object to the DB.
func (s *Service) RegisterValidator(ctx context.Context, reg []*ethpb.SignedValidatorRegistrationV1) error {
	ctx, span := trace.StartSpan(ctx, "builder.RegisterValidator")
	defer span.End()
	start := time.Now()
	defer func() {
		registerValidatorLatency.Observe(float64(time.Since(start).Milliseconds()))
	}()
	if s.c == nil {
		return ErrNoBuilder
	}

	// should be removed if db is removed
	idxs := make([]primitives.ValidatorIndex, 0)
	msgs := make([]*ethpb.ValidatorRegistrationV1, 0)

	indexToRegistration := make(map[primitives.ValidatorIndex]*ethpb.ValidatorRegistrationV1)

	valid := make([]*ethpb.SignedValidatorRegistrationV1, 0)
	for i := 0; i < len(reg); i++ {
		r := reg[i]
		nx, exists := s.cfg.headFetcher.HeadPublicKeyToValidatorIndex(bytesutil.ToBytes48(r.Message.Pubkey))
		if !exists {
			// we want to allow validators to set up keys that haven't been added to the beaconstate validator list yet,
			// so we should tolerate keys that do not seem to be valid by skipping past them.
			log.Warnf("Skipping validator registration for pubkey=%#x - not in current validator set.", r.Message.Pubkey)
			continue
		}
		idxs = append(idxs, nx)
		msgs = append(msgs, r.Message)
		valid = append(valid, r)
		indexToRegistration[nx] = r.Message
	}
	if err := s.c.RegisterValidator(ctx, valid); err != nil {
		return errors.Wrap(err, "could not register validator(s)")
	}

	if len(indexToRegistration) != len(msgs) {
		return errors.New("ids and registrations must be the same length")
	}
	if s.registrationCache != nil {
		s.registrationCache.UpdateIndexToRegisteredMap(ctx, indexToRegistration)
		return nil
	} else {
		return s.cfg.beaconDB.SaveRegistrationsByValidatorIDs(ctx, idxs, msgs)
	}
}

// RegistrationByValidatorID returns either the values from the cache or db.
func (s *Service) RegistrationByValidatorID(ctx context.Context, id primitives.ValidatorIndex) (*ethpb.ValidatorRegistrationV1, error) {
	if s.registrationCache != nil {
		return s.registrationCache.RegistrationByIndex(id)
	} else {
		if s.cfg == nil || s.cfg.beaconDB == nil {
			return nil, errors.New("nil beacon db")
		}
		return s.cfg.beaconDB.RegistrationByValidatorID(ctx, id)
	}
}

// Configured returns true if the user has configured a builder client.
func (s *Service) Configured() bool {
	return s.c != nil && !reflect.ValueOf(s.c).IsNil()
}

func (s *Service) pollRelayerStatus(ctx context.Context) {
	ticker := time.NewTicker(time.Minute)
	defer ticker.Stop()
	for {
		select {
		case <-ticker.C:
			if s.c != nil {
				if err := s.c.Status(ctx); err != nil {
					log.WithError(err).Error("Failed to call relayer status endpoint, perhaps mev-boost or relayers are down")
				}
			}
		case <-ctx.Done():
			return
		}
	}
}
