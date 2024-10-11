/*
 *
 * Copyright 2024 gRPC authors.
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

package peer

import (
	"context"
	"fmt"
	"testing"
	"time"

	"google.golang.org/grpc/credentials"
)

const defaultTestTimeout = 10 * time.Second

// A struct that implements AuthInfo interface and implements CommonAuthInfo() method.
type testAuthInfo struct {
	credentials.CommonAuthInfo
}

func (ta testAuthInfo) AuthType() string {
	return fmt.Sprintf("testAuthInfo-%d", ta.SecurityLevel)
}

type addr struct {
	ipAddress string
}

func (addr) Network() string { return "" }

func (a *addr) String() string { return a.ipAddress }

func TestPeerStringer(t *testing.T) {
	testCases := []struct {
		name string
		peer *Peer
		want string
	}{
		{
			name: "+Addr-LocalAddr+ValidAuth",
			peer: &Peer{Addr: &addr{"example.com:1234"}, AuthInfo: testAuthInfo{credentials.CommonAuthInfo{SecurityLevel: credentials.PrivacyAndIntegrity}}},
			want: "Peer{Addr: 'example.com:1234', LocalAddr: <nil>, AuthInfo: 'testAuthInfo-3'}",
		},
		{
			name: "+Addr+LocalAddr+ValidAuth",
			peer: &Peer{Addr: &addr{"example.com:1234"}, LocalAddr: &addr{"example.com:1234"}, AuthInfo: testAuthInfo{credentials.CommonAuthInfo{SecurityLevel: credentials.PrivacyAndIntegrity}}},
			want: "Peer{Addr: 'example.com:1234', LocalAddr: 'example.com:1234', AuthInfo: 'testAuthInfo-3'}",
		},
		{
			name: "+Addr-LocalAddr+emptyAuth",
			peer: &Peer{Addr: &addr{"1.2.3.4:1234"}, AuthInfo: testAuthInfo{credentials.CommonAuthInfo{}}},
			want: "Peer{Addr: '1.2.3.4:1234', LocalAddr: <nil>, AuthInfo: 'testAuthInfo-0'}",
		},
		{
			name: "-Addr-LocalAddr+emptyAuth",
			peer: &Peer{AuthInfo: testAuthInfo{}},
			want: "Peer{Addr: <nil>, LocalAddr: <nil>, AuthInfo: 'testAuthInfo-0'}",
		},
		{
			name: "zeroedPeer",
			peer: &Peer{},
			want: "Peer{Addr: <nil>, LocalAddr: <nil>, AuthInfo: <nil>}",
		},
		{
			name: "nilPeer",
			peer: nil,
			want: "Peer<nil>",
		},
	}
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()
	for _, tc := range testCases {
		t.Run(tc.name, func(t *testing.T) {
			ctx = NewContext(ctx, tc.peer)

			p, ok := FromContext(ctx)
			if !ok {
				t.Fatalf("Unable to get peer from context")
			}
			if p.String() != tc.want {
				t.Fatalf("Error using peer String(): expected %q, got %q", tc.want, p.String())
			}
		})
	}
}
