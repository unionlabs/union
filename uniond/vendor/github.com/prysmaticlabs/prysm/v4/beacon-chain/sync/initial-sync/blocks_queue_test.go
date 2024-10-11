package initialsync

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/libp2p/go-libp2p/core/peer"
	mock "github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain/testing"
	dbtest "github.com/prysmaticlabs/prysm/v4/beacon-chain/db/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/peers"
	p2pt "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/testing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/startup"
	beaconsync "github.com/prysmaticlabs/prysm/v4/beacon-chain/sync"
	"github.com/prysmaticlabs/prysm/v4/cmd/beacon-chain/flags"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/blocks"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/interfaces"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	leakybucket "github.com/prysmaticlabs/prysm/v4/container/leaky-bucket"
	"github.com/prysmaticlabs/prysm/v4/container/slice"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	eth "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/testing/assert"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	prysmTime "github.com/prysmaticlabs/prysm/v4/time"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	logTest "github.com/sirupsen/logrus/hooks/test"
)

func TestBlocksQueue_InitStartStop(t *testing.T) {
	blockBatchLimit := flags.Get().BlockBatchLimit
	mc, p2p, _ := initializeTestServices(t, []primitives.Slot{}, []*peerData{})

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
		chain: mc,
		p2p:   p2p,
		clock: startup.NewClock(mc.Genesis, mc.ValidatorsRoot),
	})

	t.Run("stop without start", func(t *testing.T) {
		ctx, cancel := context.WithCancel(ctx)
		defer cancel()
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		assert.ErrorContains(t, errQueueTakesTooLongToStop.Error(), queue.stop())
	})

	t.Run("use default fetcher", func(t *testing.T) {
		ctx, cancel := context.WithCancel(ctx)
		defer cancel()
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		assert.NoError(t, queue.start())
	})

	t.Run("stop timeout", func(t *testing.T) {
		ctx, cancel := context.WithCancel(ctx)
		defer cancel()
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		assert.NoError(t, queue.start())
		assert.ErrorContains(t, errQueueTakesTooLongToStop.Error(), queue.stop())
	})

	t.Run("check for leaked goroutines", func(t *testing.T) {
		ctx, cancel := context.WithCancel(ctx)
		defer cancel()
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		assert.NoError(t, queue.start())
		// Blocks up until all resources are reclaimed (or timeout is called)
		assert.NoError(t, queue.stop())
		select {
		case <-queue.fetchedData:
		default:
			t.Error("queue.fetchedData channel is leaked")
		}
		select {
		case <-fetcher.fetchResponses:
		default:
			t.Error("fetcher.fetchResponses channel is leaked")
		}
	})

	t.Run("re-starting of stopped queue", func(t *testing.T) {
		ctx, cancel := context.WithCancel(ctx)
		defer cancel()
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		assert.NoError(t, queue.start())
		assert.NoError(t, queue.stop())
		assert.ErrorContains(t, errQueueCtxIsDone.Error(), queue.start())
	})

	t.Run("multiple stopping attempts", func(t *testing.T) {
		ctx, cancel := context.WithCancel(ctx)
		defer cancel()
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		assert.NoError(t, queue.start())
		assert.NoError(t, queue.stop())
		assert.NoError(t, queue.stop())
	})

	t.Run("cancellation", func(t *testing.T) {
		ctx, cancel := context.WithCancel(context.Background())
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
			clock:               startup.NewClock(mc.Genesis, mc.ValidatorsRoot),
		})
		assert.NoError(t, queue.start())
		cancel()
		assert.NoError(t, queue.stop())
	})
}

