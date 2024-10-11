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

package server

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"github.com/google/uuid"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/testutils/xds/e2e"
	"google.golang.org/grpc/xds/internal/xdsclient"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource/version"

	v3routepb "github.com/envoyproxy/go-control-plane/envoy/config/route/v3"
	v3discoverypb "github.com/envoyproxy/go-control-plane/envoy/service/discovery/v3"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

const (
	defaultTestTimeout      = 10 * time.Second
	defaultTestShortTimeout = 10 * time.Millisecond // For events expected to *not* happen.
)

const (
	listenerName = "listener"
	clusterName  = "cluster"

	route1 = "route1"
	route2 = "route2"
	route3 = "route3"
	route4 = "route4"
)

// xdsSetupForTests performs the following setup actions:
//   - spins up an xDS management server
//   - creates an xDS client with a bootstrap configuration pointing to the above
//     management server
//
// Returns the following:
// - a reference to the management server
// - nodeID to use when pushing resources to the management server
// - a channel to read lds resource names received by the management server
// - a channel to read rds resource names received by the management server
// - an xDS client to pass to the rdsHandler under test
func xdsSetupForTests(t *testing.T) (*e2e.ManagementServer, string, chan []string, chan []string, xdsclient.XDSClient) {
	t.Helper()

	ldsNamesCh := make(chan []string, 1)
	rdsNamesCh := make(chan []string, 1)

	// Setup the management server to push the requested route configuration
	// resource names on to a channel for the test to inspect.
	mgmtServer := e2e.StartManagementServer(t, e2e.ManagementServerOptions{
		OnStreamRequest: func(_ int64, req *v3discoverypb.DiscoveryRequest) error {
			switch req.GetTypeUrl() {
			case version.V3ListenerURL: // Waits on the listener, and route config below...
				select {
				case <-ldsNamesCh:
				default:
				}
				select {
				case ldsNamesCh <- req.GetResourceNames():
				default:
				}
			case version.V3RouteConfigURL: // waits on route config names here...
				select {
				case <-rdsNamesCh:
				default:
				}
				select {
				case rdsNamesCh <- req.GetResourceNames():
				default:
				}
			default:
				return fmt.Errorf("unexpected resources %v of type %q requested", req.GetResourceNames(), req.GetTypeUrl())
			}
			return nil
		},
		AllowResourceSubset: true,
	})

	// Create bootstrap configuration pointing to the above management server.
	nodeID := uuid.New().String()
	bootstrapContents := e2e.DefaultBootstrapContents(t, nodeID, mgmtServer.Address)

	xdsC, cancel, err := xdsclient.NewForTesting(xdsclient.OptionsForTesting{
		Name:     t.Name(),
		Contents: bootstrapContents,
	})
	if err != nil {
		t.Fatal(err)
	}
	t.Cleanup(cancel)

	return mgmtServer, nodeID, ldsNamesCh, rdsNamesCh, xdsC
}

// Waits for the wantNames to be pushed on to namesCh. Fails the test by calling
// t.Fatal if the context expires before that.
func waitForResourceNames(ctx context.Context, t *testing.T, namesCh chan []string, wantNames []string) {
	t.Helper()

	for ; ctx.Err() == nil; <-time.After(defaultTestShortTimeout) {
		select {
		case <-ctx.Done():
		case gotNames := <-namesCh:
			if cmp.Equal(gotNames, wantNames, cmpopts.EquateEmpty(), cmpopts.SortSlices(func(s1, s2 string) bool { return s1 < s2 })) {
				return
			}
			t.Logf("Received resource names %v, want %v", gotNames, wantNames)
		}
	}
	t.Fatalf("Timeout waiting for resource to be requested from the management server")
}

func routeConfigResourceForName(name string) *v3routepb.RouteConfiguration {
	return e2e.RouteConfigResourceWithOptions(e2e.RouteConfigOptions{
		RouteConfigName:      name,
		ListenerName:         listenerName,
		ClusterSpecifierType: e2e.RouteConfigClusterSpecifierTypeCluster,
		ClusterName:          clusterName,
	})
}

type testCallbackVerify struct {
	ch chan callbackStruct
}

type callbackStruct struct {
	routeName string
	rwu       rdsWatcherUpdate
}

func (tcv *testCallbackVerify) testCallback(routeName string, rwu rdsWatcherUpdate) {
	tcv.ch <- callbackStruct{routeName: routeName, rwu: rwu}
}

