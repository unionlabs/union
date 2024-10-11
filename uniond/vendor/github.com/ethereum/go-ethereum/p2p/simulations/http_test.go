// Copyright 2017 The go-ethereum Authors
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

package simulations

import (
	"context"
	"flag"
	"fmt"
	"math/rand"
	"net/http/httptest"
	"os"
	"reflect"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/event"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/node"
	"github.com/ethereum/go-ethereum/p2p"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/simulations/adapters"
	"github.com/ethereum/go-ethereum/rpc"
	"github.com/mattn/go-colorable"
)

func TestMain(m *testing.M) {
	loglevel := flag.Int("loglevel", 2, "verbosity of logs")

	flag.Parse()
	log.PrintOrigins(true)
	log.Root().SetHandler(log.LvlFilterHandler(log.Lvl(*loglevel), log.StreamHandler(colorable.NewColorableStderr(), log.TerminalFormat(true))))
	os.Exit(m.Run())
}

// testService implements the node.Service interface and provides protocols
// and APIs which are useful for testing nodes in a simulation network
type testService struct {
	id enode.ID

	// peerCount is incremented once a peer handshake has been performed
	peerCount int64

	peers    map[enode.ID]*testPeer
	peersMtx sync.Mutex

	// state stores []byte which is used to test creating and loading
	// snapshots
	state atomic.Value
}

func newTestService(ctx *adapters.ServiceContext, stack *node.Node) (node.Lifecycle, error) {
	svc := &testService{
		id:    ctx.Config.ID,
		peers: make(map[enode.ID]*testPeer),
	}
	svc.state.Store(ctx.Snapshot)

	stack.RegisterProtocols(svc.Protocols())
	stack.RegisterAPIs(svc.APIs())
	return svc, nil
}

type testPeer struct {
	testReady chan struct{}
	dumReady  chan struct{}
}

func (t *testService) peer(id enode.ID) *testPeer {
	t.peersMtx.Lock()
	defer t.peersMtx.Unlock()
	if peer, ok := t.peers[id]; ok {
		return peer
	}
	peer := &testPeer{
		testReady: make(chan struct{}),
		dumReady:  make(chan struct{}),
	}
	t.peers[id] = peer
	return peer
}

func (t *testService) Protocols() []p2p.Protocol {
	return []p2p.Protocol{
		{
			Name:    "test",
			Version: 1,
			Length:  3,
			Run:     t.RunTest,
		},
		{
			Name:    "dum",
			Version: 1,
			Length:  1,
			Run:     t.RunDum,
		},
		{
			Name:    "prb",
			Version: 1,
			Length:  1,
			Run:     t.RunPrb,
		},
	}
}

func (t *testService) APIs() []rpc.API {
	return []rpc.API{{
		Namespace: "test",
		Version:   "1.0",
		Service: &TestAPI{
			state:     &t.state,
			peerCount: &t.peerCount,
		},
	}}
}

func (t *testService) Start() error {
	return nil
}

func (t *testService) Stop() error {
	return nil
}

// handshake performs a peer handshake by sending and expecting an empty
// message with the given code
func (t *testService) handshake(rw p2p.MsgReadWriter, code uint64) error {
	errc := make(chan error, 2)
	go func() { errc <- p2p.SendItems(rw, code) }()
	go func() { errc <- p2p.ExpectMsg(rw, code, struct{}{}) }()
	for i := 0; i < 2; i++ {
		if err := <-errc; err != nil {
			return err
		}
	}
	return nil
}

func (t *testService) RunTest(p *p2p.Peer, rw p2p.MsgReadWriter) error {
	peer := t.peer(p.ID())

	// perform three handshakes with three different message codes,
	// used to test message sending and filtering
	if err := t.handshake(rw, 2); err != nil {
		return err
	}
	if err := t.handshake(rw, 1); err != nil {
		return err
	}
	if err := t.handshake(rw, 0); err != nil {
		return err
	}

	// close the testReady channel so that other protocols can run
	close(peer.testReady)

	// track the peer
	atomic.AddInt64(&t.peerCount, 1)
	defer atomic.AddInt64(&t.peerCount, -1)

	// block until the peer is dropped
	for {
		_, err := rw.ReadMsg()
		if err != nil {
			return err
		}
	}
}

