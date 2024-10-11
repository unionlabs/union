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

package grpc

import (
	"context"
	"fmt"
	"net"
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/attributes"
	"google.golang.org/grpc/balancer"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/internal/balancer/stub"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
)

type wrapResolverBuilder struct {
	resolver.Builder
	scheme string
}

func (w *wrapResolverBuilder) Scheme() string {
	return w.scheme
}

func init() {
	resolver.Register(&wrapResolverBuilder{Builder: resolver.Get("passthrough"), scheme: "casetest"})
	resolver.Register(&wrapResolverBuilder{Builder: resolver.Get("dns"), scheme: "caseTest"})
}

func (s) TestResolverCaseSensitivity(t *testing.T) {
	// This should find the "casetest" resolver instead of the "caseTest"
	// resolver, even though the latter was registered later.  "casetest" is
	// "passthrough" and "caseTest" is "dns".  With "passthrough" the dialer
	// should see the target's address directly, but "dns" would be converted
	// into a loopback IP (v4 or v6) address.
	target := "caseTest:///localhost:1234"
	addrCh := make(chan string, 1)
	customDialer := func(_ context.Context, addr string) (net.Conn, error) {
		select {
		case addrCh <- addr:
		default:
		}
		return nil, fmt.Errorf("not dialing with custom dialer")
	}

	cc, err := Dial(target, WithContextDialer(customDialer), WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Unexpected Dial(%q) error: %v", target, err)
	}
	cc.Connect()
	if got, want := <-addrCh, "localhost:1234"; got != want {
		cc.Close()
		t.Fatalf("Dialer got address %q; wanted %q", got, want)
	}
	cc.Close()

	// Clear addrCh for future use.
	select {
	case <-addrCh:
	default:
	}

	res := &wrapResolverBuilder{Builder: resolver.Get("dns"), scheme: "caseTest2"}
	// This should not find the injected resolver due to the case not matching.
	// This results in "passthrough" being used with the address as the whole
	// target.
	target = "caseTest2:///localhost:1234"
	cc, err = Dial(target, WithContextDialer(customDialer), WithResolvers(res), WithTransportCredentials(insecure.NewCredentials()))
	if err != nil {
		t.Fatalf("Unexpected Dial(%q) error: %v", target, err)
	}
	cc.Connect()
	if got, want := <-addrCh, target; got != want {
		cc.Close()
		t.Fatalf("Dialer got address %q; wanted %q", got, want)
	}
	cc.Close()
}

// TestResolverAddressesToEndpoints ensures one Endpoint is created for each
// entry in resolver.State.Addresses automatically.
func (s) TestResolverAddressesToEndpoints(t *testing.T) {
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	const scheme = "testresolveraddressestoendpoints"
	r := manual.NewBuilderWithScheme(scheme)

	stateCh := make(chan balancer.ClientConnState, 1)
	bf := stub.BalancerFuncs{
		UpdateClientConnState: func(_ *stub.BalancerData, ccs balancer.ClientConnState) error {
			stateCh <- ccs
			return nil
		},
	}
	balancerName := "stub-balancer-" + scheme
	stub.Register(balancerName, bf)

	a1 := attributes.New("x", "y")
	a2 := attributes.New("a", "b")
	r.InitialState(resolver.State{Addresses: []resolver.Address{{Addr: "addr1", BalancerAttributes: a1}, {Addr: "addr2", BalancerAttributes: a2}}})

	cc, err := Dial(r.Scheme()+":///",
		WithTransportCredentials(insecure.NewCredentials()),
		WithResolvers(r),
		WithDefaultServiceConfig(fmt.Sprintf(`{"loadBalancingConfig": [{"%s":{}}]}`, balancerName)))
	if err != nil {
		t.Fatalf("Unexpected error dialing: %v", err)
	}
	defer cc.Close()

	select {
	case got := <-stateCh:
		want := []resolver.Endpoint{
			{Addresses: []resolver.Address{{Addr: "addr1"}}, Attributes: a1},
			{Addresses: []resolver.Address{{Addr: "addr2"}}, Attributes: a2},
		}
		if diff := cmp.Diff(got.ResolverState.Endpoints, want); diff != "" {
			t.Errorf("Did not receive expected endpoints.  Diff (-got +want):\n%v", diff)
		}
	case <-ctx.Done():
		t.Fatalf("timed out waiting for endpoints")
	}
}

// Test ensures that there is no panic if the attributes within
// resolver.State.Addresses contains a typed-nil value.
func (s) TestResolverAddressesWithTypedNilAttribute(t *testing.T) {
	r := manual.NewBuilderWithScheme(t.Name())
	resolver.Register(r)

	addrAttr := attributes.New("typed_nil", (*stringerVal)(nil))
	r.InitialState(resolver.State{Addresses: []resolver.Address{{Addr: "addr1", Attributes: addrAttr}}})

	cc, err := Dial(r.Scheme()+":///", WithTransportCredentials(insecure.NewCredentials()), WithResolvers(r))
	if err != nil {
		t.Fatalf("Unexpected error dialing: %v", err)
	}
	defer cc.Close()
}

type stringerVal struct{ s string }

func (s stringerVal) String() string { return s.s }
