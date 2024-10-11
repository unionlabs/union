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
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"reflect"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/node"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/simulations/adapters"
)

// Tests that a created snapshot with a minimal service only contains the expected connections
// and that a network when loaded with this snapshot only contains those same connections
func TestSnapshot(t *testing.T) {
	// PART I
	// create snapshot from ring network

	// this is a minimal service, whose protocol will take exactly one message OR close of connection before quitting
	adapter := adapters.NewSimAdapter(adapters.LifecycleConstructors{
		"noopwoop": func(ctx *adapters.ServiceContext, stack *node.Node) (node.Lifecycle, error) {
			return NewNoopService(nil), nil
		},
	})

	// create network
	network := NewNetwork(adapter, &NetworkConfig{
		DefaultService: "noopwoop",
	})
	// \todo consider making a member of network, set to true threadsafe when shutdown
	runningOne := true
	defer func() {
		if runningOne {
			network.Shutdown()
		}
	}()

	// create and start nodes
	nodeCount := 20
	ids := make([]enode.ID, nodeCount)
	for i := 0; i < nodeCount; i++ {
		conf := adapters.RandomNodeConfig()
		node, err := network.NewNodeWithConfig(conf)
		if err != nil {
			t.Fatalf("error creating node: %s", err)
		}
		if err := network.Start(node.ID()); err != nil {
			t.Fatalf("error starting node: %s", err)
		}
		ids[i] = node.ID()
	}

	// subscribe to peer events
	evC := make(chan *Event)
	sub := network.Events().Subscribe(evC)
	defer sub.Unsubscribe()

	// connect nodes in a ring
	// spawn separate thread to avoid deadlock in the event listeners
	connectErr := make(chan error, 1)
	go func() {
		for i, id := range ids {
			peerID := ids[(i+1)%len(ids)]
			if err := network.Connect(id, peerID); err != nil {
				connectErr <- err
				return
			}
		}
	}()

	// collect connection events up to expected number
	ctx, cancel := context.WithTimeout(context.TODO(), time.Second)
	defer cancel()
	checkIds := make(map[enode.ID][]enode.ID)
	connEventCount := nodeCount
OUTER:
	for {
		select {
		case <-ctx.Done():
			t.Fatal(ctx.Err())
		case err := <-connectErr:
			t.Fatal(err)
		case ev := <-evC:
			if ev.Type == EventTypeConn && !ev.Control {
				// fail on any disconnect
				if !ev.Conn.Up {
					t.Fatalf("unexpected disconnect: %v -> %v", ev.Conn.One, ev.Conn.Other)
				}
				checkIds[ev.Conn.One] = append(checkIds[ev.Conn.One], ev.Conn.Other)
				checkIds[ev.Conn.Other] = append(checkIds[ev.Conn.Other], ev.Conn.One)
				connEventCount--
				log.Debug("ev", "count", connEventCount)
				if connEventCount == 0 {
					break OUTER
				}
			}
		}
	}

	// create snapshot of current network
	snap, err := network.Snapshot()
	if err != nil {
		t.Fatal(err)
	}
	j, err := json.Marshal(snap)
	if err != nil {
		t.Fatal(err)
	}
	log.Debug("snapshot taken", "nodes", len(snap.Nodes), "conns", len(snap.Conns), "json", string(j))

	// verify that the snap element numbers check out
	if len(checkIds) != len(snap.Conns) || len(checkIds) != len(snap.Nodes) {
		t.Fatalf("snapshot wrong node,conn counts %d,%d != %d", len(snap.Nodes), len(snap.Conns), len(checkIds))
	}

	// shut down sim network
	runningOne = false
	sub.Unsubscribe()
	network.Shutdown()

	// check that we have all the expected connections in the snapshot
	for nodid, nodConns := range checkIds {
		for _, nodConn := range nodConns {
			var match bool
			for _, snapConn := range snap.Conns {
				if snapConn.One == nodid && snapConn.Other == nodConn {
					match = true
					break
				} else if snapConn.Other == nodid && snapConn.One == nodConn {
					match = true
					break
				}
			}
			if !match {
				t.Fatalf("snapshot missing conn %v -> %v", nodid, nodConn)
			}
		}
	}
	log.Info("snapshot checked")

	// PART II
	// load snapshot and verify that exactly same connections are formed

	adapter = adapters.NewSimAdapter(adapters.LifecycleConstructors{
		"noopwoop": func(ctx *adapters.ServiceContext, stack *node.Node) (node.Lifecycle, error) {
			return NewNoopService(nil), nil
		},
	})
	network = NewNetwork(adapter, &NetworkConfig{
		DefaultService: "noopwoop",
	})
	defer func() {
		network.Shutdown()
	}()

	// subscribe to peer events
	// every node up and conn up event will generate one additional control event
	// therefore multiply the count by two
	evC = make(chan *Event, (len(snap.Conns)*2)+(len(snap.Nodes)*2))
	sub = network.Events().Subscribe(evC)
	defer sub.Unsubscribe()

	// load the snapshot
	// spawn separate thread to avoid deadlock in the event listeners
	err = network.Load(snap)
	if err != nil {
		t.Fatal(err)
	}

	// collect connection events up to expected number
	ctx, cancel = context.WithTimeout(context.TODO(), time.Second*3)
	defer cancel()

	connEventCount = nodeCount

OuterTwo:
	for {
		select {
		case <-ctx.Done():
			t.Fatal(ctx.Err())
		case ev := <-evC:
			if ev.Type == EventTypeConn && !ev.Control {
				// fail on any disconnect
				if !ev.Conn.Up {
					t.Fatalf("unexpected disconnect: %v -> %v", ev.Conn.One, ev.Conn.Other)
				}
				log.Debug("conn", "on", ev.Conn.One, "other", ev.Conn.Other)
				checkIds[ev.Conn.One] = append(checkIds[ev.Conn.One], ev.Conn.Other)
				checkIds[ev.Conn.Other] = append(checkIds[ev.Conn.Other], ev.Conn.One)
				connEventCount--
				log.Debug("ev", "count", connEventCount)
				if connEventCount == 0 {
					break OuterTwo
				}
			}
		}
	}

	// check that we have all expected connections in the network
	for _, snapConn := range snap.Conns {
		var match bool
		for nodid, nodConns := range checkIds {
			for _, nodConn := range nodConns {
				if snapConn.One == nodid && snapConn.Other == nodConn {
					match = true
					break
				} else if snapConn.Other == nodid && snapConn.One == nodConn {
					match = true
					break
				}
			}
		}
		if !match {
			t.Fatalf("network missing conn %v -> %v", snapConn.One, snapConn.Other)
		}
	}

	// verify that network didn't generate any other additional connection events after the ones we have collected within a reasonable period of time
	ctx, cancel = context.WithTimeout(context.TODO(), time.Second)
	defer cancel()
	select {
	case <-ctx.Done():
	case ev := <-evC:
		if ev.Type == EventTypeConn {
			t.Fatalf("Superfluous conn found %v -> %v", ev.Conn.One, ev.Conn.Other)
		}
	}

	// This test validates if all connections from the snapshot
	// are created in the network.
	t.Run("conns after load", func(t *testing.T) {
		// Create new network.
		n := NewNetwork(
			adapters.NewSimAdapter(adapters.LifecycleConstructors{
				"noopwoop": func(ctx *adapters.ServiceContext, stack *node.Node) (node.Lifecycle, error) {
					return NewNoopService(nil), nil
				},
			}),
			&NetworkConfig{
				DefaultService: "noopwoop",
			},
		)
		defer n.Shutdown()

		// Load the same snapshot.
		err := n.Load(snap)
		if err != nil {
			t.Fatal(err)
		}

		// Check every connection from the snapshot
		// if it is in the network, too.
		for _, c := range snap.Conns {
			if n.GetConn(c.One, c.Other) == nil {
				t.Errorf("missing connection: %s -> %s", c.One, c.Other)
			}
		}
	})
}

