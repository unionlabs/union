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
	"errors"
	"fmt"

	"google.golang.org/grpc/balancer"
	rlspb "google.golang.org/grpc/internal/proto/grpc_lookup_v1"
	internalserviceconfig "google.golang.org/grpc/internal/serviceconfig"
	"google.golang.org/grpc/serviceconfig"

	"google.golang.org/protobuf/encoding/protojson"
)

// RLSConfig is a utility type to build service config for the RLS LB policy.
type RLSConfig struct {
	RouteLookupConfig                *rlspb.RouteLookupConfig
	RouteLookupChannelServiceConfig  string
	ChildPolicy                      *internalserviceconfig.BalancerConfig
	ChildPolicyConfigTargetFieldName string
}

// ServiceConfigJSON generates service config with a load balancing config
// corresponding to the RLS LB policy.
func (c *RLSConfig) ServiceConfigJSON() (string, error) {
	m := protojson.MarshalOptions{
		Multiline:     true,
		Indent:        "  ",
		UseProtoNames: true,
	}
	routeLookupCfg, err := m.Marshal(c.RouteLookupConfig)
	if err != nil {
		return "", err
	}
	childPolicy, err := c.ChildPolicy.MarshalJSON()
	if err != nil {
		return "", err
	}

	return fmt.Sprintf(`
{
  "loadBalancingConfig": [
    {
      "rls_experimental": {
        "routeLookupConfig": %s,
				"routeLookupChannelServiceConfig": %s,
        "childPolicy": %s,
        "childPolicyConfigTargetFieldName": %q
      }
    }
  ]
}`, string(routeLookupCfg), c.RouteLookupChannelServiceConfig, string(childPolicy), c.ChildPolicyConfigTargetFieldName), nil
}

// LoadBalancingConfig generates load balancing config which can used as part of
// a ClientConnState update to the RLS LB policy.
func (c *RLSConfig) LoadBalancingConfig() (serviceconfig.LoadBalancingConfig, error) {
	m := protojson.MarshalOptions{
		Multiline:     true,
		Indent:        "  ",
		UseProtoNames: true,
	}
	routeLookupCfg, err := m.Marshal(c.RouteLookupConfig)
	if err != nil {
		return nil, err
	}
	childPolicy, err := c.ChildPolicy.MarshalJSON()
	if err != nil {
		return nil, err
	}
	lbConfigJSON := fmt.Sprintf(`
{
  "routeLookupConfig": %s,
  "routeLookupChannelServiceConfig": %s,
  "childPolicy": %s,
  "childPolicyConfigTargetFieldName": %q
}`, string(routeLookupCfg), c.RouteLookupChannelServiceConfig, string(childPolicy), c.ChildPolicyConfigTargetFieldName)

	builder := balancer.Get("rls_experimental")
	if builder == nil {
		return nil, errors.New("balancer builder not found for RLS LB policy")
	}
	parser := builder.(balancer.ConfigParser)
	return parser.ParseConfig([]byte(lbConfigJSON))
}
