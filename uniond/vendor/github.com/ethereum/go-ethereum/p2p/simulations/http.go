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
	"bufio"
	"bytes"
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"html"
	"io"
	"net/http"
	"strconv"
	"strings"
	"sync"

	"github.com/ethereum/go-ethereum/event"
	"github.com/ethereum/go-ethereum/p2p"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/simulations/adapters"
	"github.com/ethereum/go-ethereum/rpc"
	"github.com/gorilla/websocket"
	"github.com/julienschmidt/httprouter"
)

// DefaultClient is the default simulation API client which expects the API
// to be running at http://localhost:8888
var DefaultClient = NewClient("http://localhost:8888")

// Client is a client for the simulation HTTP API which supports creating
// and managing simulation networks
type Client struct {
	URL string

	client *http.Client
}

// NewClient returns a new simulation API client
func NewClient(url string) *Client {
	return &Client{
		URL:    url,
		client: http.DefaultClient,
	}
}

// GetNetwork returns details of the network
func (c *Client) GetNetwork() (*Network, error) {
	network := &Network{}
	return network, c.Get("/", network)
}

// StartNetwork starts all existing nodes in the simulation network
func (c *Client) StartNetwork() error {
	return c.Post("/start", nil, nil)
}

// StopNetwork stops all existing nodes in a simulation network
func (c *Client) StopNetwork() error {
	return c.Post("/stop", nil, nil)
}

// CreateSnapshot creates a network snapshot
func (c *Client) CreateSnapshot() (*Snapshot, error) {
	snap := &Snapshot{}
	return snap, c.Get("/snapshot", snap)
}

// LoadSnapshot loads a snapshot into the network
func (c *Client) LoadSnapshot(snap *Snapshot) error {
	return c.Post("/snapshot", snap, nil)
}

// SubscribeOpts is a collection of options to use when subscribing to network
// events
type SubscribeOpts struct {
	// Current instructs the server to send events for existing nodes and
	// connections first
	Current bool

	// Filter instructs the server to only send a subset of message events
	Filter string
}

// SubscribeNetwork subscribes to network events which are sent from the server
// as a server-sent-events stream, optionally receiving events for existing
// nodes and connections and filtering message events
func (c *Client) SubscribeNetwork(events chan *Event, opts SubscribeOpts) (event.Subscription, error) {
	url := fmt.Sprintf("%s/events?current=%t&filter=%s", c.URL, opts.Current, opts.Filter)
	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		return nil, err
	}
	req.Header.Set("Accept", "text/event-stream")
	res, err := c.client.Do(req)
	if err != nil {
		return nil, err
	}
	if res.StatusCode != http.StatusOK {
		response, _ := io.ReadAll(res.Body)
		res.Body.Close()
		return nil, fmt.Errorf("unexpected HTTP status: %s: %s", res.Status, response)
	}

	// define a producer function to pass to event.Subscription
	// which reads server-sent events from res.Body and sends
	// them to the events channel
	producer := func(stop <-chan struct{}) error {
		defer res.Body.Close()

		// read lines from res.Body in a goroutine so that we are
		// always reading from the stop channel
		lines := make(chan string)
		errC := make(chan error, 1)
		go func() {
			s := bufio.NewScanner(res.Body)
			for s.Scan() {
				select {
				case lines <- s.Text():
				case <-stop:
					return
				}
			}
			errC <- s.Err()
		}()

		// detect any lines which start with "data:", decode the data
		// into an event and send it to the events channel
		for {
			select {
			case line := <-lines:
				if !strings.HasPrefix(line, "data:") {
					continue
				}
				data := strings.TrimSpace(strings.TrimPrefix(line, "data:"))
				event := &Event{}
				if err := json.Unmarshal([]byte(data), event); err != nil {
					return fmt.Errorf("error decoding SSE event: %s", err)
				}
				select {
				case events <- event:
				case <-stop:
					return nil
				}
			case err := <-errC:
				return err
			case <-stop:
				return nil
			}
		}
	}

	return event.NewSubscription(producer), nil
}

// GetNodes returns all nodes which exist in the network
func (c *Client) GetNodes() ([]*p2p.NodeInfo, error) {
	var nodes []*p2p.NodeInfo
	return nodes, c.Get("/nodes", &nodes)
}

// CreateNode creates a node in the network using the given configuration
func (c *Client) CreateNode(config *adapters.NodeConfig) (*p2p.NodeInfo, error) {
	node := &p2p.NodeInfo{}
	return node, c.Post("/nodes", config, node)
}

