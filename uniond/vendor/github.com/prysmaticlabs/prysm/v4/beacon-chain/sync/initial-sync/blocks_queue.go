package initialsync

import (
	"context"
	"errors"
	"time"

	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/db"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	beaconsync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"github.com/sirupsen/logrus"
)

const (
	// queueStopCallTimeout is time allowed for queue to release resources when quitting.
	queueStopCallTimeout = 1 * time.Second
	// pollingInterval defines how often state machine needs to check for new events.
	pollingInterval = 200 * time.Millisecond
	// staleEpochTimeout is an period after which epoch's state is considered stale.
	staleEpochTimeout = 1 * time.Second
	// skippedMachineTimeout is a period after which skipped machine is considered as stuck
	// and is reset (if machine is the last one, then all machines are reset and search for
	// skipped slot or backtracking takes place).
	skippedMachineTimeout = 10 * staleEpochTimeout
	// lookaheadSteps is a limit on how many forward steps are loaded into queue.
	// Each step is managed by assigned finite state machine. Must be >= 2.
	lookaheadSteps = 8
	// noRequiredPeersErrMaxRetries defines number of retries when no required peers are found.
	noRequiredPeersErrMaxRetries = 1000
	// noRequiredPeersErrRefreshInterval defines interval for which queue will be paused before
	// making the next attempt to obtain data.
	noRequiredPeersErrRefreshInterval = 15 * time.Second
	// maxResetAttempts number of times stale FSM is reset, before backtracking is triggered.
	maxResetAttempts = 4
	// startBackSlots defines number of slots before the current head, which defines a start position
	// of the initial machine. This allows more robustness in case of normal sync sets head to some
	// orphaned block: in that case starting earlier and re-fetching blocks allows to reorganize chain.
	startBackSlots = 32
)

var (
	errQueueCtxIsDone             = errors.New("queue's context is done, reinitialize")
	errQueueTakesTooLongToStop    = errors.New("queue takes too long to stop")
	errInvalidInitialState        = errors.New("invalid initial state")
	errInputNotFetchRequestParams = errors.New("input data is not type *fetchRequestParams")
	errNoRequiredPeers            = errors.New("no peers with required blocks are found")
)

const (
	modeStopOnFinalizedEpoch syncMode = iota
	modeNonConstrained
)

// syncMode specifies sync mod type.
type syncMode uint8

// blocksQueueConfig is a config to setup block queue service.
type blocksQueueConfig struct {
	blocksFetcher       *blocksFetcher
	chain               blockchainService
	clock               *startup.Clock
	ctxMap              beaconsync.ContextByteVersions
	highestExpectedSlot primitives.Slot
	p2p                 p2p.P2P
	db                  db.ReadOnlyDatabase
	mode                syncMode
}

// blocksQueue is a priority queue that serves as a intermediary between block fetchers (producers)
// and block processing goroutine (consumer). Consumer can rely on order of incoming blocks.
type blocksQueue struct {
	ctx                 context.Context
	cancel              context.CancelFunc
	smm                 *stateMachineManager
	blocksFetcher       *blocksFetcher
	chain               blockchainService
	highestExpectedSlot primitives.Slot
	mode                syncMode
	exitConditions      struct {
		noRequiredPeersErrRetries int
	}
	fetchedData chan *blocksQueueFetchedData // output channel for ready blocks
	staleEpochs map[primitives.Epoch]uint8   // counter to keep track of stale FSMs
	quit        chan struct{}                // termination notifier
}

// blocksQueueFetchedData is a data container that is returned from a queue on each step.
type blocksQueueFetchedData struct {
	pid peer.ID
	bwb []blocks.BlockWithROBlobs
}

