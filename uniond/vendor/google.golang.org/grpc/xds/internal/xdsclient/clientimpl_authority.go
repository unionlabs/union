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

package xdsclient

import (
	"errors"
	"fmt"

	"google.golang.org/grpc/internal/grpclog"
	"google.golang.org/grpc/internal/xds/bootstrap"
	"google.golang.org/grpc/xds/internal/xdsclient/xdsresource"
)

// findAuthority returns the authority for this name. If it doesn't already
// exist, one will be created.
//
// Note that this doesn't always create new authority. authorities with the same
// config but different names are shared.
//
// The returned unref function must be called when the caller is done using this
// authority, without holding c.authorityMu.
//
// Caller must not hold c.authorityMu.
func (c *clientImpl) findAuthority(n *xdsresource.Name) (*authority, func(), error) {
	scheme, authority := n.Scheme, n.Authority

	c.authorityMu.Lock()
	defer c.authorityMu.Unlock()
	if c.done.HasFired() {
		return nil, nil, errors.New("the xds-client is closed")
	}

	config := c.config.XDSServers()[0]
	if scheme == xdsresource.FederationScheme {
		authorities := c.config.Authorities()
		if authorities == nil {
			return nil, nil, fmt.Errorf("xds: failed to find authority %q", authority)
		}
		cfg, ok := authorities[authority]
		if !ok {
			return nil, nil, fmt.Errorf("xds: failed to find authority %q", authority)
		}
		if len(cfg.XDSServers) >= 1 {
			config = cfg.XDSServers[0]
		}
	}

	a, err := c.newAuthorityLocked(config)
	if err != nil {
		return nil, nil, fmt.Errorf("xds: failed to connect to the control plane for authority %q: %v", authority, err)
	}
	// All returned authority from this function will be used by a watch,
	// holding the ref here.
	//
	// Note that this must be done while c.authorityMu is held, to avoid the
	// race that an authority is returned, but before the watch starts, the
	// old last watch is canceled (in another goroutine), causing this
	// authority to be removed, and then a watch will start on a removed
	// authority.
	//
	// unref() will be done when the watch is canceled.
	a.refLocked()
	return a, func() { c.unrefAuthority(a) }, nil
}

// newAuthorityLocked creates a new authority for the given config.  If an
// authority for the given config exists in the cache, it is returned instead of
// creating a new one.
//
// The caller must take a reference of the returned authority before using, and
// unref afterwards.
//
// caller must hold c.authorityMu
func (c *clientImpl) newAuthorityLocked(config *bootstrap.ServerConfig) (_ *authority, retErr error) {
	// First check if there's already an authority for this config. If found, it
	// means this authority is used by other watches (could be the same
	// authority name, or a different authority name but the same server
	// config). Return it.
	configStr := config.String()
	if a, ok := c.authorities[configStr]; ok {
		return a, nil
	}
	// Second check if there's an authority in the idle cache. If found, it
	// means this authority was created, but moved to the idle cache because the
	// watch was canceled. Move it from idle cache to the authority cache, and
	// return.
	if old, ok := c.idleAuthorities.Remove(configStr); ok {
		oldA, _ := old.(*authority)
		if oldA != nil {
			c.authorities[configStr] = oldA
			return oldA, nil
		}
	}

	// Make a new authority since there's no existing authority for this config.
	ret, err := newAuthority(authorityArgs{
		serverCfg:          config,
		bootstrapCfg:       c.config,
		serializer:         c.serializer,
		resourceTypeGetter: c.resourceTypes.get,
		watchExpiryTimeout: c.watchExpiryTimeout,
		logger:             grpclog.NewPrefixLogger(logger, authorityPrefix(c, config.ServerURI())),
	})
	if err != nil {
		return nil, fmt.Errorf("creating new authority for config %q: %v", config.String(), err)
	}
	// Add it to the cache, so it will be reused.
	c.authorities[configStr] = ret
	return ret, nil
}

// unrefAuthority unrefs the authority. It also moves the authority to idle
// cache if it's ref count is 0.
//
// This function doesn't need to called explicitly. It's called by the returned
// unref from findAuthority().
//
// Caller must not hold c.authorityMu.
func (c *clientImpl) unrefAuthority(a *authority) {
	c.authorityMu.Lock()
	defer c.authorityMu.Unlock()
	if a.unrefLocked() > 0 {
		return
	}
	configStr := a.serverCfg.String()
	delete(c.authorities, configStr)
	c.idleAuthorities.Add(configStr, a, func() {
		a.close()
	})
}