// GetNode returns details of a node
func (c *Client) GetNode(nodeID string) (*p2p.NodeInfo, error) {
	node := &p2p.NodeInfo{}
	return node, c.Get(fmt.Sprintf("/nodes/%s", nodeID), node)
}

// StartNode starts a node
func (c *Client) StartNode(nodeID string) error {
	return c.Post(fmt.Sprintf("/nodes/%s/start", nodeID), nil, nil)
}

// StopNode stops a node
func (c *Client) StopNode(nodeID string) error {
	return c.Post(fmt.Sprintf("/nodes/%s/stop", nodeID), nil, nil)
}

// ConnectNode connects a node to a peer node
func (c *Client) ConnectNode(nodeID, peerID string) error {
	return c.Post(fmt.Sprintf("/nodes/%s/conn/%s", nodeID, peerID), nil, nil)
}

// DisconnectNode disconnects a node from a peer node
func (c *Client) DisconnectNode(nodeID, peerID string) error {
	return c.Delete(fmt.Sprintf("/nodes/%s/conn/%s", nodeID, peerID))
}

// RPCClient returns an RPC client connected to a node
func (c *Client) RPCClient(ctx context.Context, nodeID string) (*rpc.Client, error) {
	baseURL := strings.Replace(c.URL, "http", "ws", 1)
	return rpc.DialWebsocket(ctx, fmt.Sprintf("%s/nodes/%s/rpc", baseURL, nodeID), "")
}

// Get performs a HTTP GET request decoding the resulting JSON response
// into "out"
func (c *Client) Get(path string, out interface{}) error {
	return c.Send(http.MethodGet, path, nil, out)
}

// Post performs a HTTP POST request sending "in" as the JSON body and
// decoding the resulting JSON response into "out"
func (c *Client) Post(path string, in, out interface{}) error {
	return c.Send(http.MethodPost, path, in, out)
}

// Delete performs a HTTP DELETE request
func (c *Client) Delete(path string) error {
	return c.Send(http.MethodDelete, path, nil, nil)
}

// Send performs a HTTP request, sending "in" as the JSON request body and
// decoding the JSON response into "out"
func (c *Client) Send(method, path string, in, out interface{}) error {
	var body []byte
	if in != nil {
		var err error
		body, err = json.Marshal(in)
		if err != nil {
			return err
		}
	}
	req, err := http.NewRequest(method, c.URL+path, bytes.NewReader(body))
	if err != nil {
		return err
	}
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Accept", "application/json")
	res, err := c.client.Do(req)
	if err != nil {
		return err
	}
	defer res.Body.Close()
	if res.StatusCode != http.StatusOK && res.StatusCode != http.StatusCreated {
		response, _ := io.ReadAll(res.Body)
		return fmt.Errorf("unexpected HTTP status: %s: %s", res.Status, response)
	}
	if out != nil {
		if err := json.NewDecoder(res.Body).Decode(out); err != nil {
			return err
		}
	}
	return nil
}

// Server is an HTTP server providing an API to manage a simulation network
type Server struct {
	router     *httprouter.Router
	network    *Network
	mockerStop chan struct{} // when set, stops the current mocker
	mockerMtx  sync.Mutex    // synchronises access to the mockerStop field
}

// NewServer returns a new simulation API server
func NewServer(network *Network) *Server {
	s := &Server{
		router:  httprouter.New(),
		network: network,
	}

	s.OPTIONS("/", s.Options)
	s.GET("/", s.GetNetwork)
	s.POST("/start", s.StartNetwork)
	s.POST("/stop", s.StopNetwork)
	s.POST("/mocker/start", s.StartMocker)
	s.POST("/mocker/stop", s.StopMocker)
	s.GET("/mocker", s.GetMockers)
	s.POST("/reset", s.ResetNetwork)
	s.GET("/events", s.StreamNetworkEvents)
	s.GET("/snapshot", s.CreateSnapshot)
	s.POST("/snapshot", s.LoadSnapshot)
	s.POST("/nodes", s.CreateNode)
	s.GET("/nodes", s.GetNodes)
	s.GET("/nodes/:nodeid", s.GetNode)
	s.POST("/nodes/:nodeid/start", s.StartNode)
	s.POST("/nodes/:nodeid/stop", s.StopNode)
	s.POST("/nodes/:nodeid/conn/:peerid", s.ConnectNode)
	s.DELETE("/nodes/:nodeid/conn/:peerid", s.DisconnectNode)
	s.GET("/nodes/:nodeid/rpc", s.NodeRPC)

	return s
}