func TestBlocksQueue_Loop(t *testing.T) {
	currentPeriod := blockLimiterPeriod
	blockLimiterPeriod = 1 * time.Second
	defer func() {
		blockLimiterPeriod = currentPeriod
	}()
	tests := []struct {
		name                string
		highestExpectedSlot primitives.Slot
		expectedBlockSlots  []primitives.Slot
		peers               []*peerData
	}{
		{
			name:                "Single peer with all blocks",
			highestExpectedSlot: 251, // will be auto-fixed to 256 (to 8th epoch), by queue
			expectedBlockSlots:  makeSequence(1, 256),
			peers: []*peerData{
				{
					blocks:         makeSequence(1, 320),
					finalizedEpoch: 8,
					headSlot:       320,
				},
			},
		},
		{
			name:                "Multiple peers with all blocks",
			highestExpectedSlot: 256,
			expectedBlockSlots:  makeSequence(1, 256),
			peers: []*peerData{
				{
					blocks:         makeSequence(1, 320),
					finalizedEpoch: 8,
					headSlot:       320,
				},
				{
					blocks:         makeSequence(1, 320),
					finalizedEpoch: 8,
					headSlot:       320,
				},
				{
					blocks:         makeSequence(1, 320),
					finalizedEpoch: 8,
					headSlot:       320,
				},
				{
					blocks:         makeSequence(1, 320),
					finalizedEpoch: 8,
					headSlot:       320,
				},
				{
					blocks:         makeSequence(1, 320),
					finalizedEpoch: 8,
					headSlot:       320,
				},
			},
		},
		{
			name:                "Multiple peers with skipped slots",
			highestExpectedSlot: 576,
			expectedBlockSlots:  append(makeSequence(1, 64), makeSequence(500, 576)...), // up to 18th epoch
			peers: []*peerData{
				{
					blocks:         append(makeSequence(1, 64), makeSequence(500, 640)...),
					finalizedEpoch: 18,
					headSlot:       640,
				},
				{
					blocks:         append(makeSequence(1, 64), makeSequence(500, 640)...),
					finalizedEpoch: 18,
					headSlot:       640,
				},
				{
					blocks:         append(makeSequence(1, 64), makeSequence(500, 640)...),
					finalizedEpoch: 18,
					headSlot:       640,
				},
			},
		},
		{
			name:                "Multiple peers with failures",
			highestExpectedSlot: 128,
			expectedBlockSlots:  makeSequence(1, 256),
			peers: []*peerData{
				{
					blocks:         makeSequence(1, 320),
					finalizedEpoch: 8,
					headSlot:       320,
					failureSlots:   makeSequence(32*3+1, 32*3+32),
				},
				{
					blocks:         makeSequence(1, 320),
					finalizedEpoch: 8,
					headSlot:       320,
					failureSlots:   makeSequence(1, 32*3),
				},
				{
					blocks:         makeSequence(1, 320),
					finalizedEpoch: 8,
					headSlot:       320,
				},
				{
					blocks:         makeSequence(1, 320),
					finalizedEpoch: 8,
					headSlot:       320,
				},
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			mc, p2p, beaconDB := initializeTestServices(t, tt.expectedBlockSlots, tt.peers)

			ctx, cancel := context.WithCancel(context.Background())
			defer cancel()

			fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
				chain: mc,
				p2p:   p2p,
				clock: startup.NewClock(mc.Genesis, mc.ValidatorsRoot),
			})
			queue := newBlocksQueue(ctx, &blocksQueueConfig{
				blocksFetcher:       fetcher,
				chain:               mc,
				highestExpectedSlot: tt.highestExpectedSlot,
			})
			assert.NoError(t, queue.start())
			processBlock := func(b blocks.BlockWithROBlobs) error {
				block := b.Block
				if !beaconDB.HasBlock(ctx, block.Block().ParentRoot()) {
					return fmt.Errorf("%w: %#x", errParentDoesNotExist, block.Block().ParentRoot())
				}
				root, err := block.Block().HashTreeRoot()
				if err != nil {
					return err
				}
				return mc.ReceiveBlock(ctx, block, root, nil)
			}

			var blocks []blocks.BlockWithROBlobs
			for data := range queue.fetchedData {
				for _, b := range data.bwb {
					if err := processBlock(b); err != nil {
						continue
					}
					blocks = append(blocks, b)
				}
			}

			assert.NoError(t, queue.stop())

			if queue.chain.HeadSlot() < tt.highestExpectedSlot {
				t.Errorf("Not enough slots synced, want: %v, got: %v",
					len(tt.expectedBlockSlots), queue.chain.HeadSlot())
			}
			assert.Equal(t, len(tt.expectedBlockSlots), len(blocks), "Processes wrong number of blocks")
			var receivedBlockSlots []primitives.Slot
			for _, b := range blocks {
				receivedBlockSlots = append(receivedBlockSlots, b.Block.Block().Slot())
			}
			missing := slice.NotSlot(slice.IntersectionSlot(tt.expectedBlockSlots, receivedBlockSlots), tt.expectedBlockSlots)
			if len(missing) > 0 {
				t.Errorf("Missing blocks at slots %v", missing)
			}
		})
	}
}

func TestBlocksQueue_onScheduleEvent(t *testing.T) {
	blockBatchLimit := flags.Get().BlockBatchLimit
	mc, p2p, _ := initializeTestServices(t, []primitives.Slot{}, []*peerData{})

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
		chain: mc,
		p2p:   p2p,
	})

	t.Run("expired context", func(t *testing.T) {
		ctx, cancel := context.WithCancel(ctx)
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		handlerFn := queue.onScheduleEvent(ctx)
		cancel()
		updatedState, err := handlerFn(&stateMachine{
			state: stateNew,
		}, nil)
		assert.ErrorContains(t, context.Canceled.Error(), err)
		assert.Equal(t, stateNew, updatedState)
	})

	t.Run("invalid input state", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		invalidStates := []stateID{stateScheduled, stateDataParsed, stateSkipped, stateSent}
		for _, state := range invalidStates {
			t.Run(state.String(), func(t *testing.T) {
				handlerFn := queue.onScheduleEvent(ctx)
				updatedState, err := handlerFn(&stateMachine{
					state: state,
				}, nil)
				assert.ErrorContains(t, errInvalidInitialState.Error(), err)
				assert.Equal(t, state, updatedState)
			})
		}
	})

	t.Run("slot is too high", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		handlerFn := queue.onScheduleEvent(ctx)
		updatedState, err := handlerFn(&stateMachine{
			state: stateNew,
			start: queue.highestExpectedSlot + 1,
		}, nil)
		assert.ErrorContains(t, errSlotIsTooHigh.Error(), err)
		assert.Equal(t, stateSkipped, updatedState)
	})

	t.Run("fetcher fails scheduling", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		// Cancel to make fetcher spit error when trying to schedule next FSM.
		requestCtx, requestCtxCancel := context.WithCancel(context.Background())
		requestCtxCancel()
		handlerFn := queue.onScheduleEvent(requestCtx)
		updatedState, err := handlerFn(&stateMachine{
			state: stateNew,
		}, nil)
		assert.ErrorContains(t, context.Canceled.Error(), err)
		assert.Equal(t, stateNew, updatedState)
	})

	t.Run("schedule next fetch ok", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		handlerFn := queue.onScheduleEvent(ctx)
		updatedState, err := handlerFn(&stateMachine{
			state: stateNew,
		}, nil)
		assert.NoError(t, err)
		assert.Equal(t, stateScheduled, updatedState)
	})
}

