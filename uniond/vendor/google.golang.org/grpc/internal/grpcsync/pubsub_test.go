/*
 *
 * Copyright 2023 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package grpcsync

import (
	"context"
	"sync"
	"testing"
	"time"
)

type testSubscriber struct {
	onMsgCh chan int
}

func newTestSubscriber(chSize int) *testSubscriber {
	return &testSubscriber{onMsgCh: make(chan int, chSize)}
}

func (ts *testSubscriber) OnMessage(msg any) {
	select {
	case ts.onMsgCh <- msg.(int):
	default:
	}
}

func (s) TestPubSub_PublishNoMsg(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	pubsub := NewPubSub(ctx)

	ts := newTestSubscriber(1)
	pubsub.Subscribe(ts)

	select {
	case <-ts.onMsgCh:
		t.Fatal("Subscriber callback invoked when no message was published")
	case <-time.After(defaultTestShortTimeout):
	}
}

func (s) TestPubSub_PublishMsgs_RegisterSubs_And_Stop(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	pubsub := NewPubSub(ctx)

	const numPublished = 10

	ts1 := newTestSubscriber(numPublished)
	pubsub.Subscribe(ts1)

	var wg sync.WaitGroup
	wg.Add(2)
	// Publish ten messages on the pubsub and ensure that they are received in order by the subscriber.
	go func() {
		for i := 0; i < numPublished; i++ {
			pubsub.Publish(i)
		}
		wg.Done()
	}()

	go func() {
		defer wg.Done()
		for i := 0; i < numPublished; i++ {
			select {
			case m := <-ts1.onMsgCh:
				if m != i {
					t.Errorf("Received unexpected message: %q; want: %q", m, i)
					return
				}
			case <-time.After(defaultTestTimeout):
				t.Error("Timeout when expecting the onMessage() callback to be invoked")
				return
			}
		}
	}()
	wg.Wait()
	if t.Failed() {
		t.FailNow()
	}

	// Register another subscriber and ensure that it receives the last published message.
	ts2 := newTestSubscriber(numPublished)
	pubsub.Subscribe(ts2)

	select {
	case m := <-ts2.onMsgCh:
		if m != numPublished-1 {
			t.Fatalf("Received unexpected message: %q; want: %q", m, numPublished-1)
		}
	case <-time.After(defaultTestShortTimeout):
		t.Fatal("Timeout when expecting the onMessage() callback to be invoked")
	}

	wg.Add(3)
	// Publish ten messages on the pubsub and ensure that they are received in order by the subscribers.
	go func() {
		for i := 0; i < numPublished; i++ {
			pubsub.Publish(i)
		}
		wg.Done()
	}()
	go func() {
		defer wg.Done()
		for i := 0; i < numPublished; i++ {
			select {
			case m := <-ts1.onMsgCh:
				if m != i {
					t.Errorf("Received unexpected message: %q; want: %q", m, i)
					return
				}
			case <-time.After(defaultTestTimeout):
				t.Error("Timeout when expecting the onMessage() callback to be invoked")
				return
			}
		}

	}()
	go func() {
		defer wg.Done()
		for i := 0; i < numPublished; i++ {
			select {
			case m := <-ts2.onMsgCh:
				if m != i {
					t.Errorf("Received unexpected message: %q; want: %q", m, i)
					return
				}
			case <-time.After(defaultTestTimeout):
				t.Error("Timeout when expecting the onMessage() callback to be invoked")
				return
			}
		}
	}()
	wg.Wait()
	if t.Failed() {
		t.FailNow()
	}

	cancel()
	<-pubsub.Done()

	go func() {
		pubsub.Publish(99)
	}()
	// Ensure that the subscriber callback is not invoked as instantiated
	// pubsub has already closed.
	select {
	case <-ts1.onMsgCh:
		t.Fatal("The callback was invoked after pubsub being stopped")
	case <-ts2.onMsgCh:
		t.Fatal("The callback was invoked after pubsub being stopped")
	case <-time.After(defaultTestShortTimeout):
	}
}

func (s) TestPubSub_PublishMsgs_BeforeRegisterSub(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	pubsub := NewPubSub(ctx)

	const numPublished = 3
	for i := 0; i < numPublished; i++ {
		pubsub.Publish(i)
	}

	ts := newTestSubscriber(numPublished)
	pubsub.Subscribe(ts)

	// Ensure that the subscriber callback is invoked with a previously
	// published message.
	select {
	case d := <-ts.onMsgCh:
		if d != numPublished-1 {
			t.Fatalf("Unexpected message received: %q; %q", d, numPublished-1)
		}

	case <-time.After(defaultTestShortTimeout):
		t.Fatal("Timeout when expecting the onMessage() callback to be invoked")
	}
}