// GetNetwork returns details of the network
func (s *Server) GetNetwork(w http.ResponseWriter, req *http.Request) {
	s.JSON(w, http.StatusOK, s.network)
}

// StartNetwork starts all nodes in the network
func (s *Server) StartNetwork(w http.ResponseWriter, req *http.Request) {
	if err := s.network.StartAll(); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusOK)
}

// StopNetwork stops all nodes in the network
func (s *Server) StopNetwork(w http.ResponseWriter, req *http.Request) {
	if err := s.network.StopAll(); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	w.WriteHeader(http.StatusOK)
}

// StartMocker starts the mocker node simulation
func (s *Server) StartMocker(w http.ResponseWriter, req *http.Request) {
	s.mockerMtx.Lock()
	defer s.mockerMtx.Unlock()
	if s.mockerStop != nil {
		http.Error(w, "mocker already running", http.StatusInternalServerError)
		return
	}
	mockerType := req.FormValue("mocker-type")
	mockerFn := LookupMocker(mockerType)
	if mockerFn == nil {
		http.Error(w, fmt.Sprintf("unknown mocker type %q", html.EscapeString(mockerType)), http.StatusBadRequest)
		return
	}
	nodeCount, err := strconv.Atoi(req.FormValue("node-count"))
	if err != nil {
		http.Error(w, "invalid node-count provided", http.StatusBadRequest)
		return
	}
	s.mockerStop = make(chan struct{})
	go mockerFn(s.network, s.mockerStop, nodeCount)

	w.WriteHeader(http.StatusOK)
}

// StopMocker stops the mocker node simulation
func (s *Server) StopMocker(w http.ResponseWriter, req *http.Request) {
	s.mockerMtx.Lock()
	defer s.mockerMtx.Unlock()
	if s.mockerStop == nil {
		http.Error(w, "stop channel not initialized", http.StatusInternalServerError)
		return
	}
	close(s.mockerStop)
	s.mockerStop = nil

	w.WriteHeader(http.StatusOK)
}

// GetMockers returns a list of available mockers
func (s *Server) GetMockers(w http.ResponseWriter, req *http.Request) {
	list := GetMockerList()
	s.JSON(w, http.StatusOK, list)
}

// ResetNetwork resets all properties of a network to its initial (empty) state
func (s *Server) ResetNetwork(w http.ResponseWriter, req *http.Request) {
	s.network.Reset()

	w.WriteHeader(http.StatusOK)
}

// StreamNetworkEvents streams network events as a server-sent-events stream
func (s *Server) StreamNetworkEvents(w http.ResponseWriter, req *http.Request) {
	events := make(chan *Event)
	sub := s.network.events.Subscribe(events)
	defer sub.Unsubscribe()

	// write writes the given event and data to the stream like:
	//
	// event: <event>
	// data: <data>
	//
	write := func(event, data string) {
		fmt.Fprintf(w, "event: %s\n", event)
		fmt.Fprintf(w, "data: %s\n\n", data)
		if fw, ok := w.(http.Flusher); ok {
			fw.Flush()
		}
	}
	writeEvent := func(event *Event) error {
		data, err := json.Marshal(event)
		if err != nil {
			return err
		}
		write("network", string(data))
		return nil
	}
	writeErr := func(err error) {
		write("error", err.Error())
	}

	// check if filtering has been requested
	var filters MsgFilters
	if filterParam := req.URL.Query().Get("filter"); filterParam != "" {
		var err error
		filters, err = NewMsgFilters(filterParam)
		if err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
	}

	w.Header().Set("Content-Type", "text/event-stream; charset=utf-8")
	w.WriteHeader(http.StatusOK)
	fmt.Fprintf(w, "\n\n")
	if fw, ok := w.(http.Flusher); ok {
		fw.Flush()
	}

	// optionally send the existing nodes and connections
	if req.URL.Query().Get("current") == "true" {
		snap, err := s.network.Snapshot()
		if err != nil {
			writeErr(err)
			return
		}
		for _, node := range snap.Nodes {
			event := NewEvent(&node.Node)
			if err := writeEvent(event); err != nil {
				writeErr(err)
				return
			}
		}
		for _, conn := range snap.Conns {
			conn := conn
			event := NewEvent(&conn)
			if err := writeEvent(event); err != nil {
				writeErr(err)
				return
			}
		}
	}

	clientGone := req.Context().Done()
	for {
		select {
		case event := <-events:
			// only send message events which match the filters
			if event.Msg != nil && !filters.Match(event.Msg) {
				continue
			}
			if err := writeEvent(event); err != nil {
				writeErr(err)
				return
			}
		case <-clientGone:
			return
		}
	}
}

