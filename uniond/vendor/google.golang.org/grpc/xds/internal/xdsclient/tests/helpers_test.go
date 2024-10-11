/*
 *
 * Copyright 2024 gRPC authors.
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

package xdsclient_test

import (
	"fmt"
	"strings"
	"testing"
	"time"

	"google.golang.org/grpc/internal/grpctest"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

const (
	defaultTestWatchExpiryTimeout   = 500 * time.Millisecond
	defaultTestIdleAuthorityTimeout = 50 * time.Millisecond
	defaultTestTimeout              = 10 * time.Second
	defaultTestShortTimeout         = 10 * time.Millisecond // For events expected to *not* happen.

	ldsName         = "xdsclient-test-lds-resource"
	rdsName         = "xdsclient-test-rds-resource"
	cdsName         = "xdsclient-test-cds-resource"
	edsName         = "xdsclient-test-eds-resource"
	ldsNameNewStyle = "xdstp:///envoy.config.listener.v3.Listener/xdsclient-test-lds-resource"
	rdsNameNewStyle = "xdstp:///envoy.config.route.v3.RouteConfiguration/xdsclient-test-rds-resource"
	cdsNameNewStyle = "xdstp:///envoy.config.cluster.v3.Cluster/xdsclient-test-cds-resource"
	edsNameNewStyle = "xdstp:///envoy.config.endpoint.v3.ClusterLoadAssignment/xdsclient-test-eds-resource"
)

func makeAuthorityName(name string) string {
	segs := strings.Split(name, "/")
	return strings.Join(segs, "")
}

func makeNewStyleLDSName(authority string) string {
	return fmt.Sprintf("xdstp://%s/envoy.config.listener.v3.Listener/xdsclient-test-lds-resource", authority)
}

func makeNewStyleRDSName(authority string) string {
	return fmt.Sprintf("xdstp://%s/envoy.config.route.v3.RouteConfiguration/xdsclient-test-rds-resource", authority)
}

func makeNewStyleCDSName(authority string) string {
	return fmt.Sprintf("xdstp://%s/envoy.config.cluster.v3.Cluster/xdsclient-test-cds-resource", authority)
}

func makeNewStyleEDSName(authority string) string {
	return fmt.Sprintf("xdstp://%s/envoy.config.endpoint.v3.ClusterLoadAssignment/xdsclient-test-eds-resource", authority)
}
