// Copyright 2021 The go-ethereum Authors
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
	"errors"
	"math/rand"
	"reflect"
	"sync"
	"sync/atomic"
	"time"

	"github.com/ethereum/go-ethereum/common/mclock"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/les/utils"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/metrics"
	"github.com/ethereum/go-ethereum/p2p/enode"
	"github.com/ethereum/go-ethereum/p2p/enr"
	"github.com/ethereum/go-ethereum/p2p/nodestate"
	"github.com/ethereum/go-ethereum/rlp"
)

const (
	minTimeout          = time.Millisecond * 500 // minimum request timeout suggested by the server pool
	timeoutRefresh      = time.Second * 5        // recalculate timeout if older than this
	dialCost            = 10000                  // cost of a TCP dial (used for known node selection weight calculation)
	dialWaitStep        = 1.5                    // exponential multiplier of redial wait time when no value was provided by the server
	queryCost           = 500                    // cost of a UDP pre-negotiation query
	queryWaitStep       = 1.02                   // exponential multiplier of redial wait time when no value was provided by the server
	waitThreshold       = time.Hour * 2000       // drop node if waiting time is over the threshold
	nodeWeightMul       = 1000000                // multiplier constant for node weight calculation
	nodeWeightThreshold = 100                    // minimum weight for keeping a node in the known (valuable) set
	minRedialWait       = 10                     // minimum redial wait time in seconds
	preNegLimit         = 5                      // maximum number of simultaneous pre-negotiation queries
	warnQueryFails      = 20                     // number of consecutive UDP query failures before we print a warning
	maxQueryFails       = 100                    // number of consecutive UDP query failures when then chance of skipping a query reaches 50%
)

// ServerPool provides a node iterator for dial candidates. The output is a mix of newly discovered
// nodes, a weighted random selection of known (previously valuable) nodes and trusted/paid nodes.
type ServerPool struct {
	clock    mclock.Clock
	unixTime func() int64
	db       ethdb.KeyValueStore

	ns                  *nodestate.NodeStateMachine
	vt                  *ValueTracker
	mixer               *enode.FairMix
	mixSources          []enode.Iterator
	dialIterator        enode.Iterator
	validSchemes        enr.IdentityScheme
	trustedURLs         []string
	fillSet             *FillSet
	started, queryFails uint32

	timeoutLock      sync.RWMutex
	timeout          time.Duration
	timeWeights      ResponseTimeWeights
	timeoutRefreshed mclock.AbsTime

	suggestedTimeoutGauge, totalValueGauge metrics.Gauge
	sessionValueMeter                      metrics.Meter
}

// nodeHistory keeps track of dial costs which determine node weight together with the
// service value calculated by ValueTracker.
type nodeHistory struct {
	dialCost                       utils.ExpiredValue
	redialWaitStart, redialWaitEnd int64 // unix time (seconds)
}

type nodeHistoryEnc struct {
	DialCost                       utils.ExpiredValue
	RedialWaitStart, RedialWaitEnd uint64
}

// QueryFunc sends a pre-negotiation query and blocks until a response arrives or timeout occurs.
// It returns 1 if the remote node has confirmed that connection is possible, 0 if not
// possible and -1 if no response arrived (timeout).
type QueryFunc func(*enode.Node) int

