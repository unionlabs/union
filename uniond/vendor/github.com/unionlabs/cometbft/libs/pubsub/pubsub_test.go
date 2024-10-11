package pubsub_test

import (
	"context"
	"fmt"
	"runtime/debug"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/cometbft/cometbft/libs/log"
	"github.com/cometbft/cometbft/libs/pubsub"
	"github.com/cometbft/cometbft/libs/pubsub/query"
)

const (
	clientID = "test-client"
)

func TestSubscribe(t *testing.T) {
	s := pubsub.NewServer()
	s.SetLogger(log.TestingLogger())
	err := s.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := s.Stop(); err != nil {
			t.Error(err)
		}
	})

	ctx := context.Background()
	subscription, err := s.Subscribe(ctx, clientID, query.All)
	require.NoError(t, err)

	assert.Equal(t, 1, s.NumClients())
	assert.Equal(t, 1, s.NumClientSubscriptions(clientID))

	err = s.Publish(ctx, "Ka-Zar")
	require.NoError(t, err)
	assertReceive(t, "Ka-Zar", subscription.Out())

	published := make(chan struct{})
	go func() {
		defer close(published)

		err := s.Publish(ctx, "Quicksilver")
		require.NoError(t, err)

		err = s.Publish(ctx, "Asylum")
		require.NoError(t, err)

		err = s.Publish(ctx, "Ivan")
		require.NoError(t, err)
	}()

	select {
	case <-published:
		assertReceive(t, "Quicksilver", subscription.Out())
		assertCancelled(t, subscription, pubsub.ErrOutOfCapacity)
	case <-time.After(3 * time.Second):
		t.Fatal("Expected Publish(Asylum) not to block")
	}
}

func TestSubscribeWithCapacity(t *testing.T) {
	s := pubsub.NewServer()
	s.SetLogger(log.TestingLogger())
	err := s.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := s.Stop(); err != nil {
			t.Error(err)
		}
	})

	ctx := context.Background()
	assert.Panics(t, func() {
		_, err = s.Subscribe(ctx, clientID, query.All, -1)
		require.NoError(t, err)
	})
	assert.Panics(t, func() {
		_, err = s.Subscribe(ctx, clientID, query.All, 0)
		require.NoError(t, err)
	})
	subscription, err := s.Subscribe(ctx, clientID, query.All, 1)
	require.NoError(t, err)
	err = s.Publish(ctx, "Aggamon")
	require.NoError(t, err)
	assertReceive(t, "Aggamon", subscription.Out())
}

func TestSubscribeUnbuffered(t *testing.T) {
	s := pubsub.NewServer()
	s.SetLogger(log.TestingLogger())
	err := s.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := s.Stop(); err != nil {
			t.Error(err)
		}
	})

	ctx := context.Background()
	subscription, err := s.SubscribeUnbuffered(ctx, clientID, query.All)
	require.NoError(t, err)

	published := make(chan struct{})
	go func() {
		defer close(published)

		err := s.Publish(ctx, "Ultron")
		require.NoError(t, err)

		err = s.Publish(ctx, "Darkhawk")
		require.NoError(t, err)
	}()

	select {
	case <-published:
		t.Fatal("Expected Publish(Darkhawk) to block")
	case <-time.After(3 * time.Second):
		assertReceive(t, "Ultron", subscription.Out())
		assertReceive(t, "Darkhawk", subscription.Out())
	}
}

func TestSlowClientIsRemovedWithErrOutOfCapacity(t *testing.T) {
	s := pubsub.NewServer()
	s.SetLogger(log.TestingLogger())
	err := s.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := s.Stop(); err != nil {
			t.Error(err)
		}
	})

	ctx := context.Background()
	subscription, err := s.Subscribe(ctx, clientID, query.All)
	require.NoError(t, err)
	err = s.Publish(ctx, "Fat Cobra")
	require.NoError(t, err)
	err = s.Publish(ctx, "Viper")
	require.NoError(t, err)

	assertCancelled(t, subscription, pubsub.ErrOutOfCapacity)
}

