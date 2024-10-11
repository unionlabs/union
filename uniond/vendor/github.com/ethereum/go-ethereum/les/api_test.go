// Copyright 2019 The go-ethereum Authors
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

package les

import (
	"context"
	crand "crypto/rand"
	"errors"
	"flag"
	"math/rand"
	"os"
	"sync"
	"sync/atomic"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/eth"
	"github.com/ethereum/go-ethereum/eth/downloader"
	"github.com/ethereum/go-ethereum/eth/ethconfig"
	"github.com/ethereum/go-ethereum/les/flowcontrol"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/node"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/simulations"
	"github.com/ethereum/go-ethereum/p2p/simulations/adapters"
	"github.com/ethereum/go-ethereum/rpc"
	"github.com/mattn/go-colorable"
)

// Additional command line flags for the test binary.
var (
	loglevel   = flag.Int("loglevel", 0, "verbosity of logs")
	simAdapter = flag.String("adapter", "exec", "type of simulation: sim|socket|exec|docker")
)

func TestMain(m *testing.M) {
	flag.Parse()
	log.PrintOrigins(true)
	log.Root().SetHandler(log.LvlFilterHandler(log.Lvl(*loglevel), log.StreamHandler(colorable.NewColorableStderr(), log.TerminalFormat(true))))
	// register the Delivery service which will run as a devp2p
	// protocol when using the exec adapter
	adapters.RegisterLifecycles(services)
	os.Exit(m.Run())
}

// This test is not meant to be a part of the automatic testing process because it
// runs for a long time and also requires a large database in order to do a meaningful
// request performance test. When testServerDataDir is empty, the test is skipped.

const (
	testServerDataDir  = "" // should always be empty on the master branch
	testServerCapacity = 200
	testMaxClients     = 10
	testTolerance      = 0.1
	minRelCap          = 0.2
)

func TestCapacityAPI3(t *testing.T) {
	testCapacityAPI(t, 3)
}

func TestCapacityAPI6(t *testing.T) {
	testCapacityAPI(t, 6)
}

func TestCapacityAPI10(t *testing.T) {
	testCapacityAPI(t, 10)
}

