// Package pubsub implements a pub-sub model with a single publisher (Server)
// and multiple subscribers (clients).
//
// Though you can have multiple publishers by sharing a pointer to a server or
// by giving the same channel to each publisher and publishing messages from
// that channel (fan-in).
//
// Clients subscribe for messages, which could be of any type, using a query.
// When some message is published, we match it with all queries. If there is a
// match, this message will be pushed to all clients, subscribed to that query.
// See query subpackage for our implementation.
//
// Example:
//
//	q, err := query.New("account.name='John'")
//	if err != nil {
//	    return err
//	}
//	ctx, cancel := context.WithTimeout(context.Background(), 1 * time.Second)
//	defer cancel()
//	subscription, err := pubsub.Subscribe(ctx, "johns-transactions", q)
//	if err != nil {
//	    return err
//	}
//
//	for {
//	    select {
//	    case msg <- subscription.Out():
//	        // handle msg.Data() and msg.Events()
//	    case <-subscription.Canceled():
//	        return subscription.Err()
//	    }
//	}
package pubsub

import (
	"context"
	"errors"
	"fmt"

	"github.com/cometbft/cometbft/libs/service"
	cmtsync "github.com/cometbft/cometbft/libs/sync"
)

type operation int

const (
	sub operation = iota
	pub
	unsub
	shutdown
)

var (
	// ErrSubscriptionNotFound is returned when a client tries to unsubscribe
	// from not existing subscription.
	ErrSubscriptionNotFound = errors.New("subscription not found")

	// ErrAlreadySubscribed is returned when a client tries to subscribe twice or
	// more using the same query.
	ErrAlreadySubscribed = errors.New("already subscribed")
)

// Query defines an interface for a query to be used for subscribing. A query
// matches against a map of events. Each key in this map is a composite of the
// even type and an attribute key (e.g. "{eventType}.{eventAttrKey}") and the
// values are the event values that are contained under that relationship. This
// allows event types to repeat themselves with the same set of keys and
// different values.
type Query interface {
	Matches(events map[string][]string) (bool, error)
	String() string
}

type cmd struct {
	op operation

	// subscribe, unsubscribe
	query        Query
	subscription *Subscription
	clientID     string

	// publish
	msg    interface{}
	events map[string][]string
}

// Server allows clients to subscribe/unsubscribe for messages, publishing
// messages with or without events, and manages internal state.
type Server struct {
	service.BaseService

	cmds    chan cmd
	cmdsCap int

	// check if we have subscription before
	// subscribing or unsubscribing
	mtx           cmtsync.RWMutex
	subscriptions map[string]map[string]struct{} // subscriber -> query (string) -> empty struct
}

// Option sets a parameter for the server.
type Option func(*Server)

// NewServer returns a new server. See the commentary on the Option functions
// for a detailed description of how to configure buffering. If no options are
// provided, the resulting server's queue is unbuffered.
func NewServer(options ...Option) *Server {
	s := &Server{
		subscriptions: make(map[string]map[string]struct{}),
	}
	s.BaseService = *service.NewBaseService(nil, "PubSub", s)

	for _, option := range options {
		option(s)
	}

	// if BufferCapacity option was not set, the channel is unbuffered
	s.cmds = make(chan cmd, s.cmdsCap)

	return s
}

// BufferCapacity allows you to specify capacity for the internal server's
// queue. Since the server, given Y subscribers, could only process X messages,
// this option could be used to survive spikes (e.g. high amount of
// transactions during peak hours).
func BufferCapacity(cap int) Option {
	return func(s *Server) {
		if cap > 0 {
			s.cmdsCap = cap
		}
	}
}

// BufferCapacity returns capacity of the internal server's queue.
func (s *Server) BufferCapacity() int {
	return s.cmdsCap
}

// Subscribe creates a subscription for the given client.
//
// An error will be returned to the caller if the context is canceled or if
// subscription already exist for pair clientID and query.
//
// outCapacity can be used to set a capacity for Subscription#Out channel (1 by
// default). Panics if outCapacity is less than or equal to zero. If you want
// an unbuffered channel, use SubscribeUnbuffered.
func (s *Server) Subscribe(
	ctx context.Context,
	clientID string,
	query Query,
	outCapacity ...int) (*Subscription, error) {
	outCap := 1
	if len(outCapacity) > 0 {
		if outCapacity[0] <= 0 {
			panic("Negative or zero capacity. Use SubscribeUnbuffered if you want an unbuffered channel")
		}
		outCap = outCapacity[0]
	}

	return s.subscribe(ctx, clientID, query, outCap)
}