// TestNetworkSimulation creates a multi-node simulation network with each node
// connected in a ring topology, checks that all nodes successfully handshake
// with each other and that a snapshot fully represents the desired topology
func TestNetworkSimulation(t *testing.T) {
	// create simulation network with 20 testService nodes
	adapter := adapters.NewSimAdapter(adapters.LifecycleConstructors{
		"test": newTestService,
	})
	network := NewNetwork(adapter, &NetworkConfig{
		DefaultService: "test",
	})
	defer network.Shutdown()
	nodeCount := 20
	ids := make([]enode.ID, nodeCount)
	for i := 0; i < nodeCount; i++ {
		conf := adapters.RandomNodeConfig()
		node, err := network.NewNodeWithConfig(conf)
		if err != nil {
			t.Fatalf("error creating node: %s", err)
		}
		if err := network.Start(node.ID()); err != nil {
			t.Fatalf("error starting node: %s", err)
		}
		ids[i] = node.ID()
	}

	// perform a check which connects the nodes in a ring (so each node is
	// connected to exactly two peers) and then checks that all nodes
	// performed two handshakes by checking their peerCount
	action := func(_ context.Context) error {
		for i, id := range ids {
			peerID := ids[(i+1)%len(ids)]
			if err := network.Connect(id, peerID); err != nil {
				return err
			}
		}
		return nil
	}
	check := func(ctx context.Context, id enode.ID) (bool, error) {
		// check we haven't run out of time
		select {
		case <-ctx.Done():
			return false, ctx.Err()
		default:
		}

		// get the node
		node := network.GetNode(id)
		if node == nil {
			return false, fmt.Errorf("unknown node: %s", id)
		}

		// check it has exactly two peers
		client, err := node.Client()
		if err != nil {
			return false, err
		}
		var peerCount int64
		if err := client.CallContext(ctx, &peerCount, "test_peerCount"); err != nil {
			return false, err
		}
		switch {
		case peerCount < 2:
			return false, nil
		case peerCount == 2:
			return true, nil
		default:
			return false, fmt.Errorf("unexpected peerCount: %d", peerCount)
		}
	}

	timeout := 30 * time.Second
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()

	// trigger a check every 100ms
	trigger := make(chan enode.ID)
	go triggerChecks(ctx, ids, trigger, 100*time.Millisecond)

	result := NewSimulation(network).Run(ctx, &Step{
		Action:  action,
		Trigger: trigger,
		Expect: &Expectation{
			Nodes: ids,
			Check: check,
		},
	})
	if result.Error != nil {
		t.Fatalf("simulation failed: %s", result.Error)
	}

	// take a network snapshot and check it contains the correct topology
	snap, err := network.Snapshot()
	if err != nil {
		t.Fatal(err)
	}
	if len(snap.Nodes) != nodeCount {
		t.Fatalf("expected snapshot to contain %d nodes, got %d", nodeCount, len(snap.Nodes))
	}
	if len(snap.Conns) != nodeCount {
		t.Fatalf("expected snapshot to contain %d connections, got %d", nodeCount, len(snap.Conns))
	}
	for i, id := range ids {
		conn := snap.Conns[i]
		if conn.One != id {
			t.Fatalf("expected conn[%d].One to be %s, got %s", i, id, conn.One)
		}
		peerID := ids[(i+1)%len(ids)]
		if conn.Other != peerID {
			t.Fatalf("expected conn[%d].Other to be %s, got %s", i, peerID, conn.Other)
		}
	}
}

