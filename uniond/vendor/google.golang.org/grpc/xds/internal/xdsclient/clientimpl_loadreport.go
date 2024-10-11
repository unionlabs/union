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
 */

package xdsclient

import (
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal/xdsclient/load"
)

// ReportLoad starts a load reporting stream to the given server. All load
// reports to the same server share the LRS stream.
//
// It returns a Store for the user to report loads, a function to cancel the
// load reporting stream.
func (c *clientImpl) ReportLoad(server *bootstrap.ServerConfig) (*load.Store, func()) {
	c.authorityMu.Lock()
	a, err := c.newAuthorityLocked(server)
	if err != nil {
		c.authorityMu.Unlock()
		c.logger.Warningf("Failed to connect to the management server to report load for authority %q: %v", server, err)
		return nil, func() {}
	}
	// Hold the ref before starting load reporting.
	a.refLocked()
	c.authorityMu.Unlock()

	store, cancelF := a.reportLoad()
	return store, func() {
		cancelF()
		c.unrefAuthority(a)
	}
}