// newBlocksQueue creates initialized priority queue.
func newBlocksQueue(ctx context.Context, cfg *blocksQueueConfig) *blocksQueue {
	ctx, cancel := context.WithCancel(ctx)

	blocksFetcher := cfg.blocksFetcher
	if blocksFetcher == nil {
		blocksFetcher = newBlocksFetcher(ctx, &blocksFetcherConfig{
			ctxMap: cfg.ctxMap,
			chain:  cfg.chain,
			p2p:    cfg.p2p,
			db:     cfg.db,
			clock:  cfg.clock,
		})
	}
	highestExpectedSlot := cfg.highestExpectedSlot
	if highestExpectedSlot == 0 {
		if cfg.mode == modeStopOnFinalizedEpoch {
			highestExpectedSlot = blocksFetcher.bestFinalizedSlot()
		} else {
			highestExpectedSlot = blocksFetcher.bestNonFinalizedSlot()
		}
	}

	// Override fetcher's sync mode.
	blocksFetcher.mode = cfg.mode

	queue := &blocksQueue{
		ctx:                 ctx,
		cancel:              cancel,
		highestExpectedSlot: highestExpectedSlot,
		blocksFetcher:       blocksFetcher,
		chain:               cfg.chain,
		mode:                cfg.mode,
		fetchedData:         make(chan *blocksQueueFetchedData, 1),
		quit:                make(chan struct{}),
		staleEpochs:         make(map[primitives.Epoch]uint8),
	}

	// Configure state machines.
	queue.smm = newStateMachineManager()
	queue.smm.addEventHandler(eventTick, stateNew, queue.onScheduleEvent(ctx))
	queue.smm.addEventHandler(eventDataReceived, stateScheduled, queue.onDataReceivedEvent(ctx))
	queue.smm.addEventHandler(eventTick, stateDataParsed, queue.onReadyToSendEvent(ctx))
	queue.smm.addEventHandler(eventTick, stateSkipped, queue.onProcessSkippedEvent(ctx))
	queue.smm.addEventHandler(eventTick, stateSent, queue.onCheckStaleEvent(ctx))

	return queue
}

// start boots up the queue processing.
func (q *blocksQueue) start() error {
	select {
	case <-q.ctx.Done():
		return errQueueCtxIsDone
	default:
		go q.loop()
		return nil
	}
}

// stop terminates all queue operations.
func (q *blocksQueue) stop() error {
	q.cancel()
	select {
	case <-q.quit:
		return nil
	case <-time.After(queueStopCallTimeout):
		return errQueueTakesTooLongToStop
	}
}

// loop is a main queue loop.
func (q *blocksQueue) loop() {
	defer close(q.quit)

	defer func() {
		q.blocksFetcher.stop()
		close(q.fetchedData)
	}()

	if err := q.blocksFetcher.start(); err != nil {
		log.WithError(err).Debug("Can not start blocks provider")
	}

	// Define initial state machines.
	startSlot := q.chain.HeadSlot()
	if startSlot > startBackSlots {
		startSlot -= startBackSlots
	}
	blocksPerRequest := q.blocksFetcher.blocksPerPeriod
	for i := startSlot; i < startSlot.Add(blocksPerRequest*lookaheadSteps); i += primitives.Slot(blocksPerRequest) {
		q.smm.addStateMachine(i)
	}

	ticker := time.NewTicker(pollingInterval)
	defer ticker.Stop()
	for {
		if waitHighestExpectedSlot(q) {
			continue
		}

		log.WithFields(logrus.Fields{
			"highestExpectedSlot": q.highestExpectedSlot,
			"headSlot":            q.chain.HeadSlot(),
			"state":               q.smm.String(),
			"staleEpoch":          q.staleEpochs,
		}).Trace("tick")

		select {
		case <-ticker.C:
			for _, key := range q.smm.keys {
				fsm := q.smm.machines[key]
				if err := fsm.trigger(eventTick, nil); err != nil {
					log.WithFields(logrus.Fields{
						"highestExpectedSlot":       q.highestExpectedSlot,
						"noRequiredPeersErrRetries": q.exitConditions.noRequiredPeersErrRetries,
						"event":                     eventTick,
						"epoch":                     slots.ToEpoch(fsm.start),
						"start":                     fsm.start,
						"error":                     err.Error(),
					}).Debug("Can not trigger event")
					if errors.Is(err, errNoRequiredPeers) {
						forceExit := q.exitConditions.noRequiredPeersErrRetries > noRequiredPeersErrMaxRetries
						if q.mode == modeStopOnFinalizedEpoch || forceExit {
							q.cancel()
						} else {
							q.exitConditions.noRequiredPeersErrRetries++
							log.Debug("Waiting for finalized peers")
							time.Sleep(noRequiredPeersErrRefreshInterval)
						}
						continue
					}
				}
				// Do garbage collection, and advance sliding window forward.
				if q.chain.HeadSlot() >= fsm.start.Add(blocksPerRequest-1) {
					highestStartSlot, err := q.smm.highestStartSlot()
					if err != nil {
						log.WithError(err).Debug("Cannot obtain highest epoch state number")
						continue
					}
					if err := q.smm.removeStateMachine(fsm.start); err != nil {
						log.WithError(err).Debug("Can not remove state machine")
					}
					if len(q.smm.machines) < lookaheadSteps {
						q.smm.addStateMachine(highestStartSlot.Add(blocksPerRequest))
					}
				}
			}
		case response, ok := <-q.blocksFetcher.requestResponses():
			if !ok {
				log.Debug("Fetcher closed output channel")
				q.cancel()
				return
			}
			// Update state of an epoch for which data is received.
			if fsm, ok := q.smm.findStateMachine(response.start); ok {
				if err := fsm.trigger(eventDataReceived, response); err != nil {
					log.WithFields(logrus.Fields{
						"event": eventDataReceived,
						"epoch": slots.ToEpoch(fsm.start),
						"error": err.Error(),
					}).Debug("Can not process event")
					fsm.setState(stateNew)
					continue
				}
			}
		case <-q.ctx.Done():
			log.Debug("Context closed, exiting goroutine (blocks queue)")
			return
		}
	}
}

