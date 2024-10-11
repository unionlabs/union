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

package main

import (
	"flag"
	"fmt"
	"io"
	"net/http"
	"os"
	"sync/atomic"
	"time"

	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/node"
	"github.com/ethereum/go-ethereum/p2p"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/simulations"
	"github.com/ethereum/go-ethereum/p2p/simulations/adapters"
)

var adapterType = flag.String("adapter", "sim", `node adapter to use (one of "sim", "exec" or "docker")`)

// main() starts a simulation network which contains nodes running a simple
// ping-pong protocol
func main() {
	flag.Parse()

	// set the log level to Trace
	log.Root().SetHandler(log.LvlFilterHandler(log.LvlTrace, log.StreamHandler(os.Stderr, log.TerminalFormat(false))))

	// register a single ping-pong service
	services := map[string]adapters.LifecycleConstructor{
		"ping-pong": func(ctx *adapters.ServiceContext, stack *node.Node) (node.Lifecycle, error) {
			pps := newPingPongService(ctx.Config.ID)
			stack.RegisterProtocols(pps.Protocols())
			return pps, nil
		},
	}
	adapters.RegisterLifecycles(services)

	// create the NodeAdapter
	var adapter adapters.NodeAdapter

	switch *adapterType {

	case "sim":
		log.Info("using sim adapter")
		adapter = adapters.NewSimAdapter(services)

	case "exec":
		tmpdir, err := os.MkdirTemp("", "p2p-example")
		if err != nil {
			log.Crit("error creating temp dir", "err", err)
		}
		defer os.RemoveAll(tmpdir)
		log.Info("using exec adapter", "tmpdir", tmpdir)
		adapter = adapters.NewExecAdapter(tmpdir)

	default:
		log.Crit(fmt.Sprintf("unknown node adapter %q", *adapterType))
	}

	// start the HTTP API
	log.Info("starting simulation server on 0.0.0.0:8888...")
	network := simulations.NewNetwork(adapter, &simulations.NetworkConfig{
		DefaultService: "ping-pong",
	})
	if err := http.ListenAndServe(":8888", simulations.NewServer(network)); err != nil {
		log.Crit("error starting simulation server", "err", err)
	}
}

// pingPongService runs a ping-pong protocol between nodes where each node
// sends a ping to all its connected peers every 10s and receives a pong in
// return
type pingPongService struct {
	id       enode.ID
	log      log.Logger
	received atomic.Int64
}

func newPingPongService(id enode.ID) *pingPongService {
	return &pingPongService{
		id:  id,
		log: log.New("node.id", id),
	}
}

func (p *pingPongService) Protocols() []p2p.Protocol {
	return []p2p.Protocol{{
		Name:     "ping-pong",
		Version:  1,
		Length:   2,
		Run:      p.Run,
		NodeInfo: p.Info,
	}}
}

func (p *pingPongService) Start() error {
	p.log.Info("ping-pong service starting")
	return nil
}

func (p *pingPongService) Stop() error {
	p.log.Info("ping-pong service stopping")
	return nil
}

func (p *pingPongService) Info() interface{} {
	return struct {
		Received int64 `json:"received"`
	}{
		p.received.Load(),
	}
}

const (
	pingMsgCode = iota
	pongMsgCode
)

// Run implements the ping-pong protocol which sends ping messages to the peer
// at 10s intervals, and responds to pings with pong messages.
func (p *pingPongService) Run(peer *p2p.Peer, rw p2p.MsgReadWriter) error {
	log := p.log.New("peer.id", peer.ID())

	errC := make(chan error, 1)
	go func() {
		for range time.Tick(10 * time.Second) {
			log.Info("sending ping")
			if err := p2p.Send(rw, pingMsgCode, "PING"); err != nil {
				errC <- err
				return
			}
		}
	}()
	go func() {
		for {
			msg, err := rw.ReadMsg()
			if err != nil {
				errC <- err
				return
			}
			payload, err := io.ReadAll(msg.Payload)
			if err != nil {
				errC <- err
				return
			}
			log.Info("received message", "msg.code", msg.Code, "msg.payload", string(payload))
			p.received.Add(1)
			if msg.Code == pingMsgCode {
				log.Info("sending pong")
				go p2p.Send(rw, pongMsgCode, "PONG")
			}
		}
	}()
	return <-errC
}