func createTestNodes(count int, network *Network) (nodes []*Node, err error) {
	for i := 0; i < count; i++ {
		nodeConf := adapters.RandomNodeConfig()
		node, err := network.NewNodeWithConfig(nodeConf)
		if err != nil {
			return nil, err
		}
		if err := network.Start(node.ID()); err != nil {
			return nil, err
		}

		nodes = append(nodes, node)
	}

	return nodes, nil
}

func createTestNodesWithProperty(property string, count int, network *Network) (propertyNodes []*Node, err error) {
	for i := 0; i < count; i++ {
		nodeConf := adapters.RandomNodeConfig()
		nodeConf.Properties = append(nodeConf.Properties, property)

		node, err := network.NewNodeWithConfig(nodeConf)
		if err != nil {
			return nil, err
		}
		if err := network.Start(node.ID()); err != nil {
			return nil, err
		}

		propertyNodes = append(propertyNodes, node)
	}

	return propertyNodes, nil
}

// TestGetNodeIDs creates a set of nodes and attempts to retrieve their IDs,.
// It then tests again whilst excluding a node ID from being returned.
// If a node ID is not returned, or more node IDs than expected are returned, the test fails.
func TestGetNodeIDs(t *testing.T) {
	adapter := adapters.NewSimAdapter(adapters.LifecycleConstructors{
		"test": newTestService,
	})
	network := NewNetwork(adapter, &NetworkConfig{
		DefaultService: "test",
	})
	defer network.Shutdown()

	numNodes := 5
	nodes, err := createTestNodes(numNodes, network)
	if err != nil {
		t.Fatalf("Could not create test nodes %v", err)
	}

	gotNodeIDs := network.GetNodeIDs()
	if len(gotNodeIDs) != numNodes {
		t.Fatalf("Expected %d nodes, got %d", numNodes, len(gotNodeIDs))
	}

	for _, node1 := range nodes {
		match := false
		for _, node2ID := range gotNodeIDs {
			if bytes.Equal(node1.ID().Bytes(), node2ID.Bytes()) {
				match = true
				break
			}
		}

		if !match {
			t.Fatalf("A created node was not returned by GetNodes(), ID: %s", node1.ID().String())
		}
	}

	excludeNodeID := nodes[3].ID()
	gotNodeIDsExcl := network.GetNodeIDs(excludeNodeID)
	if len(gotNodeIDsExcl) != numNodes-1 {
		t.Fatalf("Expected one less node ID to be returned")
	}
	for _, nodeID := range gotNodeIDsExcl {
		if bytes.Equal(excludeNodeID.Bytes(), nodeID.Bytes()) {
			t.Fatalf("GetNodeIDs returned the node ID we excluded, ID: %s", nodeID.String())
		}
	}
}