// testCapacityAPI runs an end-to-end simulation test connecting one server with
// a given number of clients. It sets different priority capacities to all clients
// except a randomly selected one which runs in free client mode. All clients send
// similar requests at the maximum allowed rate and the test verifies whether the
// ratio of processed requests is close enough to the ratio of assigned capacities.
// Running multiple rounds with different settings ensures that changing capacity
// while connected and going back and forth between free and priority mode with
// the supplied API calls is also thoroughly tested.
func testCapacityAPI(t *testing.T, clientCount int) {
	// Skip test if no data dir specified
	if testServerDataDir == "" {
		return
	}
	for !testSim(t, 1, clientCount, []string{testServerDataDir}, nil, func(ctx context.Context, net *simulations.Network, servers []*simulations.Node, clients []*simulations.Node) bool {
		if len(servers) != 1 {
			t.Fatalf("Invalid number of servers: %d", len(servers))
		}
		server := servers[0]

		serverRpcClient, err := server.Client()
		if err != nil {
			t.Fatalf("Failed to obtain rpc client: %v", err)
		}
		headNum, headHash := getHead(ctx, t, serverRpcClient)
		minCap, totalCap := getCapacityInfo(ctx, t, serverRpcClient)
		testCap := totalCap * 3 / 4
		t.Logf("Server testCap: %d  minCap: %d  head number: %d  head hash: %064x\n", testCap, minCap, headNum, headHash)
		reqMinCap := uint64(float64(testCap) * minRelCap / (minRelCap + float64(len(clients)-1)))
		if minCap > reqMinCap {
			t.Fatalf("Minimum client capacity (%d) bigger than required minimum for this test (%d)", minCap, reqMinCap)
		}
		freeIdx := rand.Intn(len(clients))

		clientRpcClients := make([]*rpc.Client, len(clients))
		for i, client := range clients {
			var err error
			clientRpcClients[i], err = client.Client()
			if err != nil {
				t.Fatalf("Failed to obtain rpc client: %v", err)
			}
			t.Log("connecting client", i)
			if i != freeIdx {
				setCapacity(ctx, t, serverRpcClient, client.ID(), testCap/uint64(len(clients)))
			}
			net.Connect(client.ID(), server.ID())

			for {
				select {
				case <-ctx.Done():
					t.Fatalf("Timeout")
				default:
				}
				num, hash := getHead(ctx, t, clientRpcClients[i])
				if num == headNum && hash == headHash {
					t.Log("client", i, "synced")
					break
				}
				time.Sleep(time.Millisecond * 200)
			}
		}

		var wg sync.WaitGroup
		stop := make(chan struct{})

		reqCount := make([]atomic.Uint64, len(clientRpcClients))

		// Send light request like crazy.
		for i, c := range clientRpcClients {
			wg.Add(1)
			i, c := i, c
			go func() {
				defer wg.Done()

				queue := make(chan struct{}, 100)
				reqCount[i].Store(0)
				for {
					select {
					case queue <- struct{}{}:
						select {
						case <-stop:
							return
						case <-ctx.Done():
							return
						default:
							wg.Add(1)
							go func() {
								ok := testRequest(ctx, t, c)
								wg.Done()
								<-queue
								if ok {
									if reqCount[i].Add(1)%10000 == 0 {
										freezeClient(ctx, t, serverRpcClient, clients[i].ID())
									}
								}
							}()
						}
					case <-stop:
						return
					case <-ctx.Done():
						return
					}
				}
			}()
		}

		processedSince := func(start []uint64) []uint64 {
			res := make([]uint64, len(reqCount))
			for i := range reqCount {
				res[i] = reqCount[i].Load()
				if start != nil {
					res[i] -= start[i]
				}
			}
			return res
		}

		weights := make([]float64, len(clients))
		for c := 0; c < 5; c++ {
			setCapacity(ctx, t, serverRpcClient, clients[freeIdx].ID(), minCap)
			freeIdx = rand.Intn(len(clients))
			var sum float64
			for i := range clients {
				if i == freeIdx {
					weights[i] = 0
				} else {
					weights[i] = rand.Float64()*(1-minRelCap) + minRelCap
				}
				sum += weights[i]
			}
			for i, client := range clients {
				weights[i] *= float64(testCap-minCap-100) / sum
				capacity := uint64(weights[i])
				if i != freeIdx && capacity < getCapacity(ctx, t, serverRpcClient, client.ID()) {
					setCapacity(ctx, t, serverRpcClient, client.ID(), capacity)
				}
			}
			setCapacity(ctx, t, serverRpcClient, clients[freeIdx].ID(), 0)
			for i, client := range clients {
				capacity := uint64(weights[i])
				if i != freeIdx && capacity > getCapacity(ctx, t, serverRpcClient, client.ID()) {
					setCapacity(ctx, t, serverRpcClient, client.ID(), capacity)
				}
			}
			weights[freeIdx] = float64(minCap)
			for i := range clients {
				weights[i] /= float64(testCap)
			}

			time.Sleep(flowcontrol.DecParamDelay)
			t.Log("Starting measurement")
			t.Logf("Relative weights:")
			for i := range clients {
				t.Logf("  %f", weights[i])
			}
			t.Log()
			start := processedSince(nil)
			for {
				select {
				case <-ctx.Done():
					t.Fatalf("Timeout")
				default:
				}

				_, totalCap = getCapacityInfo(ctx, t, serverRpcClient)
				if totalCap < testCap {
					t.Log("Total capacity underrun")
					close(stop)
					wg.Wait()
					return false
				}

				processed := processedSince(start)
				var avg uint64
				t.Logf("Processed")
				for i, p := range processed {
					t.Logf(" %d", p)
					processed[i] = uint64(float64(p) / weights[i])
					avg += processed[i]
				}
				avg /= uint64(len(processed))

				if avg >= 10000 {
					var maxDev float64
					for _, p := range processed {
						dev := float64(int64(p-avg)) / float64(avg)
						t.Logf(" %7.4f", dev)
						if dev < 0 {
							dev = -dev
						}
						if dev > maxDev {
							maxDev = dev
						}
					}
					t.Logf("  max deviation: %f  totalCap: %d\n", maxDev, totalCap)
					if maxDev <= testTolerance {
						t.Log("success")
						break
					}
				} else {
					t.Log()
				}
				time.Sleep(time.Millisecond * 200)
			}
		}

		close(stop)
		wg.Wait()

		for i := range reqCount {
			t.Log("client", i, "processed", reqCount[i].Load())
		}
		return true
	}) {
		t.Log("restarting test")
	}
}