func TestBlocksQueue_onDataReceivedEvent(t *testing.T) {
	blockBatchLimit := flags.Get().BlockBatchLimit
	mc, p2p, _ := initializeTestServices(t, []primitives.Slot{}, []*peerData{})

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
		chain: mc,
		p2p:   p2p,
	})

	t.Run("expired context", func(t *testing.T) {
		ctx, cancel := context.WithCancel(ctx)
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		handlerFn := queue.onDataReceivedEvent(ctx)
		cancel()
		updatedState, err := handlerFn(&stateMachine{
			state: stateScheduled,
		}, nil)
		assert.ErrorContains(t, context.Canceled.Error(), err)
		assert.Equal(t, stateScheduled, updatedState)
	})

	t.Run("invalid input state", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		invalidStates := []stateID{stateNew, stateDataParsed, stateSkipped, stateSent}
		for _, state := range invalidStates {
			t.Run(state.String(), func(t *testing.T) {
				handlerFn := queue.onDataReceivedEvent(ctx)
				updatedState, err := handlerFn(&stateMachine{
					state: state,
				}, nil)
				assert.ErrorContains(t, errInvalidInitialState.Error(), err)
				assert.Equal(t, state, updatedState)
			})
		}
	})

	t.Run("invalid input param", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		handlerFn := queue.onDataReceivedEvent(ctx)
		updatedState, err := handlerFn(&stateMachine{
			state: stateScheduled,
		}, nil)
		assert.ErrorContains(t, errInputNotFetchRequestParams.Error(), err)
		assert.Equal(t, stateScheduled, updatedState)
	})

	t.Run("slot is too high do nothing", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		handlerFn := queue.onDataReceivedEvent(ctx)
		updatedState, err := handlerFn(&stateMachine{
			state: stateScheduled,
		}, &fetchRequestResponse{
			pid: "abc",
			err: errSlotIsTooHigh,
		})
		assert.ErrorContains(t, errSlotIsTooHigh.Error(), err)
		assert.Equal(t, stateScheduled, updatedState)
	})

	t.Run("slot is too high force re-request on previous epoch", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		// Mark previous machine as skipped - to test effect of re-requesting.
		queue.smm.addStateMachine(250)
		queue.smm.machines[250].state = stateSkipped
		assert.Equal(t, stateSkipped, queue.smm.machines[250].state)

		handlerFn := queue.onDataReceivedEvent(ctx)
		updatedState, err := handlerFn(&stateMachine{
			state: stateScheduled,
		}, &fetchRequestResponse{
			pid:   "abc",
			err:   errSlotIsTooHigh,
			start: 256,
		})
		assert.ErrorContains(t, errSlotIsTooHigh.Error(), err)
		assert.Equal(t, stateScheduled, updatedState)
		assert.Equal(t, stateNew, queue.smm.machines[250].state)
	})

	t.Run("invalid data returned", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		hook := logTest.NewGlobal()
		defer hook.Reset()
		handlerFn := queue.onDataReceivedEvent(ctx)
		updatedState, err := handlerFn(&stateMachine{
			state: stateScheduled,
		}, &fetchRequestResponse{
			pid: "abc",
			err: beaconsync.ErrInvalidFetchedData,
		})
		assert.ErrorContains(t, beaconsync.ErrInvalidFetchedData.Error(), err)
		assert.Equal(t, stateScheduled, updatedState)
		assert.LogsContain(t, hook, "msg=\"Peer is penalized for invalid blocks\" pid=ZiCa")
	})

	t.Run("transition ok", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		wsb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
		require.NoError(t, err)
		handlerFn := queue.onDataReceivedEvent(ctx)
		wsbCopy, err := wsb.Copy()
		require.NoError(t, err)
		response := &fetchRequestResponse{
			pid: "abc",
			bwb: []blocks.BlockWithROBlobs{
				{Block: blocks.ROBlock{ReadOnlySignedBeaconBlock: wsb}},
				{Block: blocks.ROBlock{ReadOnlySignedBeaconBlock: wsbCopy}},
			},
		}
		fsm := &stateMachine{
			state: stateScheduled,
		}
		assert.Equal(t, peer.ID(""), fsm.pid)
		assert.Equal(t, 0, len(fsm.bwb))
		updatedState, err := handlerFn(fsm, response)
		assert.NoError(t, err)
		assert.Equal(t, stateDataParsed, updatedState)
		assert.Equal(t, response.pid, fsm.pid)
		assert.DeepSSZEqual(t, response.bwb, fsm.bwb)
	})
}

func TestBlocksQueue_onReadyToSendEvent(t *testing.T) {
	blockBatchLimit := flags.Get().BlockBatchLimit
	mc, p2p, _ := initializeTestServices(t, []primitives.Slot{}, []*peerData{})

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
		chain: mc,
		p2p:   p2p,
	})

	t.Run("expired context", func(t *testing.T) {
		ctx, cancel := context.WithCancel(ctx)
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		handlerFn := queue.onReadyToSendEvent(ctx)
		cancel()
		updatedState, err := handlerFn(&stateMachine{
			state: stateNew,
		}, nil)
		assert.ErrorContains(t, context.Canceled.Error(), err)
		assert.Equal(t, stateNew, updatedState)
	})

	t.Run("invalid input state", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		invalidStates := []stateID{stateNew, stateScheduled, stateSkipped, stateSent}
		for _, state := range invalidStates {
			t.Run(state.String(), func(t *testing.T) {
				handlerFn := queue.onReadyToSendEvent(ctx)
				updatedState, err := handlerFn(&stateMachine{
					state: state,
				}, nil)
				assert.ErrorContains(t, errInvalidInitialState.Error(), err)
				assert.Equal(t, state, updatedState)
			})
		}
	})

	t.Run("no blocks to send", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		handlerFn := queue.onReadyToSendEvent(ctx)
		updatedState, err := handlerFn(&stateMachine{
			state: stateDataParsed,
		}, nil)
		// No error, but state is marked as skipped - as no blocks were produced for range.
		assert.NoError(t, err)
		assert.Equal(t, stateSkipped, updatedState)
	})

	const pidDataParsed = "abc"
	t.Run("send from the first machine", func(t *testing.T) {
		fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
			chain: mc,
			p2p:   p2p,
		})
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		wsb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
		require.NoError(t, err)
		queue.smm.addStateMachine(256)
		queue.smm.addStateMachine(320)
		queue.smm.machines[256].state = stateDataParsed
		queue.smm.machines[256].pid = pidDataParsed
		rwsb, err := blocks.NewROBlock(wsb)
		require.NoError(t, err)
		queue.smm.machines[256].bwb = []blocks.BlockWithROBlobs{
			{Block: rwsb},
		}

		handlerFn := queue.onReadyToSendEvent(ctx)
		updatedState, err := handlerFn(queue.smm.machines[256], nil)
		// Machine is the first, has blocks, send them.
		assert.NoError(t, err)
		assert.Equal(t, stateSent, updatedState)
	})

	t.Run("previous machines are not processed - do not send", func(t *testing.T) {
		fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
			chain: mc,
			p2p:   p2p,
		})
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		wsb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
		require.NoError(t, err)
		queue.smm.addStateMachine(128)
		queue.smm.machines[128].state = stateNew
		queue.smm.addStateMachine(192)
		queue.smm.machines[192].state = stateScheduled
		queue.smm.addStateMachine(256)
		queue.smm.machines[256].state = stateDataParsed
		queue.smm.addStateMachine(320)
		queue.smm.machines[320].state = stateDataParsed
		queue.smm.machines[320].pid = pidDataParsed
		rwsb, err := blocks.NewROBlock(wsb)
		require.NoError(t, err)
		queue.smm.machines[320].bwb = []blocks.BlockWithROBlobs{
			{Block: rwsb},
		}

		handlerFn := queue.onReadyToSendEvent(ctx)
		updatedState, err := handlerFn(queue.smm.machines[320], nil)
		// Previous machines have stateNew, stateScheduled, stateDataParsed states, so current
		// machine should wait before sending anything. So, no state change.
		assert.NoError(t, err)
		assert.Equal(t, stateDataParsed, updatedState)
	})

	t.Run("previous machines are processed - send", func(t *testing.T) {
		fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
			chain: mc,
			p2p:   p2p,
		})
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		wsb, err := blocks.NewSignedBeaconBlock(util.NewBeaconBlock())
		require.NoError(t, err)
		queue.smm.addStateMachine(256)
		queue.smm.machines[256].state = stateSkipped
		queue.smm.addStateMachine(320)
		queue.smm.machines[320].state = stateDataParsed
		queue.smm.machines[320].pid = pidDataParsed
		rwsb, err := blocks.NewROBlock(wsb)
		require.NoError(t, err)
		queue.smm.machines[320].bwb = []blocks.BlockWithROBlobs{
			{Block: rwsb},
		}

		handlerFn := queue.onReadyToSendEvent(ctx)
		updatedState, err := handlerFn(queue.smm.machines[320], nil)
		assert.NoError(t, err)
		assert.Equal(t, stateSent, updatedState)
	})
}

