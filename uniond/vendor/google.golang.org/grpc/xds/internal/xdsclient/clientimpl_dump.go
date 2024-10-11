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

package xdsclient

import (
	v3statuspb "github.com/envoyproxy/go-control-plane/envoy/service/status/v3"
)

// dumpResources returns the status and contents of all xDS resources.
func (c *clientImpl) dumpResources() *v3statuspb.ClientConfig {
	c.authorityMu.Lock()
	defer c.authorityMu.Unlock()

	var retCfg []*v3statuspb.ClientConfig_GenericXdsConfig
	for _, a := range c.authorities {
		retCfg = append(retCfg, a.dumpResources()...)
	}

	return &v3statuspb.ClientConfig{
		Node:              c.config.Node(),
		GenericXdsConfigs: retCfg,
	}
}

// DumpResources returns the status and contents of all xDS resources.
func DumpResources() *v3statuspb.ClientStatusResponse {
	clientsMu.Lock()
	defer clientsMu.Unlock()

	resp := &v3statuspb.ClientStatusResponse{}
	for key, client := range clients {
		cfg := client.dumpResources()
		cfg.ClientScope = key
		resp.Config = append(resp.Config, cfg)
	}
	return resp
}