var (
	clientSetup       = &nodestate.Setup{Version: 2}
	sfHasValue        = clientSetup.NewPersistentFlag("hasValue")
	sfQuery           = clientSetup.NewFlag("query")
	sfCanDial         = clientSetup.NewFlag("canDial")
	sfDialing         = clientSetup.NewFlag("dialed")
	sfWaitDialTimeout = clientSetup.NewFlag("dialTimeout")
	sfConnected       = clientSetup.NewFlag("connected")
	sfRedialWait      = clientSetup.NewFlag("redialWait")
	sfAlwaysConnect   = clientSetup.NewFlag("alwaysConnect")
	sfDialProcess     = nodestate.MergeFlags(sfQuery, sfCanDial, sfDialing, sfConnected, sfRedialWait)

	sfiNodeHistory = clientSetup.NewPersistentField("nodeHistory", reflect.TypeOf(nodeHistory{}),
		func(field interface{}) ([]byte, error) {
			if n, ok := field.(nodeHistory); ok {
				ne := nodeHistoryEnc{
					DialCost:        n.dialCost,
					RedialWaitStart: uint64(n.redialWaitStart),
					RedialWaitEnd:   uint64(n.redialWaitEnd),
				}
				enc, err := rlp.EncodeToBytes(&ne)
				return enc, err
			}
			return nil, errors.New("invalid field type")
		},
		func(enc []byte) (interface{}, error) {
			var ne nodeHistoryEnc
			err := rlp.DecodeBytes(enc, &ne)
			n := nodeHistory{
				dialCost:        ne.DialCost,
				redialWaitStart: int64(ne.RedialWaitStart),
				redialWaitEnd:   int64(ne.RedialWaitEnd),
			}
			return n, err
		},
	)
	sfiNodeWeight     = clientSetup.NewField("nodeWeight", reflect.TypeOf(uint64(0)))
	sfiConnectedStats = clientSetup.NewField("connectedStats", reflect.TypeOf(ResponseTimeStats{}))
	sfiLocalAddress   = clientSetup.NewPersistentField("localAddress", reflect.TypeOf(&enr.Record{}),
		func(field interface{}) ([]byte, error) {
			if enr, ok := field.(*enr.Record); ok {
				enc, err := rlp.EncodeToBytes(enr)
				return enc, err
			}
			return nil, errors.New("invalid field type")
		},
		func(enc []byte) (interface{}, error) {
			var enr enr.Record
			if err := rlp.DecodeBytes(enc, &enr); err != nil {
				return nil, err
			}
			return &enr, nil
		},
	)
)

// NewServerPool creates a new server pool
func NewServerPool(db ethdb.KeyValueStore, dbKey []byte, mixTimeout time.Duration, query QueryFunc, clock mclock.Clock, trustedURLs []string, requestList []RequestInfo) (*ServerPool, enode.Iterator) {
	s := &ServerPool{
		db:           db,
		clock:        clock,
		unixTime:     func() int64 { return time.Now().Unix() },
		validSchemes: enode.ValidSchemes,
		trustedURLs:  trustedURLs,
		vt:           NewValueTracker(db, &mclock.System{}, requestList, time.Minute, 1/float64(time.Hour), 1/float64(time.Hour*100), 1/float64(time.Hour*1000)),
		ns:           nodestate.NewNodeStateMachine(db, []byte(string(dbKey)+"ns:"), clock, clientSetup),
	}
	s.recalTimeout()
	s.mixer = enode.NewFairMix(mixTimeout)
	knownSelector := NewWrsIterator(s.ns, sfHasValue, sfDialProcess, sfiNodeWeight)
	alwaysConnect := NewQueueIterator(s.ns, sfAlwaysConnect, sfDialProcess, true, nil)
	s.mixSources = append(s.mixSources, knownSelector)
	s.mixSources = append(s.mixSources, alwaysConnect)

	s.dialIterator = s.mixer
	if query != nil {
		s.dialIterator = s.addPreNegFilter(s.dialIterator, query)
	}

	s.ns.SubscribeState(nodestate.MergeFlags(sfWaitDialTimeout, sfConnected), func(n *enode.Node, oldState, newState nodestate.Flags) {
		if oldState.Equals(sfWaitDialTimeout) && newState.IsEmpty() {
			// dial timeout, no connection
			s.setRedialWait(n, dialCost, dialWaitStep)
			s.ns.SetStateSub(n, nodestate.Flags{}, sfDialing, 0)
		}
	})

	return s, &serverPoolIterator{
		dialIterator: s.dialIterator,
		nextFn: func(node *enode.Node) {
			s.ns.Operation(func() {
				s.ns.SetStateSub(node, sfDialing, sfCanDial, 0)
				s.ns.SetStateSub(node, sfWaitDialTimeout, nodestate.Flags{}, time.Second*10)
			})
		},
		nodeFn: s.DialNode,
	}
}

type serverPoolIterator struct {
	dialIterator enode.Iterator
	nextFn       func(*enode.Node)
	nodeFn       func(*enode.Node) *enode.Node
}

// Next implements enode.Iterator
func (s *serverPoolIterator) Next() bool {
	if s.dialIterator.Next() {
		s.nextFn(s.dialIterator.Node())
		return true
	}
	return false
}

