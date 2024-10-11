// Package blockchain defines the life-cycle of the blockchain at the core of
// Ethereum, including processing of new blocks and attestations using proof of stake.
package blockchain

import (
	"bytes"
	"context"
	"fmt"
	"runtime"
	"sync"
	"time"

	"github.com/pkg/errors"
	"go.opencensus.io/trace"

	"github.com/prysmaticlabs/prysm/v4/async/event"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/kzg"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/cache"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed"
	statefeed "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/feed/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/helpers"
	coreTime "github.com/prysmaticlabs/prysm/v4/beacon-chain/core/time"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/transition"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db/filesystem"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/execution"
	f "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice"
	forkchoicetypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/forkchoice/types"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/attestations"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/blstoexec"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/slashings"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/operations/voluntaryexits"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/state/stategen"
	"github.com/prysmaticlabs/prysm/v4/config/features"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	ethpb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	prysmTime "github.com/prysmaticlabs/prysm/v4/time"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// Service represents a service that handles the internal
// logic of managing the full PoS beacon chain.
type Service struct {
	cfg                           *config
	ctx                           context.Context
	cancel                        context.CancelFunc
	genesisTime                   time.Time
	head                          *head
	headLock                      sync.RWMutex
	originBlockRoot               [32]byte // genesis root, or weak subjectivity checkpoint root, depending on how the node is initialized
	boundaryRoots                 [][32]byte
	checkpointStateCache          *cache.CheckpointStateCache
	initSyncBlocks                map[[32]byte]interfaces.ReadOnlySignedBeaconBlock
	initSyncBlocksLock            sync.RWMutex
	wsVerifier                    *WeakSubjectivityVerifier
	clockSetter                   startup.ClockSetter
	clockWaiter                   startup.ClockWaiter
	syncComplete                  chan struct{}
	blobNotifiers                 *blobNotifierMap
	blockBeingSynced              *currentlySyncingBlock
	blobStorage                   *filesystem.BlobStorage
	lastPublishedLightClientEpoch primitives.Epoch
}

// config options for the service.
type config struct {
	BeaconBlockBuf          int
	ChainStartFetcher       execution.ChainStartFetcher
	BeaconDB                db.HeadAccessDatabase
	DepositCache            cache.DepositCache
	PayloadIDCache          *cache.PayloadIDCache
	TrackedValidatorsCache  *cache.TrackedValidatorsCache
	AttPool                 attestations.Pool
	ExitPool                voluntaryexits.PoolManager
	SlashingPool            slashings.PoolManager
	BLSToExecPool           blstoexec.PoolManager
	P2p                     p2p.Broadcaster
	MaxRoutines             int
	StateNotifier           statefeed.Notifier
	ForkChoiceStore         f.ForkChoicer
	AttService              *attestations.Service
	StateGen                *stategen.State
	SlasherAttestationsFeed *event.Feed
	WeakSubjectivityCheckpt *ethpb.Checkpoint
	BlockFetcher            execution.POWBlockFetcher
	FinalizedStateAtStartUp state.BeaconState
	ExecutionEngineCaller   execution.EngineCaller
}

var ErrMissingClockSetter = errors.New("blockchain Service initialized without a startup.ClockSetter")

type blobNotifierMap struct {
	sync.RWMutex
	notifiers map[[32]byte]chan uint64
	seenIndex map[[32]byte][fieldparams.MaxBlobsPerBlock]bool
}

// notifyIndex notifies a blob by its index for a given root.
// It uses internal maps to keep track of seen indices and notifier channels.
func (bn *blobNotifierMap) notifyIndex(root [32]byte, idx uint64) {
	if idx >= fieldparams.MaxBlobsPerBlock {
		return
	}

	bn.Lock()
	seen := bn.seenIndex[root]
	if seen[idx] {
		bn.Unlock()
		return
	}
	seen[idx] = true
	bn.seenIndex[root] = seen

	// Retrieve or create the notifier channel for the given root.
	c, ok := bn.notifiers[root]
	if !ok {
		c = make(chan uint64, fieldparams.MaxBlobsPerBlock)
		bn.notifiers[root] = c
	}

	bn.Unlock()

	c <- idx
}

func (bn *blobNotifierMap) forRoot(root [32]byte) chan uint64 {
	bn.Lock()
	defer bn.Unlock()
	c, ok := bn.notifiers[root]
	if !ok {
		c = make(chan uint64, fieldparams.MaxBlobsPerBlock)
		bn.notifiers[root] = c
	}
	return c
}

func (bn *blobNotifierMap) delete(root [32]byte) {
	bn.Lock()
	defer bn.Unlock()
	delete(bn.seenIndex, root)
	delete(bn.notifiers, root)
}

// NewService instantiates a new block service instance that will
// be registered into a running beacon node.
func NewService(ctx context.Context, opts ...Option) (*Service, error) {
	var err error
	if params.DenebEnabled() {
		err = kzg.Start()
		if err != nil {
			return nil, errors.Wrap(err, "could not initialize go-kzg context")
		}
	}
	ctx, cancel := context.WithCancel(ctx)
	bn := &blobNotifierMap{
		notifiers: make(map[[32]byte]chan uint64),
		seenIndex: make(map[[32]byte][fieldparams.MaxBlobsPerBlock]bool),
	}
	srv := &Service{
		ctx:                  ctx,
		cancel:               cancel,
		boundaryRoots:        [][32]byte{},
		checkpointStateCache: cache.NewCheckpointStateCache(),
		initSyncBlocks:       make(map[[32]byte]interfaces.ReadOnlySignedBeaconBlock),
		blobNotifiers:        bn,
		cfg:                  &config{},
		blockBeingSynced:     &currentlySyncingBlock{roots: make(map[[32]byte]struct{})},
	}
	for _, opt := range opts {
		if err := opt(srv); err != nil {
			return nil, err
		}
	}
	if srv.clockSetter == nil {
		return nil, ErrMissingClockSetter
	}
	srv.wsVerifier, err = NewWeakSubjectivityVerifier(srv.cfg.WeakSubjectivityCheckpt, srv.cfg.BeaconDB)
	if err != nil {
		return nil, err
	}
	return srv, nil
}

// Start a blockchain service's main event loop.
func (s *Service) Start() {
	saved := s.cfg.FinalizedStateAtStartUp

	if saved != nil && !saved.IsNil() {
		if err := s.StartFromSavedState(saved); err != nil {
			log.Fatal(err)
		}
	} else {
		if err := s.startFromExecutionChain(); err != nil {
			log.Fatal(err)
		}
	}
	s.spawnProcessAttestationsRoutine()
	go s.runLateBlockTasks()
}

// Stop the blockchain service's main event loop and associated goroutines.
func (s *Service) Stop() error {
	defer s.cancel()

	// lock before accessing s.head, s.head.state, s.head.state.FinalizedCheckpoint().Root
	s.headLock.RLock()
	if s.cfg.StateGen != nil && s.head != nil && s.head.state != nil {
		r := s.head.state.FinalizedCheckpoint().Root
		s.headLock.RUnlock()
		// Save the last finalized state so that starting up in the following run will be much faster.
		if err := s.cfg.StateGen.ForceCheckpoint(s.ctx, r); err != nil {
			return err
		}
	} else {
		s.headLock.RUnlock()
	}
	// Save initial sync cached blocks to the DB before stop.
	return s.cfg.BeaconDB.SaveBlocks(s.ctx, s.getInitSyncBlocks())
}

// Status always returns nil unless there is an error condition that causes
// this service to be unhealthy.
func (s *Service) Status() error {
	optimistic, err := s.IsOptimistic(s.ctx)
	if err != nil {
		return errors.Wrap(err, "failed to check if service is optimistic")
	}
	if optimistic {
		return errors.New("service is optimistic, and only limited service functionality is provided " +
			"please check if execution layer is fully synced")
	}

	if s.originBlockRoot == params.BeaconConfig().ZeroHash {
		return errors.New("genesis state has not been created")
	}
	if runtime.NumGoroutine() > s.cfg.MaxRoutines {
		return fmt.Errorf("too many goroutines (%d)", runtime.NumGoroutine())
	}
	return nil
}

// StartFromSavedState initializes the blockchain using a previously saved finalized checkpoint.
func (s *Service) StartFromSavedState(saved state.BeaconState) error {
	log.Info("Blockchain data already exists in DB, initializing...")
	s.genesisTime = time.Unix(int64(saved.GenesisTime()), 0) // lint:ignore uintcast -- Genesis time will not exceed int64 in your lifetime.
	s.cfg.AttService.SetGenesisTime(saved.GenesisTime())

	originRoot, err := s.originRootFromSavedState(s.ctx)
	if err != nil {
		return err
	}
	s.originBlockRoot = originRoot

	if err := s.initializeHeadFromDB(s.ctx); err != nil {
		return errors.Wrap(err, "could not set up chain info")
	}
	spawnCountdownIfPreGenesis(s.ctx, s.genesisTime, s.cfg.BeaconDB)

	justified, err := s.cfg.BeaconDB.JustifiedCheckpoint(s.ctx)
	if err != nil {
		return errors.Wrap(err, "could not get justified checkpoint")
	}
	if justified == nil {
		return errNilJustifiedCheckpoint
	}
	finalized, err := s.cfg.BeaconDB.FinalizedCheckpoint(s.ctx)
	if err != nil {
		return errors.Wrap(err, "could not get finalized checkpoint")
	}
	if finalized == nil {
		return errNilFinalizedCheckpoint
	}

	fRoot := s.ensureRootNotZeros(bytesutil.ToBytes32(finalized.Root))
	s.cfg.ForkChoiceStore.Lock()
	defer s.cfg.ForkChoiceStore.Unlock()
	if err := s.cfg.ForkChoiceStore.UpdateJustifiedCheckpoint(s.ctx, &forkchoicetypes.Checkpoint{Epoch: justified.Epoch,
		Root: bytesutil.ToBytes32(justified.Root)}); err != nil {
		return errors.Wrap(err, "could not update forkchoice's justified checkpoint")
	}
	if err := s.cfg.ForkChoiceStore.UpdateFinalizedCheckpoint(&forkchoicetypes.Checkpoint{Epoch: finalized.Epoch,
		Root: bytesutil.ToBytes32(finalized.Root)}); err != nil {
		return errors.Wrap(err, "could not update forkchoice's finalized checkpoint")
	}
	s.cfg.ForkChoiceStore.SetGenesisTime(uint64(s.genesisTime.Unix()))

	st, err := s.cfg.StateGen.StateByRoot(s.ctx, fRoot)
	if err != nil {
		return errors.Wrap(err, "could not get finalized checkpoint state")
	}
	if err := s.cfg.ForkChoiceStore.InsertNode(s.ctx, st, fRoot); err != nil {
		return errors.Wrap(err, "could not insert finalized block to forkchoice")
	}
	if !features.Get().EnableStartOptimistic {
		lastValidatedCheckpoint, err := s.cfg.BeaconDB.LastValidatedCheckpoint(s.ctx)
		if err != nil {
			return errors.Wrap(err, "could not get last validated checkpoint")
		}
		if bytes.Equal(finalized.Root, lastValidatedCheckpoint.Root) {
			if err := s.cfg.ForkChoiceStore.SetOptimisticToValid(s.ctx, fRoot); err != nil {
				return errors.Wrap(err, "could not set finalized block as validated")
			}
		}
	}
	// not attempting to save initial sync blocks here, because there shouldn't be any until
	// after the statefeed.Initialized event is fired (below)
	if err := s.wsVerifier.VerifyWeakSubjectivity(s.ctx, finalized.Epoch); err != nil {
		// Exit run time if the node failed to verify weak subjectivity checkpoint.
		return errors.Wrap(err, "could not verify initial checkpoint provided for chain sync")
	}

	vr := bytesutil.ToBytes32(saved.GenesisValidatorsRoot())
	if err := s.clockSetter.SetClock(startup.NewClock(s.genesisTime, vr)); err != nil {
		return errors.Wrap(err, "failed to initialize blockchain service")
	}

	return nil
}

func (s *Service) originRootFromSavedState(ctx context.Context) ([32]byte, error) {
	// first check if we have started from checkpoint sync and have a root
	originRoot, err := s.cfg.BeaconDB.OriginCheckpointBlockRoot(ctx)
	if err == nil {
		return originRoot, nil
	}
	if !errors.Is(err, db.ErrNotFound) {
		return originRoot, errors.Wrap(err, "could not retrieve checkpoint sync chain origin data from db")
	}

	// we got here because OriginCheckpointBlockRoot gave us an ErrNotFound. this means the node was started from a genesis state,
	// so we should have a value for GenesisBlock
	genesisBlock, err := s.cfg.BeaconDB.GenesisBlock(ctx)
	if err != nil {
		return originRoot, errors.Wrap(err, "could not get genesis block from db")
	}
	if err := blocks.BeaconBlockIsNil(genesisBlock); err != nil {
		return originRoot, err
	}
	genesisBlkRoot, err := genesisBlock.Block().HashTreeRoot()
	if err != nil {
		return genesisBlkRoot, errors.Wrap(err, "could not get signing root of genesis block")
	}
	return genesisBlkRoot, nil
}

// initializeHeadFromDB uses the finalized checkpoint and head block found in the database to set the current head.
// Note that this may block until stategen replays blocks between the finalized and head blocks
// if the head sync flag was specified and the gap between the finalized and head blocks is at least 128 epochs long.
func (s *Service) initializeHeadFromDB(ctx context.Context) error {
	finalized, err := s.cfg.BeaconDB.FinalizedCheckpoint(ctx)
	if err != nil {
		return errors.Wrap(err, "could not get finalized checkpoint from db")
	}
	if finalized == nil {
		// This should never happen. At chain start, the finalized checkpoint
		// would be the genesis state and block.
		return errors.New("no finalized epoch in the database")
	}
	finalizedRoot := s.ensureRootNotZeros(bytesutil.ToBytes32(finalized.Root))
	var finalizedState state.BeaconState

	finalizedState, err = s.cfg.StateGen.Resume(ctx, s.cfg.FinalizedStateAtStartUp)
	if err != nil {
		return errors.Wrap(err, "could not get finalized state from db")
	}

	if finalizedState == nil || finalizedState.IsNil() {
		return errors.New("finalized state can't be nil")
	}

	finalizedBlock, err := s.getBlock(ctx, finalizedRoot)
	if err != nil {
		return errors.Wrap(err, "could not get finalized block")
	}
	if err := s.setHead(&head{
		finalizedRoot,
		finalizedBlock,
		finalizedState,
		finalizedBlock.Block().Slot(),
		false,
	}); err != nil {
		return errors.Wrap(err, "could not set head")
	}

	return nil
}

func (s *Service) startFromExecutionChain() error {
	log.Info("Waiting to reach the validator deposit threshold to start the beacon chain...")
	if s.cfg.ChainStartFetcher == nil {
		return errors.New("not configured execution chain")
	}
	go func() {
		stateChannel := make(chan *feed.Event, 1)
		stateSub := s.cfg.StateNotifier.StateFeed().Subscribe(stateChannel)
		defer stateSub.Unsubscribe()
		for {
			select {
			case e := <-stateChannel:
				if e.Type == statefeed.ChainStarted {
					data, ok := e.Data.(*statefeed.ChainStartedData)
					if !ok {
						log.Error("event data is not type *statefeed.ChainStartedData")
						return
					}
					log.WithField("starttime", data.StartTime).Debug("Received chain start event")
					s.onExecutionChainStart(s.ctx, data.StartTime)
					return
				}
			case <-s.ctx.Done():
				log.Debug("Context closed, exiting goroutine")
				return
			case err := <-stateSub.Err():
				log.WithError(err).Error("Subscription to state forRoot failed")
				return
			}
		}
	}()

	return nil
}

// onExecutionChainStart initializes a series of deposits from the ChainStart deposits in the eth1
// deposit contract, initializes the beacon chain's state, and kicks off the beacon chain.
func (s *Service) onExecutionChainStart(ctx context.Context, genesisTime time.Time) {
	preGenesisState := s.cfg.ChainStartFetcher.PreGenesisState()
	initializedState, err := s.initializeBeaconChain(ctx, genesisTime, preGenesisState, s.cfg.ChainStartFetcher.ChainStartEth1Data())
	if err != nil {
		log.WithError(err).Fatal("Could not initialize beacon chain")
	}
	// We start a counter to genesis, if needed.
	gRoot, err := initializedState.HashTreeRoot(s.ctx)
	if err != nil {
		log.WithError(err).Fatal("Could not hash tree root genesis state")
	}
	go slots.CountdownToGenesis(ctx, genesisTime, uint64(initializedState.NumValidators()), gRoot)

	vr := bytesutil.ToBytes32(initializedState.GenesisValidatorsRoot())
	if err := s.clockSetter.SetClock(startup.NewClock(genesisTime, vr)); err != nil {
		log.WithError(err).Fatal("failed to initialize blockchain service from execution start event")
	}
}

// initializes the state and genesis block of the beacon chain to persistent storage
// based on a genesis timestamp value obtained from the ChainStart event emitted
// by the ETH1.0 Deposit Contract and the POWChain service of the node.
func (s *Service) initializeBeaconChain(
	ctx context.Context,
	genesisTime time.Time,
	preGenesisState state.BeaconState,
	eth1data *ethpb.Eth1Data) (state.BeaconState, error) {
	ctx, span := trace.StartSpan(ctx, "beacon-chain.Service.initializeBeaconChain")
	defer span.End()
	s.genesisTime = genesisTime
	unixTime := uint64(genesisTime.Unix())

	genesisState, err := transition.OptimizedGenesisBeaconState(unixTime, preGenesisState, eth1data)
	if err != nil {
		return nil, errors.Wrap(err, "could not initialize genesis state")
	}

	if err := s.saveGenesisData(ctx, genesisState); err != nil {
		return nil, errors.Wrap(err, "could not save genesis data")
	}

	log.Info("Initialized beacon chain genesis state")

	// Clear out all pre-genesis data now that the state is initialized.
	s.cfg.ChainStartFetcher.ClearPreGenesisData()

	// Update committee shuffled indices for genesis epoch.
	if err := helpers.UpdateCommitteeCache(ctx, genesisState, 0); err != nil {
		return nil, err
	}
	if err := helpers.UpdateProposerIndicesInCache(ctx, genesisState, coreTime.CurrentEpoch(genesisState)); err != nil {
		return nil, err
	}

	s.cfg.AttService.SetGenesisTime(genesisState.GenesisTime())

	return genesisState, nil
}

// This gets called when beacon chain is first initialized to save genesis data (state, block, and more) in db.
func (s *Service) saveGenesisData(ctx context.Context, genesisState state.BeaconState) error {
	if err := s.cfg.BeaconDB.SaveGenesisData(ctx, genesisState); err != nil {
		return errors.Wrap(err, "could not save genesis data")
	}
	genesisBlk, err := s.cfg.BeaconDB.GenesisBlock(ctx)
	if err != nil || genesisBlk == nil || genesisBlk.IsNil() {
		return fmt.Errorf("could not load genesis block: %v", err)
	}
	genesisBlkRoot, err := genesisBlk.Block().HashTreeRoot()
	if err != nil {
		return errors.Wrap(err, "could not get genesis block root")
	}

	s.originBlockRoot = genesisBlkRoot
	s.cfg.StateGen.SaveFinalizedState(0 /*slot*/, genesisBlkRoot, genesisState)

	s.cfg.ForkChoiceStore.Lock()
	defer s.cfg.ForkChoiceStore.Unlock()
	if err := s.cfg.ForkChoiceStore.InsertNode(ctx, genesisState, genesisBlkRoot); err != nil {
		log.WithError(err).Fatal("Could not process genesis block for fork choice")
	}
	s.cfg.ForkChoiceStore.SetOriginRoot(genesisBlkRoot)
	// Set genesis as fully validated
	if err := s.cfg.ForkChoiceStore.SetOptimisticToValid(ctx, genesisBlkRoot); err != nil {
		return errors.Wrap(err, "Could not set optimistic status of genesis block to false")
	}
	s.cfg.ForkChoiceStore.SetGenesisTime(uint64(s.genesisTime.Unix()))

	if err := s.setHead(&head{
		genesisBlkRoot,
		genesisBlk,
		genesisState,
		genesisBlk.Block().Slot(),
		false,
	}); err != nil {
		log.WithError(err).Fatal("Could not set head")
	}
	return nil
}

// This returns true if block has been processed before. Two ways to verify the block has been processed:
// 1.) Check fork choice store.
// 2.) Check DB.
// Checking 1.) is ten times faster than checking 2.)
// this function requires a lock in forkchoice
func (s *Service) hasBlock(ctx context.Context, root [32]byte) bool {
	if s.cfg.ForkChoiceStore.HasNode(root) {
		return true
	}

	return s.cfg.BeaconDB.HasBlock(ctx, root)
}

func spawnCountdownIfPreGenesis(ctx context.Context, genesisTime time.Time, db db.HeadAccessDatabase) {
	currentTime := prysmTime.Now()
	if currentTime.After(genesisTime) {
		return
	}

	gState, err := db.GenesisState(ctx)
	if err != nil {
		log.WithError(err).Fatal("Could not retrieve genesis state")
	}
	gRoot, err := gState.HashTreeRoot(ctx)
	if err != nil {
		log.WithError(err).Fatal("Could not hash tree root genesis state")
	}
	go slots.CountdownToGenesis(ctx, genesisTime, uint64(gState.NumValidators()), gRoot)
}
