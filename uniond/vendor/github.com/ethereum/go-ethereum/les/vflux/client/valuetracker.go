// Copyright 2020 The go-ethereum Authors
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

package client

import (
	"bytes"
	"fmt"
	"math"
	"sync"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/les/utils"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/rlp"
)

const (
	vtVersion  = 1 // database encoding format for ValueTracker
	nvtVersion = 1 // database encoding format for NodeValueTracker
)

var (
	vtKey     = []byte("vt:")
	vtNodeKey = []byte("vtNode:")
)

// NodeValueTracker collects service value statistics for a specific server node
type NodeValueTracker struct {
	lock sync.Mutex

	vt                   *ValueTracker
	rtStats, lastRtStats ResponseTimeStats
	lastTransfer         mclock.AbsTime
	basket               serverBasket
	reqCosts             []uint64
	reqValues            []float64
}

// UpdateCosts updates the node value tracker's request cost table
func (nv *NodeValueTracker) UpdateCosts(reqCosts []uint64) {
	nv.vt.lock.Lock()
	defer nv.vt.lock.Unlock()

	nv.updateCosts(reqCosts, nv.vt.refBasket.reqValues, nv.vt.refBasket.reqValueFactor(reqCosts))
}

// updateCosts updates the request cost table of the server. The request value factor
// is also updated based on the given cost table and the current reference basket.
// Note that the contents of the referenced reqValues slice will not change; a new
// reference is passed if the values are updated by ValueTracker.
func (nv *NodeValueTracker) updateCosts(reqCosts []uint64, reqValues []float64, rvFactor float64) {
	nv.lock.Lock()
	defer nv.lock.Unlock()

	nv.reqCosts = reqCosts
	nv.reqValues = reqValues
	nv.basket.updateRvFactor(rvFactor)
}

// transferStats returns request basket and response time statistics that should be
// added to the global statistics. The contents of the server's own request basket are
// gradually transferred to the main reference basket and removed from the server basket
// with the specified transfer rate.
// The response time statistics are retained at both places and therefore the global
// distribution is always the sum of the individual server distributions.
func (nv *NodeValueTracker) transferStats(now mclock.AbsTime, transferRate float64) (requestBasket, ResponseTimeStats) {
	nv.lock.Lock()
	defer nv.lock.Unlock()

	dt := now - nv.lastTransfer
	nv.lastTransfer = now
	if dt < 0 {
		dt = 0
	}
	recentRtStats := nv.rtStats
	recentRtStats.SubStats(&nv.lastRtStats)
	nv.lastRtStats = nv.rtStats
	return nv.basket.transfer(-math.Expm1(-transferRate * float64(dt))), recentRtStats
}

type ServedRequest struct {
	ReqType, Amount uint32
}

// Served adds a served request to the node's statistics. An actual request may be composed
// of one or more request types (service vector indices).
func (nv *NodeValueTracker) Served(reqs []ServedRequest, respTime time.Duration) {
	nv.vt.statsExpLock.RLock()
	expFactor := nv.vt.statsExpFactor
	nv.vt.statsExpLock.RUnlock()

	nv.lock.Lock()
	defer nv.lock.Unlock()

	var value float64
	for _, r := range reqs {
		nv.basket.add(r.ReqType, r.Amount, nv.reqCosts[r.ReqType]*uint64(r.Amount), expFactor)
		value += nv.reqValues[r.ReqType] * float64(r.Amount)
	}
	nv.rtStats.Add(respTime, value, expFactor)
}

// RtStats returns the node's own response time distribution statistics
func (nv *NodeValueTracker) RtStats() ResponseTimeStats {
	nv.lock.Lock()
	defer nv.lock.Unlock()

	return nv.rtStats
}

// ValueTracker coordinates service value calculation for individual servers and updates
// global statistics
type ValueTracker struct {
	clock        mclock.Clock
	lock         sync.Mutex
	quit         chan chan struct{}
	db           ethdb.KeyValueStore
	connected    map[enode.ID]*NodeValueTracker
	reqTypeCount int

	refBasket      referenceBasket
	mappings       [][]string
	currentMapping int
	initRefBasket  requestBasket
	rtStats        ResponseTimeStats

	transferRate                 float64
	statsExpLock                 sync.RWMutex
	statsExpRate, offlineExpRate float64
	statsExpirer                 utils.Expirer
	statsExpFactor               utils.ExpirationFactor
}