func getHead(ctx context.Context, t *testing.T, client *rpc.Client) (uint64, common.Hash) {
	res := make(map[string]interface{})
	if err := client.CallContext(ctx, &res, "eth_getBlockByNumber", "latest", false); err != nil {
		t.Fatalf("Failed to obtain head block: %v", err)
	}
	numStr, ok := res["number"].(string)
	if !ok {
		t.Fatalf("RPC block number field invalid")
	}
	num, err := hexutil.DecodeUint64(numStr)
	if err != nil {
		t.Fatalf("Failed to decode RPC block number: %v", err)
	}
	hashStr, ok := res["hash"].(string)
	if !ok {
		t.Fatalf("RPC block number field invalid")
	}
	hash := common.HexToHash(hashStr)
	return num, hash
}

func testRequest(ctx context.Context, t *testing.T, client *rpc.Client) bool {
	var res string
	var addr common.Address
	crand.Read(addr[:])
	c, cancel := context.WithTimeout(ctx, time.Second*12)
	defer cancel()
	err := client.CallContext(c, &res, "eth_getBalance", addr, "latest")
	if err != nil {
		t.Log("request error:", err)
	}
	return err == nil
}

func freezeClient(ctx context.Context, t *testing.T, server *rpc.Client, clientID enode.ID) {
	if err := server.CallContext(ctx, nil, "debug_freezeClient", clientID); err != nil {
		t.Fatalf("Failed to freeze client: %v", err)
	}
}

func setCapacity(ctx context.Context, t *testing.T, server *rpc.Client, clientID enode.ID, cap uint64) {
	params := make(map[string]interface{})
	params["capacity"] = cap
	if err := server.CallContext(ctx, nil, "les_setClientParams", []enode.ID{clientID}, []string{}, params); err != nil {
		t.Fatalf("Failed to set client capacity: %v", err)
	}
}

func getCapacity(ctx context.Context, t *testing.T, server *rpc.Client, clientID enode.ID) uint64 {
	var res map[enode.ID]map[string]interface{}
	if err := server.CallContext(ctx, &res, "les_clientInfo", []enode.ID{clientID}, []string{}); err != nil {
		t.Fatalf("Failed to get client info: %v", err)
	}
	info, ok := res[clientID]
	if !ok {
		t.Fatalf("Missing client info")
	}
	v, ok := info["capacity"]
	if !ok {
		t.Fatalf("Missing field in client info: capacity")
	}
	vv, ok := v.(float64)
	if !ok {
		t.Fatalf("Failed to decode capacity field")
	}
	return uint64(vv)
}

func getCapacityInfo(ctx context.Context, t *testing.T, server *rpc.Client) (minCap, totalCap uint64) {
	var res map[string]interface{}
	if err := server.CallContext(ctx, &res, "les_serverInfo"); err != nil {
		t.Fatalf("Failed to query server info: %v", err)
	}
	decode := func(s string) uint64 {
		v, ok := res[s]
		if !ok {
			t.Fatalf("Missing field in server info: %s", s)
		}
		vv, ok := v.(float64)
		if !ok {
			t.Fatalf("Failed to decode server info field: %s", s)
		}
		return uint64(vv)
	}
	minCap = decode("minimumCapacity")
	totalCap = decode("totalCapacity")
	return
}

