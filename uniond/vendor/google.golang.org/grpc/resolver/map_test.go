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

package resolver

import (
	"fmt"
	"sort"
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/attributes"
)

// Note: each address is different from addr1 by one value.  addr7 matches
// addr1, since the only difference is BalancerAttributes, which are not
// compared.
var (
	addr1 = Address{Addr: "a1", Attributes: attributes.New("a1", 3), ServerName: "s1"}
	addr2 = Address{Addr: "a2", Attributes: attributes.New("a1", 3), ServerName: "s1"}
	addr3 = Address{Addr: "a1", Attributes: attributes.New("a2", 3), ServerName: "s1"}
	addr4 = Address{Addr: "a1", Attributes: attributes.New("a1", 2), ServerName: "s1"}
	addr5 = Address{Addr: "a1", Attributes: attributes.New("a1", "3"), ServerName: "s1"}
	addr6 = Address{Addr: "a1", Attributes: attributes.New("a1", 3), ServerName: "s2"}
	addr7 = Address{Addr: "a1", Attributes: attributes.New("a1", 3), ServerName: "s1", BalancerAttributes: attributes.New("xx", 3)}

	endpoint1   = Endpoint{Addresses: []Address{{Addr: "addr1"}}}
	endpoint2   = Endpoint{Addresses: []Address{{Addr: "addr2"}}}
	endpoint3   = Endpoint{Addresses: []Address{{Addr: "addr3"}}}
	endpoint4   = Endpoint{Addresses: []Address{{Addr: "addr4"}}}
	endpoint5   = Endpoint{Addresses: []Address{{Addr: "addr5"}}}
	endpoint6   = Endpoint{Addresses: []Address{{Addr: "addr6"}}}
	endpoint7   = Endpoint{Addresses: []Address{{Addr: "addr7"}}}
	endpoint12  = Endpoint{Addresses: []Address{{Addr: "addr1"}, {Addr: "addr2"}}}
	endpoint21  = Endpoint{Addresses: []Address{{Addr: "addr2"}, {Addr: "addr1"}}}
	endpoint123 = Endpoint{Addresses: []Address{{Addr: "addr1"}, {Addr: "addr2"}, {Addr: "addr3"}}}
)

func (s) TestAddressMap_Length(t *testing.T) {
	addrMap := NewAddressMap()
	if got := addrMap.Len(); got != 0 {
		t.Fatalf("addrMap.Len() = %v; want 0", got)
	}
	for i := 0; i < 10; i++ {
		addrMap.Set(addr1, nil)
		if got, want := addrMap.Len(), 1; got != want {
			t.Fatalf("addrMap.Len() = %v; want %v", got, want)
		}
		addrMap.Set(addr7, nil) // aliases addr1
	}
	for i := 0; i < 10; i++ {
		addrMap.Set(addr2, nil)
		if got, want := addrMap.Len(), 2; got != want {
			t.Fatalf("addrMap.Len() = %v; want %v", got, want)
		}
	}
}

func (s) TestAddressMap_Get(t *testing.T) {
	addrMap := NewAddressMap()
	addrMap.Set(addr1, 1)

	if got, ok := addrMap.Get(addr2); ok || got != nil {
		t.Fatalf("addrMap.Get(addr1) = %v, %v; want nil, false", got, ok)
	}

	addrMap.Set(addr2, 2)
	addrMap.Set(addr3, 3)
	addrMap.Set(addr4, 4)
	addrMap.Set(addr5, 5)
	addrMap.Set(addr6, 6)
	addrMap.Set(addr7, 7) // aliases addr1
	if got, ok := addrMap.Get(addr1); !ok || got.(int) != 7 {
		t.Fatalf("addrMap.Get(addr1) = %v, %v; want %v, true", got, ok, 7)
	}
	if got, ok := addrMap.Get(addr2); !ok || got.(int) != 2 {
		t.Fatalf("addrMap.Get(addr2) = %v, %v; want %v, true", got, ok, 2)
	}
	if got, ok := addrMap.Get(addr3); !ok || got.(int) != 3 {
		t.Fatalf("addrMap.Get(addr3) = %v, %v; want %v, true", got, ok, 3)
	}
	if got, ok := addrMap.Get(addr4); !ok || got.(int) != 4 {
		t.Fatalf("addrMap.Get(addr4) = %v, %v; want %v, true", got, ok, 4)
	}
	if got, ok := addrMap.Get(addr5); !ok || got.(int) != 5 {
		t.Fatalf("addrMap.Get(addr5) = %v, %v; want %v, true", got, ok, 5)
	}
	if got, ok := addrMap.Get(addr6); !ok || got.(int) != 6 {
		t.Fatalf("addrMap.Get(addr6) = %v, %v; want %v, true", got, ok, 6)
	}
	if got, ok := addrMap.Get(addr7); !ok || got.(int) != 7 {
		t.Fatalf("addrMap.Get(addr7) = %v, %v; want %v, true", got, ok, 7)
	}
}