// TestGetNodes creates a set of nodes and attempts to retrieve them again.
// It then tests again whilst excluding a node from being returned.
// If a node is not returned, or more nodes than expected are returned, the test fails.
func TestGetNodes(t *testing.T) {
	adapter := adapters.NewSimAdapter(adapters.LifecycleConstructors{
		"test": newTestService,
	})
	network := NewNetwork(adapter, &NetworkConfig{
		DefaultService: "test",
	})
	defer network.Shutdown()

	numNodes := 5
	nodes, err := createTestNodes(numNodes, network)
	if err != nil {
		t.Fatalf("Could not create test nodes %v", err)
	}

	gotNodes := network.GetNodes()
	if len(gotNodes) != numNodes {
		t.Fatalf("Expected %d nodes, got %d", numNodes, len(gotNodes))
	}

	for _, node1 := range nodes {
		match := false
		for _, node2 := range gotNodes {
			if bytes.Equal(node1.ID().Bytes(), node2.ID().Bytes()) {
				match = true
				break
			}
		}

		if !match {
			t.Fatalf("A created node was not returned by GetNodes(), ID: %s", node1.ID().String())
		}
	}

	excludeNodeID := nodes[3].ID()
	gotNodesExcl := network.GetNodes(excludeNodeID)
	if len(gotNodesExcl) != numNodes-1 {
		t.Fatalf("Expected one less node to be returned")
	}
	for _, node := range gotNodesExcl {
		if bytes.Equal(excludeNodeID.Bytes(), node.ID().Bytes()) {
			t.Fatalf("GetNodes returned the node we excluded, ID: %s", node.ID().String())
		}
	}
}

// TestGetNodesByID creates a set of nodes and attempts to retrieve a subset of them by ID
// If a node is not returned, or more nodes than expected are returned, the test fails.
func TestGetNodesByID(t *testing.T) {
	adapter := adapters.NewSimAdapter(adapters.LifecycleConstructors{
		"test": newTestService,
	})
	network := NewNetwork(adapter, &NetworkConfig{
		DefaultService: "test",
	})
	defer network.Shutdown()

	numNodes := 5
	nodes, err := createTestNodes(numNodes, network)
	if err != nil {
		t.Fatalf("Could not create test nodes: %v", err)
	}

	numSubsetNodes := 2
	subsetNodes := nodes[0:numSubsetNodes]
	var subsetNodeIDs []enode.ID
	for _, node := range subsetNodes {
		subsetNodeIDs = append(subsetNodeIDs, node.ID())
	}

	gotNodesByID := network.GetNodesByID(subsetNodeIDs)
	if len(gotNodesByID) != numSubsetNodes {
		t.Fatalf("Expected %d nodes, got %d", numSubsetNodes, len(gotNodesByID))
	}

	for _, node1 := range subsetNodes {
		match := false
		for _, node2 := range gotNodesByID {
			if bytes.Equal(node1.ID().Bytes(), node2.ID().Bytes()) {
				match = true
				break
			}
		}

		if !match {
			t.Fatalf("A created node was not returned by GetNodesByID(), ID: %s", node1.ID().String())
		}
	}
}