func (t *testService) RunDum(p *p2p.Peer, rw p2p.MsgReadWriter) error {
	peer := t.peer(p.ID())

	// wait for the test protocol to perform its handshake
	<-peer.testReady

	// perform a handshake
	if err := t.handshake(rw, 0); err != nil {
		return err
	}

	// close the dumReady channel so that other protocols can run
	close(peer.dumReady)

	// block until the peer is dropped
	for {
		_, err := rw.ReadMsg()
		if err != nil {
			return err
		}
	}
}
func (t *testService) RunPrb(p *p2p.Peer, rw p2p.MsgReadWriter) error {
	peer := t.peer(p.ID())

	// wait for the dum protocol to perform its handshake
	<-peer.dumReady

	// perform a handshake
	if err := t.handshake(rw, 0); err != nil {
		return err
	}

	// block until the peer is dropped
	for {
		_, err := rw.ReadMsg()
		if err != nil {
			return err
		}
	}
}

func (t *testService) Snapshot() ([]byte, error) {
	return t.state.Load().([]byte), nil
}

// TestAPI provides a test API to:
// * get the peer count
// * get and set an arbitrary state byte slice
// * get and increment a counter
// * subscribe to counter increment events
type TestAPI struct {
	state     *atomic.Value
	peerCount *int64
	counter   int64
	feed      event.Feed
}

func (t *TestAPI) PeerCount() int64 {
	return atomic.LoadInt64(t.peerCount)
}

func (t *TestAPI) Get() int64 {
	return atomic.LoadInt64(&t.counter)
}

func (t *TestAPI) Add(delta int64) {
	atomic.AddInt64(&t.counter, delta)
	t.feed.Send(delta)
}

func (t *TestAPI) GetState() []byte {
	return t.state.Load().([]byte)
}

func (t *TestAPI) SetState(state []byte) {
	t.state.Store(state)
}

func (t *TestAPI) Events(ctx context.Context) (*rpc.Subscription, error) {
	notifier, supported := rpc.NotifierFromContext(ctx)
	if !supported {
		return nil, rpc.ErrNotificationsUnsupported
	}

	rpcSub := notifier.CreateSubscription()

	go func() {
		events := make(chan int64)
		sub := t.feed.Subscribe(events)
		defer sub.Unsubscribe()

		for {
			select {
			case event := <-events:
				notifier.Notify(rpcSub.ID, event)
			case <-sub.Err():
				return
			case <-rpcSub.Err():
				return
			case <-notifier.Closed():
				return
			}
		}
	}()

	return rpcSub, nil
}

var testServices = adapters.LifecycleConstructors{
	"test": newTestService,
}

func testHTTPServer(t *testing.T) (*Network, *httptest.Server) {
	t.Helper()
	adapter := adapters.NewSimAdapter(testServices)
	network := NewNetwork(adapter, &NetworkConfig{
		DefaultService: "test",
	})
	return network, httptest.NewServer(NewServer(network))
}

// TestHTTPNetwork tests interacting with a simulation network using the HTTP
// API
func TestHTTPNetwork(t *testing.T) {
	// start the server
	network, s := testHTTPServer(t)
	defer s.Close()

	// subscribe to events so we can check them later
	client := NewClient(s.URL)
	events := make(chan *Event, 100)
	var opts SubscribeOpts
	sub, err := client.SubscribeNetwork(events, opts)
	if err != nil {
		t.Fatalf("error subscribing to network events: %s", err)
	}
	defer sub.Unsubscribe()

	// check we can retrieve details about the network
	gotNetwork, err := client.GetNetwork()
	if err != nil {
		t.Fatalf("error getting network: %s", err)
	}
	if gotNetwork.ID != network.ID {
		t.Fatalf("expected network to have ID %q, got %q", network.ID, gotNetwork.ID)
	}

	// start a simulation network
	nodeIDs := startTestNetwork(t, client)

	// check we got all the events
	x := &expectEvents{t, events, sub}
	x.expect(
		x.nodeEvent(nodeIDs[0], false),
		x.nodeEvent(nodeIDs[1], false),
		x.nodeEvent(nodeIDs[0], true),
		x.nodeEvent(nodeIDs[1], true),
		x.connEvent(nodeIDs[0], nodeIDs[1], false),
		x.connEvent(nodeIDs[0], nodeIDs[1], true),
	)

	// reconnect the stream and check we get the current nodes and conns
	events = make(chan *Event, 100)
	opts.Current = true
	sub, err = client.SubscribeNetwork(events, opts)
	if err != nil {
		t.Fatalf("error subscribing to network events: %s", err)
	}
	defer sub.Unsubscribe()
	x = &expectEvents{t, events, sub}
	x.expect(
		x.nodeEvent(nodeIDs[0], true),
		x.nodeEvent(nodeIDs[1], true),
		x.connEvent(nodeIDs[0], nodeIDs[1], true),
	)
}

