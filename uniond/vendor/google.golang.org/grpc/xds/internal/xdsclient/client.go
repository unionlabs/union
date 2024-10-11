/*
 *
 * Copyright 2019 gRPC authors.
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

// Package xdsclient implements a full fledged gRPC client for the xDS API used
// by the xds resolver and balancer implementations.
package xdsclient

import (
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal/xdsclient/load"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"
)

// XDSClient is a full fledged gRPC client which queries a set of discovery APIs
// (collectively termed as xDS) on a remote management server, to discover
// various dynamic resources.
type XDSClient interface {
	// WatchResource uses xDS to discover the resource associated with the
	// provided resource name. The resource type implementation determines how
	// xDS requests are sent out and how responses are deserialized and
	// validated. Upon receipt of a response from the management server, an
	// appropriate callback on the watcher is invoked.
	//
	// Most callers will not have a need to use this API directly. They will
	// instead use a resource-type-specific wrapper API provided by the relevant
	// resource type implementation.
	//
	//
	// During a race (e.g. an xDS response is received while the user is calling
	// cancel()), there's a small window where the callback can be called after
	// the watcher is canceled. Callers need to handle this case.
	WatchResource(rType xdsresource.Type, resourceName string, watcher xdsresource.ResourceWatcher) (cancel func())

	ReportLoad(*bootstrap.ServerConfig) (*load.Store, func())

	BootstrapConfig() *bootstrap.Config
}
