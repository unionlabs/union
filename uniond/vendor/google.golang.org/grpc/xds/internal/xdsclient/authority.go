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
 */

package xdsclient

import (
	"context"
	"errors"
	"fmt"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"google.golang.org/grpc/internal/grpclog"
	"google.golang.org/grpc/internal/grpcsync"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal/xdsclient/load"
	"google.golang.org/grpc/xds/internal/xdsclient/transport"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"
	"google.golang.org/protobuf/types/known/anypb"
	"google.golang.org/protobuf/types/known/timestamppb"

	v3adminpb "github.com/envoyproxy/go-control-plane/envoy/admin/v3"
	v3statuspb "github.com/envoyproxy/go-control-plane/envoy/service/status/v3"
)

type watchState int

const (
	watchStateStarted   watchState = iota // Watch started, request not yet set.
	watchStateRequested                   // Request sent for resource being watched.
	watchStateReceived                    // Response received for resource being watched.
	watchStateTimeout                     // Watch timer expired, no response.
	watchStateCanceled                    // Watch cancelled.
)

type resourceState struct {
	watchers        map[xdsresource.ResourceWatcher]bool // Set of watchers for this resource
	cache           xdsresource.ResourceData             // Most recent ACKed update for this resource
	md              xdsresource.UpdateMetadata           // Metadata for the most recent update
	deletionIgnored bool                                 // True if resource deletion was ignored for a prior update

	// Common watch state for all watchers of this resource.
	wTimer *time.Timer // Expiry timer
	wState watchState  // State of the watch
}

// authority wraps all state associated with a single management server. It
// contains the transport used to communicate with the management server and a
// cache of resource state for resources requested from the management server.
//
// Bootstrap configuration could contain multiple entries in the authorities map
// that share the same server config (server address and credentials to use). We
// share the same authority instance amongst these entries, and the reference
// counting is taken care of by the `clientImpl` type.
type authority struct {
	serverCfg          *bootstrap.ServerConfig       // Server config for this authority
	bootstrapCfg       *bootstrap.Config             // Full bootstrap configuration
	refCount           int                           // Reference count of watches referring to this authority
	serializer         *grpcsync.CallbackSerializer  // Callback serializer for invoking watch callbacks
	resourceTypeGetter func(string) xdsresource.Type // ResourceType registry lookup
	transport          *transport.Transport          // Underlying xDS transport to the management server
	watchExpiryTimeout time.Duration                 // Resource watch expiry timeout
	logger             *grpclog.PrefixLogger

	// A two level map containing the state of all the resources being watched.
	//
	// The first level map key is the ResourceType (Listener, Route etc). This
	// allows us to have a single map for all resources instead of having per
	// resource-type maps.
	//
	// The second level map key is the resource name, with the value being the
	// actual state of the resource.
	resourcesMu sync.Mutex
	resources   map[xdsresource.Type]map[string]*resourceState
	closed      bool
}

// authorityArgs is a convenience struct to wrap arguments required to create a
// new authority. All fields here correspond directly to appropriate fields
// stored in the authority struct.
type authorityArgs struct {
	// The reason for passing server config and bootstrap config separately
	// (although the former is part of the latter) is because authorities in the
	// bootstrap config might contain an empty server config, and in this case,
	// the top-level server config is to be used.
	serverCfg          *bootstrap.ServerConfig
	bootstrapCfg       *bootstrap.Config
	serializer         *grpcsync.CallbackSerializer
	resourceTypeGetter func(string) xdsresource.Type
	watchExpiryTimeout time.Duration
	logger             *grpclog.PrefixLogger
}

func newAuthority(args authorityArgs) (*authority, error) {
	ret := &authority{
		serverCfg:          args.serverCfg,
		bootstrapCfg:       args.bootstrapCfg,
		serializer:         args.serializer,
		resourceTypeGetter: args.resourceTypeGetter,
		watchExpiryTimeout: args.watchExpiryTimeout,
		logger:             args.logger,
		resources:          make(map[xdsresource.Type]map[string]*resourceState),
	}

	tr, err := transport.New(transport.Options{
		ServerCfg:      args.serverCfg,
		OnRecvHandler:  ret.handleResourceUpdate,
		OnErrorHandler: ret.newConnectionError,
		OnSendHandler:  ret.transportOnSendHandler,
		Logger:         args.logger,
		NodeProto:      args.bootstrapCfg.Node(),
	})
	if err != nil {
		return nil, fmt.Errorf("creating new transport to %q: %v", args.serverCfg, err)
	}
	ret.transport = tr
	return ret, nil
}