func startTestNetwork(t *testing.T, client *Client) []string {
	// create two nodes
	nodeCount := 2
	nodeIDs := make([]string, nodeCount)
	for i := 0; i < nodeCount; i++ {
		config := adapters.RandomNodeConfig()
		node, err := client.CreateNode(config)
		if err != nil {
			t.Fatalf("error creating node: %s", err)
		}
		nodeIDs[i] = node.ID
	}

	// check both nodes exist
	nodes, err := client.GetNodes()
	if err != nil {
		t.Fatalf("error getting nodes: %s", err)
	}
	if len(nodes) != nodeCount {
		t.Fatalf("expected %d nodes, got %d", nodeCount, len(nodes))
	}
	for i, nodeID := range nodeIDs {
		if nodes[i].ID != nodeID {
			t.Fatalf("expected node %d to have ID %q, got %q", i, nodeID, nodes[i].ID)
		}
		node, err := client.GetNode(nodeID)
		if err != nil {
			t.Fatalf("error getting node %d: %s", i, err)
		}
		if node.ID != nodeID {
			t.Fatalf("expected node %d to have ID %q, got %q", i, nodeID, node.ID)
		}
	}

	// start both nodes
	for _, nodeID := range nodeIDs {
		if err := client.StartNode(nodeID); err != nil {
			t.Fatalf("error starting node %q: %s", nodeID, err)
		}
	}

	// connect the nodes
	for i := 0; i < nodeCount-1; i++ {
		peerId := i + 1
		if i == nodeCount-1 {
			peerId = 0
		}
		if err := client.ConnectNode(nodeIDs[i], nodeIDs[peerId]); err != nil {
			t.Fatalf("error connecting nodes: %s", err)
		}
	}

	return nodeIDs
}

type expectEvents struct {
	*testing.T

	events chan *Event
	sub    event.Subscription
}

func (t *expectEvents) nodeEvent(id string, up bool) *Event {
	config := &adapters.NodeConfig{ID: enode.HexID(id)}
	return &Event{Type: EventTypeNode, Node: newNode(nil, config, up)}
}

func (t *expectEvents) connEvent(one, other string, up bool) *Event {
	return &Event{
		Type: EventTypeConn,
		Conn: &Conn{
			One:   enode.HexID(one),
			Other: enode.HexID(other),
			Up:    up,
		},
	}
}

func (t *expectEvents) expectMsgs(expected map[MsgFilter]int) {
	actual := make(map[MsgFilter]int)
	timeout := time.After(10 * time.Second)
loop:
	for {
		select {
		case event := <-t.events:
			t.Logf("received %s event: %v", event.Type, event)

			if event.Type != EventTypeMsg || event.Msg.Received {
				continue loop
			}
			if event.Msg == nil {
				t.Fatal("expected event.Msg to be set")
			}
			filter := MsgFilter{
				Proto: event.Msg.Protocol,
				Code:  int64(event.Msg.Code),
			}
			actual[filter]++
			if actual[filter] > expected[filter] {
				t.Fatalf("received too many msgs for filter: %v", filter)
			}
			if reflect.DeepEqual(actual, expected) {
				return
			}

		case err := <-t.sub.Err():
			t.Fatalf("network stream closed unexpectedly: %s", err)

		case <-timeout:
			t.Fatal("timed out waiting for expected events")
		}
	}
}