// SubscribeUnbuffered does the same as Subscribe, except it returns a
// subscription with unbuffered channel. Use with caution as it can freeze the
// server.
func (s *Server) SubscribeUnbuffered(ctx context.Context, clientID string, query Query) (*Subscription, error) {
	return s.subscribe(ctx, clientID, query, 0)
}

func (s *Server) subscribe(ctx context.Context, clientID string, query Query, outCapacity int) (*Subscription, error) {
	s.mtx.RLock()
	clientSubscriptions, ok := s.subscriptions[clientID]
	if ok {
		_, ok = clientSubscriptions[query.String()]
	}
	s.mtx.RUnlock()
	if ok {
		return nil, ErrAlreadySubscribed
	}

	subscription := NewSubscription(outCapacity)
	select {
	case s.cmds <- cmd{op: sub, clientID: clientID, query: query, subscription: subscription}:
		s.mtx.Lock()
		if _, ok = s.subscriptions[clientID]; !ok {
			s.subscriptions[clientID] = make(map[string]struct{})
		}
		s.subscriptions[clientID][query.String()] = struct{}{}
		s.mtx.Unlock()
		return subscription, nil
	case <-ctx.Done():
		return nil, ctx.Err()
	case <-s.Quit():
		return nil, errors.New("service is shutting down")
	}
}

// Unsubscribe removes the subscription on the given query. An error will be
// returned to the caller if the context is canceled or if subscription does
// not exist.
func (s *Server) Unsubscribe(ctx context.Context, clientID string, query Query) error {
	s.mtx.RLock()
	clientSubscriptions, ok := s.subscriptions[clientID]
	if ok {
		_, ok = clientSubscriptions[query.String()]
	}
	s.mtx.RUnlock()
	if !ok {
		return ErrSubscriptionNotFound
	}

	select {
	case s.cmds <- cmd{op: unsub, clientID: clientID, query: query}:
		s.mtx.Lock()
		delete(clientSubscriptions, query.String())
		if len(clientSubscriptions) == 0 {
			delete(s.subscriptions, clientID)
		}
		s.mtx.Unlock()
		return nil
	case <-ctx.Done():
		return ctx.Err()
	case <-s.Quit():
		return nil
	}
}

// UnsubscribeAll removes all client subscriptions. An error will be returned
// to the caller if the context is canceled or if subscription does not exist.
func (s *Server) UnsubscribeAll(ctx context.Context, clientID string) error {
	s.mtx.RLock()
	_, ok := s.subscriptions[clientID]
	s.mtx.RUnlock()
	if !ok {
		return ErrSubscriptionNotFound
	}

	select {
	case s.cmds <- cmd{op: unsub, clientID: clientID}:
		s.mtx.Lock()
		delete(s.subscriptions, clientID)
		s.mtx.Unlock()
		return nil
	case <-ctx.Done():
		return ctx.Err()
	case <-s.Quit():
		return nil
	}
}

// NumClients returns the number of clients.
func (s *Server) NumClients() int {
	s.mtx.RLock()
	defer s.mtx.RUnlock()
	return len(s.subscriptions)
}

// NumClientSubscriptions returns the number of subscriptions the client has.
func (s *Server) NumClientSubscriptions(clientID string) int {
	s.mtx.RLock()
	defer s.mtx.RUnlock()
	return len(s.subscriptions[clientID])
}

// Publish publishes the given message. An error will be returned to the caller
// if the context is canceled.
func (s *Server) Publish(ctx context.Context, msg interface{}) error {
	return s.PublishWithEvents(ctx, msg, make(map[string][]string))
}

// PublishWithEvents publishes the given message with the set of events. The set
// is matched with clients queries. If there is a match, the message is sent to
// the client.
func (s *Server) PublishWithEvents(ctx context.Context, msg interface{}, events map[string][]string) error {
	select {
	case s.cmds <- cmd{op: pub, msg: msg, events: events}:
		return nil
	case <-ctx.Done():
		return ctx.Err()
	case <-s.Quit():
		return nil
	}
}