var services = adapters.LifecycleConstructors{
	"lesclient": newLesClientService,
	"lesserver": newLesServerService,
}

func NewNetwork() (*simulations.Network, func(), error) {
	adapter, adapterTeardown, err := NewAdapter(*simAdapter, services)
	if err != nil {
		return nil, adapterTeardown, err
	}
	defaultService := "streamer"
	net := simulations.NewNetwork(adapter, &simulations.NetworkConfig{
		ID:             "0",
		DefaultService: defaultService,
	})
	teardown := func() {
		adapterTeardown()
		net.Shutdown()
	}
	return net, teardown, nil
}

func NewAdapter(adapterType string, services adapters.LifecycleConstructors) (adapter adapters.NodeAdapter, teardown func(), err error) {
	teardown = func() {}
	switch adapterType {
	case "sim":
		adapter = adapters.NewSimAdapter(services)
		//	case "socket":
		//		adapter = adapters.NewSocketAdapter(services)
	case "exec":
		baseDir, err0 := os.MkdirTemp("", "les-test")
		if err0 != nil {
			return nil, teardown, err0
		}
		teardown = func() { os.RemoveAll(baseDir) }
		adapter = adapters.NewExecAdapter(baseDir)
	/*case "docker":
	adapter, err = adapters.NewDockerAdapter()
	if err != nil {
		return nil, teardown, err
	}*/
	default:
		return nil, teardown, errors.New("adapter needs to be one of sim, socket, exec, docker")
	}
	return adapter, teardown, nil
}

func testSim(t *testing.T, serverCount, clientCount int, serverDir, clientDir []string, test func(ctx context.Context, net *simulations.Network, servers []*simulations.Node, clients []*simulations.Node) bool) bool {
	net, teardown, err := NewNetwork()
	defer teardown()
	if err != nil {
		t.Fatalf("Failed to create network: %v", err)
	}
	timeout := 1800 * time.Second
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()

	servers := make([]*simulations.Node, serverCount)
	clients := make([]*simulations.Node, clientCount)

	for i := range clients {
		clientconf := adapters.RandomNodeConfig()
		clientconf.Lifecycles = []string{"lesclient"}
		if len(clientDir) == clientCount {
			clientconf.DataDir = clientDir[i]
		}
		client, err := net.NewNodeWithConfig(clientconf)
		if err != nil {
			t.Fatalf("Failed to create client: %v", err)
		}
		clients[i] = client
	}

	for i := range servers {
		serverconf := adapters.RandomNodeConfig()
		serverconf.Lifecycles = []string{"lesserver"}
		if len(serverDir) == serverCount {
			serverconf.DataDir = serverDir[i]
		}
		server, err := net.NewNodeWithConfig(serverconf)
		if err != nil {
			t.Fatalf("Failed to create server: %v", err)
		}
		servers[i] = server
	}

	for _, client := range clients {
		if err := net.Start(client.ID()); err != nil {
			t.Fatalf("Failed to start client node: %v", err)
		}
	}
	for _, server := range servers {
		if err := net.Start(server.ID()); err != nil {
			t.Fatalf("Failed to start server node: %v", err)
		}
	}

	return test(ctx, net, servers, clients)
}

func newLesClientService(ctx *adapters.ServiceContext, stack *node.Node) (node.Lifecycle, error) {
	config := ethconfig.Defaults
	config.SyncMode = downloader.LightSync
	return New(stack, &config)
}

func newLesServerService(ctx *adapters.ServiceContext, stack *node.Node) (node.Lifecycle, error) {
	config := ethconfig.Defaults
	config.SyncMode = downloader.FullSync
	config.LightServ = testServerCapacity
	config.LightPeers = testMaxClients
	ethereum, err := eth.New(stack, &config)
	if err != nil {
		return nil, err
	}
	_, err = NewLesServer(stack, ethereum, &config)
	if err != nil {
		return nil, err
	}
	return ethereum, nil
}
