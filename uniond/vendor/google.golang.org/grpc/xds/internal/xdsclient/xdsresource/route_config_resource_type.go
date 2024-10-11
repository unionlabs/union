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

package xdsresource

import (
	"google.golang.org/grpc/internal/pretty"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource/version"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/anypb"
)

const (
	// RouteConfigTypeName represents the transport agnostic name for the
	// route config resource.
	RouteConfigTypeName = "RouteConfigResource"
)

var (
	// Compile time interface checks.
	_ Type = routeConfigResourceType{}

	// Singleton instantiation of the resource type implementation.
	routeConfigType = routeConfigResourceType{
		resourceTypeState: resourceTypeState{
			typeURL:                    version.V3RouteConfigURL,
			typeName:                   "RouteConfigResource",
			allResourcesRequiredInSotW: false,
		},
	}
)

// routeConfigResourceType provides the resource-type specific functionality for
// a RouteConfiguration resource.
//
// Implements the Type interface.
type routeConfigResourceType struct {
	resourceTypeState
}

// Decode deserializes and validates an xDS resource serialized inside the
// provided `Any` proto, as received from the xDS management server.
func (routeConfigResourceType) Decode(_ *DecodeOptions, resource *anypb.Any) (*DecodeResult, error) {
	name, rc, err := unmarshalRouteConfigResource(resource)
	switch {
	case name == "":
		// Name is unset only when protobuf deserialization fails.
		return nil, err
	case err != nil:
		// Protobuf deserialization succeeded, but resource validation failed.
		return &DecodeResult{Name: name, Resource: &RouteConfigResourceData{Resource: RouteConfigUpdate{}}}, err
	}

	return &DecodeResult{Name: name, Resource: &RouteConfigResourceData{Resource: rc}}, nil

}

// RouteConfigResourceData wraps the configuration of a RouteConfiguration
// resource as received from the management server.
//
// Implements the ResourceData interface.
type RouteConfigResourceData struct {
	ResourceData

	// TODO: We have always stored update structs by value. See if this can be
	// switched to a pointer?
	Resource RouteConfigUpdate
}

// Equal returns true if other is equal to r.
func (r *RouteConfigResourceData) Equal(other ResourceData) bool {
	if r == nil && other == nil {
		return true
	}
	if (r == nil) != (other == nil) {
		return false
	}
	return proto.Equal(r.Resource.Raw, other.Raw())

}

// ToJSON returns a JSON string representation of the resource data.
func (r *RouteConfigResourceData) ToJSON() string {
	return pretty.ToJSON(r.Resource)
}

// Raw returns the underlying raw protobuf form of the route configuration
// resource.
func (r *RouteConfigResourceData) Raw() *anypb.Any {
	return r.Resource.Raw
}

// RouteConfigWatcher wraps the callbacks to be invoked for different
// events corresponding to the route configuration resource being watched.
type RouteConfigWatcher interface {
	// OnUpdate is invoked to report an update for the resource being watched.
	OnUpdate(*RouteConfigResourceData, OnDoneFunc)

	// OnError is invoked under different error conditions including but not
	// limited to the following:
	//	- authority mentioned in the resource is not found
	//	- resource name parsing error
	//	- resource deserialization error
	//	- resource validation error
	//	- ADS stream failure
	//	- connection failure
	OnError(error, OnDoneFunc)

	// OnResourceDoesNotExist is invoked for a specific error condition where
	// the requested resource is not found on the xDS management server.
	OnResourceDoesNotExist(OnDoneFunc)
}

type delegatingRouteConfigWatcher struct {
	watcher RouteConfigWatcher
}

func (d *delegatingRouteConfigWatcher) OnUpdate(data ResourceData, onDone OnDoneFunc) {
	rc := data.(*RouteConfigResourceData)
	d.watcher.OnUpdate(rc, onDone)
}

func (d *delegatingRouteConfigWatcher) OnError(err error, onDone OnDoneFunc) {
	d.watcher.OnError(err, onDone)
}

func (d *delegatingRouteConfigWatcher) OnResourceDoesNotExist(onDone OnDoneFunc) {
	d.watcher.OnResourceDoesNotExist(onDone)
}

// WatchRouteConfig uses xDS to discover the configuration associated with the
// provided route configuration resource name.
func WatchRouteConfig(p Producer, name string, w RouteConfigWatcher) (cancel func()) {
	delegator := &delegatingRouteConfigWatcher{watcher: w}
	return p.WatchResource(routeConfigType, name, delegator)
}
