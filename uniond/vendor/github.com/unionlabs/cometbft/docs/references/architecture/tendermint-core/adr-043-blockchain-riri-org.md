# ADR 043: Blockhchain Reactor Riri-Org

## Changelog

- 18-06-2019: Initial draft
- 08-07-2019: Reviewed
- 29-11-2019: Implemented
- 14-02-2020: Updated with the implementation details

## Context

The blockchain reactor is responsible for two high level processes:sending/receiving blocks from peers and FastSync-ing blocks to catch upnode who is far behind. The goal of [ADR-40](https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-040-blockchain-reactor-refactor.md) was to refactor these two processes by separating business logic currently wrapped up in go-channels into pure `handle*` functions. While the ADR specified what the final form of the reactor might look like it lacked guidance on intermediary steps to get there.
The following diagram illustrates the state of the [blockchain-reorg](https://github.com/tendermint/tendermint/pull/3561) reactor which will be referred to as `v1`.

![v1 Blockchain Reactor Architecture
Diagram](https://github.com/tendermint/tendermint/blob/f9e556481654a24aeb689bdadaf5eab3ccd66829/docs/architecture/img/blockchain-reactor-v1.png)

While `v1` of the blockchain reactor has shown significant improvements in terms of simplifying the concurrency model, the current PR has run into few roadblocks.

- The current PR large and difficult to review.
- Block gossiping and fast sync processes are highly coupled to the shared `Pool` data structure.
- Peer communication is spread over multiple components creating complex dependency graph which must be mocked out during testing.
- Timeouts modeled as stateful tickers introduce non-determinism in tests

This ADR is meant to specify the missing components and control necessary to achieve [ADR-40](https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-040-blockchain-reactor-refactor.md).

## Decision

Partition the responsibilities of the blockchain reactor into a set of components which communicate exclusively with events. Events will contain timestamps allowing each component to track time as internal state. The internal state will be mutated by a set of `handle*` which will produce event(s). The integration between components will happen in the reactor and reactor tests will then become integration tests between components. This design will be known as `v2`.

![v2 Blockchain Reactor Architecture
Diagram](https://github.com/tendermint/tendermint/blob/584e67ac3fac220c5c3e0652e3582eca8231e814/docs/architecture/img/blockchain-reactor-v2.png)

### Fast Sync Related Communication Channels

The diagram below shows the fast sync routines and the types of channels and queues used to communicate with each other.
In addition the per reactor channels used by the sendRoutine to send messages over the Peer MConnection are shown.

![v2 Blockchain Channels and Queues
Diagram](https://github.com/tendermint/tendermint/blob/5cf570690f989646fb3b615b734da503f038891f/docs/architecture/img/blockchain-v2-channels.png)

### Reactor changes in detail

The reactor will include a demultiplexing routine which will send each message to each sub routine for independent processing. Each sub routine will then select the messages it's interested in and call the handle specific function specified in [ADR-40](https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-040-blockchain-reactor-refactor.md). The demuxRoutine acts as "pacemaker" setting the time in which events are expected to be handled.

```go
func demuxRoutine(msgs, scheduleMsgs, processorMsgs, ioMsgs) {
	timer := time.NewTicker(interval)
	for {
		select {
			case <-timer.C:
				now := evTimeCheck{time.Now()}
				schedulerMsgs <- now
				processorMsgs <- now
				ioMsgs <- now
			case msg:= <- msgs:
				msg.time = time.Now()
				// These channels should produce backpressure before
				// being full to avoid starving each other
				schedulerMsgs <- msg
				processorMsgs <- msg
				ioMesgs <- msg
				if msg == stop {
					break;
				}
		}
	}
}

func processRoutine(input chan Message, output chan Message) {
	processor := NewProcessor(..)
	for {
		msg := <- input
		switch msg := msg.(type) {
			case bcBlockRequestMessage:
				output <- processor.handleBlockRequest(msg))
			...
			case stop:
				processor.stop()
				break;
	}
}

func scheduleRoutine(input chan Message, output chan Message) {
	schelduer = NewScheduler(...)
	for {
		msg := <-msgs
		switch msg := input.(type) {
			case bcBlockResponseMessage:
				output <- scheduler.handleBlockResponse(msg)
			...
			case stop:
				schedule.stop()
				break;
		}
	}
}
```

## Lifecycle management

A set of routines for individual processes allow processes to run in parallel with clear lifecycle management. `Start`, `Stop`, and `AddPeer` hooks currently present in the reactor will delegate to the sub-routines allowing them to manage internal state independent without further coupling to the reactor.

```go
func (r *BlockChainReactor) Start() {
	r.msgs := make(chan Message, maxInFlight)
	schedulerMsgs := make(chan Message)
	processorMsgs := make(chan Message)
	ioMsgs := make(chan Message)

	go processorRoutine(processorMsgs, r.msgs)
	go scheduleRoutine(schedulerMsgs, r.msgs)
	go ioRoutine(ioMsgs, r.msgs)
	...
}

func (bcR *BlockchainReactor) Receive(...) {
	...
	r.msgs <- msg
	...
}

func (r *BlockchainReactor) Stop() {
	...
	r.msgs <- stop
	...
}

...
func (r *BlockchainReactor) Stop() {
	...
	r.msgs <- stop
	...
}
...

func (r *BlockchainReactor) AddPeer(peer p2p.Peer) {
	...
	r.msgs <- bcAddPeerEv{peer.ID}
	...
}

```

## IO handling

An io handling routine within the reactor will isolate peer communication. Message going through the ioRoutine will usually be one way, using `p2p` APIs. In the case in which the `p2p` API such as `trySend` return errors, the ioRoutine can funnel those message back to the demuxRoutine for distribution to the other routines. For instance errors from the ioRoutine can be consumed by the scheduler to inform better peer selection implementations.

```go
func (r *BlockchainReacor) ioRoutine(ioMesgs chan Message, outMsgs chan Message) {
	...
	for {
		msg := <-ioMsgs
		switch msg := msg.(type) {
			case scBlockRequestMessage:
				queued := r.sendBlockRequestToPeer(...)
				if queued {
					outMsgs <- ioSendQueued{...}
				}
			case scStatusRequestMessage
				r.sendStatusRequestToPeer(...)
			case bcPeerError
				r.Switch.StopPeerForError(msg.src)
				...
			...
			case bcFinished
				break;
		}
	}
}

```

### Processor Internals

The processor is responsible for ordering, verifying and executing blocks. The Processor will maintain an internal cursor `height` referring to the last processed block. As a set of blocks arrive unordered, the Processor will check if it has `height+1` necessary to process the next block. The processor also maintains the map `blockPeers` of peers to height, to keep track of which peer provided the block at `height`. `blockPeers` can be used in`handleRemovePeer(...)` to reschedule all unprocessed blocks provided by a peer who has errored.

```go
type Processor struct {
	height int64 // the height cursor
	state ...
	blocks [height]*Block	 // keep a set of blocks in memory until they are processed
	blockPeers [height]PeerID // keep track of which heights came from which peerID
	lastTouch timestamp
}

func (proc *Processor) handleBlockResponse(peerID, block) {
    if block.height <= height || block[block.height] {
	} else if blocks[block.height] {
		return errDuplicateBlock{}
	} else  {
		blocks[block.height] = block
	}

	if blocks[height] && blocks[height+1] {
		... = state.Validators.VerifyCommit(...)
		... = store.SaveBlock(...)
		state, err = blockExec.ApplyBlock(...)
		...
		if err == nil {
			delete blocks[height]
			height++
			lastTouch = msg.time
			return pcBlockProcessed{height-1}
		} else {
			... // Delete all unprocessed block from the peer
			return pcBlockProcessError{peerID, height}
		}
	}
}

func (proc *Processor) handleRemovePeer(peerID) {
	events = []
	// Delete all unprocessed blocks from peerID
	for i = height; i < len(blocks); i++ {
		if blockPeers[i] == peerID {
			events = append(events, pcBlockReschedule{height})

			delete block[height]
		}
	}
	return events
}

func handleTimeCheckEv(time) {
	if time - lastTouch > timeout {
		// Timeout the processor
		...
	}
}
```

## Schedule

The Schedule maintains the internal state used for scheduling blockRequestMessages based on some scheduling algorithm. The schedule needs to maintain state on:

- The state `blockState` of every block seem up to height of maxHeight
- The set of peers and their peer state `peerState`
- which peers have which blocks
- which blocks have been requested from which peers

```go
type blockState int

const (
	blockStateNew = iota
	blockStatePending,
	blockStateReceived,
	blockStateProcessed
)

type schedule {
    // a list of blocks in which blockState
	blockStates        map[height]blockState

    // a map of which blocks are available from which peers
	blockPeers         map[height]map[p2p.ID]scPeer

    // a map of peerID to schedule specific peer struct `scPeer`
	peers              map[p2p.ID]scPeer

    // a map of heights to the peer we are waiting for a response from
	pending map[height]scPeer

	targetPending  int // the number of blocks we want in blockStatePending
	targetReceived int // the number of blocks we want in blockStateReceived

	peerTimeout        int
	peerMinSpeed       int
}

func (sc *schedule) numBlockInState(state blockState) uint32 {
	num := 0
	for i := sc.minHeight(); i <= sc.maxHeight(); i++ {
		if sc.blockState[i] == state {
			num++
		}
	}
	return num
}


func (sc *schedule) popSchedule(maxRequest int) []scBlockRequestMessage {
	// We only want to schedule requests such that we have less than sc.targetPending and sc.targetReceived
	// This ensures we don't saturate the network or flood the processor with unprocessed blocks
	todo := min(sc.targetPending - sc.numBlockInState(blockStatePending), sc.numBlockInState(blockStateReceived))
	events := []scBlockRequestMessage{}
	for i := sc.minHeight(); i < sc.maxMaxHeight(); i++ {
		if todo == 0 {
			break
		}
		if blockStates[i] == blockStateNew {
			peer = sc.selectPeer(blockPeers[i])
			sc.blockStates[i] = blockStatePending
			sc.pending[i] = peer
			events = append(events, scBlockRequestMessage{peerID: peer.peerID, height: i})
			todo--
		}
	}
	return events
}
...

type scPeer struct {
	peerID               p2p.ID
	numOustandingRequest int
	lastTouched          time.Time
	monitor              flow.Monitor
}

```

# Scheduler

The scheduler is configured to maintain a target `n` of in flight
messages and will use feedback from `_blockResponseMessage`,
`_statusResponseMessage` and `_peerError` produce an optimal assignment
of scBlockRequestMessage at each `timeCheckEv`.

```

func handleStatusResponse(peerID, height, time) {
	schedule.touchPeer(peerID, time)
	schedule.setPeerHeight(peerID, height)
}

func handleBlockResponseMessage(peerID, height, block, time) {
	schedule.touchPeer(peerID, time)
	schedule.markReceived(peerID, height, size(block))
}

func handleNoBlockResponseMessage(peerID, height, time) {
	schedule.touchPeer(peerID, time)
	// reschedule that block, punish peer...
    ...
}

func handlePeerError(peerID)  {
    // Remove the peer, reschedule the requests
    ...
}

func handleTimeCheckEv(time) {
	// clean peer list

    events = []
	for peerID := range schedule.peersNotTouchedSince(time) {
		pending = schedule.pendingFrom(peerID)
		schedule.setPeerState(peerID, timedout)
		schedule.resetBlocks(pending)
		events = append(events, peerTimeout{peerID})
    }

	events = append(events, schedule.popSchedule())

	return events
}
```

## Peer

The Peer Stores per peer state based on messages received by the scheduler.

```go
type Peer struct {
	lastTouched timestamp
	lastDownloaded timestamp
	pending map[height]struct{}
	height height // max height for the peer
	state {
		pending,   // we know the peer but not the height
		active,    // we know the height
		timeout    // the peer has timed out
	}
}
```

## Status

Implemented

## Consequences

### Positive

- Test become deterministic
- Simulation becomes a-termporal: no need wait for a wall-time timeout
- Peer Selection can be independently tested/simulated
- Develop a general approach to refactoring reactors

### Negative

### Neutral

### Implementation Path

- Implement the scheduler, test the scheduler, review the rescheduler
- Implement the processor, test the processor, review the processor
- Implement the demuxer, write integration test, review integration tests

## References

- [ADR-40](https://github.com/tendermint/tendermint/blob/main/docs/architecture/adr-040-blockchain-reactor-refactor.md): The original blockchain reactor re-org proposal
- [Blockchain re-org](https://github.com/tendermint/tendermint/pull/3561): The current blockchain reactor re-org implementation (v1)