func waitHighestExpectedSlot(q *blocksQueue) bool {
	// Check highest expected slot when we approach chain's head slot.
	if q.chain.HeadSlot() >= q.highestExpectedSlot {
		// By the time initial sync is complete, highest slot may increase, re-check.
		if q.mode == modeStopOnFinalizedEpoch {
			if q.highestExpectedSlot < q.blocksFetcher.bestFinalizedSlot() {
				q.highestExpectedSlot = q.blocksFetcher.bestFinalizedSlot()
				return true
			}
		} else {
			if q.highestExpectedSlot < q.blocksFetcher.bestNonFinalizedSlot() {
				q.highestExpectedSlot = q.blocksFetcher.bestNonFinalizedSlot()
				return true
			}
		}
		log.WithField("slot", q.highestExpectedSlot).Debug("Highest expected slot reached")
		q.cancel()
	}
	return false
}

// onScheduleEvent is an event called on newly arrived epochs. Transforms state to scheduled.
func (q *blocksQueue) onScheduleEvent(ctx context.Context) eventHandlerFn {
	return func(m *stateMachine, in interface{}) (stateID, error) {
		if m.state != stateNew {
			return m.state, errInvalidInitialState
		}
		if m.start > q.highestExpectedSlot {
			m.setState(stateSkipped)
			return m.state, errSlotIsTooHigh
		}
		blocksPerRequest := q.blocksFetcher.blocksPerPeriod
		if err := q.blocksFetcher.scheduleRequest(ctx, m.start, blocksPerRequest); err != nil {
			return m.state, err
		}
		return stateScheduled, nil
	}
}

// onDataReceivedEvent is an event called when data is received from fetcher.
func (q *blocksQueue) onDataReceivedEvent(ctx context.Context) eventHandlerFn {
	return func(m *stateMachine, in interface{}) (stateID, error) {
		if ctx.Err() != nil {
			return m.state, ctx.Err()
		}
		if m.state != stateScheduled {
			return m.state, errInvalidInitialState
		}
		response, ok := in.(*fetchRequestResponse)
		if !ok {
			return m.state, errInputNotFetchRequestParams
		}
		if response.err != nil {
			if errors.Is(response.err, errSlotIsTooHigh) {
				// Current window is already too big, re-request previous epochs.
				for _, fsm := range q.smm.machines {
					if fsm.start < response.start && fsm.state == stateSkipped {
						fsm.setState(stateNew)
					}
				}
			}
			if errors.Is(response.err, beaconsync.ErrInvalidFetchedData) {
				// Peer returned invalid data, penalize.
				q.blocksFetcher.p2p.Peers().Scorers().BadResponsesScorer().Increment(m.pid)
				log.WithField("pid", response.pid).Debug("Peer is penalized for invalid blocks")
			}
			return m.state, response.err
		}
		m.pid = response.pid
		m.bwb = response.bwb
		return stateDataParsed, nil
	}
}