func TestBlocksQueue_onProcessSkippedEvent(t *testing.T) {
	blockBatchLimit := flags.Get().BlockBatchLimit
	mc, p2p, _ := initializeTestServices(t, []primitives.Slot{}, []*peerData{})

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
		chain: mc,
		p2p:   p2p,
	})

	t.Run("expired context", func(t *testing.T) {
		ctx, cancel := context.WithCancel(ctx)
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		handlerFn := queue.onProcessSkippedEvent(ctx)
		cancel()
		updatedState, err := handlerFn(&stateMachine{
			state: stateSkipped,
		}, nil)
		assert.ErrorContains(t, context.Canceled.Error(), err)
		assert.Equal(t, stateSkipped, updatedState)
	})

	t.Run("invalid input state", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		invalidStates := []stateID{stateNew, stateScheduled, stateDataParsed, stateSent}
		for _, state := range invalidStates {
			t.Run(state.String(), func(t *testing.T) {
				handlerFn := queue.onProcessSkippedEvent(ctx)
				updatedState, err := handlerFn(&stateMachine{
					state: state,
				}, nil)
				assert.ErrorContains(t, errInvalidInitialState.Error(), err)
				assert.Equal(t, state, updatedState)
			})
		}
	})

	t.Run("not the last machine - do nothing", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		queue.smm.addStateMachine(256)
		// Machine is not skipped for too long. Do not mark as new just yet.
		queue.smm.machines[256].updated = prysmTime.Now().Add(-1 * (skippedMachineTimeout / 2))
		queue.smm.machines[256].state = stateSkipped
		queue.smm.addStateMachine(320)
		queue.smm.machines[320].state = stateScheduled
		handlerFn := queue.onProcessSkippedEvent(ctx)
		updatedState, err := handlerFn(queue.smm.machines[256], nil)
		assert.NoError(t, err)
		assert.Equal(t, stateSkipped, updatedState)
	})

	t.Run("not the last machine - reset", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		queue.smm.addStateMachine(256)
		// Machine is skipped for too long. Reset.
		queue.smm.machines[256].updated = prysmTime.Now().Add(-1 * skippedMachineTimeout)
		queue.smm.machines[256].state = stateSkipped
		queue.smm.addStateMachine(320)
		queue.smm.machines[320].state = stateScheduled
		handlerFn := queue.onProcessSkippedEvent(ctx)
		updatedState, err := handlerFn(queue.smm.machines[256], nil)
		assert.NoError(t, err)
		assert.Equal(t, stateNew, updatedState)
	})

	t.Run("not all machines are skipped", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		queue.smm.addStateMachine(192)
		queue.smm.machines[192].state = stateSkipped
		queue.smm.addStateMachine(256)
		queue.smm.machines[256].state = stateScheduled
		queue.smm.addStateMachine(320)
		queue.smm.machines[320].state = stateSkipped
		handlerFn := queue.onProcessSkippedEvent(ctx)
		updatedState, err := handlerFn(queue.smm.machines[320], nil)
		assert.NoError(t, err)
		assert.Equal(t, stateSkipped, updatedState)
	})

	t.Run("not enough peers", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		queue.smm.addStateMachine(192)
		queue.smm.machines[192].state = stateSkipped
		queue.smm.addStateMachine(256)
		queue.smm.machines[256].state = stateSkipped
		queue.smm.addStateMachine(320)
		queue.smm.machines[320].state = stateSkipped
		// Mode 1: Stop on finalized epoch.
		handlerFn := queue.onProcessSkippedEvent(ctx)
		updatedState, err := handlerFn(queue.smm.machines[320], nil)
		assert.ErrorContains(t, errNoRequiredPeers.Error(), err)
		assert.Equal(t, stateSkipped, updatedState)
		// Mode 2: Do not on finalized epoch.
		queue.mode = modeNonConstrained
		handlerFn = queue.onProcessSkippedEvent(ctx)
		updatedState, err = handlerFn(queue.smm.machines[320], nil)
		assert.ErrorContains(t, errNoRequiredPeers.Error(), err)
		assert.Equal(t, stateSkipped, updatedState)
	})

	t.Run("ready to update machines - non-skipped slot not found", func(t *testing.T) {
		p := p2pt.NewTestP2P(t)
		connectPeers(t, p, []*peerData{
			{blocks: makeSequence(1, 160), finalizedEpoch: 5, headSlot: 128},
		}, p.Peers())
		fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
			chain: mc,
			p2p:   p,
		})
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		startSlot := queue.chain.HeadSlot()
		blocksPerRequest := queue.blocksFetcher.blocksPerPeriod
		for i := startSlot; i < startSlot.Add(blocksPerRequest*lookaheadSteps); i += primitives.Slot(blocksPerRequest) {
			queue.smm.addStateMachine(i).setState(stateSkipped)
		}

		handlerFn := queue.onProcessSkippedEvent(ctx)
		updatedState, err := handlerFn(queue.smm.machines[primitives.Slot(blocksPerRequest*(lookaheadSteps-1))], nil)
		assert.ErrorContains(t, "invalid range for non-skipped slot", err)
		assert.Equal(t, stateSkipped, updatedState)
	})

	t.Run("ready to update machines - constrained mode", func(t *testing.T) {
		p := p2pt.NewTestP2P(t)
		connectPeers(t, p, []*peerData{
			{blocks: makeSequence(500, 628), finalizedEpoch: 16, headSlot: 600},
		}, p.Peers())
		fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
			chain: mc,
			p2p:   p,
		})
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		assert.Equal(t, primitives.Slot(blockBatchLimit), queue.highestExpectedSlot)

		startSlot := queue.chain.HeadSlot()
		blocksPerRequest := queue.blocksFetcher.blocksPerPeriod
		var machineSlots []primitives.Slot
		for i := startSlot; i < startSlot.Add(blocksPerRequest*lookaheadSteps); i += primitives.Slot(blocksPerRequest) {
			queue.smm.addStateMachine(i).setState(stateSkipped)
			machineSlots = append(machineSlots, i)
		}
		for _, slot := range machineSlots {
			_, ok := queue.smm.findStateMachine(slot)
			assert.Equal(t, true, ok)
		}
		// Update head slot, so that machines are re-arranged starting from the next slot i.e.
		// there's no point to reset machines for some slot that has already been processed.
		updatedSlot := primitives.Slot(100)
		defer func() {
			require.NoError(t, mc.State.SetSlot(0))
		}()
		require.NoError(t, mc.State.SetSlot(updatedSlot))

		handlerFn := queue.onProcessSkippedEvent(ctx)
		updatedState, err := handlerFn(queue.smm.machines[primitives.Slot(blocksPerRequest*(lookaheadSteps-1))], nil)
		assert.NoError(t, err)
		assert.Equal(t, stateSkipped, updatedState)
		// Assert that machines have been re-arranged.
		for i, slot := range machineSlots {
			_, ok := queue.smm.findStateMachine(slot)
			assert.Equal(t, false, ok)
			_, ok = queue.smm.findStateMachine(updatedSlot.Add(1 + uint64(i)*blocksPerRequest))
			assert.Equal(t, true, ok)
		}
		// Assert highest expected slot is extended.
		assert.Equal(t, primitives.Slot(blocksPerRequest*lookaheadSteps), queue.highestExpectedSlot)
	})

	t.Run("ready to update machines - unconstrained mode", func(t *testing.T) {
		p := p2pt.NewTestP2P(t)
		connectPeers(t, p, []*peerData{
			{blocks: makeSequence(500, 628), finalizedEpoch: 16, headSlot: 600},
		}, p.Peers())
		fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
			chain: mc,
			p2p:   p,
		})
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		queue.mode = modeNonConstrained
		assert.Equal(t, primitives.Slot(blockBatchLimit), queue.highestExpectedSlot)

		startSlot := queue.chain.HeadSlot()
		blocksPerRequest := queue.blocksFetcher.blocksPerPeriod
		var machineSlots []primitives.Slot
		for i := startSlot; i < startSlot.Add(blocksPerRequest*lookaheadSteps); i += primitives.Slot(blocksPerRequest) {
			queue.smm.addStateMachine(i).setState(stateSkipped)
			machineSlots = append(machineSlots, i)
		}
		for _, slot := range machineSlots {
			_, ok := queue.smm.findStateMachine(slot)
			assert.Equal(t, true, ok)
		}
		// Update head slot, so that machines are re-arranged starting from the next slot i.e.
		// there's no point to reset machines for some slot that has already been processed.
		updatedSlot := primitives.Slot(100)
		require.NoError(t, mc.State.SetSlot(updatedSlot))

		handlerFn := queue.onProcessSkippedEvent(ctx)
		updatedState, err := handlerFn(queue.smm.machines[primitives.Slot(blocksPerRequest*(lookaheadSteps-1))], nil)
		assert.NoError(t, err)
		assert.Equal(t, stateSkipped, updatedState)
		// Assert that machines have been re-arranged.
		for i, slot := range machineSlots {
			_, ok := queue.smm.findStateMachine(slot)
			assert.Equal(t, false, ok)
			_, ok = queue.smm.findStateMachine(updatedSlot.Add(1 + uint64(i)*blocksPerRequest))
			assert.Equal(t, true, ok)
		}
		// Assert highest expected slot is extended.
		assert.Equal(t, primitives.Slot(blocksPerRequest*(lookaheadSteps+1)), queue.highestExpectedSlot)
	})
}

