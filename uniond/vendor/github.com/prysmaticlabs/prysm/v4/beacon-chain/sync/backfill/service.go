package backfill

import (
	"context"

	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/proto/dbval"
	"github.com/prysmaticlabs/prysm/v4/runtime"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	log "github.com/sirupsen/logrus"
)

type Service struct {
	ctx           context.Context
	store         *Store
	ms            minimumSlotter
	cw            startup.ClockWaiter
	enabled       bool // service is disabled by default while feature is experimental
	nWorkers      int
	batchSeq      *batchSequencer
	batchSize     uint64
	pool          batchWorkerPool
	verifier      *verifier
	p2p           p2p.P2P
	pa            PeerAssigner
	batchImporter batchImporter
}

var _ runtime.Service = (*Service)(nil)

type ServiceOption func(*Service) error

func WithEnableBackfill(enabled bool) ServiceOption {
	return func(s *Service) error {
		s.enabled = enabled
		return nil
	}
}

func WithWorkerCount(n int) ServiceOption {
	return func(s *Service) error {
		s.nWorkers = n
		return nil
	}
}

func WithBatchSize(n uint64) ServiceOption {
	return func(s *Service) error {
		s.batchSize = n
		return nil
	}
}

type minimumSlotter interface {
	minimumSlot() primitives.Slot
	setClock(*startup.Clock)
}

type defaultMinimumSlotter struct {
	clock *startup.Clock
	cw    startup.ClockWaiter
	ctx   context.Context
}

func (d defaultMinimumSlotter) minimumSlot() primitives.Slot {
	if d.clock == nil {
		var err error
		d.clock, err = d.cw.WaitForClock(d.ctx)
		if err != nil {
			log.WithError(err).Fatal("failed to obtain system/genesis clock, unable to start backfill service")
		}
	}
	return minimumBackfillSlot(d.clock.CurrentSlot())
}

func (d defaultMinimumSlotter) setClock(c *startup.Clock) {
	//nolint:all
	d.clock = c
}

var _ minimumSlotter = &defaultMinimumSlotter{}

type batchImporter func(ctx context.Context, b batch, su *Store) (*dbval.BackfillStatus, error)

func defaultBatchImporter(ctx context.Context, b batch, su *Store) (*dbval.BackfillStatus, error) {
	status := su.status()
	if err := b.ensureParent(bytesutil.ToBytes32(status.LowParentRoot)); err != nil {
		return status, err
	}
	// Import blocks to db and update db state to reflect the newly imported blocks.
	// Other parts of the beacon node may use the same StatusUpdater instance
	// via the coverage.AvailableBlocker interface to safely determine if a given slot has been backfilled.
	status, err := su.fillBack(ctx, b.results)
	if err != nil {
		log.WithError(err).Fatal("Non-recoverable db error in backfill service, quitting.")
	}
	return status, nil
}

// PeerAssigner describes a type that provides an Assign method, which can assign the best peer
// to service an RPC request. The Assign method takes a map of peers that should be excluded,
// allowing the caller to avoid making multiple concurrent requests to the same peer.
type PeerAssigner interface {
	Assign(busy map[peer.ID]bool, n int) ([]peer.ID, error)
}

// NewService initializes the backfill Service. Like all implementations of the Service interface,
// the service won't begin its runloop until Start() is called.
func NewService(ctx context.Context, su *Store, cw startup.ClockWaiter, p p2p.P2P, pa PeerAssigner, opts ...ServiceOption) (*Service, error) {
	s := &Service{
		ctx:           ctx,
		store:         su,
		cw:            cw,
		ms:            &defaultMinimumSlotter{cw: cw, ctx: ctx},
		p2p:           p,
		pa:            pa,
		batchImporter: defaultBatchImporter,
	}
	for _, o := range opts {
		if err := o(s); err != nil {
			return nil, err
		}
	}
	s.pool = newP2PBatchWorkerPool(p, s.nWorkers)

	return s, nil
}

func (s *Service) initVerifier(ctx context.Context) (*verifier, error) {
	cps, err := s.store.originState(ctx)
	if err != nil {
		return nil, err
	}
	keys, err := cps.PublicKeys()
	if err != nil {
		return nil, errors.Wrap(err, "Unable to retrieve public keys for all validators in the origin state")
	}
	return newBackfillVerifier(cps.GenesisValidatorsRoot(), keys)
}

func (s *Service) updateComplete() bool {
	b, err := s.pool.complete()
	if err != nil {
		if errors.Is(err, errEndSequence) {
			log.WithField("backfill_slot", b.begin).Info("Backfill is complete")
			return true
		}
		log.WithError(err).Fatal("Non-recoverable error in backfill service, quitting.")
	}
	s.batchSeq.update(b)
	return false
}

