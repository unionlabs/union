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

package clusterresolver

import (
	"encoding/json"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/balancer/roundrobin"
	iserviceconfig "google.golang.org/grpc/internal/serviceconfig"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal/balancer/outlierdetection"
	"google.golang.org/grpc/xds/internal/balancer/ringhash"
)

func TestDiscoveryMechanismTypeMarshalJSON(t *testing.T) {
	tests := []struct {
		name string
		typ  DiscoveryMechanismType
		want string
	}{
		{
			name: "eds",
			typ:  DiscoveryMechanismTypeEDS,
			want: `"EDS"`,
		},
		{
			name: "dns",
			typ:  DiscoveryMechanismTypeLogicalDNS,
			want: `"LOGICAL_DNS"`,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got, err := json.Marshal(tt.typ); err != nil || string(got) != tt.want {
				t.Fatalf("DiscoveryMechanismTypeEDS.MarshalJSON() = (%v, %v), want (%s, nil)", string(got), err, tt.want)
			}
		})
	}
}
func TestDiscoveryMechanismTypeUnmarshalJSON(t *testing.T) {
	tests := []struct {
		name    string
		js      string
		want    DiscoveryMechanismType
		wantErr bool
	}{
		{
			name: "eds",
			js:   `"EDS"`,
			want: DiscoveryMechanismTypeEDS,
		},
		{
			name: "dns",
			js:   `"LOGICAL_DNS"`,
			want: DiscoveryMechanismTypeLogicalDNS,
		},
		{
			name:    "error",
			js:      `"1234"`,
			wantErr: true,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			var got DiscoveryMechanismType
			err := json.Unmarshal([]byte(tt.js), &got)
			if (err != nil) != tt.wantErr {
				t.Fatalf("DiscoveryMechanismTypeEDS.UnmarshalJSON() error = %v, wantErr %v", err, tt.wantErr)
			}
			if diff := cmp.Diff(got, tt.want); diff != "" {
				t.Fatalf("DiscoveryMechanismTypeEDS.UnmarshalJSON() got unexpected output, diff (-got +want): %v", diff)
			}
		})
	}
}

const (
	testJSONConfig1 = `{
  "discoveryMechanisms": [{
    "cluster": "test-cluster-name",
    "lrsLoadReportingServer": {
      "server_uri": "trafficdirector.googleapis.com:443",
      "channel_creds": [ { "type": "google_default" } ]
    },
    "maxConcurrentRequests": 314,
    "type": "EDS",
    "edsServiceName": "test-eds-service-name",
    "outlierDetection": {}
  }],
  "xdsLbPolicy":[{"round_robin":{}}]
}`
	testJSONConfig2 = `{
  "discoveryMechanisms": [{
    "cluster": "test-cluster-name",
    "lrsLoadReportingServer": {
      "server_uri": "trafficdirector.googleapis.com:443",
      "channel_creds": [ { "type": "google_default" } ]
    },
    "maxConcurrentRequests": 314,
    "type": "EDS",
    "edsServiceName": "test-eds-service-name",
    "outlierDetection": {}
  },{
    "type": "LOGICAL_DNS",
    "outlierDetection": {}
  }],
  "xdsLbPolicy":[{"round_robin":{}}]
}`
	testJSONConfig3 = `{
  "discoveryMechanisms": [{
    "cluster": "test-cluster-name",
    "lrsLoadReportingServer": {
      "server_uri": "trafficdirector.googleapis.com:443",
      "channel_creds": [ { "type": "google_default" } ]
    },
    "maxConcurrentRequests": 314,
    "type": "EDS",
    "edsServiceName": "test-eds-service-name",
    "outlierDetection": {}
  }],
  "xdsLbPolicy":[{"round_robin":{}}]
}`
	testJSONConfig4 = `{
  "discoveryMechanisms": [{
    "cluster": "test-cluster-name",
    "lrsLoadReportingServer": {
      "server_uri": "trafficdirector.googleapis.com:443",
      "channel_creds": [ { "type": "google_default" } ]
    },
    "maxConcurrentRequests": 314,
    "type": "EDS",
    "edsServiceName": "test-eds-service-name",
    "outlierDetection": {}
  }],
  "xdsLbPolicy":[{"ring_hash_experimental":{}}]
}`
	testJSONConfig5 = `{
  "discoveryMechanisms": [{
    "cluster": "test-cluster-name",
    "lrsLoadReportingServer": {
      "server_uri": "trafficdirector.googleapis.com:443",
      "channel_creds": [ { "type": "google_default" } ]
    },
    "maxConcurrentRequests": 314,
    "type": "EDS",
    "edsServiceName": "test-eds-service-name",
    "outlierDetection": {}
  }],
  "xdsLbPolicy":[{"round_robin":{}}]
}`
)