// transportOnSendHandler is called by the underlying transport when it sends a
// resource request successfully. Timers are activated for resources waiting for
// a response.
func (a *authority) transportOnSendHandler(u *transport.ResourceSendInfo) {
	rType := a.resourceTypeGetter(u.URL)
	// Resource type not found is not expected under normal circumstances, since
	// the resource type url passed to the transport is determined by the authority.
	if rType == nil {
		a.logger.Warningf("Unknown resource type url: %s.", u.URL)
		return
	}
	a.resourcesMu.Lock()
	defer a.resourcesMu.Unlock()
	a.startWatchTimersLocked(rType, u.ResourceNames)
}

func (a *authority) handleResourceUpdate(resourceUpdate transport.ResourceUpdate, onDone func()) error {
	rType := a.resourceTypeGetter(resourceUpdate.URL)
	if rType == nil {
		return xdsresource.NewErrorf(xdsresource.ErrorTypeResourceTypeUnsupported, "Resource URL %v unknown in response from server", resourceUpdate.URL)
	}

	opts := &xdsresource.DecodeOptions{
		BootstrapConfig: a.bootstrapCfg,
		ServerConfig:    a.serverCfg,
	}
	updates, md, err := decodeAllResources(opts, rType, resourceUpdate)
	a.updateResourceStateAndScheduleCallbacks(rType, updates, md, onDone)
	return err
}

