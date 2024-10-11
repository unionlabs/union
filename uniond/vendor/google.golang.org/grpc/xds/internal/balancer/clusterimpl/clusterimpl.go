/*
 *
 * Copyright 2020 gRPC authors.
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

// Package clusterimpl implements the xds_cluster_impl balancing policy. It
// handles the cluster features (e.g. circuit_breaking, RPC dropping).
//
// Note that it doesn't handle name resolution, which is done by policy
// xds_cluster_resolver.
package clusterimpl

import (
	"context"
	"encoding/json"
	"fmt"
	"sync"
	"sync/atomic"

	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/connectivity"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/balancer/gracefulswitch"
	"google.golang.org/grpc/internal/grpclog"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/internal/pretty"
	"google.golang.org/grpc/internal/xds"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/serviceconfig"
	xdsinternal "google.golang.org/grpc/xds/internal"
	"google.golang.org/grpc/xds/internal/balancer/loadstore"
	"google.golang.org/grpc/xds/internal/xdsclient"
	"google.golang.org/grpc/xds/internal/xdsclient/load"
)

const (
	// Name is the name of the cluster_impl balancer.
	Name                   = "xds_cluster_impl_experimental"
	defaultRequestCountMax = 1024
)

var (
	connectedAddress  = internal.ConnectedAddress.(func(balancer.SubConnState) resolver.Address)
	errBalancerClosed = fmt.Errorf("%s LB policy is closed", Name)
)

func init() {
	balancer.Register(bb{})
}

type bb struct{}

func (bb) Build(cc balancer.ClientConn, bOpts balancer.BuildOptions) balancer.Balancer {
	ctx, cancel := context.WithCancel(context.Background())
	b := &clusterImplBalancer{
		ClientConn:       cc,
		bOpts:            bOpts,
		loadWrapper:      loadstore.NewWrapper(),
		requestCountMax:  defaultRequestCountMax,
		serializer:       grpcsync.NewCallbackSerializer(ctx),
		serializerCancel: cancel,
	}
	b.logger = prefixLogger(b)
	b.child = gracefulswitch.NewBalancer(b, bOpts)
	b.logger.Infof("Created")
	return b
}

func (bb) Name() string {
	return Name
}

func (bb) ParseConfig(c json.RawMessage) (serviceconfig.LoadBalancingConfig, error) {
	return parseConfig(c)
}

type clusterImplBalancer struct {
	balancer.ClientConn

	bOpts     balancer.BuildOptions
	logger    *grpclog.PrefixLogger
	xdsClient xdsclient.XDSClient

	config           *LBConfig
	child            *gracefulswitch.Balancer
	cancelLoadReport func()
	edsServiceName   string
	lrsServer        *bootstrap.ServerConfig
	loadWrapper      *loadstore.Wrapper

	clusterNameMu sync.Mutex
	clusterName   string

	serializer       *grpcsync.CallbackSerializer
	serializerCancel context.CancelFunc

	// childState/drops/requestCounter keeps the state used by the most recently
	// generated picker.
	childState            balancer.State
	dropCategories        []DropConfig // The categories for drops.
	drops                 []*dropper
	requestCounterCluster string // The cluster name for the request counter.
	requestCounterService string // The service name for the request counter.
	requestCounter        *xdsclient.ClusterRequestsCounter
	requestCountMax       uint32
	telemetryLabels       map[string]string
}

// updateLoadStore checks the config for load store, and decides whether it
// needs to restart the load reporting stream.
func (b *clusterImplBalancer) updateLoadStore(newConfig *LBConfig) error {
	var updateLoadClusterAndService bool

	// ClusterName is different, restart. ClusterName is from ClusterName and
	// EDSServiceName.
	clusterName := b.getClusterName()
	if clusterName != newConfig.Cluster {
		updateLoadClusterAndService = true
		b.setClusterName(newConfig.Cluster)
		clusterName = newConfig.Cluster
	}
	if b.edsServiceName != newConfig.EDSServiceName {
		updateLoadClusterAndService = true
		b.edsServiceName = newConfig.EDSServiceName
	}
	if updateLoadClusterAndService {
		// This updates the clusterName and serviceName that will be reported
		// for the loads. The update here is too early, the perfect timing is
		// when the picker is updated with the new connection. But from this
		// balancer's point of view, it's impossible to tell.
		//
		// On the other hand, this will almost never happen. Each LRS policy
		// shouldn't get updated config. The parent should do a graceful switch
		// when the clusterName or serviceName is changed.
		b.loadWrapper.UpdateClusterAndService(clusterName, b.edsServiceName)
	}

	var (
		stopOldLoadReport  bool
		startNewLoadReport bool
	)

	// Check if it's necessary to restart load report.
	if b.lrsServer == nil {
		if newConfig.LoadReportingServer != nil {
			// Old is nil, new is not nil, start new LRS.
			b.lrsServer = newConfig.LoadReportingServer
			startNewLoadReport = true
		}
		// Old is nil, new is nil, do nothing.
	} else if newConfig.LoadReportingServer == nil {
		// Old is not nil, new is nil, stop old, don't start new.
		b.lrsServer = newConfig.LoadReportingServer
		stopOldLoadReport = true
	} else {
		// Old is not nil, new is not nil, compare string values, if
		// different, stop old and start new.
		if !b.lrsServer.Equal(newConfig.LoadReportingServer) {
			b.lrsServer = newConfig.LoadReportingServer
			stopOldLoadReport = true
			startNewLoadReport = true
		}
	}

	if stopOldLoadReport {
		if b.cancelLoadReport != nil {
			b.cancelLoadReport()
			b.cancelLoadReport = nil
			if !startNewLoadReport {
				// If a new LRS stream will be started later, no need to update
				// it to nil here.
				b.loadWrapper.UpdateLoadStore(nil)
			}
		}
	}
	if startNewLoadReport {
		var loadStore *load.Store
		if b.xdsClient != nil {
			loadStore, b.cancelLoadReport = b.xdsClient.ReportLoad(b.lrsServer)
		}
		b.loadWrapper.UpdateLoadStore(loadStore)
	}

	return nil
}

func (b *clusterImplBalancer) updateClientConnState(s balancer.ClientConnState) error {
	if b.logger.V(2) {
		b.logger.Infof("Received configuration: %s", pretty.ToJSON(s.BalancerConfig))
	}
	newConfig, ok := s.BalancerConfig.(*LBConfig)
	if !ok {
		return fmt.Errorf("unexpected balancer config with type: %T", s.BalancerConfig)
	}

	// Need to check for potential errors at the beginning of this function, so
	// that on errors, we reject the whole config, instead of applying part of
	// it.
	bb := balancer.Get(newConfig.ChildPolicy.Name)
	if bb == nil {
		return fmt.Errorf("child policy %q not registered", newConfig.ChildPolicy.Name)
	}

	if b.xdsClient == nil {
		c := xdsclient.FromResolverState(s.ResolverState)
		if c == nil {
			return balancer.ErrBadResolverState
		}
		b.xdsClient = c
	}

	// Update load reporting config. This needs to be done before updating the
	// child policy because we need the loadStore from the updated client to be
	// passed to the ccWrapper, so that the next picker from the child policy
	// will pick up the new loadStore.
	if err := b.updateLoadStore(newConfig); err != nil {
		return err
	}

	if b.config == nil || b.config.ChildPolicy.Name != newConfig.ChildPolicy.Name {
		if err := b.child.SwitchTo(bb); err != nil {
			return fmt.Errorf("error switching to child of type %q: %v", newConfig.ChildPolicy.Name, err)
		}
	}
	b.config = newConfig

	b.telemetryLabels = newConfig.TelemetryLabels
	dc := b.handleDropAndRequestCount(newConfig)
	if dc != nil && b.childState.Picker != nil {
		b.ClientConn.UpdateState(balancer.State{
			ConnectivityState: b.childState.ConnectivityState,
			Picker:            b.newPicker(dc),
		})
	}

	// Addresses and sub-balancer config are sent to sub-balancer.
	return b.child.UpdateClientConnState(balancer.ClientConnState{
		ResolverState:  s.ResolverState,
		BalancerConfig: b.config.ChildPolicy.Config,
	})
}

func (b *clusterImplBalancer) UpdateClientConnState(s balancer.ClientConnState) error {
	// Handle the update in a blocking fashion.
	errCh := make(chan error, 1)
	callback := func(context.Context) {
		errCh <- b.updateClientConnState(s)
	}
	onFailure := func() {
		// An attempt to schedule callback fails only when an update is received
		// after Close().
		errCh <- errBalancerClosed
	}
	b.serializer.ScheduleOr(callback, onFailure)
	return <-errCh
}

func (b *clusterImplBalancer) ResolverError(err error) {
	b.serializer.TrySchedule(func(context.Context) {
		b.child.ResolverError(err)
	})
}

func (b *clusterImplBalancer) updateSubConnState(_ balancer.SubConn, s balancer.SubConnState, cb func(balancer.SubConnState)) {
	// Trigger re-resolution when a SubConn turns transient failure. This is
	// necessary for the LogicalDNS in cluster_resolver policy to re-resolve.
	//
	// Note that this happens not only for the addresses from DNS, but also for
	// EDS (cluster_impl doesn't know if it's DNS or EDS, only the parent
	// knows). The parent priority policy is configured to ignore re-resolution
	// signal from the EDS children.
	if s.ConnectivityState == connectivity.TransientFailure {
		b.ClientConn.ResolveNow(resolver.ResolveNowOptions{})
	}

	if cb != nil {
		cb(s)
	}
}

func (b *clusterImplBalancer) UpdateSubConnState(sc balancer.SubConn, s balancer.SubConnState) {
	b.logger.Errorf("UpdateSubConnState(%v, %+v) called unexpectedly", sc, s)
}

func (b *clusterImplBalancer) Close() {
	b.serializer.TrySchedule(func(_ context.Context) {
		b.child.Close()
		b.childState = balancer.State{}

		if b.cancelLoadReport != nil {
			b.cancelLoadReport()
			b.cancelLoadReport = nil
		}
		b.logger.Infof("Shutdown")
	})
	b.serializerCancel()
	<-b.serializer.Done()
}

func (b *clusterImplBalancer) ExitIdle() {
	b.serializer.TrySchedule(func(context.Context) {
		b.child.ExitIdle()
	})
}

// Override methods to accept updates from the child LB.

func (b *clusterImplBalancer) UpdateState(state balancer.State) {
	b.serializer.TrySchedule(func(context.Context) {
		b.childState = state
		b.ClientConn.UpdateState(balancer.State{
			ConnectivityState: b.childState.ConnectivityState,
			Picker: b.newPicker(&dropConfigs{
				drops:           b.drops,
				requestCounter:  b.requestCounter,
				requestCountMax: b.requestCountMax,
			}),
		})
	})
}

func (b *clusterImplBalancer) setClusterName(n string) {
	b.clusterNameMu.Lock()
	defer b.clusterNameMu.Unlock()
	b.clusterName = n
}

func (b *clusterImplBalancer) getClusterName() string {
	b.clusterNameMu.Lock()
	defer b.clusterNameMu.Unlock()
	return b.clusterName
}

// scWrapper is a wrapper of SubConn with locality ID. The locality ID can be
// retrieved from the addresses when creating SubConn.
//
// All SubConns passed to the child policies are wrapped in this, so that the
// picker can get the localityID from the picked SubConn, and do load reporting.
//
// After wrapping, all SubConns to and from the parent ClientConn (e.g. for
// SubConn state update, update/remove SubConn) must be the original SubConns.
// All SubConns to and from the child policy (NewSubConn, forwarding SubConn
// state update) must be the wrapper. The balancer keeps a map from the original
// SubConn to the wrapper for this purpose.
type scWrapper struct {
	balancer.SubConn
	// locality needs to be atomic because it can be updated while being read by
	// the picker.
	locality atomic.Value // type xdsinternal.LocalityID
}

func (scw *scWrapper) updateLocalityID(lID xdsinternal.LocalityID) {
	scw.locality.Store(lID)
}

func (scw *scWrapper) localityID() xdsinternal.LocalityID {
	lID, _ := scw.locality.Load().(xdsinternal.LocalityID)
	return lID
}

func (b *clusterImplBalancer) NewSubConn(addrs []resolver.Address, opts balancer.NewSubConnOptions) (balancer.SubConn, error) {
	clusterName := b.getClusterName()
	newAddrs := make([]resolver.Address, len(addrs))
	for i, addr := range addrs {
		newAddrs[i] = xds.SetXDSHandshakeClusterName(addr, clusterName)
	}
	var sc balancer.SubConn
	scw := &scWrapper{}
	oldListener := opts.StateListener
	opts.StateListener = func(state balancer.SubConnState) {
		b.serializer.TrySchedule(func(context.Context) {
			b.updateSubConnState(sc, state, oldListener)
			if state.ConnectivityState != connectivity.Ready {
				return
			}
			// Read connected address and call updateLocalityID() based on the connected
			// address's locality. https://github.com/grpc/grpc-go/issues/7339
			addr := connectedAddress(state)
			lID := xdsinternal.GetLocalityID(addr)
			if lID.Empty() {
				if b.logger.V(2) {
					b.logger.Infof("Locality ID for %s unexpectedly empty", addr)
				}
				return
			}
			scw.updateLocalityID(lID)
		})
	}
	sc, err := b.ClientConn.NewSubConn(newAddrs, opts)
	if err != nil {
		return nil, err
	}
	scw.SubConn = sc
	return scw, nil
}

func (b *clusterImplBalancer) RemoveSubConn(sc balancer.SubConn) {
	b.logger.Errorf("RemoveSubConn(%v) called unexpectedly", sc)
}

func (b *clusterImplBalancer) UpdateAddresses(sc balancer.SubConn, addrs []resolver.Address) {
	clusterName := b.getClusterName()
	newAddrs := make([]resolver.Address, len(addrs))
	var lID xdsinternal.LocalityID
	for i, addr := range addrs {
		newAddrs[i] = xds.SetXDSHandshakeClusterName(addr, clusterName)
		lID = xdsinternal.GetLocalityID(newAddrs[i])
	}
	if scw, ok := sc.(*scWrapper); ok {
		scw.updateLocalityID(lID)
		// Need to get the original SubConn from the wrapper before calling
		// parent ClientConn.
		sc = scw.SubConn
	}
	b.ClientConn.UpdateAddresses(sc, newAddrs)
}

type dropConfigs struct {
	drops           []*dropper
	requestCounter  *xdsclient.ClusterRequestsCounter
	requestCountMax uint32
}

// handleDropAndRequestCount compares drop and request counter in newConfig with
// the one currently used by picker. It returns a new dropConfigs if a new
// picker needs to be generated, otherwise it returns nil.
func (b *clusterImplBalancer) handleDropAndRequestCount(newConfig *LBConfig) *dropConfigs {
	// Compare new drop config. And update picker if it's changed.
	var updatePicker bool
	if !equalDropCategories(b.dropCategories, newConfig.DropCategories) {
		b.dropCategories = newConfig.DropCategories
		b.drops = make([]*dropper, 0, len(newConfig.DropCategories))
		for _, c := range newConfig.DropCategories {
			b.drops = append(b.drops, newDropper(c))
		}
		updatePicker = true
	}

	// Compare cluster name. And update picker if it's changed, because circuit
	// breaking's stream counter will be different.
	if b.requestCounterCluster != newConfig.Cluster || b.requestCounterService != newConfig.EDSServiceName {
		b.requestCounterCluster = newConfig.Cluster
		b.requestCounterService = newConfig.EDSServiceName
		b.requestCounter = xdsclient.GetClusterRequestsCounter(newConfig.Cluster, newConfig.EDSServiceName)
		updatePicker = true
	}
	// Compare upper bound of stream count. And update picker if it's changed.
	// This is also for circuit breaking.
	var newRequestCountMax uint32 = 1024
	if newConfig.MaxConcurrentRequests != nil {
		newRequestCountMax = *newConfig.MaxConcurrentRequests
	}
	if b.requestCountMax != newRequestCountMax {
		b.requestCountMax = newRequestCountMax
		updatePicker = true
	}

	if !updatePicker {
		return nil
	}
	return &dropConfigs{
		drops:           b.drops,
		requestCounter:  b.requestCounter,
		requestCountMax: b.requestCountMax,
	}
}