func TestParseConfig(t *testing.T) {
	testLRSServerConfig, err := bootstrap.ServerConfigForTesting(bootstrap.ServerConfigTestingOptions{
		URI:          "trafficdirector.googleapis.com:443",
		ChannelCreds: []bootstrap.ChannelCreds{{Type: "google_default"}},
	})
	if err != nil {
		t.Fatalf("Failed to create LRS server config for testing: %v", err)
	}
	tests := []struct {
		name    string
		js      string
		want    *LBConfig
		wantErr bool
	}{
		{
			name:    "empty json",
			js:      "",
			want:    nil,
			wantErr: true,
		},
		{
			name: "OK with one discovery mechanism",
			js:   testJSONConfig1,
			want: &LBConfig{
				DiscoveryMechanisms: []DiscoveryMechanism{
					{
						Cluster:               testClusterName,
						LoadReportingServer:   testLRSServerConfig,
						MaxConcurrentRequests: newUint32(testMaxRequests),
						Type:                  DiscoveryMechanismTypeEDS,
						EDSServiceName:        testEDSService,
						outlierDetection: outlierdetection.LBConfig{
							Interval:           iserviceconfig.Duration(10 * time.Second), // default interval
							BaseEjectionTime:   iserviceconfig.Duration(30 * time.Second),
							MaxEjectionTime:    iserviceconfig.Duration(300 * time.Second),
							MaxEjectionPercent: 10,
							// sre and fpe are both nil
						},
					},
				},
				xdsLBPolicy: iserviceconfig.BalancerConfig{ // do we want to make this not pointer
					Name:   roundrobin.Name,
					Config: nil,
				},
			},
			wantErr: false,
		},
		{
			name: "OK with multiple discovery mechanisms",
			js:   testJSONConfig2,
			want: &LBConfig{
				DiscoveryMechanisms: []DiscoveryMechanism{
					{
						Cluster:               testClusterName,
						LoadReportingServer:   testLRSServerConfig,
						MaxConcurrentRequests: newUint32(testMaxRequests),
						Type:                  DiscoveryMechanismTypeEDS,
						EDSServiceName:        testEDSService,
						outlierDetection: outlierdetection.LBConfig{
							Interval:           iserviceconfig.Duration(10 * time.Second), // default interval
							BaseEjectionTime:   iserviceconfig.Duration(30 * time.Second),
							MaxEjectionTime:    iserviceconfig.Duration(300 * time.Second),
							MaxEjectionPercent: 10,
							// sre and fpe are both nil
						},
					},
					{
						Type: DiscoveryMechanismTypeLogicalDNS,
						outlierDetection: outlierdetection.LBConfig{
							Interval:           iserviceconfig.Duration(10 * time.Second), // default interval
							BaseEjectionTime:   iserviceconfig.Duration(30 * time.Second),
							MaxEjectionTime:    iserviceconfig.Duration(300 * time.Second),
							MaxEjectionPercent: 10,
							// sre and fpe are both nil
						},
					},
				},
				xdsLBPolicy: iserviceconfig.BalancerConfig{
					Name:   roundrobin.Name,
					Config: nil,
				},
			},
			wantErr: false,
		},
		{
			name: "OK with picking policy round_robin",
			js:   testJSONConfig3,
			want: &LBConfig{
				DiscoveryMechanisms: []DiscoveryMechanism{
					{
						Cluster:               testClusterName,
						LoadReportingServer:   testLRSServerConfig,
						MaxConcurrentRequests: newUint32(testMaxRequests),
						Type:                  DiscoveryMechanismTypeEDS,
						EDSServiceName:        testEDSService,
						outlierDetection: outlierdetection.LBConfig{
							Interval:           iserviceconfig.Duration(10 * time.Second), // default interval
							BaseEjectionTime:   iserviceconfig.Duration(30 * time.Second),
							MaxEjectionTime:    iserviceconfig.Duration(300 * time.Second),
							MaxEjectionPercent: 10,
							// sre and fpe are both nil
						},
					},
				},
				xdsLBPolicy: iserviceconfig.BalancerConfig{
					Name:   roundrobin.Name,
					Config: nil,
				},
			},
			wantErr: false,
		},
		{
			name: "OK with picking policy ring_hash",
			js:   testJSONConfig4,
			want: &LBConfig{
				DiscoveryMechanisms: []DiscoveryMechanism{
					{
						Cluster:               testClusterName,
						LoadReportingServer:   testLRSServerConfig,
						MaxConcurrentRequests: newUint32(testMaxRequests),
						Type:                  DiscoveryMechanismTypeEDS,
						EDSServiceName:        testEDSService,
						outlierDetection: outlierdetection.LBConfig{
							Interval:           iserviceconfig.Duration(10 * time.Second), // default interval
							BaseEjectionTime:   iserviceconfig.Duration(30 * time.Second),
							MaxEjectionTime:    iserviceconfig.Duration(300 * time.Second),
							MaxEjectionPercent: 10,
							// sre and fpe are both nil
						},
					},
				},
				xdsLBPolicy: iserviceconfig.BalancerConfig{
					Name:   ringhash.Name,
					Config: &ringhash.LBConfig{MinRingSize: 1024, MaxRingSize: 4096}, // Ringhash LB config with default min and max.
				},
			},
			wantErr: false,
		},
		{
			name: "noop-outlier-detection",
			js:   testJSONConfig5,
			want: &LBConfig{
				DiscoveryMechanisms: []DiscoveryMechanism{
					{
						Cluster:               testClusterName,
						LoadReportingServer:   testLRSServerConfig,
						MaxConcurrentRequests: newUint32(testMaxRequests),
						Type:                  DiscoveryMechanismTypeEDS,
						EDSServiceName:        testEDSService,
						outlierDetection: outlierdetection.LBConfig{
							Interval:           iserviceconfig.Duration(10 * time.Second), // default interval
							BaseEjectionTime:   iserviceconfig.Duration(30 * time.Second),
							MaxEjectionTime:    iserviceconfig.Duration(300 * time.Second),
							MaxEjectionPercent: 10,
							// sre and fpe are both nil
						},
					},
				},
				xdsLBPolicy: iserviceconfig.BalancerConfig{
					Name:   roundrobin.Name,
					Config: nil,
				},
			},
			wantErr: false,
		},
	}
	for _, tt := range tests {
		b := balancer.Get(Name)
		if b == nil {
			t.Fatalf("LB policy %q not registered", Name)
		}
		cfgParser, ok := b.(balancer.ConfigParser)
		if !ok {
			t.Fatalf("LB policy %q does not support config parsing", Name)
		}
		t.Run(tt.name, func(t *testing.T) {
			got, err := cfgParser.ParseConfig([]byte(tt.js))
			if (err != nil) != tt.wantErr {
				t.Fatalf("parseConfig() error = %v, wantErr %v", err, tt.wantErr)
			}
			if tt.wantErr {
				return
			}
			if diff := cmp.Diff(got, tt.want, cmp.AllowUnexported(LBConfig{}), cmpopts.IgnoreFields(LBConfig{}, "XDSLBPolicy")); diff != "" {
				t.Errorf("parseConfig() got unexpected output, diff (-got +want): %v", diff)
			}
		})
	}
}

func newUint32(i uint32) *uint32 {
	return &i
}