func (s *Service) importBatches(ctx context.Context) {
	importable := s.batchSeq.importable()
	imported := 0
	defer func() {
		if imported == 0 {
			return
		}
		backfillBatchesImported.Add(float64(imported))
	}()
	for i := range importable {
		ib := importable[i]
		if len(ib.results) == 0 {
			log.WithFields(ib.logFields()).Error("Batch with no results, skipping importer.")
		}
		_, err := s.batchImporter(ctx, ib, s.store)
		if err != nil {
			log.WithError(err).WithFields(ib.logFields()).Debug("Backfill batch failed to import.")
			s.downscore(ib)
			s.batchSeq.update(ib.withState(batchErrRetryable))
			// If a batch fails, the subsequent batches are no longer considered importable.
			break
		}
		s.batchSeq.update(ib.withState(batchImportComplete))
		imported += 1
		// Calling update with state=batchImportComplete will advance the batch list.
	}

	nt := s.batchSeq.numTodo()
	log.WithField("imported", imported).WithField("importable", len(importable)).
		WithField("batches_remaining", nt).
		Info("Backfill batches processed.")

	backfillRemainingBatches.Set(float64(nt))
}

func (s *Service) scheduleTodos() {
	batches, err := s.batchSeq.sequence()
	if err != nil {
		// This typically means we have several importable batches, but they are stuck behind a batch that needs
		// to complete first so that we can chain parent roots across batches.
		// ie backfilling [[90..100), [80..90), [70..80)], if we complete [70..80) and [80..90) but not [90..100),
		// we can't move forward until [90..100) completes, because we need to confirm 99 connects to 100,
		// and then we'll have the parent_root expected by 90 to ensure it matches the root for 89,
		// at which point we know we can process [80..90).
		if errors.Is(err, errMaxBatches) {
			log.Debug("Backfill batches waiting for descendent batch to complete.")
			return
		}
	}
	for _, b := range batches {
		s.pool.todo(b)
	}
}

// Start begins the runloop of backfill.Service in the current goroutine.
func (s *Service) Start() {
	if !s.enabled {
		log.Info("Exiting backfill service; not enabled.")
		return
	}
	ctx, cancel := context.WithCancel(s.ctx)
	defer func() {
		cancel()
	}()
	clock, err := s.cw.WaitForClock(ctx)
	if err != nil {
		log.WithError(err).Fatal("Backfill service failed to start while waiting for genesis data.")
	}
	s.ms.setClock(clock)

	if s.store.isGenesisSync() {
		log.Info("Exiting backfill service as the node has been initialized with a genesis state or the backfill status is missing")
		return
	}
	status := s.store.status()
	// Exit early if there aren't going to be any batches to backfill.
	if primitives.Slot(status.LowSlot) <= s.ms.minimumSlot() {
		log.WithField("minimum_required_slot", s.ms.minimumSlot()).
			WithField("backfill_lowest_slot", status.LowSlot).
			Info("Exiting backfill service; minimum block retention slot > lowest backfilled block.")
		return
	}
	s.verifier, err = s.initVerifier(ctx)
	if err != nil {
		log.WithError(err).Fatal("Unable to initialize backfill verifier, quitting.")
	}
	s.pool.spawn(ctx, s.nWorkers, clock, s.pa, s.verifier)

	s.batchSeq = newBatchSequencer(s.nWorkers, s.ms.minimumSlot(), primitives.Slot(status.LowSlot), primitives.Slot(s.batchSize))
	if err = s.initBatches(); err != nil {
		log.WithError(err).Fatal("Non-recoverable error in backfill service, quitting.")
	}

	for {
		if ctx.Err() != nil {
			return
		}
		if allComplete := s.updateComplete(); allComplete {
			return
		}
		s.importBatches(ctx)
		batchesWaiting.Set(float64(s.batchSeq.countWithState(batchImportable)))
		if err := s.batchSeq.moveMinimum(s.ms.minimumSlot()); err != nil {
			log.WithError(err).Fatal("Non-recoverable error in backfill service, quitting.")
		}
		s.scheduleTodos()
	}
}

func (s *Service) initBatches() error {
	batches, err := s.batchSeq.sequence()
	if err != nil {
		return err
	}
	for _, b := range batches {
		s.pool.todo(b)
	}
	return nil
}

func (s *Service) downscore(b batch) {
	s.p2p.Peers().Scorers().BadResponsesScorer().Increment(b.pid)
}

func (s *Service) Stop() error {
	return nil
}

func (s *Service) Status() error {
	return nil
}

// minimumBackfillSlot determines the lowest slot that backfill needs to download based on looking back
// MIN_EPOCHS_FOR_BLOCK_REQUESTS from the current slot.
func minimumBackfillSlot(current primitives.Slot) primitives.Slot {
	oe := helpers.MinEpochsForBlockRequests()
	if oe > slots.MaxSafeEpoch() {
		oe = slots.MaxSafeEpoch()
	}
	offset := slots.UnsafeEpochStart(oe)
	if offset >= current {
		// Slot 0 is the genesis block, therefore the signature in it is invalid.
		// To prevent us from rejecting a batch, we restrict the minimum backfill batch till only slot 1
		return 1
	}
	return current - offset
}
