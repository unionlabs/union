package initialsync

import (
	"context"
	"errors"

	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
)

// resetWithBlocks removes all state machines, then re-adds enough machines to contain all provided
// blocks (machines are set into stateDataParsed state, so that their content is immediately
// consumable). It is assumed that blocks come in an ascending order.
func (q *blocksQueue) resetFromFork(fork *forkData) error {
	if fork == nil {
		return errors.New("nil fork data")
	}
	if len(fork.bwb) == 0 {
		return errors.New("no blocks to reset from")
	}
	firstBlock := fork.bwb[0].Block.Block()

	blocksPerRequest := q.blocksFetcher.blocksPerPeriod
	if err := q.smm.removeAllStateMachines(); err != nil {
		return err
	}
	fsm := q.smm.addStateMachine(firstBlock.Slot())
	fsm.pid = fork.peer
	fsm.bwb = fork.bwb
	fsm.state = stateDataParsed

	// The rest of machines are in skipped state.
	startSlot := firstBlock.Slot().Add(uint64(len(fork.bwb)))
	for i := startSlot; i < startSlot.Add(blocksPerRequest*(lookaheadSteps-1)); i += primitives.Slot(blocksPerRequest) {
		fsm := q.smm.addStateMachine(i)
		fsm.state = stateSkipped
	}
	return nil
}

// resetFromSlot removes all state machines, and re-adds them starting with a given slot.
// The last machine added relies on calculated non-skipped slot (to allow FSMs to jump over
// long periods with skipped slots).
func (q *blocksQueue) resetFromSlot(ctx context.Context, startSlot primitives.Slot) error {
	// Shift start position of all the machines except for the last one.
	blocksPerRequest := q.blocksFetcher.blocksPerPeriod
	if err := q.smm.removeAllStateMachines(); err != nil {
		return err
	}
	for i := startSlot; i < startSlot.Add(blocksPerRequest*(lookaheadSteps-1)); i += primitives.Slot(blocksPerRequest) {
		q.smm.addStateMachine(i)
	}

	// Replace the last (currently activated) state machine to start with best known non-skipped slot.
	nonSkippedSlot, err := q.blocksFetcher.nonSkippedSlotAfter(ctx, startSlot.Add(blocksPerRequest*(lookaheadSteps-1)-1))
	if err != nil {
		return err
	}
	if q.mode == modeStopOnFinalizedEpoch {
		if q.highestExpectedSlot < q.blocksFetcher.bestFinalizedSlot() {
			q.highestExpectedSlot = q.blocksFetcher.bestFinalizedSlot()
		}
	} else {
		if q.highestExpectedSlot < q.blocksFetcher.bestNonFinalizedSlot() {
			q.highestExpectedSlot = q.blocksFetcher.bestNonFinalizedSlot()
		}
	}
	if nonSkippedSlot > q.highestExpectedSlot {
		nonSkippedSlot = startSlot.Add(blocksPerRequest * (lookaheadSteps - 1))
	}
	q.smm.addStateMachine(nonSkippedSlot)
	return nil
}
