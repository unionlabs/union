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

package manual_test

import (
	"errors"
	"testing"

	"google.golang.org/grpc"
	"google.golang.org/grpc/credentials/insecure"
	"google.golang.org/grpc/resolver"
	"google.golang.org/grpc/resolver/manual"
)

func TestResolver(t *testing.T) {
	r := manual.NewBuilderWithScheme("whatever")
	r.InitialState(resolver.State{
		Addresses: []resolver.Address{
			{Addr: "address"},
		},
	})

	t.Run("update_state_panics", func(t *testing.T) {
		defer func() {
			want := "cannot update state as grpc.Dial with resolver has not been called"
			if r := recover(); r != want {
				t.Errorf("expected panic %q, got %q", want, r)
			}
		}()
		r.UpdateState(resolver.State{Addresses: []resolver.Address{
			{Addr: "address"},
			{Addr: "anotheraddress"},
		}})
	})
	t.Run("report_error_panics", func(t *testing.T) {
		defer func() {
			want := "cannot report error as grpc.Dial with resolver has not been called"
			if r := recover(); r != want {
				t.Errorf("expected panic %q, got %q", want, r)
			}
		}()
		r.ReportError(errors.New("example"))
	})

	t.Run("happy_path", func(t *testing.T) {
		_, err := grpc.Dial("whatever://localhost",
			grpc.WithTransportCredentials(insecure.NewCredentials()),
			grpc.WithResolvers(r))
		if err != nil {
			t.Errorf("dial setup error: %v", err)
		}
		r.UpdateState(resolver.State{Addresses: []resolver.Address{
			{Addr: "ok"},
		}})
		r.ReportError(errors.New("example"))
	})
}
