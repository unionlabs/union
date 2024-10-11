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

package ringhash

import (
	"fmt"
	"math"
	"testing"

	xxhash "github.com/cespare/xxhash/v2"
	"google.golang.org/grpc/balancer/weightedroundrobin"
	"google.golang.org/grpc/resolver"
)

var testAddrs []resolver.Address
var testSubConnMap *resolver.AddressMap

func init() {
	testAddrs = []resolver.Address{
		testAddr("a", 3),
		testAddr("b", 3),
		testAddr("c", 4),
	}
	testSubConnMap = resolver.NewAddressMap()
	testSubConnMap.Set(testAddrs[0], &subConn{addr: "a", weight: 3})
	testSubConnMap.Set(testAddrs[1], &subConn{addr: "b", weight: 3})
	testSubConnMap.Set(testAddrs[2], &subConn{addr: "c", weight: 4})
}

func testAddr(addr string, weight uint32) resolver.Address {
	return weightedroundrobin.SetAddrInfo(resolver.Address{Addr: addr}, weightedroundrobin.AddrInfo{Weight: weight})
}

func (s) TestRingNew(t *testing.T) {
	var totalWeight float64 = 10
	for _, min := range []uint64{3, 4, 6, 8} {
		for _, max := range []uint64{20, 8} {
			t.Run(fmt.Sprintf("size-min-%v-max-%v", min, max), func(t *testing.T) {
				r := newRing(testSubConnMap, min, max, nil)
				totalCount := len(r.items)
				if totalCount < int(min) || totalCount > int(max) {
					t.Fatalf("unexpected size %v, want min %v, max %v", totalCount, min, max)
				}
				for _, a := range testAddrs {
					var count int
					for _, ii := range r.items {
						if ii.sc.addr == a.Addr {
							count++
						}
					}
					got := float64(count) / float64(totalCount)
					want := float64(getWeightAttribute(a)) / totalWeight
					if !equalApproximately(got, want) {
						t.Fatalf("unexpected item weight in ring: %v != %v", got, want)
					}
				}
			})
		}
	}
}

func equalApproximately(x, y float64) bool {
	delta := math.Abs(x - y)
	mean := math.Abs(x+y) / 2.0
	return delta/mean < 0.25
}

func (s) TestRingPick(t *testing.T) {
	r := newRing(testSubConnMap, 10, 20, nil)
	for _, h := range []uint64{xxhash.Sum64String("1"), xxhash.Sum64String("2"), xxhash.Sum64String("3"), xxhash.Sum64String("4")} {
		t.Run(fmt.Sprintf("picking-hash-%v", h), func(t *testing.T) {
			e := r.pick(h)
			var low uint64
			if e.idx > 0 {
				low = r.items[e.idx-1].hash
			}
			high := e.hash
			// h should be in [low, high).
			if h < low || h >= high {
				t.Fatalf("unexpected item picked, hash: %v, low: %v, high: %v", h, low, high)
			}
		})
	}
}

func (s) TestRingNext(t *testing.T) {
	r := newRing(testSubConnMap, 10, 20, nil)

	for _, e := range r.items {
		ne := r.next(e)
		if ne.idx != (e.idx+1)%len(r.items) {
			t.Fatalf("next(%+v) returned unexpected %+v", e, ne)
		}
	}
}
