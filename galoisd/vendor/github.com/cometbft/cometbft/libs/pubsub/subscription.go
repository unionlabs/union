package pubsub

import (
	"errors"

	cmtsync "github.com/cometbft/cometbft/libs/sync"
)

var (
	// ErrUnsubscribed is returned by Err when a client unsubscribes.
	ErrUnsubscribed = errors.New("client unsubscribed")

	// ErrOutOfCapacity is returned by Err when a client is not pulling messages
	// fast enough. Note the client's subscription will be terminated.
	ErrOutOfCapacity = errors.New("internal subscription event buffer is out of capacity")
)

// A Subscription represents a client subscription for a particular query and
// consists of three things:
// 1) channel onto which messages and events are published
// 2) channel which is closed if a client is too slow or choose to unsubscribe
// 3) err indicating the reason for (2)
type Subscription struct {
	out chan Message

	canceled chan struct{}
	mtx      cmtsync.RWMutex
	err      error
}

// NewSubscription returns a new subscription with the given outCapacity.
func NewSubscription(outCapacity int) *Subscription {
	return &Subscription{
		out:      make(chan Message, outCapacity),
		canceled: make(chan struct{}),
	}
}

// Out returns a channel onto which messages and events are published.
// Unsubscribe/UnsubscribeAll does not close the channel to avoid clients from
// receiving a nil message.
func (s *Subscription) Out() <-chan Message {
	return s.out
}

// Canceled returns a channel that's closed when the subscription is
// terminated and supposed to be used in a select statement.
func (s *Subscription) Canceled() <-chan struct{} {
	return s.canceled
}

// Err returns nil if the channel returned is not yet closed.
// If the channel is closed, Err returns a non-nil error explaining why:
//   - ErrUnsubscribed if the subscriber choose to unsubscribe,
//   - ErrOutOfCapacity if the subscriber is not pulling messages fast enough
//     and the channel returned by Out became full,
//
// After Err returns a non-nil error, successive calls to Err return the same
// error.
func (s *Subscription) Err() error {
	s.mtx.RLock()
	defer s.mtx.RUnlock()
	return s.err
}

func (s *Subscription) cancel(err error) {
	s.mtx.Lock()
	s.err = err
	s.mtx.Unlock()
	close(s.canceled)
}

// Message glues data and events together.
type Message struct {
	data   interface{}
	events map[string][]string
}

func NewMessage(data interface{}, events map[string][]string) Message {
	return Message{data, events}
}

// Data returns an original data published.
func (msg Message) Data() interface{} {
	return msg.data
}

// Events returns events, which matched the client's query.
func (msg Message) Events() map[string][]string {
	return msg.events
}
