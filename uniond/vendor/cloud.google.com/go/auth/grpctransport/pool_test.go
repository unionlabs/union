// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package grpctransport

import (
	"context"
	"errors"
	"net"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
)

func TestPool_RoundRobin(t *testing.T) {
	conn1 := &grpc.ClientConn{}
	conn2 := &grpc.ClientConn{}

	pool := &roundRobinConnPool{
		conns: []*grpc.ClientConn{
			conn1, conn2,
		},
	}

	if got := pool.Connection(); got != conn2 {
		t.Errorf("pool.Conn() #1 = %v, want conn2 (%v)", got, conn2)
	}
	if got := pool.Connection(); got != conn1 {
		t.Errorf("pool.Conn() #2 = %v, want conn1 (%v)", got, conn1)
	}
	if got := pool.Connection(); got != conn2 {
		t.Errorf("pool.Conn() #3 = %v, want conn2 (%v)", got, conn2)
	}
	if got := pool.Len(); got != 2 {
		t.Errorf("pool.Len() = %v, want %v", got, 2)
	}
}

func TestPool_SingleConn(t *testing.T) {
	conn1 := &grpc.ClientConn{}
	pool := &singleConnPool{conn1}

	if got := pool.Connection(); got != conn1 {
		t.Errorf("pool.Conn() #1 = %v, want conn2 (%v)", got, conn1)
	}
	if got := pool.Connection(); got != conn1 {
		t.Errorf("pool.Conn() #2 = %v, want conn1 (%v)", got, conn1)
	}
	if got := pool.Len(); got != 1 {
		t.Errorf("pool.Len() = %v, want %v", got, 1)
	}
}

func TestClose(t *testing.T) {
	_, l := mockServer(t)

	pool := &roundRobinConnPool{}
	for i := 0; i < 4; i++ {
		conn, err := grpc.Dial(l.Addr().String(), grpc.WithTransportCredentials(insecure.NewCredentials()))
		if err != nil {
			t.Fatal(err)
		}
		pool.conns = append(pool.conns, conn)
	}

	if err := pool.Close(); err != nil {
		t.Fatalf("pool.Close: %v", err)
	}
}

func TestWithEndpointAndPoolSize(t *testing.T) {
	_, l := mockServer(t)
	ctx := context.Background()
	connPool, err := Dial(ctx, false, &Options{
		Endpoint: l.Addr().String(),
		PoolSize: 4,
	})
	if err != nil {
		t.Fatal(err)
	}

	if err := connPool.Close(); err != nil {
		t.Fatalf("pool.Close: %v", err)
	}
}

func TestMultiError(t *testing.T) {
	tests := []struct {
		name string
		errs multiError
		want string
	}{
		{
			name: "0 errors",
			want: "(0 errors)",
		},
		{
			name: "1 errors",
			errs: []error{errors.New("the full error message")},
			want: "the full error message",
		},
		{
			name: "2 errors",
			errs: []error{errors.New("foo"), errors.New("bar")},
			want: "foo (and 1 other error)",
		},
		{
			name: "3 errors",
			errs: []error{errors.New("foo"), errors.New("bar"), errors.New("baz")},
			want: "foo (and 2 other errors)",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := tt.errs.Error()
			if got != tt.want {
				t.Fatalf("got %v, want %v", got, tt.want)
			}
		})
	}
}

func mockServer(t *testing.T) (*grpc.Server, net.Listener) {
	t.Helper()

	l, err := net.Listen("tcp", "localhost:0")
	if err != nil {
		t.Fatal(err)
	}

	s := grpc.NewServer()
	go s.Serve(l)

	return s, l
}