// onReadyToSendEvent is an event called to allow epochs with available blocks to send them downstream.
func (q *blocksQueue) onReadyToSendEvent(ctx context.Context) eventHandlerFn {
	return func(m *stateMachine, in interface{}) (stateID, error) {
		if ctx.Err() != nil {
			return m.state, ctx.Err()
		}
		if m.state != stateDataParsed {
			return m.state, errInvalidInitialState
		}

		if len(m.bwb) == 0 {
			return stateSkipped, nil
		}

		send := func() (stateID, error) {
			data := &blocksQueueFetchedData{
				pid: m.pid,
				bwb: m.bwb,
			}
			select {
			case <-ctx.Done():
				return m.state, ctx.Err()
			case q.fetchedData <- data:
			}
			return stateSent, nil
		}

		// Make sure that we send epochs in a correct order.
		// If machine is the first (has lowest start block), send.
		if m.isFirst() {
			return send()
		}

		// Make sure that previous epoch is already processed.
		for _, fsm := range q.smm.machines {
			// Review only previous slots.
			if fsm.start < m.start {
				switch fsm.state {
				case stateNew, stateScheduled, stateDataParsed:
					return m.state, nil
				}
			}
		}

		return send()
	}
}

// onProcessSkippedEvent is an event triggered on skipped machines, allowing handlers to
// extend lookahead window, in case where progress is not possible otherwise.
func (q *blocksQueue) onProcessSkippedEvent(ctx context.Context) eventHandlerFn {
	return func(m *stateMachine, in interface{}) (stateID, error) {
		if ctx.Err() != nil {
			return m.state, ctx.Err()
		}
		if m.state != stateSkipped {
			return m.state, errInvalidInitialState
		}

		// Only the highest epoch with skipped state can trigger extension.
		if !m.isLast() {
			// When a state machine stays in skipped state for too long - reset it.
			if time.Since(m.updated) > skippedMachineTimeout {
				return stateNew, nil
			}
			return m.state, nil
		}

		// Make sure that all machines are in skipped state i.e. manager cannot progress without reset or
		// moving the last machine's start block forward (in an attempt to find next non-skipped block).
		if !q.smm.allMachinesInState(stateSkipped) {
			return m.state, nil
		}

		// Check if we have enough peers to progress, or sync needs to halt (due to no peers available).
		bestFinalizedSlot := q.blocksFetcher.bestFinalizedSlot()
		if q.mode == modeStopOnFinalizedEpoch {
			if bestFinalizedSlot <= q.chain.HeadSlot() {
				return stateSkipped, errNoRequiredPeers
			}
		} else {
			if q.blocksFetcher.bestNonFinalizedSlot() <= q.chain.HeadSlot() {
				return stateSkipped, errNoRequiredPeers
			}
		}

		// All machines are skipped, FSMs need reset.
		startSlot := q.chain.HeadSlot() + 1
		if q.mode == modeNonConstrained && startSlot > bestFinalizedSlot {
			q.staleEpochs[slots.ToEpoch(startSlot)]++
			// If FSMs have been reset enough times, try to explore alternative forks.
			if q.staleEpochs[slots.ToEpoch(startSlot)] >= maxResetAttempts {
				delete(q.staleEpochs, slots.ToEpoch(startSlot))
				fork, err := q.blocksFetcher.findFork(ctx, startSlot)
				if err == nil {
					return stateSkipped, q.resetFromFork(fork)
				}
				log.WithFields(logrus.Fields{
					"epoch": slots.ToEpoch(startSlot),
					"error": err.Error(),
				}).Debug("Can not explore alternative branches")
			}
		}
		return stateSkipped, q.resetFromSlot(ctx, startSlot)
	}
}

// onCheckStaleEvent is an event that allows to mark stale epochs,
// so that they can be re-processed.
func (_ *blocksQueue) onCheckStaleEvent(ctx context.Context) eventHandlerFn {
	return func(m *stateMachine, in interface{}) (stateID, error) {
		if ctx.Err() != nil {
			return m.state, ctx.Err()
		}
		if m.state != stateSent {
			return m.state, errInvalidInitialState
		}

		// Break out immediately if bucket is not stale.
		if time.Since(m.updated) < staleEpochTimeout {
			return m.state, nil
		}

		return stateSkipped, nil
	}
}
