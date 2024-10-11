/*
 *
 * Copyright 2023 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

// Package leastrequest implements a least request load balancer.
package leastrequest

import (
	"encoding/json"
	"fmt"
	"math/rand"
	"sync/atomic"

	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/balancer/base"
	"google.golang.org/grpc/grpclog"
	"google.golang.org/grpc/serviceconfig"
)

// randuint32 is a global to stub out in tests.
var randuint32 = rand.Uint32

// Name is the name of the least request balancer.
const Name = "least_request_experimental"

var logger = grpclog.Component("least-request")

func init() {
	balancer.Register(bb{})
}

// LBConfig is the balancer config for least_request_experimental balancer.
type LBConfig struct {
	serviceconfig.LoadBalancingConfig `json:"-"`

	// ChoiceCount is the number of random SubConns to sample to find the one
	// with the fewest outstanding requests. If unset, defaults to 2. If set to
	// < 2, the config will be rejected, and if set to > 10, will become 10.
	ChoiceCount uint32 `json:"choiceCount,omitempty"`
}

type bb struct{}

func (bb) ParseConfig(s json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
	lbConfig := &LBConfig{
		ChoiceCount: 2,
	}
	if err := json.Unmarshal(s, lbConfig); err != nil {
		return nil, fmt.Errorf("least-request: unable to unmarshal LBConfig: %v", err)
	}
	// "If `choice_count < 2`, the config will be rejected." - A48
	if lbConfig.ChoiceCount < 2 { // sweet
		return nil, fmt.Errorf("least-request: lbConfig.choiceCount: %v, must be >= 2", lbConfig.ChoiceCount)
	}
	// "If a LeastRequestLoadBalancingConfig with a choice_count > 10 is
	// received, the least_request_experimental policy will set choice_count =
	// 10." - A48
	if lbConfig.ChoiceCount > 10 {
		lbConfig.ChoiceCount = 10
	}
	return lbConfig, nil
}

func (bb) Name() string {
	return Name
}

func (bb) Build(cc balancer.ClientConn, bOpts balancer.BuildOptions) balancer.Balancer {
	b := &leastRequestBalancer{scRPCCounts: make(map[balancer.SubConn]*atomic.Int32)}
	baseBuilder := base.NewBalancerBuilder(Name, b, base.Config{HealthCheck: true})
	b.Balancer = baseBuilder.Build(cc, bOpts)
	return b
}

type leastRequestBalancer struct {
	// Embeds balancer.Balancer because needs to intercept UpdateClientConnState
	// to learn about choiceCount.
	balancer.Balancer

	choiceCount uint32
	scRPCCounts map[balancer.SubConn]*atomic.Int32 // Hold onto RPC counts to keep track for subsequent picker updates.
}

func (lrb *leastRequestBalancer) UpdateClientConnState(s balancer.ClientConnState) error {
	lrCfg, ok := s.BalancerConfig.(*LBConfig)
	if !ok {
		logger.Errorf("least-request: received config with unexpected type %T: %v", s.BalancerConfig, s.BalancerConfig)
		return balancer.ErrBadResolverState
	}

	lrb.choiceCount = lrCfg.ChoiceCount
	return lrb.Balancer.UpdateClientConnState(s)
}

type scWithRPCCount struct {
	sc      balancer.SubConn
	numRPCs *atomic.Int32
}

func (lrb *leastRequestBalancer) Build(info base.PickerBuildInfo) balancer.Picker {
	if logger.V(2) {
		logger.Infof("least-request: Build called with info: %v", info)
	}
	if len(info.ReadySCs) == 0 {
		return base.NewErrPicker(balancer.ErrNoSubConnAvailable)
	}

	for sc := range lrb.scRPCCounts {
		if _, ok := info.ReadySCs[sc]; !ok { // If no longer ready, no more need for the ref to count active RPCs.
			delete(lrb.scRPCCounts, sc)
		}
	}

	// Create new refs if needed.
	for sc := range info.ReadySCs {
		if _, ok := lrb.scRPCCounts[sc]; !ok {
			lrb.scRPCCounts[sc] = new(atomic.Int32)
		}
	}

	// Copy refs to counters into picker.
	scs := make([]scWithRPCCount, 0, len(info.ReadySCs))
	for sc := range info.ReadySCs {
		scs = append(scs, scWithRPCCount{
			sc:      sc,
			numRPCs: lrb.scRPCCounts[sc], // guaranteed to be present due to algorithm
		})
	}

	return &picker{
		choiceCount: lrb.choiceCount,
		subConns:    scs,
	}
}

type picker struct {
	// choiceCount is the number of random SubConns to find the one with
	// the least request.
	choiceCount uint32
	// Built out when receives list of ready RPCs.
	subConns []scWithRPCCount
}

func (p *picker) Pick(balancer.PickInfo) (balancer.PickResult, error) {
	var pickedSC *scWithRPCCount
	var pickedSCNumRPCs int32
	for i := 0; i < int(p.choiceCount); i++ {
		index := randuint32() % uint32(len(p.subConns))
		sc := p.subConns[index]
		n := sc.numRPCs.Load()
		if pickedSC == nil || n < pickedSCNumRPCs {
			pickedSC = &sc
			pickedSCNumRPCs = n
		}
	}
	// "The counter for a subchannel should be atomically incremented by one
	// after it has been successfully picked by the picker." - A48
	pickedSC.numRPCs.Add(1)
	// "the picker should add a callback for atomically decrementing the
	// subchannel counter once the RPC finishes (regardless of Status code)." -
	// A48.
	done := func(balancer.DoneInfo) {
		pickedSC.numRPCs.Add(-1)
	}
	return balancer.PickResult{
		SubConn: pickedSC.sc,
		Done:    done,
	}, nil
}
