// Copyright 2022 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package node

import (
	"context"
	crand "crypto/rand"
	"fmt"
	"net/http"
	"os"
	"path"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common/hexutil"
	"github.com/ethereum/go-ethereum/rpc"
	"github.com/golang-jwt/jwt/v4"
)

type helloRPC string

func (ta helloRPC) HelloWorld() (string, error) {
	return string(ta), nil
}

type authTest struct {
	name            string
	endpoint        string
	prov            rpc.HTTPAuth
	expectDialFail  bool
	expectCall1Fail bool
	expectCall2Fail bool
}

func (at *authTest) Run(t *testing.T) {
	ctx := context.Background()
	cl, err := rpc.DialOptions(ctx, at.endpoint, rpc.WithHTTPAuth(at.prov))
	if at.expectDialFail {
		if err == nil {
			t.Fatal("expected initial dial to fail")
		} else {
			return
		}
	}
	if err != nil {
		t.Fatalf("failed to dial rpc endpoint: %v", err)
	}

	var x string
	err = cl.CallContext(ctx, &x, "engine_helloWorld")
	if at.expectCall1Fail {
		if err == nil {
			t.Fatal("expected call 1 to fail")
		} else {
			return
		}
	}
	if err != nil {
		t.Fatalf("failed to call rpc endpoint: %v", err)
	}
	if x != "hello engine" {
		t.Fatalf("method was silent but did not return expected value: %q", x)
	}

	err = cl.CallContext(ctx, &x, "eth_helloWorld")
	if at.expectCall2Fail {
		if err == nil {
			t.Fatal("expected call 2 to fail")
		} else {
			return
		}
	}
	if err != nil {
		t.Fatalf("failed to call rpc endpoint: %v", err)
	}
	if x != "hello eth" {
		t.Fatalf("method was silent but did not return expected value: %q", x)
	}
}