type valueTrackerEncV1 struct {
	Mappings           [][]string
	RefBasketMapping   uint
	RefBasket          requestBasket
	RtStats            ResponseTimeStats
	ExpOffset, SavedAt uint64
}

type nodeValueTrackerEncV1 struct {
	RtStats             ResponseTimeStats
	ServerBasketMapping uint
	ServerBasket        requestBasket
}

// RequestInfo is an initializer structure for the service vector.
type RequestInfo struct {
	// Name identifies the request type and is used for re-mapping the service vector if necessary
	Name string
	// InitAmount and InitValue are used to initialize the reference basket
	InitAmount, InitValue float64
}

// NewValueTracker creates a new ValueTracker and loads its previously saved state from
// the database if possible.
func NewValueTracker(db ethdb.KeyValueStore, clock mclock.Clock, reqInfo []RequestInfo, updatePeriod time.Duration, transferRate, statsExpRate, offlineExpRate float64) *ValueTracker {
	now := clock.Now()

	initRefBasket := requestBasket{items: make([]basketItem, len(reqInfo))}
	mapping := make([]string, len(reqInfo))

	var sumAmount, sumValue float64
	for _, req := range reqInfo {
		sumAmount += req.InitAmount
		sumValue += req.InitAmount * req.InitValue
	}
	scaleValues := sumAmount * basketFactor / sumValue
	for i, req := range reqInfo {
		mapping[i] = req.Name
		initRefBasket.items[i].amount = uint64(req.InitAmount * basketFactor)
		initRefBasket.items[i].value = uint64(req.InitAmount * req.InitValue * scaleValues)
	}

	vt := &ValueTracker{
		clock:          clock,
		connected:      make(map[enode.ID]*NodeValueTracker),
		quit:           make(chan chan struct{}),
		db:             db,
		reqTypeCount:   len(initRefBasket.items),
		initRefBasket:  initRefBasket,
		transferRate:   transferRate,
		statsExpRate:   statsExpRate,
		offlineExpRate: offlineExpRate,
	}
	if vt.loadFromDb(mapping) != nil {
		// previous state not saved or invalid, init with default values
		vt.refBasket.basket = initRefBasket
		vt.mappings = [][]string{mapping}
		vt.currentMapping = 0
	}
	vt.statsExpirer.SetRate(now, statsExpRate)
	vt.refBasket.init(vt.reqTypeCount)
	vt.periodicUpdate()

	go func() {
		for {
			select {
			case <-clock.After(updatePeriod):
				vt.lock.Lock()
				vt.periodicUpdate()
				vt.lock.Unlock()
			case quit := <-vt.quit:
				close(quit)
				return
			}
		}
	}()
	return vt
}

// StatsExpirer returns the statistics expirer so that other values can be expired
// with the same rate as the service value statistics.
func (vt *ValueTracker) StatsExpirer() *utils.Expirer {
	return &vt.statsExpirer
}

// StatsExpFactor returns the current expiration factor so that other values can be expired
// with the same rate as the service value statistics.
func (vt *ValueTracker) StatsExpFactor() utils.ExpirationFactor {
	vt.statsExpLock.RLock()
	defer vt.statsExpLock.RUnlock()

	return vt.statsExpFactor
}