func (s) TestAddressMap_Delete(t *testing.T) {
	addrMap := NewAddressMap()
	addrMap.Set(addr1, 1)
	addrMap.Set(addr2, 2)
	if got, want := addrMap.Len(), 2; got != want {
		t.Fatalf("addrMap.Len() = %v; want %v", got, want)
	}
	addrMap.Delete(addr3)
	addrMap.Delete(addr4)
	addrMap.Delete(addr5)
	addrMap.Delete(addr6)
	addrMap.Delete(addr7) // aliases addr1
	if got, ok := addrMap.Get(addr1); ok || got != nil {
		t.Fatalf("addrMap.Get(addr1) = %v, %v; want nil, false", got, ok)
	}
	if got, ok := addrMap.Get(addr7); ok || got != nil {
		t.Fatalf("addrMap.Get(addr7) = %v, %v; want nil, false", got, ok)
	}
	if got, ok := addrMap.Get(addr2); !ok || got.(int) != 2 {
		t.Fatalf("addrMap.Get(addr2) = %v, %v; want %v, true", got, ok, 2)
	}
}

func (s) TestAddressMap_Keys(t *testing.T) {
	addrMap := NewAddressMap()
	addrMap.Set(addr1, 1)
	addrMap.Set(addr2, 2)
	addrMap.Set(addr3, 3)
	addrMap.Set(addr4, 4)
	addrMap.Set(addr5, 5)
	addrMap.Set(addr6, 6)
	addrMap.Set(addr7, 7) // aliases addr1

	want := []Address{addr1, addr2, addr3, addr4, addr5, addr6}
	got := addrMap.Keys()
	if d := cmp.Diff(want, got, cmp.Transformer("sort", func(in []Address) []Address {
		out := append([]Address(nil), in...)
		sort.Slice(out, func(i, j int) bool { return fmt.Sprint(out[i]) < fmt.Sprint(out[j]) })
		return out
	})); d != "" {
		t.Fatalf("addrMap.Keys returned unexpected elements (-want, +got):\n%v", d)
	}
}

func (s) TestAddressMap_Values(t *testing.T) {
	addrMap := NewAddressMap()
	addrMap.Set(addr1, 1)
	addrMap.Set(addr2, 2)
	addrMap.Set(addr3, 3)
	addrMap.Set(addr4, 4)
	addrMap.Set(addr5, 5)
	addrMap.Set(addr6, 6)
	addrMap.Set(addr7, 7) // aliases addr1

	want := []int{2, 3, 4, 5, 6, 7}
	var got []int
	for _, v := range addrMap.Values() {
		got = append(got, v.(int))
	}
	sort.Ints(got)
	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("addrMap.Values returned unexpected elements (-want, +got):\n%v", diff)
	}
}

func (s) TestEndpointMap_Length(t *testing.T) {
	em := NewEndpointMap()
	// Should be empty at creation time.
	if got := em.Len(); got != 0 {
		t.Fatalf("em.Len() = %v; want 0", got)
	}
	// Add two endpoints with the same unordered set of addresses. This should
	// amount to one endpoint. It should also not take into account attributes.
	em.Set(endpoint12, struct{}{})
	em.Set(endpoint21, struct{}{})

	if got := em.Len(); got != 1 {
		t.Fatalf("em.Len() = %v; want 1", got)
	}

	// Add another unique endpoint. This should cause the length to be 2.
	em.Set(endpoint123, struct{}{})
	if got := em.Len(); got != 2 {
		t.Fatalf("em.Len() = %v; want 2", got)
	}
}