func TestAuthEndpoints(t *testing.T) {
	var secret [32]byte
	if _, err := crand.Read(secret[:]); err != nil {
		t.Fatalf("failed to create jwt secret: %v", err)
	}
	// Geth must read it from a file, and does not support in-memory JWT secrets, so we create a temporary file.
	jwtPath := path.Join(t.TempDir(), "jwt_secret")
	if err := os.WriteFile(jwtPath, []byte(hexutil.Encode(secret[:])), 0600); err != nil {
		t.Fatalf("failed to prepare jwt secret file: %v", err)
	}
	// We get ports assigned by the node automatically
	conf := &Config{
		HTTPHost:  "127.0.0.1",
		HTTPPort:  0,
		WSHost:    "127.0.0.1",
		WSPort:    0,
		AuthAddr:  "127.0.0.1",
		AuthPort:  0,
		JWTSecret: jwtPath,

		WSModules:   []string{"eth", "engine"},
		HTTPModules: []string{"eth", "engine"},
	}
	node, err := New(conf)
	if err != nil {
		t.Fatalf("could not create a new node: %v", err)
	}
	// register dummy apis so we can test the modules are available and reachable with authentication
	node.RegisterAPIs([]rpc.API{
		{
			Namespace:     "engine",
			Version:       "1.0",
			Service:       helloRPC("hello engine"),
			Public:        true,
			Authenticated: true,
		},
		{
			Namespace:     "eth",
			Version:       "1.0",
			Service:       helloRPC("hello eth"),
			Public:        true,
			Authenticated: true,
		},
	})
	if err := node.Start(); err != nil {
		t.Fatalf("failed to start test node: %v", err)
	}
	defer node.Close()

	// sanity check we are running different endpoints
	if a, b := node.WSEndpoint(), node.WSAuthEndpoint(); a == b {
		t.Fatalf("expected ws and auth-ws endpoints to be different, got: %q and %q", a, b)
	}
	if a, b := node.HTTPEndpoint(), node.HTTPAuthEndpoint(); a == b {
		t.Fatalf("expected http and auth-http endpoints to be different, got: %q and %q", a, b)
	}

	goodAuth := NewJWTAuth(secret)
	var otherSecret [32]byte
	if _, err := crand.Read(otherSecret[:]); err != nil {
		t.Fatalf("failed to create jwt secret: %v", err)
	}
	badAuth := NewJWTAuth(otherSecret)

	notTooLong := time.Second * 57
	tooLong := time.Second * 60
	requestDelay := time.Second

	testCases := []authTest{
		// Auth works
		{name: "ws good", endpoint: node.WSAuthEndpoint(), prov: goodAuth, expectCall1Fail: false},
		{name: "http good", endpoint: node.HTTPAuthEndpoint(), prov: goodAuth, expectCall1Fail: false},

		// Try a bad auth
		{name: "ws bad", endpoint: node.WSAuthEndpoint(), prov: badAuth, expectDialFail: true},      // ws auth is immediate
		{name: "http bad", endpoint: node.HTTPAuthEndpoint(), prov: badAuth, expectCall1Fail: true}, // http auth is on first call

		// A common mistake with JWT is to allow the "none" algorithm, which is a valid JWT but not secure.
		{name: "ws none", endpoint: node.WSAuthEndpoint(), prov: noneAuth(secret), expectDialFail: true},
		{name: "http none", endpoint: node.HTTPAuthEndpoint(), prov: noneAuth(secret), expectCall1Fail: true},

		// claims of 5 seconds or more, older or newer, are not allowed
		{name: "ws too old", endpoint: node.WSAuthEndpoint(), prov: offsetTimeAuth(secret, -tooLong), expectDialFail: true},
		{name: "http too old", endpoint: node.HTTPAuthEndpoint(), prov: offsetTimeAuth(secret, -tooLong), expectCall1Fail: true},
		// note: for it to be too long we need to add a delay, so that once we receive the request, the difference has not dipped below the "tooLong"
		{name: "ws too new", endpoint: node.WSAuthEndpoint(), prov: offsetTimeAuth(secret, tooLong+requestDelay), expectDialFail: true},
		{name: "http too new", endpoint: node.HTTPAuthEndpoint(), prov: offsetTimeAuth(secret, tooLong+requestDelay), expectCall1Fail: true},

		// Try offset the time, but stay just within bounds
		{name: "ws old", endpoint: node.WSAuthEndpoint(), prov: offsetTimeAuth(secret, -notTooLong)},
		{name: "http old", endpoint: node.HTTPAuthEndpoint(), prov: offsetTimeAuth(secret, -notTooLong)},
		{name: "ws new", endpoint: node.WSAuthEndpoint(), prov: offsetTimeAuth(secret, notTooLong)},
		{name: "http new", endpoint: node.HTTPAuthEndpoint(), prov: offsetTimeAuth(secret, notTooLong)},

		// ws only authenticates on initial dial, then continues communication
		{name: "ws single auth", endpoint: node.WSAuthEndpoint(), prov: changingAuth(goodAuth, badAuth)},
		{name: "http call fail auth", endpoint: node.HTTPAuthEndpoint(), prov: changingAuth(goodAuth, badAuth), expectCall2Fail: true},
		{name: "http call fail time", endpoint: node.HTTPAuthEndpoint(), prov: changingAuth(goodAuth, offsetTimeAuth(secret, tooLong+requestDelay)), expectCall2Fail: true},
	}

	for _, testCase := range testCases {
		t.Run(testCase.name, testCase.Run)
	}
}

func noneAuth(secret [32]byte) rpc.HTTPAuth {
	return func(header http.Header) error {
		token := jwt.NewWithClaims(jwt.SigningMethodNone, jwt.MapClaims{
			"iat": &jwt.NumericDate{Time: time.Now()},
		})
		s, err := token.SignedString(secret[:])
		if err != nil {
			return fmt.Errorf("failed to create JWT token: %w", err)
		}
		header.Set("Authorization", "Bearer "+s)
		return nil
	}
}

func changingAuth(provs ...rpc.HTTPAuth) rpc.HTTPAuth {
	i := 0
	return func(header http.Header) error {
		i += 1
		if i > len(provs) {
			i = len(provs)
		}
		return provs[i-1](header)
	}
}

func offsetTimeAuth(secret [32]byte, offset time.Duration) rpc.HTTPAuth {
	return func(header http.Header) error {
		token := jwt.NewWithClaims(jwt.SigningMethodHS256, jwt.MapClaims{
			"iat": &jwt.NumericDate{Time: time.Now().Add(offset)},
		})
		s, err := token.SignedString(secret[:])
		if err != nil {
			return fmt.Errorf("failed to create JWT token: %w", err)
		}
		header.Set("Authorization", "Bearer "+s)
		return nil
	}
}