func TestBlocksQueue_onCheckStaleEvent(t *testing.T) {
	blockBatchLimit := flags.Get().BlockBatchLimit
	mc, p2p, _ := initializeTestServices(t, []primitives.Slot{}, []*peerData{})

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fetcher := newBlocksFetcher(ctx, &blocksFetcherConfig{
		chain: mc,
		p2p:   p2p,
	})

	t.Run("expired context", func(t *testing.T) {
		ctx, cancel := context.WithCancel(ctx)
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		handlerFn := queue.onCheckStaleEvent(ctx)
		cancel()
		updatedState, err := handlerFn(&stateMachine{
			state: stateSkipped,
		}, nil)
		assert.ErrorContains(t, context.Canceled.Error(), err)
		assert.Equal(t, stateSkipped, updatedState)
	})

	t.Run("invalid input state", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})

		invalidStates := []stateID{stateNew, stateScheduled, stateDataParsed, stateSkipped}
		for _, state := range invalidStates {
			t.Run(state.String(), func(t *testing.T) {
				handlerFn := queue.onCheckStaleEvent(ctx)
				updatedState, err := handlerFn(&stateMachine{
					state: state,
				}, nil)
				assert.ErrorContains(t, errInvalidInitialState.Error(), err)
				assert.Equal(t, state, updatedState)
			})
		}
	})

	t.Run("process non stale machine", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		handlerFn := queue.onCheckStaleEvent(ctx)
		updatedState, err := handlerFn(&stateMachine{
			state:   stateSent,
			updated: prysmTime.Now().Add(-staleEpochTimeout / 2),
		}, nil)
		// State should not change, as machine is not yet stale.
		assert.NoError(t, err)
		assert.Equal(t, stateSent, updatedState)
	})

	t.Run("process stale machine", func(t *testing.T) {
		queue := newBlocksQueue(ctx, &blocksQueueConfig{
			blocksFetcher:       fetcher,
			chain:               mc,
			highestExpectedSlot: primitives.Slot(blockBatchLimit),
		})
		handlerFn := queue.onCheckStaleEvent(ctx)
		updatedState, err := handlerFn(&stateMachine{
			state:   stateSent,
			updated: prysmTime.Now().Add(-staleEpochTimeout),
		}, nil)
		// State should change, as machine is stale.
		assert.NoError(t, err)
		assert.Equal(t, stateSkipped, updatedState)
	})
}