func (t *expectEvents) expect(events ...*Event) {
	t.Helper()
	timeout := time.After(10 * time.Second)
	i := 0
	for {
		select {
		case event := <-t.events:
			t.Logf("received %s event: %v", event.Type, event)

			expected := events[i]
			if event.Type != expected.Type {
				t.Fatalf("expected event %d to have type %q, got %q", i, expected.Type, event.Type)
			}

			switch expected.Type {
			case EventTypeNode:
				if event.Node == nil {
					t.Fatal("expected event.Node to be set")
				}
				if event.Node.ID() != expected.Node.ID() {
					t.Fatalf("expected node event %d to have id %q, got %q", i, expected.Node.ID().TerminalString(), event.Node.ID().TerminalString())
				}
				if event.Node.Up() != expected.Node.Up() {
					t.Fatalf("expected node event %d to have up=%t, got up=%t", i, expected.Node.Up(), event.Node.Up())
				}

			case EventTypeConn:
				if event.Conn == nil {
					t.Fatal("expected event.Conn to be set")
				}
				if event.Conn.One != expected.Conn.One {
					t.Fatalf("expected conn event %d to have one=%q, got one=%q", i, expected.Conn.One.TerminalString(), event.Conn.One.TerminalString())
				}
				if event.Conn.Other != expected.Conn.Other {
					t.Fatalf("expected conn event %d to have other=%q, got other=%q", i, expected.Conn.Other.TerminalString(), event.Conn.Other.TerminalString())
				}
				if event.Conn.Up != expected.Conn.Up {
					t.Fatalf("expected conn event %d to have up=%t, got up=%t", i, expected.Conn.Up, event.Conn.Up)
				}
			}

			i++
			if i == len(events) {
				return
			}

		case err := <-t.sub.Err():
			t.Fatalf("network stream closed unexpectedly: %s", err)

		case <-timeout:
			t.Fatal("timed out waiting for expected events")
		}
	}
}

// TestHTTPNodeRPC tests calling RPC methods on nodes via the HTTP API
func TestHTTPNodeRPC(t *testing.T) {
	// start the server
	_, s := testHTTPServer(t)
	defer s.Close()

	// start a node in the network
	client := NewClient(s.URL)

	config := adapters.RandomNodeConfig()
	node, err := client.CreateNode(config)
	if err != nil {
		t.Fatalf("error creating node: %s", err)
	}
	if err := client.StartNode(node.ID); err != nil {
		t.Fatalf("error starting node: %s", err)
	}

	// create two RPC clients
	ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
	defer cancel()
	rpcClient1, err := client.RPCClient(ctx, node.ID)
	if err != nil {
		t.Fatalf("error getting node RPC client: %s", err)
	}
	rpcClient2, err := client.RPCClient(ctx, node.ID)
	if err != nil {
		t.Fatalf("error getting node RPC client: %s", err)
	}

	// subscribe to events using client 1
	events := make(chan int64, 1)
	sub, err := rpcClient1.Subscribe(ctx, "test", events, "events")
	if err != nil {
		t.Fatalf("error subscribing to events: %s", err)
	}
	defer sub.Unsubscribe()

	// call some RPC methods using client 2
	if err := rpcClient2.CallContext(ctx, nil, "test_add", 10); err != nil {
		t.Fatalf("error calling RPC method: %s", err)
	}
	var result int64
	if err := rpcClient2.CallContext(ctx, &result, "test_get"); err != nil {
		t.Fatalf("error calling RPC method: %s", err)
	}
	if result != 10 {
		t.Fatalf("expected result to be 10, got %d", result)
	}

	// check we got an event from client 1
	select {
	case event := <-events:
		if event != 10 {
			t.Fatalf("expected event to be 10, got %d", event)
		}
	case <-ctx.Done():
		t.Fatal(ctx.Err())
	}
}