func (s) TestEndpointMap_Get(t *testing.T) {
	em := NewEndpointMap()
	em.Set(endpoint1, 1)
	// The second endpoint endpoint21 should override.
	em.Set(endpoint12, 1)
	em.Set(endpoint21, 2)
	em.Set(endpoint3, 3)
	em.Set(endpoint4, 4)
	em.Set(endpoint5, 5)
	em.Set(endpoint6, 6)
	em.Set(endpoint7, 7)

	if got, ok := em.Get(endpoint1); !ok || got.(int) != 1 {
		t.Fatalf("em.Get(endpoint1) = %v, %v; want %v, true", got, ok, 1)
	}
	if got, ok := em.Get(endpoint12); !ok || got.(int) != 2 {
		t.Fatalf("em.Get(endpoint12) = %v, %v; want %v, true", got, ok, 2)
	}
	if got, ok := em.Get(endpoint21); !ok || got.(int) != 2 {
		t.Fatalf("em.Get(endpoint21) = %v, %v; want %v, true", got, ok, 2)
	}
	if got, ok := em.Get(endpoint3); !ok || got.(int) != 3 {
		t.Fatalf("em.Get(endpoint1) = %v, %v; want %v, true", got, ok, 3)
	}
	if got, ok := em.Get(endpoint4); !ok || got.(int) != 4 {
		t.Fatalf("em.Get(endpoint1) = %v, %v; want %v, true", got, ok, 4)
	}
	if got, ok := em.Get(endpoint5); !ok || got.(int) != 5 {
		t.Fatalf("em.Get(endpoint1) = %v, %v; want %v, true", got, ok, 5)
	}
	if got, ok := em.Get(endpoint6); !ok || got.(int) != 6 {
		t.Fatalf("em.Get(endpoint1) = %v, %v; want %v, true", got, ok, 6)
	}
	if got, ok := em.Get(endpoint7); !ok || got.(int) != 7 {
		t.Fatalf("em.Get(endpoint1) = %v, %v; want %v, true", got, ok, 7)
	}
	if _, ok := em.Get(endpoint123); ok {
		t.Fatalf("em.Get(endpoint123) = _, %v; want _, false", ok)
	}
}

func (s) TestEndpointMap_Delete(t *testing.T) {
	em := NewEndpointMap()
	// Initial state of system: [1, 2, 3, 12]
	em.Set(endpoint1, struct{}{})
	em.Set(endpoint2, struct{}{})
	em.Set(endpoint3, struct{}{})
	em.Set(endpoint12, struct{}{})
	// Delete: [2, 21]
	em.Delete(endpoint2)
	em.Delete(endpoint21)

	// [1, 3] should be present:
	if _, ok := em.Get(endpoint1); !ok {
		t.Fatalf("em.Get(endpoint1) = %v, want true", ok)
	}
	if _, ok := em.Get(endpoint3); !ok {
		t.Fatalf("em.Get(endpoint3) = %v, want true", ok)
	}
	// [2, 12] should not be present:
	if _, ok := em.Get(endpoint2); ok {
		t.Fatalf("em.Get(endpoint2) = %v, want false", ok)
	}
	if _, ok := em.Get(endpoint12); ok {
		t.Fatalf("em.Get(endpoint12) = %v, want false", ok)
	}
	if _, ok := em.Get(endpoint21); ok {
		t.Fatalf("em.Get(endpoint21) = %v, want false", ok)
	}
}

func (s) TestEndpointMap_Values(t *testing.T) {
	em := NewEndpointMap()
	em.Set(endpoint1, 1)
	// The second endpoint endpoint21 should override.
	em.Set(endpoint12, 1)
	em.Set(endpoint21, 2)
	em.Set(endpoint3, 3)
	em.Set(endpoint4, 4)
	em.Set(endpoint5, 5)
	em.Set(endpoint6, 6)
	em.Set(endpoint7, 7)
	want := []int{1, 2, 3, 4, 5, 6, 7}
	var got []int
	for _, v := range em.Values() {
		got = append(got, v.(int))
	}
	sort.Ints(got)
	if diff := cmp.Diff(want, got); diff != "" {
		t.Fatalf("em.Values() returned unexpected elements (-want, +got):\n%v", diff)
	}
}
