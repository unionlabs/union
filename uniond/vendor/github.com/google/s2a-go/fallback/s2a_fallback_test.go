/*
 *
 * Copyright 2023 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package fallback

import (
	"context"
	"fmt"
	"net"
	"reflect"
	"testing"
)

func TestDefaultFallbackClientHandshakeFunc(t *testing.T) {
	testDialContextFunc := func(context.Context, string, string) (net.Conn, error) {
		return nil, fmt.Errorf("testDialError")
	}
	for _, tc := range []struct {
		desc              string
		inputFallbackAddr string
		funcInitialized   bool
		expectedAddr      string
		expectNilErr      bool
	}{
		{
			"working case, fallback address has port suffix",
			"example.com:443",
			true,
			"example.com:443",
			true,
		},
		{
			"working case, fallback address has no port suffix",
			"example.com",
			true,
			"example.com:443",

			true,
		},
		{
			"working case, IP address, with port",
			"192.168.1.1:443",
			true,
			"192.168.1.1:443",

			true,
		},
		{
			"working case, IP address, no port",
			"192.168.1.1",
			true,
			"192.168.1.1:443",

			true,
		},
		{
			"working case, IPv6 address, with port",
			"[2001:db8::1]:443",
			true,
			"[2001:db8::1]:443",
			true,
		},
		{
			"working case, IPv6 address, no port",
			"2001:db8::1",
			true,
			"[2001:db8::1]:443",
			true,
		},
		{
			"test empty fallback address",
			"",
			false,
			"",
			false,
		},
	} {
		fbFunc, err := defaultFallbackClientHandshakeFuncInternal(tc.inputFallbackAddr, testDialContextFunc)
		if got, want := fbFunc != nil, tc.funcInitialized; got != want {
			t.Errorf("%v: fallback handshake func is initialized=[%v], want [%v]", tc.desc, got, want)
		}
		if got, want := err == nil, tc.expectNilErr; got != want {
			t.Errorf("%v: got error [%v], want nil error [%v]", tc.desc, err, want)
		}
		if err == nil {
			_, _, err := fbFunc(context.TODO(), "", nil, fmt.Errorf("testS2AError"))
			if err == nil {
				t.Errorf("%v: expecting an error from the test dial function, got nil instead", tc.desc)
			}
			expectedErr := fmt.Sprintf("dialing to fallback server %s failed: testDialError; S2A client handshake with  error: testS2AError", tc.expectedAddr)
			if got, want := err.Error(), expectedErr; got != want {
				t.Errorf("%v: fallback handshake got error [%v], want error [%v]", tc.desc, got, want)
			}
		}
	}
}
func TestDefaultFallbackDialerAndAddress(t *testing.T) {
	for _, tc := range []struct {
		desc              string
		inputFallbackAddr string
		dialerInitialized bool
		expectedAddr      string
		expectNilErr      bool
	}{
		{
			"working case, fallback address has port suffix",
			"example.com:443",
			true,
			"example.com:443",
			true,
		},
		{
			"working case, fallback address has no port suffix",
			"example.com",
			true,
			"example.com:443",
			true,
		},
		{
			"working case, IP address, with port",
			"192.168.1.1:443",
			true,
			"192.168.1.1:443",
			true,
		},
		{
			"working case, IP address, no port",
			"192.168.1.1",
			true,
			"192.168.1.1:443",
			true,
		},
		{
			"working case, IPv6 address, with port",
			"[2001:db8::1]:443",
			true,
			"[2001:db8::1]:443",
			true,
		},
		{
			"working case, IPv6 address, no port",
			"2001:db8::1",
			true,
			"[2001:db8::1]:443",
			true,
		},
		{
			"test empty fallback address",
			"",
			false,
			"",
			false,
		},
	} {
		fbDialer, fbAddr, err := DefaultFallbackDialerAndAddress(tc.inputFallbackAddr)
		if got, want := fbDialer != nil, tc.dialerInitialized; got != want {
			t.Errorf("%v: fallback dialer is initialized=[%v], want [%v]", tc.desc, got, want)
		}
		if got, want := fbAddr, tc.expectedAddr; got != want {
			t.Errorf("%v: returned fallback address=[%v], want [%v]", tc.desc, got, want)
		}
		if got, want := err == nil, tc.expectNilErr; got != want {
			t.Errorf("%v: got error [%v], want nil error [%v]", tc.desc, err, want)
		}
		if err == nil {
			if !reflect.DeepEqual(fbDialer.Config, &FallbackTLSConfigHTTP) {
				t.Errorf("%v: unexpected tls config from fallback dialer: [%v], expected: [%v]", tc.desc, fbDialer.Config, &FallbackTLSConfigHTTP)
			}

		}
	}
}