func (a *authority) updateResourceStateAndScheduleCallbacks(rType xdsresource.Type, updates map[string]resourceDataErrTuple, md xdsresource.UpdateMetadata, onDone func()) {
	a.resourcesMu.Lock()
	defer a.resourcesMu.Unlock()

	// We build a list of callback funcs to invoke, and invoke them at the end
	// of this method instead of inline (when handling the update for a
	// particular resource), because we want to make sure that all calls to
	// increment watcherCnt happen before any callbacks are invoked. This will
	// ensure that the onDone callback is never invoked before all watcher
	// callbacks are invoked, and the watchers have processed the update.
	watcherCnt := new(atomic.Int64)
	done := func() {
		watcherCnt.Add(-1)
		if watcherCnt.Load() == 0 {
			onDone()
		}
	}
	funcsToSchedule := []func(context.Context){}
	defer func() {
		if len(funcsToSchedule) == 0 {
			// When there are no watchers for the resources received as part of
			// this update, invoke onDone explicitly to unblock the next read on
			// the ADS stream.
			onDone()
		}
		for _, f := range funcsToSchedule {
			a.serializer.ScheduleOr(f, onDone)
		}
	}()

	resourceStates := a.resources[rType]
	for name, uErr := range updates {
		if state, ok := resourceStates[name]; ok {
			// Cancel the expiry timer associated with the resource once a
			// response is received, irrespective of whether the update is a
			// good one or not.
			//
			// We check for watch states `started` and `requested` here to
			// accommodate for a race which can happen in the following
			// scenario:
			// - When a watch is registered, it is possible that the ADS stream
			//   is not yet created. In this case, the request for the resource
			//   is not sent out immediately. An entry in the `resourceStates`
			//   map is created with a watch state of `started`.
			// - Once the stream is created, it is possible that the management
			//   server might respond with the requested resource before we send
			//   out request for the same. If we don't check for `started` here,
			//   and move the state to `received`, we will end up starting the
			//   timer when the request gets sent out. And since the management
			//   server already sent us the resource, there is a good chance
			//   that it will not send it again. This would eventually lead to
			//   the timer firing, even though we have the resource in the
			//   cache.
			if state.wState == watchStateStarted || state.wState == watchStateRequested {
				// It is OK to ignore the return value from Stop() here because
				// if the timer has already fired, it means that the timer watch
				// expiry callback is blocked on the same lock that we currently
				// hold. Since we move the state to `received` here, the timer
				// callback will be a no-op.
				if state.wTimer != nil {
					state.wTimer.Stop()
				}
				state.wState = watchStateReceived
			}

			if uErr.err != nil {
				// On error, keep previous version of the resource. But update
				// status and error.
				state.md.ErrState = md.ErrState
				state.md.Status = md.Status
				for watcher := range state.watchers {
					watcher := watcher
					err := uErr.err
					watcherCnt.Add(1)
					funcsToSchedule = append(funcsToSchedule, func(context.Context) { watcher.OnError(err, done) })
				}
				continue
			}

			if state.deletionIgnored {
				state.deletionIgnored = false
				a.logger.Infof("A valid update was received for resource %q of type %q after previously ignoring a deletion", name, rType.TypeName())
			}
			// Notify watchers only if this is a first time update or it is different
			// from the one currently cached.
			if state.cache == nil || !state.cache.Equal(uErr.resource) {
				for watcher := range state.watchers {
					watcher := watcher
					resource := uErr.resource
					watcherCnt.Add(1)
					funcsToSchedule = append(funcsToSchedule, func(context.Context) { watcher.OnUpdate(resource, done) })
				}
			}
			// Sync cache.
			if a.logger.V(2) {
				a.logger.Infof("Resource type %q with name %q added to cache", rType.TypeName(), name)
			}
			state.cache = uErr.resource
			// Set status to ACK, and clear error state. The metadata might be a
			// NACK metadata because some other resources in the same response
			// are invalid.
			state.md = md
			state.md.ErrState = nil
			state.md.Status = xdsresource.ServiceStatusACKed
			if md.ErrState != nil {
				state.md.Version = md.ErrState.Version
			}
		}
	}

	// If this resource type requires that all resources be present in every
	// SotW response from the server, a response that does not include a
	// previously seen resource will be interpreted as a deletion of that
	// resource unless ignore_resource_deletion option was set in the server
	// config.
	if !rType.AllResourcesRequiredInSotW() {
		return
	}
	for name, state := range resourceStates {
		if state.cache == nil {
			// If the resource state does not contain a cached update, which can
			// happen when:
			// - resource was newly requested but has not yet been received, or,
			// - resource was removed as part of a previous update,
			// we don't want to generate an error for the watchers.
			//
			// For the first of the above two conditions, this ADS response may
			// be in reaction to an earlier request that did not yet request the
			// new resource, so its absence from the response does not
			// necessarily indicate that the resource does not exist. For that
			// case, we rely on the request timeout instead.
			//
			// For the second of the above two conditions, we already generated
			// an error when we received the first response which removed this
			// resource. So, there is no need to generate another one.
			continue
		}
		if _, ok := updates[name]; !ok {
			// The metadata status is set to "ServiceStatusNotExist" if a
			// previous update deleted this resource, in which case we do not
			// want to repeatedly call the watch callbacks with a
			// "resource-not-found" error.
			if state.md.Status == xdsresource.ServiceStatusNotExist {
				continue
			}
			// Per A53, resource deletions are ignored if the `ignore_resource_deletion`
			// server feature is enabled through the bootstrap configuration. If the
			// resource deletion is to be ignored, the resource is not removed from
			// the cache and the corresponding OnResourceDoesNotExist() callback is
			// not invoked on the watchers.
			if a.serverCfg.ServerFeaturesIgnoreResourceDeletion() {
				if !state.deletionIgnored {
					state.deletionIgnored = true
					a.logger.Warningf("Ignoring resource deletion for resource %q of type %q", name, rType.TypeName())
				}
				continue
			}
			// If resource exists in cache, but not in the new update, delete
			// the resource from cache, and also send a resource not found error
			// to indicate resource removed. Metadata for the resource is still
			// maintained, as this is required by CSDS.
			state.cache = nil
			state.md = xdsresource.UpdateMetadata{Status: xdsresource.ServiceStatusNotExist}
			for watcher := range state.watchers {
				watcher := watcher
				watcherCnt.Add(1)
				funcsToSchedule = append(funcsToSchedule, func(context.Context) { watcher.OnResourceDoesNotExist(done) })
			}
		}
	}
}

type resourceDataErrTuple struct {
	resource xdsresource.ResourceData
	err      error
}