// OnStop implements Service.OnStop by shutting down the server.
func (s *Server) OnStop() {
	s.cmds <- cmd{op: shutdown}
}

// NOTE: not goroutine safe
type state struct {
	// query string -> client -> subscription
	subscriptions map[string]map[string]*Subscription
	// query string -> queryPlusRefCount
	queries map[string]*queryPlusRefCount
}

// queryPlusRefCount holds a pointer to a query and reference counter. When
// refCount is zero, query will be removed.
type queryPlusRefCount struct {
	q        Query
	refCount int
}

// OnStart implements Service.OnStart by starting the server.
func (s *Server) OnStart() error {
	go s.loop(state{
		subscriptions: make(map[string]map[string]*Subscription),
		queries:       make(map[string]*queryPlusRefCount),
	})
	return nil
}

// OnReset implements Service.OnReset
func (s *Server) OnReset() error {
	return nil
}

func (s *Server) loop(state state) {
loop:
	for cmd := range s.cmds {
		switch cmd.op {
		case unsub:
			if cmd.query != nil {
				state.remove(cmd.clientID, cmd.query.String(), ErrUnsubscribed)
			} else {
				state.removeClient(cmd.clientID, ErrUnsubscribed)
			}
		case shutdown:
			state.removeAll(nil)
			break loop
		case sub:
			state.add(cmd.clientID, cmd.query, cmd.subscription)
		case pub:
			if err := state.send(cmd.msg, cmd.events); err != nil {
				s.Logger.Error("Error querying for events", "err", err)
			}
		}
	}
}

func (state *state) add(clientID string, q Query, subscription *Subscription) {
	qStr := q.String()

	// initialize subscription for this client per query if needed
	if _, ok := state.subscriptions[qStr]; !ok {
		state.subscriptions[qStr] = make(map[string]*Subscription)
	}
	// create subscription
	state.subscriptions[qStr][clientID] = subscription

	// initialize query if needed
	if _, ok := state.queries[qStr]; !ok {
		state.queries[qStr] = &queryPlusRefCount{q: q, refCount: 0}
	}
	// increment reference counter
	state.queries[qStr].refCount++
}

func (state *state) remove(clientID string, qStr string, reason error) {
	clientSubscriptions, ok := state.subscriptions[qStr]
	if !ok {
		return
	}

	subscription, ok := clientSubscriptions[clientID]
	if !ok {
		return
	}

	subscription.cancel(reason)

	// remove client from query map.
	// if query has no other clients subscribed, remove it.
	delete(state.subscriptions[qStr], clientID)
	if len(state.subscriptions[qStr]) == 0 {
		delete(state.subscriptions, qStr)
	}

	// decrease ref counter in queries
	state.queries[qStr].refCount--
	// remove the query if nobody else is using it
	if state.queries[qStr].refCount == 0 {
		delete(state.queries, qStr)
	}
}

func (state *state) removeClient(clientID string, reason error) {
	for qStr, clientSubscriptions := range state.subscriptions {
		if _, ok := clientSubscriptions[clientID]; ok {
			state.remove(clientID, qStr, reason)
		}
	}
}

func (state *state) removeAll(reason error) {
	for qStr, clientSubscriptions := range state.subscriptions {
		for clientID := range clientSubscriptions {
			state.remove(clientID, qStr, reason)
		}
	}
}

func (state *state) send(msg interface{}, events map[string][]string) error {
	for qStr, clientSubscriptions := range state.subscriptions {
		q := state.queries[qStr].q

		match, err := q.Matches(events)
		if err != nil {
			return fmt.Errorf("failed to match against query %s: %w", q.String(), err)
		}

		if match {
			for clientID, subscription := range clientSubscriptions {
				if cap(subscription.out) == 0 {
					// block on unbuffered channel
					subscription.out <- NewMessage(msg, events)
				} else {
					// don't block on buffered channels
					select {
					case subscription.out <- NewMessage(msg, events):
					default:
						state.remove(clientID, qStr, ErrOutOfCapacity)
					}
				}
			}
		}
	}

	return nil
}
