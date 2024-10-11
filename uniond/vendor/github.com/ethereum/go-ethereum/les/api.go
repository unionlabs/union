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
	"errors"
	"fmt"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	vfs "github.com/ethereum/go-ethereum/les/vflux/server"
	"github.com/ethereum/go-ethereum/p2p/enode"
)

var errUnknownBenchmarkType = errors.New("unknown benchmark type")

// LightServerAPI provides an API to access the LES light server.
type LightServerAPI struct {
	server                               *LesServer
	defaultPosFactors, defaultNegFactors vfs.PriceFactors
}

// NewLightServerAPI creates a new LES light server API.
func NewLightServerAPI(server *LesServer) *LightServerAPI {
	return &LightServerAPI{
		server:            server,
		defaultPosFactors: defaultPosFactors,
		defaultNegFactors: defaultNegFactors,
	}
}

// parseNode parses either an enode address a raw hex node id
func parseNode(node string) (enode.ID, error) {
	if id, err := enode.ParseID(node); err == nil {
		return id, nil
	}
	if node, err := enode.Parse(enode.ValidSchemes, node); err == nil {
		return node.ID(), nil
	} else {
		return enode.ID{}, err
	}
}

// ServerInfo returns global server parameters
func (api *LightServerAPI) ServerInfo() map[string]interface{} {
	res := make(map[string]interface{})
	res["minimumCapacity"] = api.server.minCapacity
	res["maximumCapacity"] = api.server.maxCapacity
	_, res["totalCapacity"] = api.server.clientPool.Limits()
	_, res["totalConnectedCapacity"] = api.server.clientPool.Active()
	res["priorityConnectedCapacity"] = 0 //TODO connect when token sale module is added
	return res
}

// ClientInfo returns information about clients listed in the ids list or matching the given tags
func (api *LightServerAPI) ClientInfo(nodes []string) map[enode.ID]map[string]interface{} {
	var ids []enode.ID
	for _, node := range nodes {
		if id, err := parseNode(node); err == nil {
			ids = append(ids, id)
		}
	}

	res := make(map[enode.ID]map[string]interface{})
	if len(ids) == 0 {
		ids = api.server.peers.ids()
	}
	for _, id := range ids {
		if peer := api.server.peers.peer(id); peer != nil {
			res[id] = api.clientInfo(peer, peer.balance)
		} else {
			api.server.clientPool.BalanceOperation(id, "", func(balance vfs.AtomicBalanceOperator) {
				res[id] = api.clientInfo(nil, balance)
			})
		}
	}
	return res
}

// PriorityClientInfo returns information about clients with a positive balance
// in the given ID range (stop excluded). If stop is null then the iterator stops
// only at the end of the ID space. MaxCount limits the number of results returned.
// If maxCount limit is applied but there are more potential results then the ID
// of the next potential result is included in the map with an empty structure
// assigned to it.
func (api *LightServerAPI) PriorityClientInfo(start, stop enode.ID, maxCount int) map[enode.ID]map[string]interface{} {
	res := make(map[enode.ID]map[string]interface{})
	ids := api.server.clientPool.GetPosBalanceIDs(start, stop, maxCount+1)
	if len(ids) > maxCount {
		res[ids[maxCount]] = make(map[string]interface{})
		ids = ids[:maxCount]
	}
	for _, id := range ids {
		if peer := api.server.peers.peer(id); peer != nil {
			res[id] = api.clientInfo(peer, peer.balance)
		} else {
			api.server.clientPool.BalanceOperation(id, "", func(balance vfs.AtomicBalanceOperator) {
				res[id] = api.clientInfo(nil, balance)
			})
		}
	}
	return res
}