// loadFromDb loads the value tracker's state from the database and converts saved
// request basket index mapping if it does not match the specified index to name mapping.
func (vt *ValueTracker) loadFromDb(mapping []string) error {
	enc, err := vt.db.Get(vtKey)
	if err != nil {
		return err
	}
	r := bytes.NewReader(enc)
	var version uint
	if err := rlp.Decode(r, &version); err != nil {
		log.Error("Decoding value tracker state failed", "err", err)
		return err
	}
	if version != vtVersion {
		log.Error("Unknown ValueTracker version", "stored", version, "current", nvtVersion)
		return fmt.Errorf("Unknown ValueTracker version %d (current version is %d)", version, vtVersion)
	}
	var vte valueTrackerEncV1
	if err := rlp.Decode(r, &vte); err != nil {
		log.Error("Decoding value tracker state failed", "err", err)
		return err
	}
	logOffset := utils.Fixed64(vte.ExpOffset)
	dt := time.Now().UnixNano() - int64(vte.SavedAt)
	if dt > 0 {
		logOffset += utils.Float64ToFixed64(float64(dt) * vt.offlineExpRate / math.Log(2))
	}
	vt.statsExpirer.SetLogOffset(vt.clock.Now(), logOffset)
	vt.rtStats = vte.RtStats
	vt.mappings = vte.Mappings
	vt.currentMapping = -1
loop:
	for i, m := range vt.mappings {
		if len(m) != len(mapping) {
			continue loop
		}
		for j, s := range mapping {
			if m[j] != s {
				continue loop
			}
		}
		vt.currentMapping = i
		break
	}
	if vt.currentMapping == -1 {
		vt.currentMapping = len(vt.mappings)
		vt.mappings = append(vt.mappings, mapping)
	}
	if int(vte.RefBasketMapping) == vt.currentMapping {
		vt.refBasket.basket = vte.RefBasket
	} else {
		if vte.RefBasketMapping >= uint(len(vt.mappings)) {
			log.Error("Unknown request basket mapping", "stored", vte.RefBasketMapping, "current", vt.currentMapping)
			return fmt.Errorf("Unknown request basket mapping %d (current version is %d)", vte.RefBasketMapping, vt.currentMapping)
		}
		vt.refBasket.basket = vte.RefBasket.convertMapping(vt.mappings[vte.RefBasketMapping], mapping, vt.initRefBasket)
	}
	return nil
}

// saveToDb saves the value tracker's state to the database
func (vt *ValueTracker) saveToDb() {
	vte := valueTrackerEncV1{
		Mappings:         vt.mappings,
		RefBasketMapping: uint(vt.currentMapping),
		RefBasket:        vt.refBasket.basket,
		RtStats:          vt.rtStats,
		ExpOffset:        uint64(vt.statsExpirer.LogOffset(vt.clock.Now())),
		SavedAt:          uint64(time.Now().UnixNano()),
	}
	enc1, err := rlp.EncodeToBytes(uint(vtVersion))
	if err != nil {
		log.Error("Encoding value tracker state failed", "err", err)
		return
	}
	enc2, err := rlp.EncodeToBytes(&vte)
	if err != nil {
		log.Error("Encoding value tracker state failed", "err", err)
		return
	}
	if err := vt.db.Put(vtKey, append(enc1, enc2...)); err != nil {
		log.Error("Saving value tracker state failed", "err", err)
	}
}

// Stop saves the value tracker's state and each loaded node's individual state and
// returns after shutting the internal goroutines down.
func (vt *ValueTracker) Stop() {
	quit := make(chan struct{})
	vt.quit <- quit
	<-quit
	vt.lock.Lock()
	vt.periodicUpdate()
	for id, nv := range vt.connected {
		vt.saveNode(id, nv)
	}
	vt.connected = nil
	vt.saveToDb()
	vt.lock.Unlock()
}

// Register adds a server node to the value tracker
func (vt *ValueTracker) Register(id enode.ID) *NodeValueTracker {
	vt.lock.Lock()
	defer vt.lock.Unlock()

	if vt.connected == nil {
		// ValueTracker has already been stopped
		return nil
	}
	nv := vt.loadOrNewNode(id)
	reqTypeCount := len(vt.refBasket.reqValues)
	nv.reqCosts = make([]uint64, reqTypeCount)
	nv.lastTransfer = vt.clock.Now()
	nv.reqValues = vt.refBasket.reqValues
	nv.basket.init(reqTypeCount)

	vt.connected[id] = nv
	return nv
}

// Unregister removes a server node from the value tracker
func (vt *ValueTracker) Unregister(id enode.ID) {
	vt.lock.Lock()
	defer vt.lock.Unlock()

	if nv := vt.connected[id]; nv != nil {
		vt.saveNode(id, nv)
		delete(vt.connected, id)
	}
}

// GetNode returns an individual server node's value tracker. If it did not exist before
// then a new node is created.
func (vt *ValueTracker) GetNode(id enode.ID) *NodeValueTracker {
	vt.lock.Lock()
	defer vt.lock.Unlock()

	return vt.loadOrNewNode(id)
}

