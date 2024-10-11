// Copyright 2016 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package event

import (
	"context"
	"errors"
	"runtime"
	"testing"
	"time"

	"github.com/prysmaticlabs/prysm/v4/testing/require"
)

var errInts = errors.New("error in subscribeInts")

func subscribeInts(max, fail int, c chan<- int) Subscription {
	return NewSubscription(func(quit <-chan struct{}) error {
		for i := 0; i < max; i++ {
			if i >= fail {
				return errInts
			}
			select {
			case c <- i:
			case <-quit:
				return nil
			}
		}
		return nil
	})
}

func TestNewSubscriptionError(t *testing.T) {
	t.Parallel()

	channel := make(chan int)
	sub := subscribeInts(10, 2, channel)
loop:
	for want := 0; want < 10; want++ {
		select {
		case got := <-channel:
			require.Equal(t, want, got)
		case err := <-sub.Err():
			require.Equal(t, errInts, err)
			require.Equal(t, 2, want)
			break loop
		}
	}
	sub.Unsubscribe()

	err, ok := <-sub.Err()
	require.NoError(t, err)
	if ok {
		t.Fatal("channel still open after Unsubscribe")
	}
}

func TestResubscribe(t *testing.T) {
	t.Parallel()

	var i int
	nfails := 6
	sub := Resubscribe(100*time.Millisecond, func(ctx context.Context) (Subscription, error) {
		// fmt.Printf("call #%d @ %v\n", i, time.Now())
		i++
		if i == 2 {
			// Delay the second failure a bit to reset the resubscribe interval.
			time.Sleep(200 * time.Millisecond)
		}
		if i < nfails {
			return nil, errors.New("oops")
		}
		sub := NewSubscription(func(unsubscribed <-chan struct{}) error { return nil })
		return sub, nil
	})

	<-sub.Err()
	require.Equal(t, nfails, i)
}

func TestResubscribeAbort(t *testing.T) {
	t.Parallel()

	done := make(chan error)
	sub := Resubscribe(0, func(ctx context.Context) (Subscription, error) {
		select {
		case <-ctx.Done():
			done <- nil
		case <-time.After(2 * time.Second):
			done <- errors.New("context given to resubscribe function not canceled within 2s")
		}
		return nil, nil
	})

	sub.Unsubscribe()
	require.NoError(t, <-done)
}

func TestResubscribeNonBlocking(t *testing.T) {
	done := make(chan struct{})
	sub := Resubscribe(0, func(ctx context.Context) (Subscription, error) {
		<-done
		return nil, nil
	})

	resub, ok := sub.(*resubscribeSub)
	require.Equal(t, true, ok)
	currNum := runtime.NumGoroutine()
	resub.unsub <- struct{}{}
	done <- struct{}{}
	require.Equal(t, currNum-1, runtime.NumGoroutine())
}