// clientInfo creates a client info data structure
func (api *LightServerAPI) clientInfo(peer *clientPeer, balance vfs.ReadOnlyBalance) map[string]interface{} {
	info := make(map[string]interface{})
	pb, nb := balance.GetBalance()
	info["isConnected"] = peer != nil
	info["pricing/balance"] = pb
	info["priority"] = pb != 0
	//		cb := api.server.clientPool.ndb.getCurrencyBalance(id)
	//		info["pricing/currency"] = cb.amount
	if peer != nil {
		info["connectionTime"] = float64(mclock.Now()-peer.connectedAt) / float64(time.Second)
		info["capacity"] = peer.getCapacity()
		info["pricing/negBalance"] = nb
	}
	return info
}

// setParams either sets the given parameters for a single connected client (if specified)
// or the default parameters applicable to clients connected in the future
func (api *LightServerAPI) setParams(params map[string]interface{}, client *clientPeer, posFactors, negFactors *vfs.PriceFactors) (updateFactors bool, err error) {
	defParams := client == nil
	for name, value := range params {
		errValue := func() error {
			return fmt.Errorf("invalid value for parameter '%s'", name)
		}
		setFactor := func(v *float64) {
			if val, ok := value.(float64); ok && val >= 0 {
				*v = val / float64(time.Second)
				updateFactors = true
			} else {
				err = errValue()
			}
		}

		switch {
		case name == "pricing/timeFactor":
			setFactor(&posFactors.TimeFactor)
		case name == "pricing/capacityFactor":
			setFactor(&posFactors.CapacityFactor)
		case name == "pricing/requestCostFactor":
			setFactor(&posFactors.RequestFactor)
		case name == "pricing/negative/timeFactor":
			setFactor(&negFactors.TimeFactor)
		case name == "pricing/negative/capacityFactor":
			setFactor(&negFactors.CapacityFactor)
		case name == "pricing/negative/requestCostFactor":
			setFactor(&negFactors.RequestFactor)
		case !defParams && name == "capacity":
			if capacity, ok := value.(float64); ok && uint64(capacity) >= api.server.minCapacity {
				_, err = api.server.clientPool.SetCapacity(client.Node(), uint64(capacity), 0, false)
				// time factor recalculation is performed automatically by the balance tracker
			} else {
				err = errValue()
			}
		default:
			if defParams {
				err = fmt.Errorf("invalid default parameter '%s'", name)
			} else {
				err = fmt.Errorf("invalid client parameter '%s'", name)
			}
		}
		if err != nil {
			return
		}
	}
	return
}

// SetClientParams sets client parameters for all clients listed in the ids list
// or all connected clients if the list is empty
func (api *LightServerAPI) SetClientParams(nodes []string, params map[string]interface{}) error {
	var err error
	for _, node := range nodes {
		var id enode.ID
		if id, err = parseNode(node); err != nil {
			return err
		}
		if peer := api.server.peers.peer(id); peer != nil {
			posFactors, negFactors := peer.balance.GetPriceFactors()
			update, e := api.setParams(params, peer, &posFactors, &negFactors)
			if update {
				peer.balance.SetPriceFactors(posFactors, negFactors)
			}
			if e != nil {
				err = e
			}
		} else {
			err = fmt.Errorf("client %064x is not connected", id)
		}
	}
	return err
}

// SetDefaultParams sets the default parameters applicable to clients connected in the future
func (api *LightServerAPI) SetDefaultParams(params map[string]interface{}) error {
	update, err := api.setParams(params, nil, &api.defaultPosFactors, &api.defaultNegFactors)
	if update {
		api.server.clientPool.SetDefaultFactors(api.defaultPosFactors, api.defaultNegFactors)
	}
	return err
}

// SetConnectedBias set the connection bias, which is applied to already connected clients
// So that already connected client won't be kicked out very soon and we can ensure all
// connected clients can have enough time to request or sync some data.
// When the input parameter `bias` < 0 (illegal), return error.
func (api *LightServerAPI) SetConnectedBias(bias time.Duration) error {
	if bias < time.Duration(0) {
		return fmt.Errorf("bias illegal: %v less than 0", bias)
	}
	api.server.clientPool.SetConnectedBias(bias)
	return nil
}

