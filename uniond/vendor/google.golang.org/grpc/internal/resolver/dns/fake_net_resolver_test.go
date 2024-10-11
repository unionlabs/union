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

package dns_test

import (
	"context"
	"net"
	"sync"

	"google.golang.org/grpc/internal/testutils"
)

// A fake implementation of the internal.NetResolver interface for use in tests.
type testNetResolver struct {
	// A write to this channel is made when this resolver receives a resolution
	// request. Tests can rely on reading from this channel to be notified about
	// resolution requests instead of sleeping for a predefined period of time.
	lookupHostCh *testutils.Channel

	mu              sync.Mutex
	hostLookupTable map[string][]string   // Name --> list of addresses
	srvLookupTable  map[string][]*net.SRV // Name --> list of SRV records
	txtLookupTable  map[string][]string   // Name --> service config for TXT record
}

func (tr *testNetResolver) LookupHost(ctx context.Context, host string) ([]string, error) {
	if tr.lookupHostCh != nil {
		if err := tr.lookupHostCh.SendContext(ctx, nil); err != nil {
			return nil, err
		}
	}

	tr.mu.Lock()
	defer tr.mu.Unlock()

	if addrs, ok := tr.hostLookupTable[host]; ok {
		return addrs, nil
	}

	return nil, &net.DNSError{
		Err:         "hostLookup error",
		Name:        host,
		Server:      "fake",
		IsTemporary: true,
	}
}

func (tr *testNetResolver) UpdateHostLookupTable(table map[string][]string) {
	tr.mu.Lock()
	tr.hostLookupTable = table
	tr.mu.Unlock()
}

func (tr *testNetResolver) LookupSRV(ctx context.Context, service, proto, name string) (string, []*net.SRV, error) {
	tr.mu.Lock()
	defer tr.mu.Unlock()

	cname := "_" + service + "._" + proto + "." + name
	if srvs, ok := tr.srvLookupTable[cname]; ok {
		return cname, srvs, nil
	}
	return "", nil, &net.DNSError{
		Err:         "srvLookup error",
		Name:        cname,
		Server:      "fake",
		IsTemporary: true,
	}
}

func (tr *testNetResolver) LookupTXT(ctx context.Context, host string) ([]string, error) {
	tr.mu.Lock()
	defer tr.mu.Unlock()

	if sc, ok := tr.txtLookupTable[host]; ok {
		return sc, nil
	}
	return nil, &net.DNSError{
		Err:         "txtLookup error",
		Name:        host,
		Server:      "fake",
		IsTemporary: true,
	}
}

func (tr *testNetResolver) UpdateTXTLookupTable(table map[string][]string) {
	tr.mu.Lock()
	tr.txtLookupTable = table
	tr.mu.Unlock()
}

// txtRecordServiceConfig generates a slice of strings (aggregately representing
// a single service config file) for the input config string, that represents
// the result from a real DNS TXT record lookup.
func txtRecordServiceConfig(cfg string) []string {
	// In DNS, service config is encoded in a TXT record via the mechanism
	// described in RFC-1464 using the attribute name grpc_config.
	b := append([]byte("grpc_config="), []byte(cfg)...)

	// Split b into multiple strings, each with a max of 255 bytes, which is
	// the DNS TXT record limit.
	var r []string
	for i := 0; i < len(b); i += txtBytesLimit {
		if i+txtBytesLimit > len(b) {
			r = append(r, string(b[i:]))
		} else {
			r = append(r, string(b[i:i+txtBytesLimit]))
		}
	}
	return r
}
