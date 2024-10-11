package types

import (
	"context"
	"fmt"
	"math/rand"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	abci "github.com/cometbft/cometbft/abci/types"
	cmtpubsub "github.com/cometbft/cometbft/libs/pubsub"
	cmtquery "github.com/cometbft/cometbft/libs/pubsub/query"
	cmttime "github.com/cometbft/cometbft/types/time"
)

func TestEventBusPublishEventTx(t *testing.T) {
	eventBus := NewEventBus()
	err := eventBus.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := eventBus.Stop(); err != nil {
			t.Error(err)
		}
	})

	tx := Tx("foo")
	result := abci.ExecTxResult{
		Data: []byte("bar"),
		Events: []abci.Event{
			{Type: "testType", Attributes: []abci.EventAttribute{{Key: "baz", Value: "1"}}},
		},
	}

	// PublishEventTx adds 3 composite keys, so the query below should work
	query := fmt.Sprintf("tm.event='Tx' AND tx.height=1 AND tx.hash='%X' AND testType.baz=1", tx.Hash())
	txsSub, err := eventBus.Subscribe(context.Background(), "test", cmtquery.MustCompile(query))
	require.NoError(t, err)

	done := make(chan struct{})
	go func() {
		msg := <-txsSub.Out()
		edt := msg.Data().(EventDataTx)
		assert.Equal(t, int64(1), edt.Height)
		assert.Equal(t, uint32(0), edt.Index)
		assert.EqualValues(t, tx, edt.Tx)
		assert.Equal(t, result, edt.Result)
		close(done)
	}()

	err = eventBus.PublishEventTx(EventDataTx{abci.TxResult{
		Height: 1,
		Index:  0,
		Tx:     tx,
		Result: result,
	}})
	require.NoError(t, err)

	select {
	case <-done:
	case <-time.After(1 * time.Second):
		t.Fatal("did not receive a transaction after 1 sec.")
	}
}

func TestEventBusPublishEventNewBlock(t *testing.T) {
	eventBus := NewEventBus()
	err := eventBus.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := eventBus.Stop(); err != nil {
			t.Error(err)
		}
	})

	block := MakeBlock(0, []Tx{}, nil, []Evidence{})
	resultFinalizeBlock := abci.FinalizeBlockResponse{
		Events: []abci.Event{
			{Type: "testType", Attributes: []abci.EventAttribute{{Key: "baz", Value: "1"}}},
		},
	}

	// PublishEventNewBlock adds the tm.event compositeKey, so the query below should work
	query := "tm.event='NewBlock' AND testType.baz=1"
	blocksSub, err := eventBus.Subscribe(context.Background(), "test", cmtquery.MustCompile(query))
	require.NoError(t, err)

	done := make(chan struct{})
	go func() {
		msg := <-blocksSub.Out()
		edt := msg.Data().(EventDataNewBlock)
		assert.Equal(t, block, edt.Block)
		assert.Equal(t, resultFinalizeBlock, edt.ResultFinalizeBlock)
		close(done)
	}()

	var ps *PartSet
	ps, err = block.MakePartSet(BlockPartSizeBytes)
	require.NoError(t, err)

	err = eventBus.PublishEventNewBlock(EventDataNewBlock{
		Block: block,
		BlockID: BlockID{
			Hash:          block.Hash(),
			PartSetHeader: ps.Header(),
		},
		ResultFinalizeBlock: resultFinalizeBlock,
	})
	require.NoError(t, err)

	select {
	case <-done:
	case <-time.After(1 * time.Second):
		t.Fatal("did not receive a block after 1 sec.")
	}
}

func TestEventBusPublishEventTxDuplicateKeys(t *testing.T) {
	eventBus := NewEventBus()
	err := eventBus.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := eventBus.Stop(); err != nil {
			t.Error(err)
		}
	})

	tx := Tx("foo")
	result := abci.ExecTxResult{
		Data: []byte("bar"),
		Events: []abci.Event{
			{
				Type: "transfer",
				Attributes: []abci.EventAttribute{
					{Key: "sender", Value: "foo"},
					{Key: "recipient", Value: "bar"},
					{Key: "amount", Value: "5"},
				},
			},
			{
				Type: "transfer",
				Attributes: []abci.EventAttribute{
					{Key: "sender", Value: "baz"},
					{Key: "recipient", Value: "cat"},
					{Key: "amount", Value: "13"},
				},
			},
			{
				Type: "withdraw.rewards",
				Attributes: []abci.EventAttribute{
					{Key: "address", Value: "bar"},
					{Key: "source", Value: "iceman"},
					{Key: "amount", Value: "33"},
				},
			},
		},
	}

	testCases := []struct {
		query         string
		expectResults bool
	}{
		{
			"tm.event='Tx' AND tx.height=1 AND transfer.sender='DoesNotExist'",
			false,
		},
		{
			"tm.event='Tx' AND tx.height=1 AND transfer.sender='foo'",
			true,
		},
		{
			"tm.event='Tx' AND tx.height=1 AND transfer.sender='baz'",
			true,
		},
		{
			"tm.event='Tx' AND tx.height=1 AND transfer.sender='foo' AND transfer.sender='baz'",
			true,
		},
		{
			"tm.event='Tx' AND tx.height=1 AND transfer.sender='foo' AND transfer.sender='DoesNotExist'",
			false,
		},
	}

	for i, tc := range testCases {
		sub, err := eventBus.Subscribe(context.Background(), fmt.Sprintf("client-%d", i), cmtquery.MustCompile(tc.query))
		require.NoError(t, err)

		done := make(chan struct{})

		go func() {
			select {
			case msg := <-sub.Out():
				data := msg.Data().(EventDataTx)
				assert.Equal(t, int64(1), data.Height)
				assert.Equal(t, uint32(0), data.Index)
				assert.EqualValues(t, tx, data.Tx)
				assert.Equal(t, result, data.Result)
				close(done)
			case <-time.After(1 * time.Second):
				return
			}
		}()

		err = eventBus.PublishEventTx(EventDataTx{abci.TxResult{
			Height: 1,
			Index:  0,
			Tx:     tx,
			Result: result,
		}})
		require.NoError(t, err)

		select {
		case <-done:
			if !tc.expectResults {
				require.Fail(t, "unexpected transaction result(s) from subscription")
			}
		case <-time.After(1 * time.Second):
			if tc.expectResults {
				require.Fail(t, "failed to receive a transaction after 1 second")
			}
		}
	}
}

