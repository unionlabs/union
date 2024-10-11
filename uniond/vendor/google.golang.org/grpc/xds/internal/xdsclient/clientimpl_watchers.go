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
 */

package xdsclient

import (
	"context"
	"fmt"
	"sync"

	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"
)

// WatchResource uses xDS to discover the resource associated with the provided
// resource name. The resource type implementation determines how xDS requests
// are sent out and how responses are deserialized and validated. Upon receipt
// of a response from the management server, an appropriate callback on the
// watcher is invoked.
func (c *clientImpl) WatchResource(rType xdsresource.Type, resourceName string, watcher xdsresource.ResourceWatcher) (cancel func()) {
	// Return early if the client is already closed.
	//
	// The client returned from the top-level API is a ref-counted client which
	// contains a pointer to `clientImpl`. When all references are released, the
	// ref-counted client sets its pointer to `nil`. And if any watch APIs are
	// made on such a closed client, we will get here with a `nil` receiver.
	if c == nil || c.done.HasFired() {
		logger.Warningf("Watch registered for name %q of type %q, but client is closed", rType.TypeName(), resourceName)
		return func() {}
	}

	if err := c.resourceTypes.maybeRegister(rType); err != nil {
		logger.Warningf("Watch registered for name %q of type %q which is already registered", rType.TypeName(), resourceName)
		c.serializer.TrySchedule(func(context.Context) { watcher.OnError(err, func() {}) })
		return func() {}
	}

	// TODO: Make ParseName return an error if parsing fails, and
	// schedule the OnError callback in that case.
	n := xdsresource.ParseName(resourceName)
	a, unref, err := c.findAuthority(n)
	if err != nil {
		logger.Warningf("Watch registered for name %q of type %q, authority %q is not found", rType.TypeName(), resourceName, n.Authority)
		c.serializer.TrySchedule(func(context.Context) { watcher.OnError(err, func() {}) })
		return func() {}
	}
	cancelF := a.watchResource(rType, n.String(), watcher)
	return func() {
		cancelF()
		unref()
	}
}

// A registry of xdsresource.Type implementations indexed by their corresponding
// type URLs. Registration of an xdsresource.Type happens the first time a watch
// for a resource of that type is invoked.
type resourceTypeRegistry struct {
	mu    sync.Mutex
	types map[string]xdsresource.Type
}

func newResourceTypeRegistry() *resourceTypeRegistry {
	return &resourceTypeRegistry{types: make(map[string]xdsresource.Type)}
}

func (r *resourceTypeRegistry) get(url string) xdsresource.Type {
	r.mu.Lock()
	defer r.mu.Unlock()
	return r.types[url]
}

func (r *resourceTypeRegistry) maybeRegister(rType xdsresource.Type) error {
	r.mu.Lock()
	defer r.mu.Unlock()

	url := rType.TypeURL()
	typ, ok := r.types[url]
	if ok && typ != rType {
		return fmt.Errorf("attempt to re-register a resource type implementation for %v", rType.TypeName())
	}
	r.types[url] = rType
	return nil
}

func (c *clientImpl) triggerResourceNotFoundForTesting(rType xdsresource.Type, resourceName string) error {
	if c == nil || c.done.HasFired() {
		return fmt.Errorf("attempt to trigger resource-not-found-error for resource %q of type %q, but client is closed", rType.TypeName(), resourceName)
	}

	n := xdsresource.ParseName(resourceName)
	a, unref, err := c.findAuthority(n)
	if err != nil {
		return fmt.Errorf("attempt to trigger resource-not-found-error for resource %q of type %q, but authority %q is not found", rType.TypeName(), resourceName, n.Authority)
	}
	defer unref()
	a.triggerResourceNotFoundForTesting(rType, n.String())
	return nil
}
