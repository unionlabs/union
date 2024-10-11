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
 *
 */

package hierarchy

import (
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/attributes"
	"google.golang.org/grpc/resolver"
)

func TestGet(t *testing.T) {
	tests := []struct {
		name string
		addr resolver.Address
		want []string
	}{
		{
			name: "not set",
			addr: resolver.Address{},
			want: nil,
		},
		{
			name: "set",
			addr: resolver.Address{
				BalancerAttributes: attributes.New(pathKey, pathValue{"a", "b"}),
			},
			want: []string{"a", "b"},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := Get(tt.addr); !cmp.Equal(got, tt.want) {
				t.Errorf("Get() = %v, want %v", got, tt.want)
			}
		})
	}
}

func TestSet(t *testing.T) {
	tests := []struct {
		name string
		addr resolver.Address
		path []string
	}{
		{
			name: "before is not set",
			addr: resolver.Address{},
			path: []string{"a", "b"},
		},
		{
			name: "before is set",
			addr: resolver.Address{
				BalancerAttributes: attributes.New(pathKey, pathValue{"before", "a", "b"}),
			},
			path: []string{"a", "b"},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			newAddr := Set(tt.addr, tt.path)
			newPath := Get(newAddr)
			if !cmp.Equal(newPath, tt.path) {
				t.Errorf("path after Set() = %v, want %v", newPath, tt.path)
			}
		})
	}
}

func TestGroup(t *testing.T) {
	tests := []struct {
		name  string
		addrs []resolver.Address
		want  map[string][]resolver.Address
	}{
		{
			name: "all with hierarchy",
			addrs: []resolver.Address{
				{Addr: "a0", BalancerAttributes: attributes.New(pathKey, pathValue{"a"})},
				{Addr: "a1", BalancerAttributes: attributes.New(pathKey, pathValue{"a"})},
				{Addr: "b0", BalancerAttributes: attributes.New(pathKey, pathValue{"b"})},
				{Addr: "b1", BalancerAttributes: attributes.New(pathKey, pathValue{"b"})},
			},
			want: map[string][]resolver.Address{
				"a": {
					{Addr: "a0", BalancerAttributes: attributes.New(pathKey, pathValue{})},
					{Addr: "a1", BalancerAttributes: attributes.New(pathKey, pathValue{})},
				},
				"b": {
					{Addr: "b0", BalancerAttributes: attributes.New(pathKey, pathValue{})},
					{Addr: "b1", BalancerAttributes: attributes.New(pathKey, pathValue{})},
				},
			},
		},
		{
			// Addresses without hierarchy are ignored.
			name: "without hierarchy",
			addrs: []resolver.Address{
				{Addr: "a0", BalancerAttributes: attributes.New(pathKey, pathValue{"a"})},
				{Addr: "a1", BalancerAttributes: attributes.New(pathKey, pathValue{"a"})},
				{Addr: "b0", BalancerAttributes: nil},
				{Addr: "b1", BalancerAttributes: nil},
			},
			want: map[string][]resolver.Address{
				"a": {
					{Addr: "a0", BalancerAttributes: attributes.New(pathKey, pathValue{})},
					{Addr: "a1", BalancerAttributes: attributes.New(pathKey, pathValue{})},
				},
			},
		},
		{
			// If hierarchy is set to a wrong type (which should never happen),
			// the address is ignored.
			name: "wrong type",
			addrs: []resolver.Address{
				{Addr: "a0", BalancerAttributes: attributes.New(pathKey, pathValue{"a"})},
				{Addr: "a1", BalancerAttributes: attributes.New(pathKey, pathValue{"a"})},
				{Addr: "b0", BalancerAttributes: attributes.New(pathKey, "b")},
				{Addr: "b1", BalancerAttributes: attributes.New(pathKey, 314)},
			},
			want: map[string][]resolver.Address{
				"a": {
					{Addr: "a0", BalancerAttributes: attributes.New(pathKey, pathValue{})},
					{Addr: "a1", BalancerAttributes: attributes.New(pathKey, pathValue{})},
				},
			},
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := Group(tt.addrs); !cmp.Equal(got, tt.want, cmp.AllowUnexported(attributes.Attributes{})) {
				t.Errorf("Group() = %v, want %v", got, tt.want)
				t.Errorf("diff: %v", cmp.Diff(got, tt.want, cmp.AllowUnexported(attributes.Attributes{})))
			}
		})
	}
}

func TestGroupE2E(t *testing.T) {
	hierarchy := map[string]map[string][]string{
		"p0": {
			"wt0": {"addr0", "addr1"},
			"wt1": {"addr2", "addr3"},
		},
		"p1": {
			"wt10": {"addr10", "addr11"},
			"wt11": {"addr12", "addr13"},
		},
	}

	var addrsWithHierarchy []resolver.Address
	for p, wts := range hierarchy {
		path1 := pathValue{p}
		for wt, addrs := range wts {
			path2 := append(pathValue(nil), path1...)
			path2 = append(path2, wt)
			for _, addr := range addrs {
				a := resolver.Address{
					Addr:               addr,
					BalancerAttributes: attributes.New(pathKey, path2),
				}
				addrsWithHierarchy = append(addrsWithHierarchy, a)
			}
		}
	}

	gotHierarchy := make(map[string]map[string][]string)
	for p1, wts := range Group(addrsWithHierarchy) {
		gotHierarchy[p1] = make(map[string][]string)
		for p2, addrs := range Group(wts) {
			for _, addr := range addrs {
				gotHierarchy[p1][p2] = append(gotHierarchy[p1][p2], addr.Addr)
			}
		}
	}

	if !cmp.Equal(gotHierarchy, hierarchy) {
		t.Errorf("diff: %v", cmp.Diff(gotHierarchy, hierarchy))
	}
}