func TestEventBusPublishEventNewBlockHeader(t *testing.T) {
	eventBus := NewEventBus()
	err := eventBus.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := eventBus.Stop(); err != nil {
			t.Error(err)
		}
	})

	block := MakeBlock(0, []Tx{}, nil, []Evidence{})
	// PublishEventNewBlockHeader adds the tm.event compositeKey, so the query below should work
	query := "tm.event='NewBlockHeader'"
	headersSub, err := eventBus.Subscribe(context.Background(), "test", cmtquery.MustCompile(query))
	require.NoError(t, err)

	done := make(chan struct{})
	go func() {
		msg := <-headersSub.Out()
		edt := msg.Data().(EventDataNewBlockHeader)
		assert.Equal(t, block.Header, edt.Header)
		close(done)
	}()

	err = eventBus.PublishEventNewBlockHeader(EventDataNewBlockHeader{
		Header: block.Header,
	})
	require.NoError(t, err)

	select {
	case <-done:
	case <-time.After(1 * time.Second):
		t.Fatal("did not receive a block header after 1 sec.")
	}
}

func TestEventBusPublishEventNewBlockEvents(t *testing.T) {
	eventBus := NewEventBus()
	err := eventBus.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := eventBus.Stop(); err != nil {
			t.Error(err)
		}
	})

	// PublishEventNewBlockHeader adds the tm.event compositeKey, so the query below should work
	query := "tm.event='NewBlockEvents'"
	headersSub, err := eventBus.Subscribe(context.Background(), "test", cmtquery.MustCompile(query))
	require.NoError(t, err)

	done := make(chan struct{})
	go func() {
		msg := <-headersSub.Out()
		edt := msg.Data().(EventDataNewBlockEvents)
		assert.Equal(t, int64(1), edt.Height)
		close(done)
	}()

	err = eventBus.PublishEventNewBlockEvents(EventDataNewBlockEvents{
		Height: 1,
		Events: []abci.Event{{
			Type: "transfer",
			Attributes: []abci.EventAttribute{{
				Key:   "currency",
				Value: "ATOM",
			}},
		}},
	})
	require.NoError(t, err)

	select {
	case <-done:
	case <-time.After(1 * time.Second):
		t.Fatal("did not receive a block header after 1 sec.")
	}
}

func TestEventBusPublishEventNewEvidence(t *testing.T) {
	eventBus := NewEventBus()
	err := eventBus.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := eventBus.Stop(); err != nil {
			t.Error(err)
		}
	})

	ev, err := NewMockDuplicateVoteEvidence(1, cmttime.Now(), "test-chain-id")
	require.NoError(t, err)

	query := "tm.event='NewEvidence'"
	evSub, err := eventBus.Subscribe(context.Background(), "test", cmtquery.MustCompile(query))
	require.NoError(t, err)

	done := make(chan struct{})
	go func() {
		msg := <-evSub.Out()
		edt := msg.Data().(EventDataNewEvidence)
		assert.Equal(t, ev, edt.Evidence)
		assert.Equal(t, int64(4), edt.Height)
		close(done)
	}()

	err = eventBus.PublishEventNewEvidence(EventDataNewEvidence{
		Evidence: ev,
		Height:   4,
	})
	require.NoError(t, err)

	select {
	case <-done:
	case <-time.After(1 * time.Second):
		t.Fatal("did not receive a block header after 1 sec.")
	}
}