// NewMsgFilters constructs a collection of message filters from a URL query
// parameter.
//
// The parameter is expected to be a dash-separated list of individual filters,
// each having the format '<proto>:<codes>', where <proto> is the name of a
// protocol and <codes> is a comma-separated list of message codes.
//
// A message code of '*' or '-1' is considered a wildcard and matches any code.
func NewMsgFilters(filterParam string) (MsgFilters, error) {
	filters := make(MsgFilters)
	for _, filter := range strings.Split(filterParam, "-") {
		protoCodes := strings.SplitN(filter, ":", 2)
		if len(protoCodes) != 2 || protoCodes[0] == "" || protoCodes[1] == "" {
			return nil, fmt.Errorf("invalid message filter: %s", filter)
		}
		proto := protoCodes[0]
		for _, code := range strings.Split(protoCodes[1], ",") {
			if code == "*" || code == "-1" {
				filters[MsgFilter{Proto: proto, Code: -1}] = struct{}{}
				continue
			}
			n, err := strconv.ParseUint(code, 10, 64)
			if err != nil {
				return nil, fmt.Errorf("invalid message code: %s", code)
			}
			filters[MsgFilter{Proto: proto, Code: int64(n)}] = struct{}{}
		}
	}
	return filters, nil
}

// MsgFilters is a collection of filters which are used to filter message
// events
type MsgFilters map[MsgFilter]struct{}

// Match checks if the given message matches any of the filters
func (m MsgFilters) Match(msg *Msg) bool {
	// check if there is a wildcard filter for the message's protocol
	if _, ok := m[MsgFilter{Proto: msg.Protocol, Code: -1}]; ok {
		return true
	}

	// check if there is a filter for the message's protocol and code
	if _, ok := m[MsgFilter{Proto: msg.Protocol, Code: int64(msg.Code)}]; ok {
		return true
	}

	return false
}

// MsgFilter is used to filter message events based on protocol and message
// code
type MsgFilter struct {
	// Proto is matched against a message's protocol
	Proto string

	// Code is matched against a message's code, with -1 matching all codes
	Code int64
}

// CreateSnapshot creates a network snapshot
func (s *Server) CreateSnapshot(w http.ResponseWriter, req *http.Request) {
	snap, err := s.network.Snapshot()
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	s.JSON(w, http.StatusOK, snap)
}

// LoadSnapshot loads a snapshot into the network
func (s *Server) LoadSnapshot(w http.ResponseWriter, req *http.Request) {
	snap := &Snapshot{}
	if err := json.NewDecoder(req.Body).Decode(snap); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	if err := s.network.Load(snap); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	s.JSON(w, http.StatusOK, s.network)
}