func decodeAllResources(opts *xdsresource.DecodeOptions, rType xdsresource.Type, update transport.ResourceUpdate) (map[string]resourceDataErrTuple, xdsresource.UpdateMetadata, error) {
	timestamp := time.Now()
	md := xdsresource.UpdateMetadata{
		Version:   update.Version,
		Timestamp: timestamp,
	}

	topLevelErrors := make([]error, 0)           // Tracks deserialization errors, where we don't have a resource name.
	perResourceErrors := make(map[string]error)  // Tracks resource validation errors, where we have a resource name.
	ret := make(map[string]resourceDataErrTuple) // Return result, a map from resource name to either resource data or error.
	for _, r := range update.Resources {
		result, err := rType.Decode(opts, r)

		// Name field of the result is left unpopulated only when resource
		// deserialization fails.
		name := ""
		if result != nil {
			name = xdsresource.ParseName(result.Name).String()
		}
		if err == nil {
			ret[name] = resourceDataErrTuple{resource: result.Resource}
			continue
		}
		if name == "" {
			topLevelErrors = append(topLevelErrors, err)
			continue
		}
		perResourceErrors[name] = err
		// Add place holder in the map so we know this resource name was in
		// the response.
		ret[name] = resourceDataErrTuple{err: err}
	}

	if len(topLevelErrors) == 0 && len(perResourceErrors) == 0 {
		md.Status = xdsresource.ServiceStatusACKed
		return ret, md, nil
	}

	md.Status = xdsresource.ServiceStatusNACKed
	errRet := combineErrors(rType.TypeName(), topLevelErrors, perResourceErrors)
	md.ErrState = &xdsresource.UpdateErrorMetadata{
		Version:   update.Version,
		Err:       errRet,
		Timestamp: timestamp,
	}
	return ret, md, errRet
}

// startWatchTimersLocked is invoked upon transport.OnSend() callback with resources
// requested on the underlying ADS stream. This satisfies the conditions to start
// watch timers per A57 [https://github.com/grpc/proposal/blob/master/A57-xds-client-failure-mode-behavior.md#handling-resources-that-do-not-exist]
//
// Caller must hold a.resourcesMu.
func (a *authority) startWatchTimersLocked(rType xdsresource.Type, resourceNames []string) {
	resourceStates := a.resources[rType]
	for _, resourceName := range resourceNames {
		if state, ok := resourceStates[resourceName]; ok {
			if state.wState != watchStateStarted {
				continue
			}
			state.wTimer = time.AfterFunc(a.watchExpiryTimeout, func() {
				a.resourcesMu.Lock()
				a.handleWatchTimerExpiryLocked(rType, resourceName, state)
				a.resourcesMu.Unlock()
			})
			state.wState = watchStateRequested
		}
	}
}

// stopWatchTimersLocked is invoked upon connection errors to stops watch timers
// for resources that have been requested, but not yet responded to by the management
// server.
//
// Caller must hold a.resourcesMu.
func (a *authority) stopWatchTimersLocked() {
	for _, rType := range a.resources {
		for resourceName, state := range rType {
			if state.wState != watchStateRequested {
				continue
			}
			if !state.wTimer.Stop() {
				// If the timer has already fired, it means that the timer watch expiry
				// callback is blocked on the same lock that we currently hold. Don't change
				// the watch state and instead let the watch expiry callback handle it.
				a.logger.Warningf("Watch timer for resource %v already fired. Ignoring here.", resourceName)
				continue
			}
			state.wTimer = nil
			state.wState = watchStateStarted
		}
	}
}

// newConnectionError is called by the underlying transport when it receives a
// connection error. The error will be forwarded to all the resource watchers.
func (a *authority) newConnectionError(err error) {
	a.resourcesMu.Lock()
	defer a.resourcesMu.Unlock()

	a.stopWatchTimersLocked()

	// We do not consider it an error if the ADS stream was closed after having received
	// a response on the stream. This is because there are legitimate reasons why the server
	// may need to close the stream during normal operations, such as needing to rebalance
	// load or the underlying connection hitting its max connection age limit.
	// See gRFC A57 for more details.
	if xdsresource.ErrType(err) == xdsresource.ErrTypeStreamFailedAfterRecv {
		a.logger.Warningf("Watchers not notified since ADS stream failed after having received at least one response: %v", err)
		return
	}

	for _, rType := range a.resources {
		for _, state := range rType {
			// Propagate the connection error from the transport layer to all watchers.
			for watcher := range state.watchers {
				watcher := watcher
				a.serializer.TrySchedule(func(context.Context) {
					watcher.OnError(xdsresource.NewErrorf(xdsresource.ErrorTypeConnection, "xds: error received from xDS stream: %v", err), func() {})
				})
			}
		}
	}
}