func TestDifferentClients(t *testing.T) {
	s := pubsub.NewServer()
	s.SetLogger(log.TestingLogger())
	err := s.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := s.Stop(); err != nil {
			t.Error(err)
		}
	})

	ctx := context.Background()
	subscription1, err := s.Subscribe(ctx, "client-1", query.MustCompile("tm.events.type='NewBlock'"))
	require.NoError(t, err)
	err = s.PublishWithEvents(ctx, "Iceman", map[string][]string{"tm.events.type": {"NewBlock"}})
	require.NoError(t, err)
	assertReceive(t, "Iceman", subscription1.Out())

	subscription2, err := s.Subscribe(
		ctx,
		"client-2",
		query.MustCompile("tm.events.type='NewBlock' AND abci.account.name='Igor'"),
	)
	require.NoError(t, err)
	err = s.PublishWithEvents(
		ctx,
		"Ultimo",
		map[string][]string{"tm.events.type": {"NewBlock"}, "abci.account.name": {"Igor"}},
	)
	require.NoError(t, err)
	assertReceive(t, "Ultimo", subscription1.Out())
	assertReceive(t, "Ultimo", subscription2.Out())

	subscription3, err := s.Subscribe(
		ctx,
		"client-3",
		query.MustCompile("tm.events.type='NewRoundStep' AND abci.account.name='Igor' AND abci.invoice.number = 10"),
	)
	require.NoError(t, err)
	err = s.PublishWithEvents(ctx, "Valeria Richards", map[string][]string{"tm.events.type": {"NewRoundStep"}})
	require.NoError(t, err)
	assert.Zero(t, len(subscription3.Out()))
}

func TestSubscribeDuplicateKeys(t *testing.T) {
	ctx := context.Background()
	s := pubsub.NewServer()
	s.SetLogger(log.TestingLogger())
	require.NoError(t, s.Start())
	t.Cleanup(func() {
		if err := s.Stop(); err != nil {
			t.Error(err)
		}
	})

	testCases := []struct {
		query    string
		expected any
	}{
		{
			"withdraw.rewards='17'",
			"Iceman",
		},
		{
			"withdraw.rewards='22'",
			"Iceman",
		},
		{
			"withdraw.rewards='1' AND withdraw.rewards='22'",
			"Iceman",
		},
		{
			"withdraw.rewards='100'",
			nil,
		},
	}

	for i, tc := range testCases {
		sub, err := s.Subscribe(ctx, fmt.Sprintf("client-%d", i), query.MustCompile(tc.query))
		require.NoError(t, err)

		err = s.PublishWithEvents(
			ctx,
			"Iceman",
			map[string][]string{
				"transfer.sender":  {"foo", "bar", "baz"},
				"withdraw.rewards": {"1", "17", "22"},
			},
		)
		require.NoError(t, err)

		if tc.expected != nil {
			assertReceive(t, tc.expected, sub.Out())
		} else {
			require.Zero(t, len(sub.Out()))
		}
	}
}

func TestClientSubscribesTwice(t *testing.T) {
	s := pubsub.NewServer()
	s.SetLogger(log.TestingLogger())
	err := s.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := s.Stop(); err != nil {
			t.Error(err)
		}
	})

	ctx := context.Background()
	q := query.MustCompile("tm.events.type='NewBlock'")

	subscription1, err := s.Subscribe(ctx, clientID, q)
	require.NoError(t, err)
	err = s.PublishWithEvents(ctx, "Goblin Queen", map[string][]string{"tm.events.type": {"NewBlock"}})
	require.NoError(t, err)
	assertReceive(t, "Goblin Queen", subscription1.Out())

	subscription2, err := s.Subscribe(ctx, clientID, q)
	require.Error(t, err)
	require.Nil(t, subscription2)

	err = s.PublishWithEvents(ctx, "Spider-Man", map[string][]string{"tm.events.type": {"NewBlock"}})
	require.NoError(t, err)
	assertReceive(t, "Spider-Man", subscription1.Out())
}