func verifyRouteName(ctx context.Context, t *testing.T, ch chan callbackStruct, want callbackStruct) {
	t.Helper()
	select {
	case got := <-ch:
		if diff := cmp.Diff(got.routeName, want.routeName); diff != "" {
			t.Fatalf("unexpected update received (-got, +want):%v, want: %v", got, want)
		}
	case <-ctx.Done():
		t.Fatalf("timeout waiting for callback")
	}
}

// TestRDSHandler tests the RDS Handler. It first configures the rds handler to
// watch route 1 and 2. Before receiving both RDS updates, it should not be
// ready, but after receiving both, it should be ready. It then tells the rds
// handler to watch route 1 2 and 3. It should not be ready until it receives
// route3 from the management server. It then configures the rds handler to
// watch route 1 and 3. It should immediately be ready. It then configures the
// rds handler to watch route 1 and 4. It should not be ready until it receives
// an rds update for route 4.
func (s) TestRDSHandler(t *testing.T) {
	mgmtServer, nodeID, _, rdsNamesCh, xdsC := xdsSetupForTests(t)

	ch := make(chan callbackStruct, 1)
	tcv := &testCallbackVerify{ch: ch}
	rh := newRDSHandler(tcv.testCallback, xdsC, nil)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Configure the management server with a single route config resource.
	routeResource1 := routeConfigResourceForName(route1)
	resources := e2e.UpdateOptions{
		NodeID:         nodeID,
		Routes:         []*v3routepb.RouteConfiguration{routeResource1},
		SkipValidation: true,
	}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	rh.updateRouteNamesToWatch(map[string]bool{route1: true, route2: true})
	waitForResourceNames(ctx, t, rdsNamesCh, []string{route1, route2})
	verifyRouteName(ctx, t, ch, callbackStruct{routeName: route1})

	// The rds handler update should not be ready.
	if got := rh.determineRouteConfigurationReady(); got != false {
		t.Fatalf("rh.determineRouteConfigurationReady: %v, want: false", false)
	}

	// Configure the management server both route config resources.
	routeResource2 := routeConfigResourceForName(route2)
	resources.Routes = []*v3routepb.RouteConfiguration{routeResource1, routeResource2}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}

	verifyRouteName(ctx, t, ch, callbackStruct{routeName: route2})

	if got := rh.determineRouteConfigurationReady(); got != true {
		t.Fatalf("rh.determineRouteConfigurationReady: %v, want: true", got)
	}

	rh.updateRouteNamesToWatch(map[string]bool{route1: true, route2: true, route3: true})
	waitForResourceNames(ctx, t, rdsNamesCh, []string{route1, route2, route3})
	if got := rh.determineRouteConfigurationReady(); got != false {
		t.Fatalf("rh.determineRouteConfigurationReady: %v, want: false", got)
	}

	// Configure the management server with route config resources.
	routeResource3 := routeConfigResourceForName(route3)
	resources.Routes = []*v3routepb.RouteConfiguration{routeResource1, routeResource2, routeResource3}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	verifyRouteName(ctx, t, ch, callbackStruct{routeName: route3})

	if got := rh.determineRouteConfigurationReady(); got != true {
		t.Fatalf("rh.determineRouteConfigurationReady: %v, want: true", got)
	}
	// Update to route 1 and route 2. Should immediately go ready.
	rh.updateRouteNamesToWatch(map[string]bool{route1: true, route3: true})
	if got := rh.determineRouteConfigurationReady(); got != true {
		t.Fatalf("rh.determineRouteConfigurationReady: %v, want: true", got)
	}

	// Update to route 1 and route 4. No route 4, so should not be ready.
	rh.updateRouteNamesToWatch(map[string]bool{route1: true, route4: true})
	waitForResourceNames(ctx, t, rdsNamesCh, []string{route1, route4})
	if got := rh.determineRouteConfigurationReady(); got != false {
		t.Fatalf("rh.determineRouteConfigurationReady: %v, want: false", got)
	}
	routeResource4 := routeConfigResourceForName(route4)
	resources.Routes = []*v3routepb.RouteConfiguration{routeResource1, routeResource2, routeResource3, routeResource4}
	if err := mgmtServer.Update(ctx, resources); err != nil {
		t.Fatal(err)
	}
	verifyRouteName(ctx, t, ch, callbackStruct{routeName: route4})
	if got := rh.determineRouteConfigurationReady(); got != true {
		t.Fatalf("rh.determineRouteConfigurationReady: %v, want: true", got)
	}
}