// TestHTTPSnapshot tests creating and loading network snapshots
func TestHTTPSnapshot(t *testing.T) {
	// start the server
	network, s := testHTTPServer(t)
	defer s.Close()

	var eventsDone = make(chan struct{}, 1)
	count := 1
	eventsDoneChan := make(chan *Event)
	eventSub := network.Events().Subscribe(eventsDoneChan)
	go func() {
		defer eventSub.Unsubscribe()
		for event := range eventsDoneChan {
			if event.Type == EventTypeConn && !event.Control {
				count--
				if count == 0 {
					eventsDone <- struct{}{}
					return
				}
			}
		}
	}()

	// create a two-node network
	client := NewClient(s.URL)
	nodeCount := 2
	nodes := make([]*p2p.NodeInfo, nodeCount)
	for i := 0; i < nodeCount; i++ {
		config := adapters.RandomNodeConfig()
		node, err := client.CreateNode(config)
		if err != nil {
			t.Fatalf("error creating node: %s", err)
		}
		if err := client.StartNode(node.ID); err != nil {
			t.Fatalf("error starting node: %s", err)
		}
		nodes[i] = node
	}
	if err := client.ConnectNode(nodes[0].ID, nodes[1].ID); err != nil {
		t.Fatalf("error connecting nodes: %s", err)
	}

	// store some state in the test services
	states := make([]string, nodeCount)
	for i, node := range nodes {
		rpc, err := client.RPCClient(context.Background(), node.ID)
		if err != nil {
			t.Fatalf("error getting RPC client: %s", err)
		}
		defer rpc.Close()
		state := fmt.Sprintf("%x", rand.Int())
		if err := rpc.Call(nil, "test_setState", []byte(state)); err != nil {
			t.Fatalf("error setting service state: %s", err)
		}
		states[i] = state
	}
	<-eventsDone
	// create a snapshot
	snap, err := client.CreateSnapshot()
	if err != nil {
		t.Fatalf("error creating snapshot: %s", err)
	}
	for i, state := range states {
		gotState := snap.Nodes[i].Snapshots["test"]
		if string(gotState) != state {
			t.Fatalf("expected snapshot state %q, got %q", state, gotState)
		}
	}

	// create another network
	network2, s := testHTTPServer(t)
	defer s.Close()
	client = NewClient(s.URL)
	count = 1
	eventSub = network2.Events().Subscribe(eventsDoneChan)
	go func() {
		defer eventSub.Unsubscribe()
		for event := range eventsDoneChan {
			if event.Type == EventTypeConn && !event.Control {
				count--
				if count == 0 {
					eventsDone <- struct{}{}
					return
				}
			}
		}
	}()

	// subscribe to events so we can check them later
	events := make(chan *Event, 100)
	var opts SubscribeOpts
	sub, err := client.SubscribeNetwork(events, opts)
	if err != nil {
		t.Fatalf("error subscribing to network events: %s", err)
	}
	defer sub.Unsubscribe()

	// load the snapshot
	if err := client.LoadSnapshot(snap); err != nil {
		t.Fatalf("error loading snapshot: %s", err)
	}
	<-eventsDone

	// check the nodes and connection exists
	net, err := client.GetNetwork()
	if err != nil {
		t.Fatalf("error getting network: %s", err)
	}
	if len(net.Nodes) != nodeCount {
		t.Fatalf("expected network to have %d nodes, got %d", nodeCount, len(net.Nodes))
	}
	for i, node := range nodes {
		id := net.Nodes[i].ID().String()
		if id != node.ID {
			t.Fatalf("expected node %d to have ID %s, got %s", i, node.ID, id)
		}
	}
	if len(net.Conns) != 1 {
		t.Fatalf("expected network to have 1 connection, got %d", len(net.Conns))
	}
	conn := net.Conns[0]
	if conn.One.String() != nodes[0].ID {
		t.Fatalf("expected connection to have one=%q, got one=%q", nodes[0].ID, conn.One)
	}
	if conn.Other.String() != nodes[1].ID {
		t.Fatalf("expected connection to have other=%q, got other=%q", nodes[1].ID, conn.Other)
	}
	if !conn.Up {
		t.Fatal("should be up")
	}

	// check the node states were restored
	for i, node := range nodes {
		rpc, err := client.RPCClient(context.Background(), node.ID)
		if err != nil {
			t.Fatalf("error getting RPC client: %s", err)
		}
		defer rpc.Close()
		var state []byte
		if err := rpc.Call(&state, "test_getState"); err != nil {
			t.Fatalf("error getting service state: %s", err)
		}
		if string(state) != states[i] {
			t.Fatalf("expected snapshot state %q, got %q", states[i], state)
		}
	}

	// check we got all the events
	x := &expectEvents{t, events, sub}
	x.expect(
		x.nodeEvent(nodes[0].ID, false),
		x.nodeEvent(nodes[0].ID, true),
		x.nodeEvent(nodes[1].ID, false),
		x.nodeEvent(nodes[1].ID, true),
		x.connEvent(nodes[0].ID, nodes[1].ID, false),
		x.connEvent(nodes[0].ID, nodes[1].ID, true),
	)
}

