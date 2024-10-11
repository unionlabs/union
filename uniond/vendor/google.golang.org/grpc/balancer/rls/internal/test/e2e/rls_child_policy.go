/*
 *
 * Copyright 2021 gRPC authors.
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

package e2e

import (
	"encoding/json"
	"errors"
	"fmt"

	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/balancer/pickfirst"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/serviceconfig"
)

const (
	// RLSChildPolicyTargetNameField is a top-level field name to add to the child
	// policy's config, whose value is set to the target for the child policy.
	RLSChildPolicyTargetNameField = "Backend"
	// RLSChildPolicyBadTarget is a value which is considered a bad target by the
	// child policy. This is useful to test bad child policy configuration.
	RLSChildPolicyBadTarget = "bad-target"
)

// ErrParseConfigBadTarget is the error returned from ParseConfig when the
// backend field is set to RLSChildPolicyBadTarget.
var ErrParseConfigBadTarget = errors.New("backend field set to RLSChildPolicyBadTarget")

// BalancerFuncs is a set of callbacks which get invoked when the corresponding
// method on the child policy is invoked.
type BalancerFuncs struct {
	UpdateClientConnState func(cfg *RLSChildPolicyConfig) error
	Close                 func()
}

// RegisterRLSChildPolicy registers a balancer builder with the given name, to
// be used as a child policy for the RLS LB policy.
//
// The child policy uses a pickfirst balancer under the hood to send all traffic
// to the single backend specified by the `RLSChildPolicyTargetNameField` field
// in its configuration which looks like: {"Backend": "Backend-address"}.
func RegisterRLSChildPolicy(name string, bf *BalancerFuncs) {
	balancer.Register(bb{name: name, bf: bf})
}

type bb struct {
	name string
	bf   *BalancerFuncs
}

func (bb bb) Name() string { return bb.name }

func (bb bb) Build(cc balancer.ClientConn, opts balancer.BuildOptions) balancer.Balancer {
	pf := balancer.Get(pickfirst.Name)
	b := &bal{
		Balancer: pf.Build(cc, opts),
		bf:       bb.bf,
		done:     grpcsync.NewEvent(),
	}
	go b.run()
	return b
}

func (bb bb) ParseConfig(c json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
	cfg := &RLSChildPolicyConfig{}
	if err := json.Unmarshal(c, cfg); err != nil {
		return nil, err
	}
	if cfg.Backend == RLSChildPolicyBadTarget {
		return nil, ErrParseConfigBadTarget
	}
	return cfg, nil
}

type bal struct {
	balancer.Balancer
	bf   *BalancerFuncs
	done *grpcsync.Event
}

// RLSChildPolicyConfig is the LB config for the test child policy.
type RLSChildPolicyConfig struct {
	serviceconfig.LoadBalancingConfig
	Backend string // The target for which this child policy was created.
	Random  string // A random field to test child policy config changes.
}

func (b *bal) UpdateClientConnState(c balancer.ClientConnState) error {
	cfg, ok := c.BalancerConfig.(*RLSChildPolicyConfig)
	if !ok {
		return fmt.Errorf("received balancer config of type %T, want %T", c.BalancerConfig, &RLSChildPolicyConfig{})
	}
	if b.bf != nil && b.bf.UpdateClientConnState != nil {
		b.bf.UpdateClientConnState(cfg)
	}
	return b.Balancer.UpdateClientConnState(balancer.ClientConnState{
		ResolverState: resolver.State{Addresses: []resolver.Address{{Addr: cfg.Backend}}},
	})
}

func (b *bal) Close() {
	b.Balancer.Close()
	if b.bf != nil && b.bf.Close != nil {
		b.bf.Close()
	}
	b.done.Fire()
}

// run is a dummy goroutine to make sure that child policies are closed at the
// end of tests. If they are not closed, these goroutines will be picked up by
// the leak checker and tests will fail.
func (b *bal) run() {
	<-b.done.Done()
}