func TestEventBusPublish(t *testing.T) {
	eventBus := NewEventBus()
	err := eventBus.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := eventBus.Stop(); err != nil {
			t.Error(err)
		}
	})

	const numEventsExpected = 14

	sub, err := eventBus.Subscribe(context.Background(), "test", cmtquery.All, numEventsExpected)
	require.NoError(t, err)

	done := make(chan struct{})
	go func() {
		numEvents := 0
		for range sub.Out() {
			numEvents++
			if numEvents >= numEventsExpected {
				close(done)
				return
			}
		}
	}()

	err = eventBus.Publish(EventNewBlockHeader, EventDataNewBlockHeader{})
	require.NoError(t, err)
	err = eventBus.PublishEventNewBlock(EventDataNewBlock{Block: &Block{Header: Header{Height: 1}}})
	require.NoError(t, err)
	err = eventBus.PublishEventNewBlockHeader(EventDataNewBlockHeader{})
	require.NoError(t, err)
	err = eventBus.PublishEventNewBlockEvents(EventDataNewBlockEvents{Height: 1})
	require.NoError(t, err)
	err = eventBus.PublishEventVote(EventDataVote{})
	require.NoError(t, err)
	err = eventBus.PublishEventNewRoundStep(EventDataRoundState{})
	require.NoError(t, err)
	err = eventBus.PublishEventTimeoutPropose(EventDataRoundState{})
	require.NoError(t, err)
	err = eventBus.PublishEventTimeoutWait(EventDataRoundState{})
	require.NoError(t, err)
	err = eventBus.PublishEventNewRound(EventDataNewRound{})
	require.NoError(t, err)
	err = eventBus.PublishEventCompleteProposal(EventDataCompleteProposal{})
	require.NoError(t, err)
	err = eventBus.PublishEventPolka(EventDataRoundState{})
	require.NoError(t, err)
	err = eventBus.PublishEventRelock(EventDataRoundState{})
	require.NoError(t, err)
	err = eventBus.PublishEventLock(EventDataRoundState{})
	require.NoError(t, err)
	err = eventBus.PublishEventValidatorSetUpdates(EventDataValidatorSetUpdates{})
	require.NoError(t, err)

	select {
	case <-done:
	case <-time.After(1 * time.Second):
		t.Fatalf("expected to receive %d events after 1 sec.", numEventsExpected)
	}
}

func BenchmarkEventBus(b *testing.B) {
	benchmarks := []struct {
		name        string
		numClients  int
		randQueries bool
		randEvents  bool
	}{
		{"10Clients1Query1Event", 10, false, false},
		{"100Clients", 100, false, false},
		{"1000Clients", 1000, false, false},

		{"10ClientsRandQueries1Event", 10, true, false},
		{"100Clients", 100, true, false},
		{"1000Clients", 1000, true, false},

		{"10ClientsRandQueriesRandEvents", 10, true, true},
		{"100Clients", 100, true, true},
		{"1000Clients", 1000, true, true},

		{"10Clients1QueryRandEvents", 10, false, true},
		{"100Clients", 100, false, true},
		{"1000Clients", 1000, false, true},
	}

	for _, bm := range benchmarks {
		b.Run(bm.name, func(b *testing.B) {
			benchmarkEventBus(b, bm.numClients, bm.randQueries, bm.randEvents)
		})
	}
}

func benchmarkEventBus(b *testing.B, numClients int, randQueries bool, randEvents bool) {
	b.Helper()
	// for random* functions
	rnd := rand.New(rand.NewSource(cmttime.Now().Unix()))

	eventBus := NewEventBusWithBufferCapacity(0) // set buffer capacity to 0 so we are not testing cache
	err := eventBus.Start()
	if err != nil {
		b.Error(err)
	}
	b.Cleanup(func() {
		if err := eventBus.Stop(); err != nil {
			b.Error(err)
		}
	})

	ctx := context.Background()
	q := EventQueryNewBlock

	for i := 0; i < numClients; i++ {
		if randQueries {
			q = randQuery(rnd)
		}
		sub, err := eventBus.Subscribe(ctx, fmt.Sprintf("client-%d", i), q)
		if err != nil {
			b.Fatal(err)
		}
		go func() {
			for {
				select {
				case <-sub.Out():
				case <-sub.Canceled():
					return
				}
			}
		}()
	}

	eventType := EventNewBlock

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		if randEvents {
			eventType = randEvent(rnd)
		}

		err := eventBus.Publish(eventType, EventDataString("Gamora"))
		if err != nil {
			b.Error(err)
		}
	}
}

var events = []string{
	EventNewBlock,
	EventNewBlockHeader,
	EventNewBlockEvents,
	EventNewRound,
	EventNewRoundStep,
	EventTimeoutPropose,
	EventCompleteProposal,
	EventPolka,
	EventLock,
	EventRelock,
	EventTimeoutWait,
	EventVote,
}

func randEvent(r *rand.Rand) string {
	return events[r.Intn(len(events))]
}

var queries = []cmtpubsub.Query{
	EventQueryNewBlock,
	EventQueryNewBlockHeader,
	EventQueryNewBlockEvents,
	EventQueryNewRound,
	EventQueryNewRoundStep,
	EventQueryTimeoutPropose,
	EventQueryCompleteProposal,
	EventQueryPolka,
	EventQueryLock,
	EventQueryRelock,
	EventQueryTimeoutWait,
	EventQueryVote,
}

func randQuery(r *rand.Rand) cmtpubsub.Query {
	return queries[r.Intn(len(queries))]
}
