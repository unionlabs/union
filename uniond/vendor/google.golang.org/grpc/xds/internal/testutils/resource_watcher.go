/*
 *
 * Copyright 2023 gRPC authors.
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

package testutils

import "google.golang.org/grpc/xds/internal/xdsclient/xdsresource"

// TestResourceWatcher implements the xdsresource.ResourceWatcher interface,
// used to receive updates on watches registered with the xDS client, when using
// the resource-type agnostic WatchResource API.
//
// Tests can use the channels provided by this type to get access to updates and
// errors sent by the xDS client.
type TestResourceWatcher struct {
	// UpdateCh is the channel on which xDS client updates are delivered.
	UpdateCh chan *xdsresource.ResourceData
	// ErrorCh is the channel on which errors from the xDS client are delivered.
	ErrorCh chan error
	// ResourceDoesNotExistCh is the channel used to indicate calls to OnResourceDoesNotExist
	ResourceDoesNotExistCh chan struct{}
}

// OnUpdate is invoked by the xDS client to report the latest update on the resource
// being watched.
func (w *TestResourceWatcher) OnUpdate(data xdsresource.ResourceData, onDone xdsresource.OnDoneFunc) {
	defer onDone()
	select {
	case <-w.UpdateCh:
	default:
	}
	w.UpdateCh <- &data
}

// OnError is invoked by the xDS client to report the latest error.
func (w *TestResourceWatcher) OnError(err error, onDone xdsresource.OnDoneFunc) {
	defer onDone()
	select {
	case <-w.ErrorCh:
	default:
	}
	w.ErrorCh <- err
}

// OnResourceDoesNotExist is used by the xDS client to report that the resource
// being watched no longer exists.
func (w *TestResourceWatcher) OnResourceDoesNotExist(onDone xdsresource.OnDoneFunc) {
	defer onDone()
	select {
	case <-w.ResourceDoesNotExistCh:
	default:
	}
	w.ResourceDoesNotExistCh <- struct{}{}
}

// NewTestResourceWatcher returns a TestResourceWatcher to watch for resources
// via the xDS client.
func NewTestResourceWatcher() *TestResourceWatcher {
	return &TestResourceWatcher{
		UpdateCh:               make(chan *xdsresource.ResourceData, 1),
		ErrorCh:                make(chan error, 1),
		ResourceDoesNotExistCh: make(chan struct{}, 1),
	}
}