// Node implements enode.Iterator
func (s *serverPoolIterator) Node() *enode.Node {
	return s.nodeFn(s.dialIterator.Node())
}

// Close implements enode.Iterator
func (s *serverPoolIterator) Close() {
	s.dialIterator.Close()
}

// AddMetrics adds metrics to the server pool. Should be called before Start().
func (s *ServerPool) AddMetrics(
	suggestedTimeoutGauge, totalValueGauge, serverSelectableGauge, serverConnectedGauge metrics.Gauge,
	sessionValueMeter, serverDialedMeter metrics.Meter) {
	s.suggestedTimeoutGauge = suggestedTimeoutGauge
	s.totalValueGauge = totalValueGauge
	s.sessionValueMeter = sessionValueMeter
	if serverSelectableGauge != nil {
		s.ns.AddLogMetrics(sfHasValue, sfDialProcess, "selectable", nil, nil, serverSelectableGauge)
	}
	if serverDialedMeter != nil {
		s.ns.AddLogMetrics(sfDialing, nodestate.Flags{}, "dialed", serverDialedMeter, nil, nil)
	}
	if serverConnectedGauge != nil {
		s.ns.AddLogMetrics(sfConnected, nodestate.Flags{}, "connected", nil, nil, serverConnectedGauge)
	}
}

// AddSource adds a node discovery source to the server pool (should be called before start)
func (s *ServerPool) AddSource(source enode.Iterator) {
	if source != nil {
		s.mixSources = append(s.mixSources, source)
	}
}

// addPreNegFilter installs a node filter mechanism that performs a pre-negotiation query.
// Nodes that are filtered out and does not appear on the output iterator are put back
// into redialWait state.
func (s *ServerPool) addPreNegFilter(input enode.Iterator, query QueryFunc) enode.Iterator {
	s.fillSet = NewFillSet(s.ns, input, sfQuery)
	s.ns.SubscribeState(sfDialProcess, func(n *enode.Node, oldState, newState nodestate.Flags) {
		if !newState.Equals(sfQuery) {
			if newState.HasAll(sfQuery) {
				// remove query flag if the node is already somewhere in the dial process
				s.ns.SetStateSub(n, nodestate.Flags{}, sfQuery, 0)
			}
			return
		}
		fails := atomic.LoadUint32(&s.queryFails)
		failMax := fails
		if failMax > maxQueryFails {
			failMax = maxQueryFails
		}
		if rand.Intn(maxQueryFails*2) < int(failMax) {
			// skip pre-negotiation with increasing chance, max 50%
			// this ensures that the client can operate even if UDP is not working at all
			s.ns.SetStateSub(n, sfCanDial, nodestate.Flags{}, time.Second*10)
			// set canDial before resetting queried so that FillSet will not read more
			// candidates unnecessarily
			s.ns.SetStateSub(n, nodestate.Flags{}, sfQuery, 0)
			return
		}
		go func() {
			q := query(n)
			if q == -1 {
				atomic.AddUint32(&s.queryFails, 1)
				fails++
				if fails%warnQueryFails == 0 {
					// warn if a large number of consecutive queries have failed
					log.Warn("UDP connection queries failed", "count", fails)
				}
			} else {
				atomic.StoreUint32(&s.queryFails, 0)
			}
			s.ns.Operation(func() {
				// we are no longer running in the operation that the callback belongs to, start a new one because of setRedialWait
				if q == 1 {
					s.ns.SetStateSub(n, sfCanDial, nodestate.Flags{}, time.Second*10)
				} else {
					s.setRedialWait(n, queryCost, queryWaitStep)
				}
				s.ns.SetStateSub(n, nodestate.Flags{}, sfQuery, 0)
			})
		}()
	})
	return NewQueueIterator(s.ns, sfCanDial, nodestate.Flags{}, false, func(waiting bool) {
		if waiting {
			s.fillSet.SetTarget(preNegLimit)
		} else {
			s.fillSet.SetTarget(0)
		}
	})
}