func TestBlocksQueue_stuckInUnfavourableFork(t *testing.T) {
	beaconDB := dbtest.SetupDB(t)
	p2p := p2pt.NewTestP2P(t)

	// The chain1 contains 250 blocks and is a dead end.
	// The chain2 contains 296 blocks, with fork started at slot 128 of chain1.
	chain1 := extendBlockSequence(t, []*eth.SignedBeaconBlock{}, 250)
	forkedSlot := primitives.Slot(201)
	chain2 := extendBlockSequence(t, chain1[:forkedSlot], 100)
	finalizedSlot := primitives.Slot(63)
	finalizedEpoch := slots.ToEpoch(finalizedSlot)

	genesisBlock := chain1[0]
	util.SaveBlock(t, context.Background(), beaconDB, genesisBlock)
	genesisRoot, err := genesisBlock.Block.HashTreeRoot()
	require.NoError(t, err)

	st, err := util.NewBeaconState()
	require.NoError(t, err)
	mc := &mock.ChainService{
		State: st,
		Root:  genesisRoot[:],
		DB:    beaconDB,
		FinalizedCheckPoint: &eth.Checkpoint{
			Epoch: finalizedEpoch,
			Root:  []byte(fmt.Sprintf("finalized_root %d", finalizedEpoch)),
		},
		Genesis:        time.Now(),
		ValidatorsRoot: [32]byte{},
	}

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	fetcher := newBlocksFetcher(
		ctx,
		&blocksFetcherConfig{
			chain: mc,
			p2p:   p2p,
			db:    beaconDB,
			clock: startup.NewClock(mc.Genesis, mc.ValidatorsRoot),
		},
	)
	fetcher.rateLimiter = leakybucket.NewCollector(6400, 6400, 1*time.Second, false)

	queue := newBlocksQueue(ctx, &blocksQueueConfig{
		blocksFetcher:       fetcher,
		chain:               mc,
		highestExpectedSlot: primitives.Slot(len(chain2) - 1),
		mode:                modeNonConstrained,
	})

	// Populate database with blocks from unfavourable fork i.e. branch that leads to dead end.
	for _, blk := range chain1[1:] {
		parentRoot := bytesutil.ToBytes32(blk.Block.ParentRoot)
		// Save block only if parent root is already in database or cache.
		if beaconDB.HasBlock(ctx, parentRoot) || mc.HasBlock(ctx, parentRoot) {
			util.SaveBlock(t, ctx, beaconDB, blk)
			require.NoError(t, st.SetSlot(blk.Block.Slot))
		}
	}
	require.Equal(t, primitives.Slot(len(chain1)-1), mc.HeadSlot())
	hook := logTest.NewGlobal()

	t.Run("unfavourable fork and no alternative branches", func(t *testing.T) {
		defer hook.Reset()
		// Reset all machines.
		require.NoError(t, queue.smm.removeAllStateMachines())

		// Add peer that will advertise high non-finalized slot, but will not be able to support
		// its claims with actual blocks.
		emptyPeer := connectPeerHavingBlocks(t, p2p, chain1, finalizedSlot, p2p.Peers())
		defer func() {
			p2p.Peers().SetConnectionState(emptyPeer, peers.PeerDisconnected)
		}()
		chainState, err := p2p.Peers().ChainState(emptyPeer)
		require.NoError(t, err)
		chainState.HeadSlot = 500
		p2p.Peers().SetChainState(emptyPeer, chainState)

		startSlot := mc.HeadSlot() + 1
		blocksPerRequest := queue.blocksFetcher.blocksPerPeriod
		machineSlots := make([]primitives.Slot, 0)
		for i := startSlot; i < startSlot.Add(blocksPerRequest*lookaheadSteps); i += primitives.Slot(blocksPerRequest) {
			queue.smm.addStateMachine(i).setState(stateSkipped)
			machineSlots = append(machineSlots, i)
		}
		for _, slot := range machineSlots {
			_, ok := queue.smm.findStateMachine(slot)
			assert.Equal(t, true, ok)
		}
		// Since counter for stale epochs hasn't exceeded threshold, backtracking is not triggered.
		handlerFn := queue.onProcessSkippedEvent(ctx)
		assert.Equal(t, lookaheadSteps, len(queue.smm.machines))
		updatedState, err := handlerFn(queue.smm.machines[machineSlots[len(machineSlots)-1]], nil)
		assert.ErrorContains(t, "invalid range for non-skipped slot", err)
		assert.Equal(t, stateSkipped, updatedState)
		assert.Equal(t, lookaheadSteps-1, len(queue.smm.machines))
		assert.LogsDoNotContain(t, hook, "Searching for alternative blocks")
		assert.LogsDoNotContain(t, hook, "No alternative blocks found for peer")
		hook.Reset()

		// The last machine got removed (it was for non-skipped slot, which fails).
		queue.smm.addStateMachine(machineSlots[len(machineSlots)-1])
		assert.Equal(t, lookaheadSteps, len(queue.smm.machines))
		for _, slot := range machineSlots {
			fsm, ok := queue.smm.findStateMachine(slot)
			require.Equal(t, true, ok)
			fsm.setState(stateSkipped)
		}

		// Update counter, and trigger backtracking.
		queue.staleEpochs[slots.ToEpoch(machineSlots[0])] = maxResetAttempts
		handlerFn = queue.onProcessSkippedEvent(ctx)
		updatedState, err = handlerFn(queue.smm.machines[machineSlots[len(machineSlots)-1]], nil)
		assert.ErrorContains(t, "invalid range for non-skipped slot", err)
		assert.Equal(t, stateSkipped, updatedState)
		assert.Equal(t, lookaheadSteps-1, len(queue.smm.machines))
		assert.LogsContain(t, hook, "Searching for alternative blocks")
		assert.LogsContain(t, hook, "No alternative blocks found for peer")
	})

	t.Run("unfavourable fork and alternative branches exist", func(t *testing.T) {
		defer hook.Reset()
		// Reset all machines.
		require.NoError(t, queue.smm.removeAllStateMachines())

		// Add peer that will advertise high non-finalized slot, but will not be able to support
		// its claims with actual blocks.
		forkedPeer := connectPeerHavingBlocks(t, p2p, chain2, finalizedSlot, p2p.Peers())
		startSlot := mc.HeadSlot() + 1
		blocksPerRequest := queue.blocksFetcher.blocksPerPeriod
		machineSlots := make([]primitives.Slot, 0)
		for i := startSlot; i < startSlot.Add(blocksPerRequest*lookaheadSteps); i += primitives.Slot(blocksPerRequest) {
			queue.smm.addStateMachine(i).setState(stateSkipped)
			machineSlots = append(machineSlots, i)
		}
		for _, slot := range machineSlots {
			_, ok := queue.smm.findStateMachine(slot)
			assert.Equal(t, true, ok)
		}
		// Since counter for stale epochs hasn't exceeded threshold, backtracking is not triggered.
		handlerFn := queue.onProcessSkippedEvent(ctx)
		assert.Equal(t, lookaheadSteps, len(queue.smm.machines))
		updatedState, err := handlerFn(queue.smm.machines[machineSlots[len(machineSlots)-1]], nil)
		assert.ErrorContains(t, "invalid range for non-skipped slot", err)
		assert.Equal(t, stateSkipped, updatedState)
		assert.Equal(t, lookaheadSteps-1, len(queue.smm.machines))
		assert.LogsDoNotContain(t, hook, "Searching for alternative blocks")
		assert.LogsDoNotContain(t, hook, "No alternative blocks found for peer")
		hook.Reset()

		// The last machine got removed (it was for non-skipped slot, which fails).
		queue.smm.addStateMachine(machineSlots[len(machineSlots)-1])
		assert.Equal(t, lookaheadSteps, len(queue.smm.machines))
		for _, slot := range machineSlots {
			fsm, ok := queue.smm.findStateMachine(slot)
			require.Equal(t, true, ok)
			fsm.setState(stateSkipped)
		}

		// Update counter, and trigger backtracking.
		queue.staleEpochs[slots.ToEpoch(machineSlots[0])] = maxResetAttempts
		handlerFn = queue.onProcessSkippedEvent(ctx)
		updatedState, err = handlerFn(queue.smm.machines[machineSlots[len(machineSlots)-1]], nil)
		require.NoError(t, err)
		assert.Equal(t, stateSkipped, updatedState)
		assert.LogsContain(t, hook, "Searching for alternative blocks")
		assert.LogsDoNotContain(t, hook, "No alternative blocks found for peer")
		require.Equal(t, lookaheadSteps, len(queue.smm.machines))

		// Alternative fork should start on slot 201, make sure that the first machine contains all
		// required forked data, including data on and after slot 201.
		forkedEpochStartSlot, err := slots.EpochStart(slots.ToEpoch(forkedSlot))
		require.NoError(t, err)
		firstFSM, ok := queue.smm.findStateMachine(forkedSlot)
		require.Equal(t, true, ok)
		require.Equal(t, stateDataParsed, firstFSM.state)
		require.Equal(t, forkedPeer, firstFSM.pid)
		reqEnd := testForkStartSlot(t, 251) + primitives.Slot(findForkReqRangeSize())
		require.Equal(t, int(reqEnd-forkedSlot), len(firstFSM.bwb))
		require.Equal(t, forkedSlot, firstFSM.bwb[0].Block.Block().Slot())

		// Assert that forked data from chain2 is available (within 64 fetched blocks).
		for i, blk := range chain2[forkedSlot:] {
			if i >= len(firstFSM.bwb) {
				break
			}
			rootFromFSM := firstFSM.bwb[i].Block.Root()
			blkRoot, err := blk.Block.HashTreeRoot()
			require.NoError(t, err)
			assert.Equal(t, blkRoot, rootFromFSM)
		}

		// Assert that machines are in the expected state.
		startSlot = forkedEpochStartSlot.Add(1 + blocksPerRequest)
		require.Equal(t, int(blocksPerRequest)-int(forkedSlot-(forkedEpochStartSlot+1)), len(firstFSM.bwb))
		for i := startSlot; i < startSlot.Add(blocksPerRequest*(lookaheadSteps-1)); i += primitives.Slot(blocksPerRequest) {
			fsm, ok := queue.smm.findStateMachine(i)
			require.Equal(t, true, ok)
			assert.Equal(t, stateSkipped, fsm.state)
		}
	})
}