// AddBalance adds the given amount to the balance of a client if possible and returns
// the balance before and after the operation
func (api *LightServerAPI) AddBalance(node string, amount int64) (balance [2]uint64, err error) {
	var id enode.ID
	if id, err = parseNode(node); err != nil {
		return
	}
	api.server.clientPool.BalanceOperation(id, "", func(nb vfs.AtomicBalanceOperator) {
		balance[0], balance[1], err = nb.AddBalance(amount)
	})
	return
}

// Benchmark runs a request performance benchmark with a given set of measurement setups
// in multiple passes specified by passCount. The measurement time for each setup in each
// pass is specified in milliseconds by length.
//
// Note: measurement time is adjusted for each pass depending on the previous ones.
// Therefore a controlled total measurement time is achievable in multiple passes.
func (api *LightServerAPI) Benchmark(setups []map[string]interface{}, passCount, length int) ([]map[string]interface{}, error) {
	benchmarks := make([]requestBenchmark, len(setups))
	for i, setup := range setups {
		if t, ok := setup["type"].(string); ok {
			getInt := func(field string, def int) int {
				if value, ok := setup[field].(float64); ok {
					return int(value)
				}
				return def
			}
			getBool := func(field string, def bool) bool {
				if value, ok := setup[field].(bool); ok {
					return value
				}
				return def
			}
			switch t {
			case "header":
				benchmarks[i] = &benchmarkBlockHeaders{
					amount:  getInt("amount", 1),
					skip:    getInt("skip", 1),
					byHash:  getBool("byHash", false),
					reverse: getBool("reverse", false),
				}
			case "body":
				benchmarks[i] = &benchmarkBodiesOrReceipts{receipts: false}
			case "receipts":
				benchmarks[i] = &benchmarkBodiesOrReceipts{receipts: true}
			case "proof":
				benchmarks[i] = &benchmarkProofsOrCode{code: false}
			case "code":
				benchmarks[i] = &benchmarkProofsOrCode{code: true}
			case "cht":
				benchmarks[i] = &benchmarkHelperTrie{
					bloom:    false,
					reqCount: getInt("amount", 1),
				}
			case "bloom":
				benchmarks[i] = &benchmarkHelperTrie{
					bloom:    true,
					reqCount: getInt("amount", 1),
				}
			case "txSend":
				benchmarks[i] = &benchmarkTxSend{}
			case "txStatus":
				benchmarks[i] = &benchmarkTxStatus{}
			default:
				return nil, errUnknownBenchmarkType
			}
		} else {
			return nil, errUnknownBenchmarkType
		}
	}
	rs := api.server.handler.runBenchmark(benchmarks, passCount, time.Millisecond*time.Duration(length))
	result := make([]map[string]interface{}, len(setups))
	for i, r := range rs {
		res := make(map[string]interface{})
		if r.err == nil {
			res["totalCount"] = r.totalCount
			res["avgTime"] = r.avgTime
			res["maxInSize"] = r.maxInSize
			res["maxOutSize"] = r.maxOutSize
		} else {
			res["error"] = r.err.Error()
		}
		result[i] = res
	}
	return result, nil
}

// DebugAPI provides an API to debug LES light server functionality.
type DebugAPI struct {
	server *LesServer
}

// NewDebugAPI creates a new LES light server debug API.
func NewDebugAPI(server *LesServer) *DebugAPI {
	return &DebugAPI{
		server: server,
	}
}

// FreezeClient forces a temporary client freeze which normally happens when the server is overloaded
func (api *DebugAPI) FreezeClient(node string) error {
	var (
		id  enode.ID
		err error
	)
	if id, err = parseNode(node); err != nil {
		return err
	}
	if peer := api.server.peers.peer(id); peer != nil {
		peer.freeze()
		return nil
	} else {
		return fmt.Errorf("client %064x is not connected", id[:])
	}
}