// Start starts the server pool. Note that NodeStateMachine should be started first.
func (s *ServerPool) Start() {
	s.ns.Start()
	for _, iter := range s.mixSources {
		// add sources to mixer at startup because the mixer instantly tries to read them
		// which should only happen after NodeStateMachine has been started
		s.mixer.AddSource(iter)
	}
	for _, url := range s.trustedURLs {
		if node, err := enode.Parse(s.validSchemes, url); err == nil {
			s.ns.SetState(node, sfAlwaysConnect, nodestate.Flags{}, 0)
		} else {
			log.Error("Invalid trusted server URL", "url", url, "error", err)
		}
	}
	unixTime := s.unixTime()
	s.ns.Operation(func() {
		s.ns.ForEach(sfHasValue, nodestate.Flags{}, func(node *enode.Node, state nodestate.Flags) {
			s.calculateWeight(node)
			if n, ok := s.ns.GetField(node, sfiNodeHistory).(nodeHistory); ok && n.redialWaitEnd > unixTime {
				wait := n.redialWaitEnd - unixTime
				lastWait := n.redialWaitEnd - n.redialWaitStart
				if wait > lastWait {
					// if the time until expiration is larger than the last suggested
					// waiting time then the system clock was probably adjusted
					wait = lastWait
				}
				s.ns.SetStateSub(node, sfRedialWait, nodestate.Flags{}, time.Duration(wait)*time.Second)
			}
		})
	})
	atomic.StoreUint32(&s.started, 1)
}

// Stop stops the server pool
func (s *ServerPool) Stop() {
	if s.fillSet != nil {
		s.fillSet.Close()
	}
	s.ns.Operation(func() {
		s.ns.ForEach(sfConnected, nodestate.Flags{}, func(n *enode.Node, state nodestate.Flags) {
			// recalculate weight of connected nodes in order to update hasValue flag if necessary
			s.calculateWeight(n)
		})
	})
	s.ns.Stop()
	s.vt.Stop()
}

// RegisterNode implements serverPeerSubscriber
func (s *ServerPool) RegisterNode(node *enode.Node) (*NodeValueTracker, error) {
	if atomic.LoadUint32(&s.started) == 0 {
		return nil, errors.New("server pool not started yet")
	}
	nvt := s.vt.Register(node.ID())
	s.ns.Operation(func() {
		s.ns.SetStateSub(node, sfConnected, sfDialing.Or(sfWaitDialTimeout), 0)
		s.ns.SetFieldSub(node, sfiConnectedStats, nvt.RtStats())
		if node.IP().IsLoopback() {
			s.ns.SetFieldSub(node, sfiLocalAddress, node.Record())
		}
	})
	return nvt, nil
}

// UnregisterNode implements serverPeerSubscriber
func (s *ServerPool) UnregisterNode(node *enode.Node) {
	s.ns.Operation(func() {
		s.setRedialWait(node, dialCost, dialWaitStep)
		s.ns.SetStateSub(node, nodestate.Flags{}, sfConnected, 0)
		s.ns.SetFieldSub(node, sfiConnectedStats, nil)
	})
	s.vt.Unregister(node.ID())
}

// recalTimeout calculates the current recommended timeout. This value is used by
// the client as a "soft timeout" value. It also affects the service value calculation
// of individual nodes.
func (s *ServerPool) recalTimeout() {
	// Use cached result if possible, avoid recalculating too frequently.
	s.timeoutLock.RLock()
	refreshed := s.timeoutRefreshed
	s.timeoutLock.RUnlock()
	now := s.clock.Now()
	if refreshed != 0 && time.Duration(now-refreshed) < timeoutRefresh {
		return
	}
	// Cached result is stale, recalculate a new one.
	rts := s.vt.RtStats()

	// Add a fake statistic here. It is an easy way to initialize with some
	// conservative values when the database is new. As soon as we have a
	// considerable amount of real stats this small value won't matter.
	rts.Add(time.Second*2, 10, s.vt.StatsExpFactor())

	// Use either 10% failure rate timeout or twice the median response time
	// as the recommended timeout.
	timeout := minTimeout
	if t := rts.Timeout(0.1); t > timeout {
		timeout = t
	}
	if t := rts.Timeout(0.5) * 2; t > timeout {
		timeout = t
	}
	s.timeoutLock.Lock()
	if s.timeout != timeout {
		s.timeout = timeout
		s.timeWeights = TimeoutWeights(s.timeout)

		if s.suggestedTimeoutGauge != nil {
			s.suggestedTimeoutGauge.Update(int64(s.timeout / time.Millisecond))
		}
		if s.totalValueGauge != nil {
			s.totalValueGauge.Update(int64(rts.Value(s.timeWeights, s.vt.StatsExpFactor())))
		}
	}
	s.timeoutRefreshed = now
	s.timeoutLock.Unlock()
}