func TestUnsubscribe(t *testing.T) {
	s := pubsub.NewServer()
	s.SetLogger(log.TestingLogger())
	err := s.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := s.Stop(); err != nil {
			t.Error(err)
		}
	})

	ctx := context.Background()
	subscription, err := s.Subscribe(ctx, clientID, query.MustCompile("tm.events.type='NewBlock'"))
	require.NoError(t, err)
	err = s.Unsubscribe(ctx, clientID, query.MustCompile("tm.events.type='NewBlock'"))
	require.NoError(t, err)

	err = s.Publish(ctx, "Nick Fury")
	require.NoError(t, err)
	assert.Zero(t, len(subscription.Out()), "Should not receive anything after Unsubscribe")

	assertCancelled(t, subscription, pubsub.ErrUnsubscribed)
}

func TestClientUnsubscribesTwice(t *testing.T) {
	s := pubsub.NewServer()
	s.SetLogger(log.TestingLogger())
	err := s.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := s.Stop(); err != nil {
			t.Error(err)
		}
	})

	ctx := context.Background()
	_, err = s.Subscribe(ctx, clientID, query.MustCompile("tm.events.type='NewBlock'"))
	require.NoError(t, err)
	err = s.Unsubscribe(ctx, clientID, query.MustCompile("tm.events.type='NewBlock'"))
	require.NoError(t, err)

	err = s.Unsubscribe(ctx, clientID, query.MustCompile("tm.events.type='NewBlock'"))
	assert.Equal(t, pubsub.ErrSubscriptionNotFound, err)
	err = s.UnsubscribeAll(ctx, clientID)
	assert.Equal(t, pubsub.ErrSubscriptionNotFound, err)
}

func TestResubscribe(t *testing.T) {
	s := pubsub.NewServer()
	s.SetLogger(log.TestingLogger())
	err := s.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := s.Stop(); err != nil {
			t.Error(err)
		}
	})

	ctx := context.Background()
	_, err = s.Subscribe(ctx, clientID, query.All)
	require.NoError(t, err)
	err = s.Unsubscribe(ctx, clientID, query.All)
	require.NoError(t, err)
	subscription, err := s.Subscribe(ctx, clientID, query.All)
	require.NoError(t, err)

	err = s.Publish(ctx, "Cable")
	require.NoError(t, err)
	assertReceive(t, "Cable", subscription.Out())
}

func TestUnsubscribeAll(t *testing.T) {
	s := pubsub.NewServer()
	s.SetLogger(log.TestingLogger())
	err := s.Start()
	require.NoError(t, err)
	t.Cleanup(func() {
		if err := s.Stop(); err != nil {
			t.Error(err)
		}
	})

	ctx := context.Background()
	subscription1, err := s.Subscribe(ctx, clientID, query.MustCompile("tm.events.type='NewBlock'"))
	require.NoError(t, err)
	subscription2, err := s.Subscribe(ctx, clientID, query.MustCompile("tm.events.type='NewBlockHeader'"))
	require.NoError(t, err)

	err = s.UnsubscribeAll(ctx, clientID)
	require.NoError(t, err)

	err = s.Publish(ctx, "Nick Fury")
	require.NoError(t, err)
	assert.Zero(t, len(subscription1.Out()), "Should not receive anything after UnsubscribeAll")
	assert.Zero(t, len(subscription2.Out()), "Should not receive anything after UnsubscribeAll")

	assertCancelled(t, subscription1, pubsub.ErrUnsubscribed)
	assertCancelled(t, subscription2, pubsub.ErrUnsubscribed)
}