// Increments the reference count. Caller must hold parent's authorityMu.
func (a *authority) refLocked() {
	a.refCount++
}

// Decrements the reference count. Caller must hold parent's authorityMu.
func (a *authority) unrefLocked() int {
	a.refCount--
	return a.refCount
}

func (a *authority) close() {
	a.transport.Close()

	a.resourcesMu.Lock()
	a.closed = true
	a.resourcesMu.Unlock()
}

func (a *authority) watchResource(rType xdsresource.Type, resourceName string, watcher xdsresource.ResourceWatcher) func() {
	if a.logger.V(2) {
		a.logger.Infof("New watch for type %q, resource name %q", rType.TypeName(), resourceName)
	}
	a.resourcesMu.Lock()
	defer a.resourcesMu.Unlock()

	// Lookup the ResourceType specific resources from the top-level map. If
	// there is no entry for this ResourceType, create one.
	resources := a.resources[rType]
	if resources == nil {
		resources = make(map[string]*resourceState)
		a.resources[rType] = resources
	}

	// Lookup the resourceState for the particular resource that the watch is
	// being registered for. If this is the first watch for this resource,
	// instruct the transport layer to send a DiscoveryRequest for the same.
	state := resources[resourceName]
	if state == nil {
		if a.logger.V(2) {
			a.logger.Infof("First watch for type %q, resource name %q", rType.TypeName(), resourceName)
		}
		state = &resourceState{
			watchers: make(map[xdsresource.ResourceWatcher]bool),
			md:       xdsresource.UpdateMetadata{Status: xdsresource.ServiceStatusRequested},
			wState:   watchStateStarted,
		}
		resources[resourceName] = state
		a.sendDiscoveryRequestLocked(rType, resources)
	}
	// Always add the new watcher to the set of watchers.
	state.watchers[watcher] = true

	// If we have a cached copy of the resource, notify the new watcher.
	if state.cache != nil {
		if a.logger.V(2) {
			a.logger.Infof("Resource type %q with resource name %q found in cache: %s", rType.TypeName(), resourceName, state.cache.ToJSON())
		}
		resource := state.cache
		a.serializer.TrySchedule(func(context.Context) { watcher.OnUpdate(resource, func() {}) })
	}

	return func() {
		a.resourcesMu.Lock()
		defer a.resourcesMu.Unlock()

		// We already have a reference to the resourceState for this particular
		// resource. Avoid indexing into the two-level map to figure this out.

		// Delete this particular watcher from the list of watchers, so that its
		// callback will not be invoked in the future.
		state.wState = watchStateCanceled
		delete(state.watchers, watcher)
		if len(state.watchers) > 0 {
			return
		}

		// There are no more watchers for this resource, delete the state
		// associated with it, and instruct the transport to send a request
		// which does not include this resource name.
		if a.logger.V(2) {
			a.logger.Infof("Removing last watch for type %q, resource name %q", rType.TypeName(), resourceName)
		}
		delete(resources, resourceName)
		a.sendDiscoveryRequestLocked(rType, resources)
	}
}

func (a *authority) handleWatchTimerExpiryLocked(rType xdsresource.Type, resourceName string, state *resourceState) {
	if a.closed {
		return
	}
	a.logger.Warningf("Watch for resource %q of type %s timed out", resourceName, rType.TypeName())

	switch state.wState {
	case watchStateRequested:
		// This is the only state where we need to handle the timer expiry by
		// invoking appropriate watch callbacks. This is handled outside the switch.
	case watchStateCanceled:
		return
	default:
		a.logger.Warningf("Unexpected watch state %q for resource %q.", state.wState, resourceName)
		return
	}

	state.wState = watchStateTimeout
	// With the watch timer firing, it is safe to assume that the resource does
	// not exist on the management server.
	state.cache = nil
	state.md = xdsresource.UpdateMetadata{Status: xdsresource.ServiceStatusNotExist}
	for watcher := range state.watchers {
		watcher := watcher
		a.serializer.TrySchedule(func(context.Context) { watcher.OnResourceDoesNotExist(func() {}) })
	}
}

