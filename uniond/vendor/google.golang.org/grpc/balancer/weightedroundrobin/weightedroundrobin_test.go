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

package weightedroundrobin

import (
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/grpc/attributes"
	"google.golang.org/grpc/resolver"
)

func TestAddrInfoToAndFromAttributes(t *testing.T) {
	tests := []struct {
		desc            string
		inputAddrInfo   AddrInfo
		inputAttributes *attributes.Attributes
		wantAddrInfo    AddrInfo
	}{
		{
			desc:            "empty attributes",
			inputAddrInfo:   AddrInfo{Weight: 100},
			inputAttributes: nil,
			wantAddrInfo:    AddrInfo{Weight: 100},
		},
		{
			desc:            "non-empty attributes",
			inputAddrInfo:   AddrInfo{Weight: 100},
			inputAttributes: attributes.New("foo", "bar"),
			wantAddrInfo:    AddrInfo{Weight: 100},
		},
		{
			desc:            "addrInfo not present in empty attributes",
			inputAddrInfo:   AddrInfo{},
			inputAttributes: nil,
			wantAddrInfo:    AddrInfo{},
		},
		{
			desc:            "addrInfo not present in non-empty attributes",
			inputAddrInfo:   AddrInfo{},
			inputAttributes: attributes.New("foo", "bar"),
			wantAddrInfo:    AddrInfo{},
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			addr := resolver.Address{Attributes: test.inputAttributes}
			addr = SetAddrInfo(addr, test.inputAddrInfo)
			gotAddrInfo := GetAddrInfo(addr)
			if !cmp.Equal(gotAddrInfo, test.wantAddrInfo) {
				t.Errorf("gotAddrInfo: %v, wantAddrInfo: %v", gotAddrInfo, test.wantAddrInfo)
			}

		})
	}
}

func TestGetAddInfoEmpty(t *testing.T) {
	addr := resolver.Address{}
	gotAddrInfo := GetAddrInfo(addr)
	wantAddrInfo := AddrInfo{}
	if !cmp.Equal(gotAddrInfo, wantAddrInfo) {
		t.Errorf("gotAddrInfo: %v, wantAddrInfo: %v", gotAddrInfo, wantAddrInfo)
	}
}