func TestBlocksQueue_stuckWhenHeadIsSetToOrphanedBlock(t *testing.T) {
	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()

	beaconDB := dbtest.SetupDB(t)
	p2p := p2pt.NewTestP2P(t)

	chain := extendBlockSequence(t, []*eth.SignedBeaconBlock{}, 128)
	finalizedSlot := primitives.Slot(82)
	finalizedEpoch := slots.ToEpoch(finalizedSlot)

	genesisBlock := chain[0]
	util.SaveBlock(t, context.Background(), beaconDB, genesisBlock)
	genesisRoot, err := genesisBlock.Block.HashTreeRoot()
	require.NoError(t, err)

	st, err := util.NewBeaconState()
	require.NoError(t, err)
	mc := &mock.ChainService{
		State: st,
		Root:  genesisRoot[:],
		DB:    beaconDB,
		FinalizedCheckPoint: &eth.Checkpoint{
			Epoch: finalizedEpoch,
			Root:  []byte(fmt.Sprintf("finalized_root %d", finalizedEpoch)),
		},
		Genesis:        time.Now(),
		ValidatorsRoot: [32]byte{},
	}

	// Populate database with blocks with part of the chain, orphaned block will be added on top.
	for _, blk := range chain[1:84] {
		parentRoot := bytesutil.ToBytes32(blk.Block.ParentRoot)
		// Save block only if parent root is already in database or cache.
		if beaconDB.HasBlock(ctx, parentRoot) || mc.HasBlock(ctx, parentRoot) {
			util.SaveBlock(t, ctx, beaconDB, blk)
			require.NoError(t, st.SetSlot(blk.Block.Slot))
		}
	}
	require.Equal(t, primitives.Slot(83), mc.HeadSlot())
	require.Equal(t, chain[83].Block.Slot, mc.HeadSlot())

	// Set head to slot 85, while we do not have block with slot 84 in DB, so block is orphaned.
	// Moreover, block with slot 85 is a forked block and should be replaced, with block from peer.
	orphanedBlock := util.NewBeaconBlock()
	orphanedBlock.Block.Slot = 85
	orphanedBlock.Block.StateRoot = util.Random32Bytes(t)
	util.SaveBlock(t, ctx, beaconDB, orphanedBlock)
	require.NoError(t, st.SetSlot(orphanedBlock.Block.Slot))
	require.Equal(t, primitives.Slot(85), mc.HeadSlot())

	fetcher := newBlocksFetcher(
		ctx,
		&blocksFetcherConfig{
			chain: mc,
			p2p:   p2p,
			db:    beaconDB,
			clock: startup.NewClock(mc.Genesis, mc.ValidatorsRoot),
		},
	)
	fetcher.rateLimiter = leakybucket.NewCollector(6400, 6400, 1*time.Second, false)

	// Connect peer that has all the blocks available.
	allBlocksPeer := connectPeerHavingBlocks(t, p2p, chain, finalizedSlot, p2p.Peers())
	defer func() {
		p2p.Peers().SetConnectionState(allBlocksPeer, peers.PeerDisconnected)
	}()

	// Queue should be able to fetch whole chain (including slot which comes before the currently set head).
	queue := newBlocksQueue(ctx, &blocksQueueConfig{
		blocksFetcher:       fetcher,
		chain:               mc,
		highestExpectedSlot: primitives.Slot(len(chain) - 1),
		mode:                modeNonConstrained,
	})

	require.NoError(t, queue.start())
	isProcessedBlock := func(ctx context.Context, blk interfaces.ReadOnlySignedBeaconBlock, blkRoot [32]byte) bool {
		cp := mc.FinalizedCheckpt()
		finalizedSlot, err := slots.EpochStart(cp.Epoch)
		if err != nil {
			return false
		}
		if blk.Block().Slot() <= finalizedSlot || (beaconDB.HasBlock(ctx, blkRoot) || mc.HasBlock(ctx, blkRoot)) {
			return true
		}
		return false
	}

	select {
	case <-time.After(3 * time.Second):
		t.Fatal("test takes too long to complete")
	case data := <-queue.fetchedData:
		for _, b := range data.bwb {
			blk := b.Block
			blkRoot, err := blk.Block().HashTreeRoot()
			require.NoError(t, err)
			if isProcessedBlock(ctx, blk, blkRoot) {
				log.Errorf("slot: %d , root %#x: %v", blk.Block().Slot(), blkRoot, errBlockAlreadyProcessed)
				continue
			}

			parentRoot := blk.Block().ParentRoot()
			if !beaconDB.HasBlock(ctx, parentRoot) && !mc.HasBlock(ctx, parentRoot) {
				log.Errorf("%v: %#x", errParentDoesNotExist, blk.Block().ParentRoot())
				continue
			}

			// Block is not already processed, and parent exists in database - process.
			require.NoError(t, beaconDB.SaveBlock(ctx, blk))
			require.NoError(t, st.SetSlot(blk.Block().Slot()))
		}
	}
	require.NoError(t, queue.stop())

	// Check that all blocks available in chain are produced by queue.
	for _, blk := range chain[:orphanedBlock.Block.Slot+32] {
		blkRoot, err := blk.Block.HashTreeRoot()
		require.NoError(t, err)
		require.Equal(t, true, beaconDB.HasBlock(ctx, blkRoot) || mc.HasBlock(ctx, blkRoot), "slot %d", blk.Block.Slot)
	}
}