// GetTimeout returns the recommended request timeout.
func (s *ServerPool) GetTimeout() time.Duration {
	s.recalTimeout()
	s.timeoutLock.RLock()
	defer s.timeoutLock.RUnlock()
	return s.timeout
}

// getTimeoutAndWeight returns the recommended request timeout as well as the
// response time weight which is necessary to calculate service value.
func (s *ServerPool) getTimeoutAndWeight() (time.Duration, ResponseTimeWeights) {
	s.recalTimeout()
	s.timeoutLock.RLock()
	defer s.timeoutLock.RUnlock()
	return s.timeout, s.timeWeights
}

// addDialCost adds the given amount of dial cost to the node history and returns the current
// amount of total dial cost
func (s *ServerPool) addDialCost(n *nodeHistory, amount int64) uint64 {
	logOffset := s.vt.StatsExpirer().LogOffset(s.clock.Now())
	if amount > 0 {
		n.dialCost.Add(amount, logOffset)
	}
	totalDialCost := n.dialCost.Value(logOffset)
	if totalDialCost < dialCost {
		totalDialCost = dialCost
	}
	return totalDialCost
}

// serviceValue returns the service value accumulated in this session and in total
func (s *ServerPool) serviceValue(node *enode.Node) (sessionValue, totalValue float64) {
	nvt := s.vt.GetNode(node.ID())
	if nvt == nil {
		return 0, 0
	}
	currentStats := nvt.RtStats()
	_, timeWeights := s.getTimeoutAndWeight()
	expFactor := s.vt.StatsExpFactor()

	totalValue = currentStats.Value(timeWeights, expFactor)
	if connStats, ok := s.ns.GetField(node, sfiConnectedStats).(ResponseTimeStats); ok {
		diff := currentStats
		diff.SubStats(&connStats)
		sessionValue = diff.Value(timeWeights, expFactor)
		if s.sessionValueMeter != nil {
			s.sessionValueMeter.Mark(int64(sessionValue))
		}
	}
	return
}

// updateWeight calculates the node weight and updates the nodeWeight field and the
// hasValue flag. It also saves the node state if necessary.
// Note: this function should run inside a NodeStateMachine operation
func (s *ServerPool) updateWeight(node *enode.Node, totalValue float64, totalDialCost uint64) {
	weight := uint64(totalValue * nodeWeightMul / float64(totalDialCost))
	if weight >= nodeWeightThreshold {
		s.ns.SetStateSub(node, sfHasValue, nodestate.Flags{}, 0)
		s.ns.SetFieldSub(node, sfiNodeWeight, weight)
	} else {
		s.ns.SetStateSub(node, nodestate.Flags{}, sfHasValue, 0)
		s.ns.SetFieldSub(node, sfiNodeWeight, nil)
		s.ns.SetFieldSub(node, sfiNodeHistory, nil)
		s.ns.SetFieldSub(node, sfiLocalAddress, nil)
	}
	s.ns.Persist(node) // saved if node history or hasValue changed
}