// TestGetNodesByProperty creates a subset of nodes with a property assigned.
// GetNodesByProperty is then checked for correctness by comparing the nodes returned to those initially created.
// If a node with a property is not found, or more nodes than expected are returned, the test fails.
func TestGetNodesByProperty(t *testing.T) {
	adapter := adapters.NewSimAdapter(adapters.LifecycleConstructors{
		"test": newTestService,
	})
	network := NewNetwork(adapter, &NetworkConfig{
		DefaultService: "test",
	})
	defer network.Shutdown()

	numNodes := 3
	_, err := createTestNodes(numNodes, network)
	if err != nil {
		t.Fatalf("Failed to create nodes: %v", err)
	}

	numPropertyNodes := 3
	propertyTest := "test"
	propertyNodes, err := createTestNodesWithProperty(propertyTest, numPropertyNodes, network)
	if err != nil {
		t.Fatalf("Failed to create nodes with property: %v", err)
	}

	gotNodesByProperty := network.GetNodesByProperty(propertyTest)
	if len(gotNodesByProperty) != numPropertyNodes {
		t.Fatalf("Expected %d nodes with a property, got %d", numPropertyNodes, len(gotNodesByProperty))
	}

	for _, node1 := range propertyNodes {
		match := false
		for _, node2 := range gotNodesByProperty {
			if bytes.Equal(node1.ID().Bytes(), node2.ID().Bytes()) {
				match = true
				break
			}
		}

		if !match {
			t.Fatalf("A created node with property was not returned by GetNodesByProperty(), ID: %s", node1.ID().String())
		}
	}
}

// TestGetNodeIDsByProperty creates a subset of nodes with a property assigned.
// GetNodeIDsByProperty is then checked for correctness by comparing the node IDs returned to those initially created.
// If a node ID with a property is not found, or more nodes IDs than expected are returned, the test fails.
func TestGetNodeIDsByProperty(t *testing.T) {
	adapter := adapters.NewSimAdapter(adapters.LifecycleConstructors{
		"test": newTestService,
	})
	network := NewNetwork(adapter, &NetworkConfig{
		DefaultService: "test",
	})
	defer network.Shutdown()

	numNodes := 3
	_, err := createTestNodes(numNodes, network)
	if err != nil {
		t.Fatalf("Failed to create nodes: %v", err)
	}

	numPropertyNodes := 3
	propertyTest := "test"
	propertyNodes, err := createTestNodesWithProperty(propertyTest, numPropertyNodes, network)
	if err != nil {
		t.Fatalf("Failed to created nodes with property: %v", err)
	}

	gotNodeIDsByProperty := network.GetNodeIDsByProperty(propertyTest)
	if len(gotNodeIDsByProperty) != numPropertyNodes {
		t.Fatalf("Expected %d nodes with a property, got %d", numPropertyNodes, len(gotNodeIDsByProperty))
	}

	for _, node1 := range propertyNodes {
		match := false
		id1 := node1.ID()
		for _, id2 := range gotNodeIDsByProperty {
			if bytes.Equal(id1.Bytes(), id2.Bytes()) {
				match = true
				break
			}
		}

		if !match {
			t.Fatalf("Not all nodes IDs were returned by GetNodeIDsByProperty(), ID: %s", id1.String())
		}
	}
}

func triggerChecks(ctx context.Context, ids []enode.ID, trigger chan enode.ID, interval time.Duration) {
	tick := time.NewTicker(interval)
	defer tick.Stop()
	for {
		select {
		case <-tick.C:
			for _, id := range ids {
				select {
				case trigger <- id:
				case <-ctx.Done():
					return
				}
			}
		case <-ctx.Done():
			return
		}
	}
}

// \todo: refactor to implement shapshots
// and connect configuration methods once these are moved from
// swarm/network/simulations/connect.go
func BenchmarkMinimalService(b *testing.B) {
	b.Run("ring/32", benchmarkMinimalServiceTmp)
}

