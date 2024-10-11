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

package google

import (
	"context"
	"testing"

	"google.golang.org/grpc/credentials"
	icredentials "google.golang.org/grpc/internal/credentials"
	"google.golang.org/grpc/internal/xds"
	"google.golang.org/grpc/resolver"
)

func (s) TestIsDirectPathCluster(t *testing.T) {
	c := func(cluster string) context.Context {
		return icredentials.NewClientHandshakeInfoContext(context.Background(), credentials.ClientHandshakeInfo{
			Attributes: xds.SetXDSHandshakeClusterName(resolver.Address{}, cluster).Attributes,
		})
	}

	testCases := []struct {
		name string
		ctx  context.Context
		want bool
	}{
		{"not an xDS cluster", context.Background(), false},
		{"cfe", c("google_cfe_bigtable.googleapis.com"), false},
		{"non-cfe", c("google_bigtable.googleapis.com"), true},
		{"starts with xdstp but not cfe format", c("xdstp:google_cfe_bigtable.googleapis.com"), true},
		{"no authority", c("xdstp:///envoy.config.cluster.v3.Cluster/google_cfe_"), true},
		{"wrong authority", c("xdstp://foo.bar/envoy.config.cluster.v3.Cluster/google_cfe_"), true},
		{"xdstp CFE", c("xdstp://traffic-director-c2p.xds.googleapis.com/envoy.config.cluster.v3.Cluster/google_cfe_"), false},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			if got := isDirectPathCluster(tc.ctx); got != tc.want {
				t.Errorf("isDirectPathCluster(_) = %v; want %v", got, tc.want)
			}
		})
	}
}