// setRedialWait calculates and sets the redialWait timeout based on the service value
// and dial cost accumulated during the last session/attempt and in total.
// The waiting time is raised exponentially if no service value has been received in order
// to prevent dialing an unresponsive node frequently for a very long time just because it
// was useful in the past. It can still be occasionally dialed though and once it provides
// a significant amount of service value again its waiting time is quickly reduced or reset
// to the minimum.
// Note: node weight is also recalculated and updated by this function.
// Note 2: this function should run inside a NodeStateMachine operation
func (s *ServerPool) setRedialWait(node *enode.Node, addDialCost int64, waitStep float64) {
	n, _ := s.ns.GetField(node, sfiNodeHistory).(nodeHistory)
	sessionValue, totalValue := s.serviceValue(node)
	totalDialCost := s.addDialCost(&n, addDialCost)

	// if the current dial session has yielded at least the average value/dial cost ratio
	// then the waiting time should be reset to the minimum. If the session value
	// is below average but still positive then timeout is limited to the ratio of
	// average / current service value multiplied by the minimum timeout. If the attempt
	// was unsuccessful then timeout is raised exponentially without limitation.
	// Note: dialCost is used in the formula below even if dial was not attempted at all
	// because the pre-negotiation query did not return a positive result. In this case
	// the ratio has no meaning anyway and waitFactor is always raised, though in smaller
	// steps because queries are cheaper and therefore we can allow more failed attempts.
	unixTime := s.unixTime()
	plannedTimeout := float64(n.redialWaitEnd - n.redialWaitStart) // last planned redialWait timeout
	var actualWait float64                                         // actual waiting time elapsed
	if unixTime > n.redialWaitEnd {
		// the planned timeout has elapsed
		actualWait = plannedTimeout
	} else {
		// if the node was redialed earlier then we do not raise the planned timeout
		// exponentially because that could lead to the timeout rising very high in
		// a short amount of time
		// Note that in case of an early redial actualWait also includes the dial
		// timeout or connection time of the last attempt but it still serves its
		// purpose of preventing the timeout rising quicker than linearly as a function
		// of total time elapsed without a successful connection.
		actualWait = float64(unixTime - n.redialWaitStart)
	}
	// raise timeout exponentially if the last planned timeout has elapsed
	// (use at least the last planned timeout otherwise)
	nextTimeout := actualWait * waitStep
	if plannedTimeout > nextTimeout {
		nextTimeout = plannedTimeout
	}
	// we reduce the waiting time if the server has provided service value during the
	// connection (but never under the minimum)
	a := totalValue * dialCost * float64(minRedialWait)
	b := float64(totalDialCost) * sessionValue
	if a < b*nextTimeout {
		nextTimeout = a / b
	}
	if nextTimeout < minRedialWait {
		nextTimeout = minRedialWait
	}
	wait := time.Duration(float64(time.Second) * nextTimeout)
	if wait < waitThreshold {
		n.redialWaitStart = unixTime
		n.redialWaitEnd = unixTime + int64(nextTimeout)
		s.ns.SetFieldSub(node, sfiNodeHistory, n)
		s.ns.SetStateSub(node, sfRedialWait, nodestate.Flags{}, wait)
		s.updateWeight(node, totalValue, totalDialCost)
	} else {
		// discard known node statistics if waiting time is very long because the node
		// hasn't been responsive for a very long time
		s.ns.SetFieldSub(node, sfiNodeHistory, nil)
		s.ns.SetFieldSub(node, sfiNodeWeight, nil)
		s.ns.SetStateSub(node, nodestate.Flags{}, sfHasValue, 0)
	}
}

// calculateWeight calculates and sets the node weight without altering the node history.
// This function should be called during startup and shutdown only, otherwise setRedialWait
// will keep the weights updated as the underlying statistics are adjusted.
// Note: this function should run inside a NodeStateMachine operation
func (s *ServerPool) calculateWeight(node *enode.Node) {
	n, _ := s.ns.GetField(node, sfiNodeHistory).(nodeHistory)
	_, totalValue := s.serviceValue(node)
	totalDialCost := s.addDialCost(&n, 0)
	s.updateWeight(node, totalValue, totalDialCost)
}

// API returns the vflux client API
func (s *ServerPool) API() *PrivateClientAPI {
	return NewPrivateClientAPI(s.vt)
}

type dummyIdentity enode.ID

func (id dummyIdentity) Verify(r *enr.Record, sig []byte) error { return nil }
func (id dummyIdentity) NodeAddr(r *enr.Record) []byte          { return id[:] }

// DialNode replaces the given enode with a locally generated one containing the ENR
// stored in the sfiLocalAddress field if present. This workaround ensures that nodes
// on the local network can be dialed at the local address if a connection has been
// successfully established previously.
// Note that NodeStateMachine always remembers the enode with the latest version of
// the remote signed ENR. ENR filtering should be performed on that version while
// dialNode should be used for dialing the node over TCP or UDP.
func (s *ServerPool) DialNode(n *enode.Node) *enode.Node {
	if enr, ok := s.ns.GetField(n, sfiLocalAddress).(*enr.Record); ok {
		n, _ := enode.New(dummyIdentity(n.ID()), enr)
		return n
	}
	return n
}

// Persist immediately stores the state of a node in the node database
func (s *ServerPool) Persist(n *enode.Node) {
	s.ns.Persist(n)
}