func benchmarkMinimalServiceTmp(b *testing.B) {
	// stop timer to discard setup time pollution
	args := strings.Split(b.Name(), "/")
	nodeCount, err := strconv.ParseInt(args[2], 10, 16)
	if err != nil {
		b.Fatal(err)
	}

	for i := 0; i < b.N; i++ {
		// this is a minimal service, whose protocol will close a channel upon run of protocol
		// making it possible to bench the time it takes for the service to start and protocol actually to be run
		protoCMap := make(map[enode.ID]map[enode.ID]chan struct{})
		adapter := adapters.NewSimAdapter(adapters.LifecycleConstructors{
			"noopwoop": func(ctx *adapters.ServiceContext, stack *node.Node) (node.Lifecycle, error) {
				protoCMap[ctx.Config.ID] = make(map[enode.ID]chan struct{})
				svc := NewNoopService(protoCMap[ctx.Config.ID])
				return svc, nil
			},
		})

		// create network
		network := NewNetwork(adapter, &NetworkConfig{
			DefaultService: "noopwoop",
		})
		defer network.Shutdown()

		// create and start nodes
		ids := make([]enode.ID, nodeCount)
		for i := 0; i < int(nodeCount); i++ {
			conf := adapters.RandomNodeConfig()
			node, err := network.NewNodeWithConfig(conf)
			if err != nil {
				b.Fatalf("error creating node: %s", err)
			}
			if err := network.Start(node.ID()); err != nil {
				b.Fatalf("error starting node: %s", err)
			}
			ids[i] = node.ID()
		}

		// ready, set, go
		b.ResetTimer()

		// connect nodes in a ring
		for i, id := range ids {
			peerID := ids[(i+1)%len(ids)]
			if err := network.Connect(id, peerID); err != nil {
				b.Fatal(err)
			}
		}

		// wait for all protocols to signal to close down
		ctx, cancel := context.WithTimeout(context.TODO(), time.Second)
		defer cancel()
		for nodid, peers := range protoCMap {
			for peerid, peerC := range peers {
				log.Debug("getting ", "node", nodid, "peer", peerid)
				select {
				case <-ctx.Done():
					b.Fatal(ctx.Err())
				case <-peerC:
				}
			}
		}
	}
}

func TestNode_UnmarshalJSON(t *testing.T) {
	t.Run("up_field", func(t *testing.T) {
		runNodeUnmarshalJSON(t, casesNodeUnmarshalJSONUpField())
	})
	t.Run("config_field", func(t *testing.T) {
		runNodeUnmarshalJSON(t, casesNodeUnmarshalJSONConfigField())
	})
}

func runNodeUnmarshalJSON(t *testing.T, tests []nodeUnmarshalTestCase) {
	t.Helper()
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var got *Node
			if err := json.Unmarshal([]byte(tt.marshaled), &got); err != nil {
				expectErrorMessageToContain(t, err, tt.wantErr)
				got = nil
			}
			expectNodeEquality(t, got, tt.want)
		})
	}
}

type nodeUnmarshalTestCase struct {
	name      string
	marshaled string
	want      *Node
	wantErr   string
}

func expectErrorMessageToContain(t *testing.T, got error, want string) {
	t.Helper()
	if got == nil && want == "" {
		return
	}

	if got == nil && want != "" {
		t.Errorf("error was expected, got: nil, want: %v", want)
		return
	}

	if !strings.Contains(got.Error(), want) {
		t.Errorf(
			"unexpected error message, got  %v, want: %v",
			want,
			got,
		)
	}
}

func expectNodeEquality(t *testing.T, got, want *Node) {
	t.Helper()
	if !reflect.DeepEqual(got, want) {
		t.Errorf("Node.UnmarshalJSON() = %v, want %v", got, want)
	}
}

func casesNodeUnmarshalJSONUpField() []nodeUnmarshalTestCase {
	return []nodeUnmarshalTestCase{
		{
			name:      "empty json",
			marshaled: "{}",
			want:      newNode(nil, nil, false),
		},
		{
			name:      "a stopped node",
			marshaled: "{\"up\": false}",
			want:      newNode(nil, nil, false),
		},
		{
			name:      "a running node",
			marshaled: "{\"up\": true}",
			want:      newNode(nil, nil, true),
		},
		{
			name:      "invalid JSON value on valid key",
			marshaled: "{\"up\": foo}",
			wantErr:   "invalid character",
		},
		{
			name:      "invalid JSON key and value",
			marshaled: "{foo: bar}",
			wantErr:   "invalid character",
		},
		{
			name:      "bool value expected but got something else (string)",
			marshaled: "{\"up\": \"true\"}",
			wantErr:   "cannot unmarshal string into Go struct",
		},
	}
}

func casesNodeUnmarshalJSONConfigField() []nodeUnmarshalTestCase {
	// Don't do a big fuss around testing, as adapters.NodeConfig should
	// handle it's own serialization. Just do a sanity check.
	return []nodeUnmarshalTestCase{
		{
			name:      "Config field is omitted",
			marshaled: "{}",
			want:      newNode(nil, nil, false),
		},
		{
			name:      "Config field is nil",
			marshaled: "{\"config\": null}",
			want:      newNode(nil, nil, false),
		},
		{
			name:      "a non default Config field",
			marshaled: "{\"config\":{\"name\":\"node_ecdd0\",\"port\":44665}}",
			want:      newNode(nil, &adapters.NodeConfig{Name: "node_ecdd0", Port: 44665}, false),
		},
	}
}