func TestBufferCapacity(t *testing.T) {
	s := pubsub.NewServer(pubsub.BufferCapacity(2))
	s.SetLogger(log.TestingLogger())

	assert.Equal(t, 2, s.BufferCapacity())

	ctx := context.Background()
	err := s.Publish(ctx, "Nighthawk")
	require.NoError(t, err)
	err = s.Publish(ctx, "Sage")
	require.NoError(t, err)

	ctx, cancel := context.WithTimeout(ctx, 10*time.Millisecond)
	defer cancel()
	err = s.Publish(ctx, "Ironclad")
	if assert.Error(t, err) { //nolint:testifylint // require.Error doesn't work with the conditional here
		assert.Equal(t, context.DeadlineExceeded, err)
	}
}

func Benchmark10Clients(b *testing.B)   { benchmarkNClients(b, 10) }
func Benchmark100Clients(b *testing.B)  { benchmarkNClients(b, 100) }
func Benchmark1000Clients(b *testing.B) { benchmarkNClients(b, 1000) }

func Benchmark10ClientsOneQuery(b *testing.B)   { benchmarkNClientsOneQuery(b, 10) }
func Benchmark100ClientsOneQuery(b *testing.B)  { benchmarkNClientsOneQuery(b, 100) }
func Benchmark1000ClientsOneQuery(b *testing.B) { benchmarkNClientsOneQuery(b, 1000) }

func benchmarkNClients(b *testing.B, n int) {
	b.Helper()
	s := pubsub.NewServer()
	err := s.Start()
	require.NoError(b, err)

	b.Cleanup(func() {
		if err := s.Stop(); err != nil {
			b.Error(err)
		}
	})

	ctx := context.Background()
	for i := 0; i < n; i++ {
		subscription, err := s.Subscribe(
			ctx,
			clientID,
			query.MustCompile(fmt.Sprintf("abci.Account.Owner = 'Ivan' AND abci.Invoices.Number = %d", i)),
		)
		if err != nil {
			b.Fatal(err)
		}
		go func() {
			for {
				select {
				case <-subscription.Out():
					continue
				case <-subscription.Canceled():
					return
				}
			}
		}()
	}

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		err = s.PublishWithEvents(
			ctx,
			"Gamora",
			map[string][]string{"abci.Account.Owner": {"Ivan"}, "abci.Invoices.Number": {string(rune(i))}},
		)
		require.NoError(b, err)
	}
}

func benchmarkNClientsOneQuery(b *testing.B, n int) {
	b.Helper()
	s := pubsub.NewServer()
	err := s.Start()
	require.NoError(b, err)
	b.Cleanup(func() {
		if err := s.Stop(); err != nil {
			b.Error(err)
		}
	})

	ctx := context.Background()
	q := query.MustCompile("abci.Account.Owner = 'Ivan' AND abci.Invoices.Number = 1")
	for i := 0; i < n; i++ {
		subscription, err := s.Subscribe(ctx, clientID, q)
		if err != nil {
			b.Fatal(err)
		}
		go func() {
			for {
				select {
				case <-subscription.Out():
					continue
				case <-subscription.Canceled():
					return
				}
			}
		}()
	}

	b.ReportAllocs()
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		err = s.PublishWithEvents(ctx, "Gamora", map[string][]string{
			"abci.Account.Owner":   {"Ivan"},
			"abci.Invoices.Number": {"1"},
		})
		require.NoError(b, err)
	}
}

// HELPERS

func assertReceive(t *testing.T, expected any, ch <-chan pubsub.Message) {
	t.Helper()
	select {
	case actual := <-ch:
		assert.Equal(t, expected, actual.Data())
	case <-time.After(1 * time.Second):
		t.Errorf("expected to receive %v from the channel, got nothing after 1s", expected)
		debug.PrintStack()
	}
}

func assertCancelled(t *testing.T, subscription *pubsub.Subscription, err error) {
	t.Helper()
	_, ok := <-subscription.Canceled()
	assert.False(t, ok)
	assert.Equal(t, err, subscription.Err())
}
