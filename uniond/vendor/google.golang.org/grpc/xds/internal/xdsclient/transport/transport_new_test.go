/*
 *
 * Copyright 2022 gRPC authors.
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
 */

package transport_test

import (
	"strings"
	"testing"

	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal/xdsclient/transport"

	v3corepb "github.com/envoyproxy/go-control-plane/envoy/config/core/v3"
)

// TestNew covers that New() returns an error if the input *ServerConfig
// contains invalid content.
func (s) TestNew(t *testing.T) {
	serverCfg, err := bootstrap.ServerConfigForTesting(bootstrap.ServerConfigTestingOptions{URI: "server-address"})
	if err != nil {
		t.Fatalf("Failed to create server config for testing: %v", err)
	}

	tests := []struct {
		name       string
		opts       transport.Options
		wantErrStr string
	}{
		{
			name: "missing onRecv handler",
			opts: transport.Options{
				ServerCfg: serverCfg,
				NodeProto: &v3corepb.Node{},
			},
			wantErrStr: "missing OnRecv callback handler when creating a new transport",
		},
		{
			name: "missing onError handler",
			opts: transport.Options{
				ServerCfg:     serverCfg,
				NodeProto:     &v3corepb.Node{},
				OnRecvHandler: noopRecvHandler, // No data model layer validation.
				OnSendHandler: func(*transport.ResourceSendInfo) {},
			},
			wantErrStr: "missing OnError callback handler when creating a new transport",
		},

		{
			name: "missing onSend handler",
			opts: transport.Options{
				ServerCfg:      serverCfg,
				NodeProto:      &v3corepb.Node{},
				OnRecvHandler:  noopRecvHandler, // No data model layer validation.
				OnErrorHandler: func(error) {},
			},
			wantErrStr: "missing OnSend callback handler when creating a new transport",
		},
		{
			name: "happy case",
			opts: transport.Options{
				ServerCfg:      serverCfg,
				NodeProto:      &v3corepb.Node{},
				OnRecvHandler:  noopRecvHandler, // No data model layer validation.
				OnErrorHandler: func(error) {},
				OnSendHandler:  func(*transport.ResourceSendInfo) {},
			},
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			c, err := transport.New(test.opts)
			defer func() {
				if c != nil {
					c.Close()
				}
			}()
			if (err != nil) != (test.wantErrStr != "") {
				t.Fatalf("New(%+v) = %v, wantErr: %v", test.opts, err, test.wantErrStr)
			}
			if err != nil && !strings.Contains(err.Error(), test.wantErrStr) {
				t.Fatalf("New(%+v) = %v, wantErr: %v", test.opts, err, test.wantErrStr)
			}
		})
	}
}