// CreateNode creates a node in the network using the given configuration
func (s *Server) CreateNode(w http.ResponseWriter, req *http.Request) {
	config := &adapters.NodeConfig{}

	err := json.NewDecoder(req.Body).Decode(config)
	if err != nil && !errors.Is(err, io.EOF) {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	node, err := s.network.NewNodeWithConfig(config)
	if err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	s.JSON(w, http.StatusCreated, node.NodeInfo())
}

// GetNodes returns all nodes which exist in the network
func (s *Server) GetNodes(w http.ResponseWriter, req *http.Request) {
	nodes := s.network.GetNodes()

	infos := make([]*p2p.NodeInfo, len(nodes))
	for i, node := range nodes {
		infos[i] = node.NodeInfo()
	}

	s.JSON(w, http.StatusOK, infos)
}

// GetNode returns details of a node
func (s *Server) GetNode(w http.ResponseWriter, req *http.Request) {
	node := req.Context().Value("node").(*Node)

	s.JSON(w, http.StatusOK, node.NodeInfo())
}

// StartNode starts a node
func (s *Server) StartNode(w http.ResponseWriter, req *http.Request) {
	node := req.Context().Value("node").(*Node)

	if err := s.network.Start(node.ID()); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	s.JSON(w, http.StatusOK, node.NodeInfo())
}

// StopNode stops a node
func (s *Server) StopNode(w http.ResponseWriter, req *http.Request) {
	node := req.Context().Value("node").(*Node)

	if err := s.network.Stop(node.ID()); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	s.JSON(w, http.StatusOK, node.NodeInfo())
}

// ConnectNode connects a node to a peer node
func (s *Server) ConnectNode(w http.ResponseWriter, req *http.Request) {
	node := req.Context().Value("node").(*Node)
	peer := req.Context().Value("peer").(*Node)

	if err := s.network.Connect(node.ID(), peer.ID()); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	s.JSON(w, http.StatusOK, node.NodeInfo())
}

// DisconnectNode disconnects a node from a peer node
func (s *Server) DisconnectNode(w http.ResponseWriter, req *http.Request) {
	node := req.Context().Value("node").(*Node)
	peer := req.Context().Value("peer").(*Node)

	if err := s.network.Disconnect(node.ID(), peer.ID()); err != nil {
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	s.JSON(w, http.StatusOK, node.NodeInfo())
}

// Options responds to the OPTIONS HTTP method by returning a 200 OK response
// with the "Access-Control-Allow-Headers" header set to "Content-Type"
func (s *Server) Options(w http.ResponseWriter, req *http.Request) {
	w.Header().Set("Access-Control-Allow-Headers", "Content-Type")
	w.WriteHeader(http.StatusOK)
}

var wsUpgrade = websocket.Upgrader{
	CheckOrigin: func(*http.Request) bool { return true },
}

// NodeRPC forwards RPC requests to a node in the network via a WebSocket
// connection
func (s *Server) NodeRPC(w http.ResponseWriter, req *http.Request) {
	conn, err := wsUpgrade.Upgrade(w, req, nil)
	if err != nil {
		return
	}
	defer conn.Close()
	node := req.Context().Value("node").(*Node)
	node.ServeRPC(conn)
}

// ServeHTTP implements the http.Handler interface by delegating to the
// underlying httprouter.Router
func (s *Server) ServeHTTP(w http.ResponseWriter, req *http.Request) {
	s.router.ServeHTTP(w, req)
}

// GET registers a handler for GET requests to a particular path
func (s *Server) GET(path string, handle http.HandlerFunc) {
	s.router.GET(path, s.wrapHandler(handle))
}

// POST registers a handler for POST requests to a particular path
func (s *Server) POST(path string, handle http.HandlerFunc) {
	s.router.POST(path, s.wrapHandler(handle))
}

// DELETE registers a handler for DELETE requests to a particular path
func (s *Server) DELETE(path string, handle http.HandlerFunc) {
	s.router.DELETE(path, s.wrapHandler(handle))
}

// OPTIONS registers a handler for OPTIONS requests to a particular path
func (s *Server) OPTIONS(path string, handle http.HandlerFunc) {
	s.router.OPTIONS("/*path", s.wrapHandler(handle))
}

// JSON sends "data" as a JSON HTTP response
func (s *Server) JSON(w http.ResponseWriter, status int, data interface{}) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(status)
	json.NewEncoder(w).Encode(data)
}

// wrapHandler returns an httprouter.Handle which wraps an http.HandlerFunc by
// populating request.Context with any objects from the URL params
func (s *Server) wrapHandler(handler http.HandlerFunc) httprouter.Handle {
	return func(w http.ResponseWriter, req *http.Request, params httprouter.Params) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")

		ctx := req.Context()

		if id := params.ByName("nodeid"); id != "" {
			var nodeID enode.ID
			var node *Node
			if nodeID.UnmarshalText([]byte(id)) == nil {
				node = s.network.GetNode(nodeID)
			} else {
				node = s.network.GetNodeByName(id)
			}
			if node == nil {
				http.NotFound(w, req)
				return
			}
			ctx = context.WithValue(ctx, "node", node)
		}

		if id := params.ByName("peerid"); id != "" {
			var peerID enode.ID
			var peer *Node
			if peerID.UnmarshalText([]byte(id)) == nil {
				peer = s.network.GetNode(peerID)
			} else {
				peer = s.network.GetNodeByName(id)
			}
			if peer == nil {
				http.NotFound(w, req)
				return
			}
			ctx = context.WithValue(ctx, "peer", peer)
		}

		handler(w, req.WithContext(ctx))
	}
}
