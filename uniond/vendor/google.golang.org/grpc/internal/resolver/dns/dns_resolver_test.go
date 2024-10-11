/*
 *
 * Copyright 2018 gRPC authors.
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
	"errors"
	"fmt"
	"net"
	"strings"
	"sync/atomic"
	"testing"
	"time"

	"github.com/google/go-cmp/cmp"
	"github.com/google/go-cmp/cmp/cmpopts"
	"google.golang.org/grpc/balancer"
	grpclbstate "google.golang.org/grpc/balancer/grpclb/state"
	"google.golang.org/grpc/internal"
	"google.golang.org/grpc/internal/envconfig"
	"google.golang.org/grpc/internal/grpctest"
	"google.golang.org/grpc/internal/resolver/dns"
	dnsinternal "google.golang.org/grpc/internal/resolver/dns/internal"
	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/resolver"
	dnspublic "google.golang.org/grpc/resolver/dns"
	"google.golang.org/grpc/serviceconfig"

	_ "google.golang.org/grpc" // To initialize internal.ParseServiceConfig
)

const (
	txtBytesLimit           = 255
	defaultTestTimeout      = 10 * time.Second
	defaultTestShortTimeout = 10 * time.Millisecond

	colonDefaultPort = ":443"
)

type s struct {
	grpctest.Tester
}

func Test(t *testing.T) {
	grpctest.RunSubTests(t, s{})
}

// Override the default net.Resolver with a test resolver.
func overrideNetResolver(t *testing.T, r *testNetResolver) {
	origNetResolver := dnsinternal.NewNetResolver
	dnsinternal.NewNetResolver = func(string) (dnsinternal.NetResolver, error) { return r, nil }
	t.Cleanup(func() { dnsinternal.NewNetResolver = origNetResolver })
}

// Override the DNS minimum resolution interval used by the resolver.
func overrideResolutionInterval(t *testing.T, d time.Duration) {
	origMinResInterval := dns.MinResolutionInterval
	dnspublic.SetMinResolutionInterval(d)
	t.Cleanup(func() { dnspublic.SetMinResolutionInterval(origMinResInterval) })
}

// Override the timer used by the DNS resolver to fire after a duration of d.
func overrideTimeAfterFunc(t *testing.T, d time.Duration) {
	origTimeAfter := dnsinternal.TimeAfterFunc
	dnsinternal.TimeAfterFunc = func(time.Duration) <-chan time.Time {
		return time.After(d)
	}
	t.Cleanup(func() { dnsinternal.TimeAfterFunc = origTimeAfter })
}

// Override the timer used by the DNS resolver as follows:
// - use the durChan to read the duration that the resolver wants to wait for
// - use the timerChan to unblock the wait on the timer
func overrideTimeAfterFuncWithChannel(t *testing.T) (durChan chan time.Duration, timeChan chan time.Time) {
	origTimeAfter := dnsinternal.TimeAfterFunc
	durChan = make(chan time.Duration, 1)
	timeChan = make(chan time.Time)
	dnsinternal.TimeAfterFunc = func(d time.Duration) <-chan time.Time {
		select {
		case durChan <- d:
		default:
		}
		return timeChan
	}
	t.Cleanup(func() { dnsinternal.TimeAfterFunc = origTimeAfter })
	return durChan, timeChan
}

// Override the current time used by the DNS resolver.
func overrideTimeNowFunc(t *testing.T, now time.Time) {
	origTimeNowFunc := dnsinternal.TimeNowFunc
	dnsinternal.TimeNowFunc = func() time.Time { return now }
	t.Cleanup(func() { dnsinternal.TimeNowFunc = origTimeNowFunc })
}

// Override the remaining wait time to allow re-resolution by DNS resolver.
// Use the timeChan to read the time until resolver needs to wait for
// and return 0 wait time.
func overrideTimeUntilFuncWithChannel(t *testing.T) (timeChan chan time.Time) {
	timeCh := make(chan time.Time, 1)
	origTimeUntil := dnsinternal.TimeUntilFunc
	dnsinternal.TimeUntilFunc = func(t time.Time) time.Duration {
		timeCh <- t
		return 0
	}
	t.Cleanup(func() { dnsinternal.TimeUntilFunc = origTimeUntil })
	return timeCh
}

func enableSRVLookups(t *testing.T) {
	origEnableSRVLookups := dns.EnableSRVLookups
	dns.EnableSRVLookups = true
	t.Cleanup(func() { dns.EnableSRVLookups = origEnableSRVLookups })
}

// Builds a DNS resolver for target and returns a couple of channels to read the
// state and error pushed by the resolver respectively.
func buildResolverWithTestClientConn(t *testing.T, target string) (resolver.Resolver, chan resolver.State, chan error) {
	t.Helper()

	b := resolver.Get("dns")
	if b == nil {
		t.Fatalf("Resolver for dns:/// scheme not registered")
	}

	stateCh := make(chan resolver.State, 1)
	updateStateF := func(s resolver.State) error {
		select {
		case stateCh <- s:
		default:
		}
		return nil
	}

	errCh := make(chan error, 1)
	reportErrorF := func(err error) {
		select {
		case errCh <- err:
		default:
		}
	}

	tcc := &testutils.ResolverClientConn{Logger: t, UpdateStateF: updateStateF, ReportErrorF: reportErrorF}
	r, err := b.Build(resolver.Target{URL: *testutils.MustParseURL(fmt.Sprintf("dns:///%s", target))}, tcc, resolver.BuildOptions{})
	if err != nil {
		t.Fatalf("Failed to build DNS resolver for target %q: %v\n", target, err)
	}
	t.Cleanup(func() { r.Close() })

	return r, stateCh, errCh
}

// Waits for a state update from the DNS resolver and verifies the following:
// - wantAddrs matches the list of addresses in the update
// - wantBalancerAddrs matches the list of grpclb addresses in the update
// - wantSC matches the service config in the update
func verifyUpdateFromResolver(ctx context.Context, t *testing.T, stateCh chan resolver.State, wantAddrs, wantBalancerAddrs []resolver.Address, wantSC string) {
	t.Helper()

	var state resolver.State
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for a state update from the resolver")
	case state = <-stateCh:
	}

	if !cmp.Equal(state.Addresses, wantAddrs, cmpopts.EquateEmpty()) {
		t.Fatalf("Got addresses: %+v, want: %+v", state.Addresses, wantAddrs)
	}
	if gs := grpclbstate.Get(state); gs == nil {
		if len(wantBalancerAddrs) > 0 {
			t.Fatalf("Got no grpclb addresses. Want %d", len(wantBalancerAddrs))
		}
	} else {
		if !cmp.Equal(gs.BalancerAddresses, wantBalancerAddrs) {
			t.Fatalf("Got grpclb addresses %+v, want %+v", gs.BalancerAddresses, wantBalancerAddrs)
		}
	}
	if wantSC == "{}" {
		if state.ServiceConfig != nil && state.ServiceConfig.Config != nil {
			t.Fatalf("Got service config:\n%s \nWant service config: {}", cmp.Diff(nil, state.ServiceConfig.Config))
		}

	} else if wantSC != "" {
		wantSCParsed := internal.ParseServiceConfig.(func(string) *serviceconfig.ParseResult)(wantSC)
		if !internal.EqualServiceConfigForTesting(state.ServiceConfig.Config, wantSCParsed.Config) {
			t.Fatalf("Got service config:\n%s \nWant service config:\n%s", cmp.Diff(nil, state.ServiceConfig.Config), cmp.Diff(nil, wantSCParsed.Config))
		}
	}
}

// This is the service config used by the fake net.Resolver in its TXT record.
//   - it contains an array of 5 entries
//   - the first three will be dropped by the DNS resolver as part of its
//     canarying rule matching functionality:
//   - the client language does not match in the first entry
//   - the percentage is set to 0 in the second entry
//   - the client host name does not match in the third entry
//   - the fourth and fifth entries will match the canarying rules, and therefore
//     the fourth entry will be used as it will be  the first matching entry.
const txtRecordGood = `
[
	{
		"clientLanguage": [
			"CPP",
			"JAVA"
		],
		"serviceConfig": {
			"loadBalancingPolicy": "grpclb",
			"methodConfig": [
				{
					"name": [
						{
							"service": "all"
						}
					],
					"timeout": "1s"
				}
			]
		}
	},
	{
		"percentage": 0,
		"serviceConfig": {
			"loadBalancingPolicy": "grpclb",
			"methodConfig": [
				{
					"name": [
						{
							"service": "all"
						}
					],
					"timeout": "1s"
				}
			]
		}
	},
	{
		"clientHostName": [
			"localhost"
		],
		"serviceConfig": {
			"loadBalancingPolicy": "grpclb",
			"methodConfig": [
				{
					"name": [
						{
							"service": "all"
						}
					],
					"timeout": "1s"
				}
			]
		}
	},
	{
		"clientLanguage": [
			"GO"
		],
		"percentage": 100,
		"serviceConfig": {
			"loadBalancingPolicy": "round_robin",
			"methodConfig": [
				{
					"name": [
						{
							"service": "foo"
						}
					],
					"waitForReady": true,
					"timeout": "1s"
				},
				{
					"name": [
						{
							"service": "bar"
						}
					],
					"waitForReady": false
				}
			]
		}
	},
	{
		"serviceConfig": {
			"loadBalancingPolicy": "round_robin",
			"methodConfig": [
				{
					"name": [
						{
							"service": "foo",
							"method": "bar"
						}
					],
					"waitForReady": true
				}
			]
		}
	}
]`

// This is the matched portion of the above TXT record entry.
const scJSON = `
{
	"loadBalancingPolicy": "round_robin",
	"methodConfig": [
		{
			"name": [
				{
					"service": "foo"
				}
			],
			"waitForReady": true,
			"timeout": "1s"
		},
		{
			"name": [
				{
					"service": "bar"
				}
			],
			"waitForReady": false
		}
	]
}`

// This service config contains three entries, but none of the match the DNS
// resolver's canarying rules and hence the resulting service config pushed by
// the DNS resolver will be an empty one.
const txtRecordNonMatching = `
[
	{
		"clientLanguage": [
			"CPP",
			"JAVA"
		],
		"serviceConfig": {
			"loadBalancingPolicy": "grpclb",
			"methodConfig": [
				{
					"name": [
						{
							"service": "all"
						}
					],
					"timeout": "1s"
				}
			]
		}
	},
	{
		"percentage": 0,
		"serviceConfig": {
			"loadBalancingPolicy": "grpclb",
			"methodConfig": [
				{
					"name": [
						{
							"service": "all"
						}
					],
					"timeout": "1s"
				}
			]
		}
	},
	{
		"clientHostName": [
			"localhost"
		],
		"serviceConfig": {
			"loadBalancingPolicy": "grpclb",
			"methodConfig": [
				{
					"name": [
						{
							"service": "all"
						}
					],
					"timeout": "1s"
				}
			]
		}
	}
]`

// Tests the scenario where a name resolves to a list of addresses, possibly
// some grpclb addresses as well, and a service config. The test verifies that
// the expected update is pushed to the channel.
func (s) TestDNSResolver_Basic(t *testing.T) {
	tests := []struct {
		name              string
		target            string
		hostLookupTable   map[string][]string
		srvLookupTable    map[string][]*net.SRV
		txtLookupTable    map[string][]string
		wantAddrs         []resolver.Address
		wantBalancerAddrs []resolver.Address
		wantSC            string
	}{
		{
			name:   "default_port",
			target: "foo.bar.com",
			hostLookupTable: map[string][]string{
				"foo.bar.com": {"1.2.3.4", "5.6.7.8"},
			},
			txtLookupTable: map[string][]string{
				"_grpc_config.foo.bar.com": txtRecordServiceConfig(txtRecordGood),
			},
			wantAddrs:         []resolver.Address{{Addr: "1.2.3.4" + colonDefaultPort}, {Addr: "5.6.7.8" + colonDefaultPort}},
			wantBalancerAddrs: nil,
			wantSC:            scJSON,
		},
		{
			name:   "specified_port",
			target: "foo.bar.com:1234",
			hostLookupTable: map[string][]string{
				"foo.bar.com": {"1.2.3.4", "5.6.7.8"},
			},
			txtLookupTable: map[string][]string{
				"_grpc_config.foo.bar.com": txtRecordServiceConfig(txtRecordGood),
			},
			wantAddrs:         []resolver.Address{{Addr: "1.2.3.4:1234"}, {Addr: "5.6.7.8:1234"}},
			wantBalancerAddrs: nil,
			wantSC:            scJSON,
		},
		{
			name:   "ipv4_with_SRV_and_single_grpclb_address",
			target: "srv.ipv4.single.fake",
			hostLookupTable: map[string][]string{
				"srv.ipv4.single.fake": {"2.4.6.8"},
				"ipv4.single.fake":     {"1.2.3.4"},
			},
			srvLookupTable: map[string][]*net.SRV{
				"_grpclb._tcp.srv.ipv4.single.fake": {&net.SRV{Target: "ipv4.single.fake", Port: 1234}},
			},
			txtLookupTable: map[string][]string{
				"_grpc_config.srv.ipv4.single.fake": txtRecordServiceConfig(txtRecordGood),
			},
			wantAddrs:         []resolver.Address{{Addr: "2.4.6.8" + colonDefaultPort}},
			wantBalancerAddrs: []resolver.Address{{Addr: "1.2.3.4:1234", ServerName: "ipv4.single.fake"}},
			wantSC:            scJSON,
		},
		{
			name:   "ipv4_with_SRV_and_multiple_grpclb_address",
			target: "srv.ipv4.multi.fake",
			hostLookupTable: map[string][]string{
				"ipv4.multi.fake": {"1.2.3.4", "5.6.7.8", "9.10.11.12"},
			},
			srvLookupTable: map[string][]*net.SRV{
				"_grpclb._tcp.srv.ipv4.multi.fake": {&net.SRV{Target: "ipv4.multi.fake", Port: 1234}},
			},
			txtLookupTable: map[string][]string{
				"_grpc_config.srv.ipv4.multi.fake": txtRecordServiceConfig(txtRecordGood),
			},
			wantAddrs: nil,
			wantBalancerAddrs: []resolver.Address{
				{Addr: "1.2.3.4:1234", ServerName: "ipv4.multi.fake"},
				{Addr: "5.6.7.8:1234", ServerName: "ipv4.multi.fake"},
				{Addr: "9.10.11.12:1234", ServerName: "ipv4.multi.fake"},
			},
			wantSC: scJSON,
		},
		{
			name:   "ipv6_with_SRV_and_single_grpclb_address",
			target: "srv.ipv6.single.fake",
			hostLookupTable: map[string][]string{
				"srv.ipv6.single.fake": nil,
				"ipv6.single.fake":     {"2607:f8b0:400a:801::1001"},
			},
			srvLookupTable: map[string][]*net.SRV{
				"_grpclb._tcp.srv.ipv6.single.fake": {&net.SRV{Target: "ipv6.single.fake", Port: 1234}},
			},
			txtLookupTable: map[string][]string{
				"_grpc_config.srv.ipv6.single.fake": txtRecordServiceConfig(txtRecordNonMatching),
			},
			wantAddrs:         nil,
			wantBalancerAddrs: []resolver.Address{{Addr: "[2607:f8b0:400a:801::1001]:1234", ServerName: "ipv6.single.fake"}},
			wantSC:            "{}",
		},
		{
			name:   "ipv6_with_SRV_and_multiple_grpclb_address",
			target: "srv.ipv6.multi.fake",
			hostLookupTable: map[string][]string{
				"srv.ipv6.multi.fake": nil,
				"ipv6.multi.fake":     {"2607:f8b0:400a:801::1001", "2607:f8b0:400a:801::1002", "2607:f8b0:400a:801::1003"},
			},
			srvLookupTable: map[string][]*net.SRV{
				"_grpclb._tcp.srv.ipv6.multi.fake": {&net.SRV{Target: "ipv6.multi.fake", Port: 1234}},
			},
			txtLookupTable: map[string][]string{
				"_grpc_config.srv.ipv6.multi.fake": txtRecordServiceConfig(txtRecordNonMatching),
			},
			wantAddrs: nil,
			wantBalancerAddrs: []resolver.Address{
				{Addr: "[2607:f8b0:400a:801::1001]:1234", ServerName: "ipv6.multi.fake"},
				{Addr: "[2607:f8b0:400a:801::1002]:1234", ServerName: "ipv6.multi.fake"},
				{Addr: "[2607:f8b0:400a:801::1003]:1234", ServerName: "ipv6.multi.fake"},
			},
			wantSC: "{}",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			overrideTimeAfterFunc(t, 2*defaultTestTimeout)
			overrideNetResolver(t, &testNetResolver{
				hostLookupTable: test.hostLookupTable,
				srvLookupTable:  test.srvLookupTable,
				txtLookupTable:  test.txtLookupTable,
			})
			enableSRVLookups(t)
			_, stateCh, _ := buildResolverWithTestClientConn(t, test.target)

			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			verifyUpdateFromResolver(ctx, t, stateCh, test.wantAddrs, test.wantBalancerAddrs, test.wantSC)
		})
	}
}

// Tests the case where the channel returns an error for the update pushed by
// the DNS resolver. Verifies that the DNS resolver backs off before trying to
// resolve. Once the channel returns a nil error, the test verifies that the DNS
// resolver does not backoff anymore.
func (s) TestDNSResolver_ExponentialBackoff(t *testing.T) {
	tests := []struct {
		name            string
		target          string
		hostLookupTable map[string][]string
		txtLookupTable  map[string][]string
		wantAddrs       []resolver.Address
		wantSC          string
	}{
		{
			name:            "happy case default port",
			target:          "foo.bar.com",
			hostLookupTable: map[string][]string{"foo.bar.com": {"1.2.3.4", "5.6.7.8"}},
			txtLookupTable: map[string][]string{
				"_grpc_config.foo.bar.com": txtRecordServiceConfig(txtRecordGood),
			},
			wantAddrs: []resolver.Address{{Addr: "1.2.3.4" + colonDefaultPort}, {Addr: "5.6.7.8" + colonDefaultPort}},
			wantSC:    scJSON,
		},
		{
			name:            "happy case specified port",
			target:          "foo.bar.com:1234",
			hostLookupTable: map[string][]string{"foo.bar.com": {"1.2.3.4", "5.6.7.8"}},
			txtLookupTable: map[string][]string{
				"_grpc_config.foo.bar.com": txtRecordServiceConfig(txtRecordGood),
			},
			wantAddrs: []resolver.Address{{Addr: "1.2.3.4:1234"}, {Addr: "5.6.7.8:1234"}},
			wantSC:    scJSON,
		},
		{
			name:   "happy case another default port",
			target: "srv.ipv4.single.fake",
			hostLookupTable: map[string][]string{
				"srv.ipv4.single.fake": {"2.4.6.8"},
				"ipv4.single.fake":     {"1.2.3.4"},
			},
			txtLookupTable: map[string][]string{
				"_grpc_config.srv.ipv4.single.fake": txtRecordServiceConfig(txtRecordGood),
			},
			wantAddrs: []resolver.Address{{Addr: "2.4.6.8" + colonDefaultPort}},
			wantSC:    scJSON,
		},
	}
	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			durChan, timeChan := overrideTimeAfterFuncWithChannel(t)
			overrideNetResolver(t, &testNetResolver{
				hostLookupTable: test.hostLookupTable,
				txtLookupTable:  test.txtLookupTable,
			})

			// Set the test clientconn to return error back to the resolver when
			// it pushes an update on the channel.
			var returnNilErr atomic.Bool
			updateStateF := func(s resolver.State) error {
				if returnNilErr.Load() {
					return nil
				}
				return balancer.ErrBadResolverState
			}
			tcc := &testutils.ResolverClientConn{Logger: t, UpdateStateF: updateStateF}

			b := resolver.Get("dns")
			if b == nil {
				t.Fatalf("Resolver for dns:/// scheme not registered")
			}
			r, err := b.Build(resolver.Target{URL: *testutils.MustParseURL(fmt.Sprintf("dns:///%s", test.target))}, tcc, resolver.BuildOptions{})
			if err != nil {
				t.Fatalf("Failed to build DNS resolver for target %q: %v\n", test.target, err)
			}
			defer r.Close()

			// Expect the DNS resolver to backoff and attempt to re-resolve.
			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			const retries = 10
			var prevDur time.Duration
			for i := 0; i < retries; i++ {
				select {
				case <-ctx.Done():
					t.Fatalf("(Iteration: %d): Timeout when waiting for DNS resolver to backoff", i)
				case dur := <-durChan:
					if dur <= prevDur {
						t.Fatalf("(Iteration: %d): Unexpected decrease in amount of time to backoff", i)
					}
				}

				// Unblock the DNS resolver's backoff by pushing the current time.
				timeChan <- time.Now()
			}

			// Update resolver.ClientConn to not return an error anymore.
			returnNilErr.Store(true)

			// Unblock the DNS resolver's backoff, if ongoing, while we set the
			// test clientConn to not return an error anymore.
			select {
			case timeChan <- time.Now():
			default:
			}

			// Verify that the DNS resolver does not backoff anymore.
			sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
			defer sCancel()
			select {
			case <-durChan:
				t.Fatal("Unexpected DNS resolver backoff")
			case <-sCtx.Done():
			}
		})
	}
}

// Tests the case where the DNS resolver is asked to re-resolve by invoking the
// ResolveNow method.
func (s) TestDNSResolver_ResolveNow(t *testing.T) {
	const target = "foo.bar.com"

	overrideResolutionInterval(t, 0)
	overrideTimeAfterFunc(t, 0)
	tr := &testNetResolver{
		hostLookupTable: map[string][]string{
			"foo.bar.com": {"1.2.3.4", "5.6.7.8"},
		},
		txtLookupTable: map[string][]string{
			"_grpc_config.foo.bar.com": txtRecordServiceConfig(txtRecordGood),
		},
	}
	overrideNetResolver(t, tr)

	r, stateCh, _ := buildResolverWithTestClientConn(t, target)

	// Verify that the first update pushed by the resolver matches expectations.
	wantAddrs := []resolver.Address{{Addr: "1.2.3.4" + colonDefaultPort}, {Addr: "5.6.7.8" + colonDefaultPort}}
	wantSC := scJSON
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	verifyUpdateFromResolver(ctx, t, stateCh, wantAddrs, nil, wantSC)

	// Update state in the fake net.Resolver to return only one address and a
	// new service config.
	tr.UpdateHostLookupTable(map[string][]string{target: {"1.2.3.4"}})
	tr.UpdateTXTLookupTable(map[string][]string{
		"_grpc_config.foo.bar.com": txtRecordServiceConfig(`[{"serviceConfig":{"loadBalancingPolicy": "grpclb"}}]`),
	})

	// Ask the resolver to re-resolve and verify that the new update matches
	// expectations.
	r.ResolveNow(resolver.ResolveNowOptions{})
	wantAddrs = []resolver.Address{{Addr: "1.2.3.4" + colonDefaultPort}}
	wantSC = `{"loadBalancingPolicy": "grpclb"}`
	verifyUpdateFromResolver(ctx, t, stateCh, wantAddrs, nil, wantSC)

	// Update state in the fake resolver to return no addresses and the same
	// service config as before.
	tr.UpdateHostLookupTable(map[string][]string{target: nil})

	// Ask the resolver to re-resolve and verify that the new update matches
	// expectations.
	r.ResolveNow(resolver.ResolveNowOptions{})
	verifyUpdateFromResolver(ctx, t, stateCh, nil, nil, wantSC)
}

// Tests the case where the given name is an IP address and verifies that the
// update pushed by the DNS resolver meets expectations.
func (s) TestIPResolver(t *testing.T) {
	tests := []struct {
		name     string
		target   string
		wantAddr []resolver.Address
	}{
		{
			name:     "localhost ipv4 default port",
			target:   "127.0.0.1",
			wantAddr: []resolver.Address{{Addr: "127.0.0.1:443"}},
		},
		{
			name:     "localhost ipv4 non-default port",
			target:   "127.0.0.1:12345",
			wantAddr: []resolver.Address{{Addr: "127.0.0.1:12345"}},
		},
		{
			name:     "localhost ipv6 default port no brackets",
			target:   "::1",
			wantAddr: []resolver.Address{{Addr: "[::1]:443"}},
		},
		{
			name:     "localhost ipv6 default port with brackets",
			target:   "[::1]",
			wantAddr: []resolver.Address{{Addr: "[::1]:443"}},
		},
		{
			name:     "localhost ipv6 non-default port",
			target:   "[::1]:12345",
			wantAddr: []resolver.Address{{Addr: "[::1]:12345"}},
		},
		{
			name:     "ipv6 default port no brackets",
			target:   "2001:db8:85a3::8a2e:370:7334",
			wantAddr: []resolver.Address{{Addr: "[2001:db8:85a3::8a2e:370:7334]:443"}},
		},
		{
			name:     "ipv6 default port with brackets",
			target:   "[2001:db8:85a3::8a2e:370:7334]",
			wantAddr: []resolver.Address{{Addr: "[2001:db8:85a3::8a2e:370:7334]:443"}},
		},
		{
			name:     "ipv6 non-default port with brackets",
			target:   "[2001:db8:85a3::8a2e:370:7334]:12345",
			wantAddr: []resolver.Address{{Addr: "[2001:db8:85a3::8a2e:370:7334]:12345"}},
		},
		{
			name:     "abbreviated ipv6 address",
			target:   "[2001:db8::1]:http",
			wantAddr: []resolver.Address{{Addr: "[2001:db8::1]:http"}},
		},
		// TODO(yuxuanli): zone support?
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			overrideResolutionInterval(t, 0)
			overrideTimeAfterFunc(t, 2*defaultTestTimeout)
			r, stateCh, _ := buildResolverWithTestClientConn(t, test.target)

			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			verifyUpdateFromResolver(ctx, t, stateCh, test.wantAddr, nil, "")

			// Attempt to re-resolve should not result in a state update.
			r.ResolveNow(resolver.ResolveNowOptions{})
			sCtx, sCancel := context.WithTimeout(ctx, defaultTestShortTimeout)
			defer sCancel()
			select {
			case <-sCtx.Done():
			case s := <-stateCh:
				t.Fatalf("Unexpected state update from the resolver: %+v", s)
			}
		})
	}
}

// Tests the DNS resolver builder with different target names.
func (s) TestResolverBuild(t *testing.T) {
	tests := []struct {
		name    string
		target  string
		wantErr string
	}{
		{
			name:   "valid url",
			target: "www.google.com",
		},
		{
			name:   "host port",
			target: "foo.bar:12345",
		},
		{
			name:   "ipv4 address with default port",
			target: "127.0.0.1",
		},
		{
			name:   "ipv6 address without brackets and default port",
			target: "::",
		},
		{
			name:   "ipv4 address with non-default port",
			target: "127.0.0.1:12345",
		},
		{
			name:   "localhost ipv6 with brackets",
			target: "[::1]:80",
		},
		{
			name:   "ipv6 address with brackets",
			target: "[2001:db8:a0b:12f0::1]:21",
		},
		{
			name:   "empty host with port",
			target: ":80",
		},
		{
			name:   "ipv6 address with zone",
			target: "[fe80::1%25lo0]:80",
		},
		{
			name:   "url with port",
			target: "golang.org:http",
		},
		{
			name:   "ipv6 address with non integer port",
			target: "[2001:db8::1]:http",
		},
		{
			name:    "address ends with colon",
			target:  "[2001:db8::1]:",
			wantErr: dnsinternal.ErrEndsWithColon.Error(),
		},
		{
			name:    "address contains only a colon",
			target:  ":",
			wantErr: dnsinternal.ErrEndsWithColon.Error(),
		},
		{
			name:    "empty address",
			target:  "",
			wantErr: dnsinternal.ErrMissingAddr.Error(),
		},
		{
			name:    "invalid address",
			target:  "[2001:db8:a0b:12f0::1",
			wantErr: "invalid target address",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			overrideTimeAfterFunc(t, 2*defaultTestTimeout)

			b := resolver.Get("dns")
			if b == nil {
				t.Fatalf("Resolver for dns:/// scheme not registered")
			}

			tcc := &testutils.ResolverClientConn{Logger: t}
			r, err := b.Build(resolver.Target{URL: *testutils.MustParseURL(fmt.Sprintf("dns:///%s", test.target))}, tcc, resolver.BuildOptions{})
			if err != nil {
				if test.wantErr == "" {
					t.Fatalf("DNS resolver build for target %q failed with error: %v", test.target, err)
				}
				if !strings.Contains(err.Error(), test.wantErr) {
					t.Fatalf("DNS resolver build for target %q failed with error: %v, wantErr: %s", test.target, err, test.wantErr)
				}
				return
			}
			if err == nil && test.wantErr != "" {
				t.Fatalf("DNS resolver build for target %q succeeded when expected to fail with error: %s", test.target, test.wantErr)
			}
			r.Close()
		})
	}
}

// Tests scenarios where fetching of service config is enabled or disabled, and
// verifies that the expected update is pushed by the DNS resolver.
func (s) TestDisableServiceConfig(t *testing.T) {
	tests := []struct {
		name                 string
		target               string
		hostLookupTable      map[string][]string
		txtLookupTable       map[string][]string
		disableServiceConfig bool
		wantAddrs            []resolver.Address
		wantSC               string
	}{
		{
			name:            "false",
			target:          "foo.bar.com",
			hostLookupTable: map[string][]string{"foo.bar.com": {"1.2.3.4", "5.6.7.8"}},
			txtLookupTable: map[string][]string{
				"_grpc_config.foo.bar.com": txtRecordServiceConfig(txtRecordGood),
			},
			disableServiceConfig: false,
			wantAddrs:            []resolver.Address{{Addr: "1.2.3.4" + colonDefaultPort}, {Addr: "5.6.7.8" + colonDefaultPort}},
			wantSC:               scJSON,
		},
		{
			name:            "true",
			target:          "foo.bar.com",
			hostLookupTable: map[string][]string{"foo.bar.com": {"1.2.3.4", "5.6.7.8"}},
			txtLookupTable: map[string][]string{
				"_grpc_config.foo.bar.com": txtRecordServiceConfig(txtRecordGood),
			},
			disableServiceConfig: true,
			wantAddrs:            []resolver.Address{{Addr: "1.2.3.4" + colonDefaultPort}, {Addr: "5.6.7.8" + colonDefaultPort}},
			wantSC:               "{}",
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			overrideTimeAfterFunc(t, 2*defaultTestTimeout)
			overrideNetResolver(t, &testNetResolver{
				hostLookupTable: test.hostLookupTable,
				txtLookupTable:  test.txtLookupTable,
			})

			b := resolver.Get("dns")
			if b == nil {
				t.Fatalf("Resolver for dns:/// scheme not registered")
			}

			stateCh := make(chan resolver.State, 1)
			updateStateF := func(s resolver.State) error {
				stateCh <- s
				return nil
			}
			tcc := &testutils.ResolverClientConn{Logger: t, UpdateStateF: updateStateF}
			r, err := b.Build(resolver.Target{URL: *testutils.MustParseURL(fmt.Sprintf("dns:///%s", test.target))}, tcc, resolver.BuildOptions{DisableServiceConfig: test.disableServiceConfig})
			if err != nil {
				t.Fatalf("Failed to build DNS resolver for target %q: %v\n", test.target, err)
			}
			defer r.Close()

			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			verifyUpdateFromResolver(ctx, t, stateCh, test.wantAddrs, nil, test.wantSC)
		})
	}
}

// Tests the case where a TXT lookup is expected to return an error. Verifies
// that errors are ignored with the corresponding env var is set.
func (s) TestTXTError(t *testing.T) {
	for _, ignore := range []bool{false, true} {
		t.Run(fmt.Sprintf("%v", ignore), func(t *testing.T) {
			overrideTimeAfterFunc(t, 2*defaultTestTimeout)
			overrideNetResolver(t, &testNetResolver{hostLookupTable: map[string][]string{"ipv4.single.fake": {"1.2.3.4"}}})

			origTXTIgnore := envconfig.TXTErrIgnore
			envconfig.TXTErrIgnore = ignore
			defer func() { envconfig.TXTErrIgnore = origTXTIgnore }()

			// There is no entry for "ipv4.single.fake" in the txtLookupTbl
			// maintained by the fake net.Resolver. So, a TXT lookup for this
			// name will return an error.
			_, stateCh, _ := buildResolverWithTestClientConn(t, "ipv4.single.fake")

			ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
			defer cancel()
			var state resolver.State
			select {
			case <-ctx.Done():
				t.Fatal("Timeout when waiting for a state update from the resolver")
			case state = <-stateCh:
			}

			if ignore {
				if state.ServiceConfig != nil {
					t.Fatalf("Received non-nil service config: %+v; want nil", state.ServiceConfig)
				}
			} else {
				if state.ServiceConfig == nil || state.ServiceConfig.Err == nil {
					t.Fatalf("Received service config %+v; want non-nil error", state.ServiceConfig)
				}
			}
		})
	}
}

// Tests different cases for a user's dial target that specifies a non-empty
// authority (or Host field of the URL).
func (s) TestCustomAuthority(t *testing.T) {
	tests := []struct {
		name          string
		authority     string
		wantAuthority string
		wantBuildErr  bool
	}{
		{
			name:          "authority with default DNS port",
			authority:     "4.3.2.1:53",
			wantAuthority: "4.3.2.1:53",
		},
		{
			name:          "authority with non-default DNS port",
			authority:     "4.3.2.1:123",
			wantAuthority: "4.3.2.1:123",
		},
		{
			name:          "authority with no port",
			authority:     "4.3.2.1",
			wantAuthority: "4.3.2.1:53",
		},
		{
			name:          "ipv6 authority with no port",
			authority:     "::1",
			wantAuthority: "[::1]:53",
		},
		{
			name:          "ipv6 authority with brackets and no port",
			authority:     "[::1]",
			wantAuthority: "[::1]:53",
		},
		{
			name:          "ipv6 authority with brackets and non-default DNS port",
			authority:     "[::1]:123",
			wantAuthority: "[::1]:123",
		},
		{
			name:          "host name with no port",
			authority:     "dnsserver.com",
			wantAuthority: "dnsserver.com:53",
		},
		{
			name:          "no host port and non-default port",
			authority:     ":123",
			wantAuthority: "localhost:123",
		},
		{
			name:          "only colon",
			authority:     ":",
			wantAuthority: "",
			wantBuildErr:  true,
		},
		{
			name:          "ipv6 name ending in colon",
			authority:     "[::1]:",
			wantAuthority: "",
			wantBuildErr:  true,
		},
		{
			name:          "host name ending in colon",
			authority:     "dnsserver.com:",
			wantAuthority: "",
			wantBuildErr:  true,
		},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			overrideTimeAfterFunc(t, 2*defaultTestTimeout)

			// Override the address dialer to verify the authority being passed.
			origAddressDialer := dnsinternal.AddressDialer
			errChan := make(chan error, 1)
			dnsinternal.AddressDialer = func(authority string) func(ctx context.Context, network, address string) (net.Conn, error) {
				if authority != test.wantAuthority {
					errChan <- fmt.Errorf("wrong custom authority passed to resolver. target: %s got authority: %s want authority: %s", test.authority, authority, test.wantAuthority)
				} else {
					errChan <- nil
				}
				return func(ctx context.Context, network, address string) (net.Conn, error) {
					return nil, errors.New("no need to dial")
				}
			}
			defer func() { dnsinternal.AddressDialer = origAddressDialer }()

			b := resolver.Get("dns")
			if b == nil {
				t.Fatalf("Resolver for dns:/// scheme not registered")
			}

			tcc := &testutils.ResolverClientConn{Logger: t}
			endpoint := "foo.bar.com"
			target := resolver.Target{URL: *testutils.MustParseURL(fmt.Sprintf("dns://%s/%s", test.authority, endpoint))}
			r, err := b.Build(target, tcc, resolver.BuildOptions{})
			if (err != nil) != test.wantBuildErr {
				t.Fatalf("DNS resolver build for target %+v returned error %v: wantErr: %v\n", target, err, test.wantBuildErr)
			}
			if err != nil {
				return
			}
			defer r.Close()

			if err := <-errChan; err != nil {
				t.Fatal(err)
			}
		})
	}
}

// TestRateLimitedResolve exercises the rate limit enforced on re-resolution
// requests. It sets the re-resolution rate to a small value and repeatedly
// calls ResolveNow() and ensures only the expected number of resolution
// requests are made.
func (s) TestRateLimitedResolve(t *testing.T) {
	const target = "foo.bar.com"
	_, timeChan := overrideTimeAfterFuncWithChannel(t)
	tr := &testNetResolver{
		lookupHostCh:    testutils.NewChannel(),
		hostLookupTable: map[string][]string{target: {"1.2.3.4", "5.6.7.8"}},
	}
	overrideNetResolver(t, tr)

	r, stateCh, _ := buildResolverWithTestClientConn(t, target)

	// Wait for the first resolution request to be done. This happens as part
	// of the first iteration of the for loop in watcher().
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	if _, err := tr.lookupHostCh.Receive(ctx); err != nil {
		t.Fatalf("Timed out waiting for lookup() call.")
	}

	// Call Resolve Now 100 times, shouldn't continue onto next iteration of
	// watcher, thus shouldn't lookup again.
	for i := 0; i <= 100; i++ {
		r.ResolveNow(resolver.ResolveNowOptions{})
	}

	continueCtx, continueCancel := context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer continueCancel()
	if _, err := tr.lookupHostCh.Receive(continueCtx); err == nil {
		t.Fatalf("Should not have looked up again as DNS Min Res Rate timer has not gone off.")
	}

	// Make the DNSMinResRate timer fire immediately, by sending the current
	// time on it. This will unblock the resolver which is currently blocked on
	// the DNS Min Res Rate timer going off, which will allow it to continue to
	// the next iteration of the watcher loop.
	select {
	case timeChan <- time.Now():
	case <-ctx.Done():
		t.Fatal("Timed out waiting for the DNS resolver to block on DNS Min Res Rate to elapse")
	}

	// Now that DNS Min Res Rate timer has gone off, it should lookup again.
	if _, err := tr.lookupHostCh.Receive(ctx); err != nil {
		t.Fatalf("Timed out waiting for lookup() call.")
	}

	// Resolve Now 1000 more times, shouldn't lookup again as DNS Min Res Rate
	// timer has not gone off.
	for i := 0; i < 1000; i++ {
		r.ResolveNow(resolver.ResolveNowOptions{})
	}
	continueCtx, continueCancel = context.WithTimeout(context.Background(), defaultTestShortTimeout)
	defer continueCancel()
	if _, err := tr.lookupHostCh.Receive(continueCtx); err == nil {
		t.Fatalf("Should not have looked up again as DNS Min Res Rate timer has not gone off.")
	}

	// Make the DNSMinResRate timer fire immediately again.
	select {
	case timeChan <- time.Now():
	case <-ctx.Done():
		t.Fatal("Timed out waiting for the DNS resolver to block on DNS Min Res Rate to elapse")
	}

	// Now that DNS Min Res Rate timer has gone off, it should lookup again.
	if _, err := tr.lookupHostCh.Receive(ctx); err != nil {
		t.Fatalf("Timed out waiting for lookup() call.")
	}

	wantAddrs := []resolver.Address{{Addr: "1.2.3.4" + colonDefaultPort}, {Addr: "5.6.7.8" + colonDefaultPort}}
	var state resolver.State
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for a state update from the resolver")
	case state = <-stateCh:
	}
	if !cmp.Equal(state.Addresses, wantAddrs, cmpopts.EquateEmpty()) {
		t.Fatalf("Got addresses: %+v, want: %+v", state.Addresses, wantAddrs)
	}
}

// Test verifies that when the DNS resolver gets an error from the underlying
// net.Resolver, it reports the error to the channel and backs off and retries.
func (s) TestReportError(t *testing.T) {
	durChan, timeChan := overrideTimeAfterFuncWithChannel(t)
	overrideNetResolver(t, &testNetResolver{})

	const target = "notfoundaddress"
	_, _, errorCh := buildResolverWithTestClientConn(t, target)

	// Should receive first error.
	ctx, ctxCancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer ctxCancel()
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for an error from the resolver")
	case err := <-errorCh:
		if !strings.Contains(err.Error(), "hostLookup error") {
			t.Fatalf(`ReportError(err=%v) called; want err contains "hostLookupError"`, err)
		}
	}

	// Expect the DNS resolver to backoff and attempt to re-resolve. Every time,
	// the DNS resolver will receive the same error from the net.Resolver and is
	// expected to push it to the channel.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	const retries = 10
	var prevDur time.Duration
	for i := 0; i < retries; i++ {
		select {
		case <-ctx.Done():
			t.Fatalf("(Iteration: %d): Timeout when waiting for DNS resolver to backoff", i)
		case dur := <-durChan:
			if dur <= prevDur {
				t.Fatalf("(Iteration: %d): Unexpected decrease in amount of time to backoff", i)
			}
		}

		// Unblock the DNS resolver's backoff by pushing the current time.
		timeChan <- time.Now()

		select {
		case <-ctx.Done():
			t.Fatal("Timeout when waiting for an error from the resolver")
		case err := <-errorCh:
			if !strings.Contains(err.Error(), "hostLookup error") {
				t.Fatalf(`ReportError(err=%v) called; want err contains "hostLookupError"`, err)
			}
		}
	}
}

// Override the default dns.ResolvingTimeout with a test duration.
func overrideResolveTimeoutDuration(t *testing.T, dur time.Duration) {
	t.Helper()

	origDur := dns.ResolvingTimeout
	dnspublic.SetResolvingTimeout(dur)

	t.Cleanup(func() { dnspublic.SetResolvingTimeout(origDur) })
}

// Test verifies that the DNS resolver gets timeout error when net.Resolver
// takes too long to resolve a target.
func (s) TestResolveTimeout(t *testing.T) {
	// Set DNS resolving timeout duration to 7ms
	timeoutDur := 7 * time.Millisecond
	overrideResolveTimeoutDuration(t, timeoutDur)

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// We are trying to resolve hostname which takes infinity time to resolve.
	const target = "infinity"

	// Define a testNetResolver with lookupHostCh, an unbuffered channel,
	// so we can block the resolver until reaching timeout.
	tr := &testNetResolver{
		lookupHostCh:    testutils.NewChannelWithSize(0),
		hostLookupTable: map[string][]string{target: {"1.2.3.4"}},
	}
	overrideNetResolver(t, tr)

	_, _, errCh := buildResolverWithTestClientConn(t, target)
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for the DNS resolver to timeout")
	case err := <-errCh:
		if err == nil || !strings.Contains(err.Error(), "context deadline exceeded") {
			t.Fatalf(`Expected to see Timeout error; got: %v`, err)
		}
	}
}

// Test verifies that changing [MinResolutionInterval] variable correctly effects
// the resolution behaviour
func (s) TestMinResolutionInterval(t *testing.T) {
	const target = "foo.bar.com"

	overrideResolutionInterval(t, 1*time.Millisecond)
	tr := &testNetResolver{
		hostLookupTable: map[string][]string{
			"foo.bar.com": {"1.2.3.4", "5.6.7.8"},
		},
		txtLookupTable: map[string][]string{
			"_grpc_config.foo.bar.com": txtRecordServiceConfig(txtRecordGood),
		},
	}
	overrideNetResolver(t, tr)

	r, stateCh, _ := buildResolverWithTestClientConn(t, target)

	wantAddrs := []resolver.Address{{Addr: "1.2.3.4" + colonDefaultPort}, {Addr: "5.6.7.8" + colonDefaultPort}}
	wantSC := scJSON

	for i := 0; i < 5; i++ {
		// set context timeout slightly higher than the min resolution interval to make sure resolutions
		// happen successfully
		ctx, cancel := context.WithTimeout(context.Background(), 100*time.Millisecond)
		defer cancel()

		verifyUpdateFromResolver(ctx, t, stateCh, wantAddrs, nil, wantSC)
		r.ResolveNow(resolver.ResolveNowOptions{})
	}
}

// TestMinResolutionInterval_NoExtraDelay verifies that there is no extra delay
// between two resolution requests apart from [MinResolutionInterval].
func (s) TestMinResolutionInterval_NoExtraDelay(t *testing.T) {
	tr := &testNetResolver{
		hostLookupTable: map[string][]string{
			"foo.bar.com": {"1.2.3.4", "5.6.7.8"},
		},
		txtLookupTable: map[string][]string{
			"_grpc_config.foo.bar.com": txtRecordServiceConfig(txtRecordGood),
		},
	}
	overrideNetResolver(t, tr)
	// Override time.Now() to return a zero value for time. This will allow us
	// to verify that the call to time.Until is made with the exact
	// [MinResolutionInterval] that we expect.
	overrideTimeNowFunc(t, time.Time{})
	// Override time.Until() to read the time passed to it
	// and return immediately without any delay
	timeCh := overrideTimeUntilFuncWithChannel(t)

	r, stateCh, errorCh := buildResolverWithTestClientConn(t, "foo.bar.com")

	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	// Ensure that the first resolution happens.
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for DNS resolver")
	case err := <-errorCh:
		t.Fatalf("Unexpected error from resolver, %v", err)
	case <-stateCh:
	}

	// Request re-resolution and verify that the resolver waits for
	// [MinResolutionInterval].
	r.ResolveNow(resolver.ResolveNowOptions{})
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for DNS resolver")
	case gotTime := <-timeCh:
		wantTime := time.Time{}.Add(dns.MinResolutionInterval)
		if !gotTime.Equal(wantTime) {
			t.Fatalf("DNS resolver waits for %v time before re-resolution, want %v", gotTime, wantTime)
		}
	}

	// Ensure that the re-resolution request actually happens.
	select {
	case <-ctx.Done():
		t.Fatal("Timeout when waiting for an error from the resolver")
	case err := <-errorCh:
		t.Fatalf("Unexpected error from resolver, %v", err)
	case <-stateCh:
	}
}