func (a *authority) triggerResourceNotFoundForTesting(rType xdsresource.Type, resourceName string) {
	a.resourcesMu.Lock()
	defer a.resourcesMu.Unlock()

	if a.closed {
		return
	}
	resourceStates := a.resources[rType]
	state, ok := resourceStates[resourceName]
	if !ok {
		return
	}
	// if watchStateTimeout already triggered resource not found above from
	// normal watch expiry.
	if state.wState == watchStateCanceled || state.wState == watchStateTimeout {
		return
	}
	state.wState = watchStateTimeout
	state.cache = nil
	state.md = xdsresource.UpdateMetadata{Status: xdsresource.ServiceStatusNotExist}
	for watcher := range state.watchers {
		watcher := watcher
		a.serializer.TrySchedule(func(context.Context) { watcher.OnResourceDoesNotExist(func() {}) })
	}
}

// sendDiscoveryRequestLocked sends a discovery request for the specified
// resource type and resource names. Even though this method does not directly
// access the resource cache, it is important that `resourcesMu` be held when
// calling this method to ensure that a consistent snapshot of resource names is
// being requested.
func (a *authority) sendDiscoveryRequestLocked(rType xdsresource.Type, resources map[string]*resourceState) {
	resourcesToRequest := make([]string, len(resources))
	i := 0
	for name := range resources {
		resourcesToRequest[i] = name
		i++
	}
	a.transport.SendRequest(rType.TypeURL(), resourcesToRequest)
}

func (a *authority) reportLoad() (*load.Store, func()) {
	return a.transport.ReportLoad()
}

func (a *authority) dumpResources() []*v3statuspb.ClientConfig_GenericXdsConfig {
	a.resourcesMu.Lock()
	defer a.resourcesMu.Unlock()

	var ret []*v3statuspb.ClientConfig_GenericXdsConfig
	for rType, resourceStates := range a.resources {
		typeURL := rType.TypeURL()
		for name, state := range resourceStates {
			var raw *anypb.Any
			if state.cache != nil {
				raw = state.cache.Raw()
			}
			config := &v3statuspb.ClientConfig_GenericXdsConfig{
				TypeUrl:      typeURL,
				Name:         name,
				VersionInfo:  state.md.Version,
				XdsConfig:    raw,
				LastUpdated:  timestamppb.New(state.md.Timestamp),
				ClientStatus: serviceStatusToProto(state.md.Status),
			}
			if errState := state.md.ErrState; errState != nil {
				config.ErrorState = &v3adminpb.UpdateFailureState{
					LastUpdateAttempt: timestamppb.New(errState.Timestamp),
					Details:           errState.Err.Error(),
					VersionInfo:       errState.Version,
				}
			}
			ret = append(ret, config)
		}
	}
	return ret
}

func serviceStatusToProto(serviceStatus xdsresource.ServiceStatus) v3adminpb.ClientResourceStatus {
	switch serviceStatus {
	case xdsresource.ServiceStatusUnknown:
		return v3adminpb.ClientResourceStatus_UNKNOWN
	case xdsresource.ServiceStatusRequested:
		return v3adminpb.ClientResourceStatus_REQUESTED
	case xdsresource.ServiceStatusNotExist:
		return v3adminpb.ClientResourceStatus_DOES_NOT_EXIST
	case xdsresource.ServiceStatusACKed:
		return v3adminpb.ClientResourceStatus_ACKED
	case xdsresource.ServiceStatusNACKed:
		return v3adminpb.ClientResourceStatus_NACKED
	default:
		return v3adminpb.ClientResourceStatus_UNKNOWN
	}
}

func combineErrors(rType string, topLevelErrors []error, perResourceErrors map[string]error) error {
	var errStrB strings.Builder
	errStrB.WriteString(fmt.Sprintf("error parsing %q response: ", rType))
	if len(topLevelErrors) > 0 {
		errStrB.WriteString("top level errors: ")
		for i, err := range topLevelErrors {
			if i != 0 {
				errStrB.WriteString(";\n")
			}
			errStrB.WriteString(err.Error())
		}
	}
	if len(perResourceErrors) > 0 {
		var i int
		for name, err := range perResourceErrors {
			if i != 0 {
				errStrB.WriteString(";\n")
			}
			i++
			errStrB.WriteString(fmt.Sprintf("resource %q: %v", name, err.Error()))
		}
	}
	return errors.New(errStrB.String())
}
