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

package priority

import (
	"context"
	"testing"
	"time"

	"google.golang.org/grpc/internal/testutils"
	"google.golang.org/grpc/resolver"
)

func (s) TestIgnoreResolveNowClientConn(t *testing.T) {
	cc := testutils.NewBalancerClientConn(t)
	ignoreCC := newIgnoreResolveNowClientConn(cc, false)

	// Call ResolveNow() on the CC, it should be forwarded.
	ctx, cancel := context.WithTimeout(context.Background(), defaultTestTimeout)
	defer cancel()

	ignoreCC.ResolveNow(resolver.ResolveNowOptions{})
	select {
	case <-cc.ResolveNowCh:
	case <-ctx.Done():
		t.Fatalf("Timeout waiting for ResolveNow()")
	}

	// Update ignoreResolveNow to true, call ResolveNow() on the CC, they should
	// all be ignored.
	ignoreCC.updateIgnoreResolveNow(true)
	for i := 0; i < 5; i++ {
		ignoreCC.ResolveNow(resolver.ResolveNowOptions{})
	}
	select {
	case <-cc.ResolveNowCh:
		t.Fatalf("got unexpected ResolveNow() call")
	case <-time.After(defaultTestShortTimeout):
	}

	// Update ignoreResolveNow to false, new ResolveNow() calls should be
	// forwarded.
	ignoreCC.updateIgnoreResolveNow(false)
	ignoreCC.ResolveNow(resolver.ResolveNowOptions{})
	select {
	case <-cc.ResolveNowCh:
	case <-ctx.Done():
		t.Fatalf("timeout waiting for ResolveNow()")
	}
}