// loadOrNewNode returns an individual server node's value tracker. If it did not exist before
// then a new node is created.
func (vt *ValueTracker) loadOrNewNode(id enode.ID) *NodeValueTracker {
	if nv, ok := vt.connected[id]; ok {
		return nv
	}
	nv := &NodeValueTracker{vt: vt, lastTransfer: vt.clock.Now()}
	enc, err := vt.db.Get(append(vtNodeKey, id[:]...))
	if err != nil {
		return nv
	}
	r := bytes.NewReader(enc)
	var version uint
	if err := rlp.Decode(r, &version); err != nil {
		log.Error("Failed to decode node value tracker", "id", id, "err", err)
		return nv
	}
	if version != nvtVersion {
		log.Error("Unknown NodeValueTracker version", "stored", version, "current", nvtVersion)
		return nv
	}
	var nve nodeValueTrackerEncV1
	if err := rlp.Decode(r, &nve); err != nil {
		log.Error("Failed to decode node value tracker", "id", id, "err", err)
		return nv
	}
	nv.rtStats = nve.RtStats
	nv.lastRtStats = nve.RtStats
	if int(nve.ServerBasketMapping) == vt.currentMapping {
		nv.basket.basket = nve.ServerBasket
	} else {
		if nve.ServerBasketMapping >= uint(len(vt.mappings)) {
			log.Error("Unknown request basket mapping", "stored", nve.ServerBasketMapping, "current", vt.currentMapping)
			return nv
		}
		nv.basket.basket = nve.ServerBasket.convertMapping(vt.mappings[nve.ServerBasketMapping], vt.mappings[vt.currentMapping], vt.initRefBasket)
	}
	return nv
}

// saveNode saves a server node's value tracker to the database
func (vt *ValueTracker) saveNode(id enode.ID, nv *NodeValueTracker) {
	recentRtStats := nv.rtStats
	recentRtStats.SubStats(&nv.lastRtStats)
	vt.rtStats.AddStats(&recentRtStats)
	nv.lastRtStats = nv.rtStats

	nve := nodeValueTrackerEncV1{
		RtStats:             nv.rtStats,
		ServerBasketMapping: uint(vt.currentMapping),
		ServerBasket:        nv.basket.basket,
	}
	enc1, err := rlp.EncodeToBytes(uint(nvtVersion))
	if err != nil {
		log.Error("Failed to encode service value information", "id", id, "err", err)
		return
	}
	enc2, err := rlp.EncodeToBytes(&nve)
	if err != nil {
		log.Error("Failed to encode service value information", "id", id, "err", err)
		return
	}
	if err := vt.db.Put(append(vtNodeKey, id[:]...), append(enc1, enc2...)); err != nil {
		log.Error("Failed to save service value information", "id", id, "err", err)
	}
}

// RtStats returns the global response time distribution statistics
func (vt *ValueTracker) RtStats() ResponseTimeStats {
	vt.lock.Lock()
	defer vt.lock.Unlock()

	vt.periodicUpdate()
	return vt.rtStats
}

// periodicUpdate transfers individual node data to the global statistics, normalizes
// the reference basket and updates request values. The global state is also saved to
// the database with each update.
func (vt *ValueTracker) periodicUpdate() {
	now := vt.clock.Now()
	vt.statsExpLock.Lock()
	vt.statsExpFactor = utils.ExpFactor(vt.statsExpirer.LogOffset(now))
	vt.statsExpLock.Unlock()

	for _, nv := range vt.connected {
		basket, rtStats := nv.transferStats(now, vt.transferRate)
		vt.refBasket.add(basket)
		vt.rtStats.AddStats(&rtStats)
	}
	vt.refBasket.normalize()
	vt.refBasket.updateReqValues()
	for _, nv := range vt.connected {
		nv.updateCosts(nv.reqCosts, vt.refBasket.reqValues, vt.refBasket.reqValueFactor(nv.reqCosts))
	}
	vt.saveToDb()
}

type RequestStatsItem struct {
	Name                string
	ReqAmount, ReqValue float64
}

// RequestStats returns the current contents of the reference request basket, with
// request values meaning average per request rather than total.
func (vt *ValueTracker) RequestStats() []RequestStatsItem {
	vt.statsExpLock.RLock()
	expFactor := vt.statsExpFactor
	vt.statsExpLock.RUnlock()
	vt.lock.Lock()
	defer vt.lock.Unlock()

	vt.periodicUpdate()
	res := make([]RequestStatsItem, len(vt.refBasket.basket.items))
	for i, item := range vt.refBasket.basket.items {
		res[i].Name = vt.mappings[vt.currentMapping][i]
		res[i].ReqAmount = expFactor.Value(float64(item.amount)/basketFactor, vt.refBasket.basket.exp)
		res[i].ReqValue = vt.refBasket.reqValues[i]
	}
	return res
}