// TestMsgFilterPassMultiple tests streaming message events using a filter
// with multiple protocols
func TestMsgFilterPassMultiple(t *testing.T) {
	// start the server
	_, s := testHTTPServer(t)
	defer s.Close()

	// subscribe to events with a message filter
	client := NewClient(s.URL)
	events := make(chan *Event, 10)
	opts := SubscribeOpts{
		Filter: "prb:0-test:0",
	}
	sub, err := client.SubscribeNetwork(events, opts)
	if err != nil {
		t.Fatalf("error subscribing to network events: %s", err)
	}
	defer sub.Unsubscribe()

	// start a simulation network
	startTestNetwork(t, client)

	// check we got the expected events
	x := &expectEvents{t, events, sub}
	x.expectMsgs(map[MsgFilter]int{
		{"test", 0}: 2,
		{"prb", 0}:  2,
	})
}

// TestMsgFilterPassWildcard tests streaming message events using a filter
// with a code wildcard
func TestMsgFilterPassWildcard(t *testing.T) {
	// start the server
	_, s := testHTTPServer(t)
	defer s.Close()

	// subscribe to events with a message filter
	client := NewClient(s.URL)
	events := make(chan *Event, 10)
	opts := SubscribeOpts{
		Filter: "prb:0,2-test:*",
	}
	sub, err := client.SubscribeNetwork(events, opts)
	if err != nil {
		t.Fatalf("error subscribing to network events: %s", err)
	}
	defer sub.Unsubscribe()

	// start a simulation network
	startTestNetwork(t, client)

	// check we got the expected events
	x := &expectEvents{t, events, sub}
	x.expectMsgs(map[MsgFilter]int{
		{"test", 2}: 2,
		{"test", 1}: 2,
		{"test", 0}: 2,
		{"prb", 0}:  2,
	})
}

// TestMsgFilterPassSingle tests streaming message events using a filter
// with a single protocol and code
func TestMsgFilterPassSingle(t *testing.T) {
	// start the server
	_, s := testHTTPServer(t)
	defer s.Close()

	// subscribe to events with a message filter
	client := NewClient(s.URL)
	events := make(chan *Event, 10)
	opts := SubscribeOpts{
		Filter: "dum:0",
	}
	sub, err := client.SubscribeNetwork(events, opts)
	if err != nil {
		t.Fatalf("error subscribing to network events: %s", err)
	}
	defer sub.Unsubscribe()

	// start a simulation network
	startTestNetwork(t, client)

	// check we got the expected events
	x := &expectEvents{t, events, sub}
	x.expectMsgs(map[MsgFilter]int{
		{"dum", 0}: 2,
	})
}

// TestMsgFilterPassSingle tests streaming message events using an invalid
// filter
func TestMsgFilterFailBadParams(t *testing.T) {
	// start the server
	_, s := testHTTPServer(t)
	defer s.Close()

	client := NewClient(s.URL)
	events := make(chan *Event, 10)
	opts := SubscribeOpts{
		Filter: "foo:",
	}
	_, err := client.SubscribeNetwork(events, opts)
	if err == nil {
		t.Fatalf("expected event subscription to fail but succeeded!")
	}

	opts.Filter = "bzz:aa"
	_, err = client.SubscribeNetwork(events, opts)
	if err == nil {
		t.Fatalf("expected event subscription to fail but succeeded!")
	}

	opts.Filter = "invalid"
	_, err = client.SubscribeNetwork(events, opts)
	if err == nil {
		t.Fatalf("expected event subscription to fail but succeeded!")
	}
}
