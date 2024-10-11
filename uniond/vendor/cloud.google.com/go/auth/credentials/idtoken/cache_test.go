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

package idtoken

import (
	"net/http"
	"sync"
	"testing"
	"time"
)

type fakeClock struct {
	mu sync.Mutex
	t  time.Time
}

func (c *fakeClock) Now() time.Time {
	c.mu.Lock()
	defer c.mu.Unlock()
	return c.t
}

func (c *fakeClock) Sleep(d time.Duration) {
	c.mu.Lock()
	defer c.mu.Unlock()
	c.t = c.t.Add(d)
}

func TestCacheHit(t *testing.T) {
	clock := &fakeClock{t: time.Now()}
	fakeResp := &certResponse{
		Keys: []jwk{
			{
				Kid: "123",
			},
		},
	}
	cache := newCachingClient(nil)
	cache.clock = clock.Now

	// Cache should be empty
	cert, ok := cache.get(googleSACertsURL)
	if ok || cert != nil {
		t.Fatal("cache for SA certs should be empty")
	}

	// Add an item, but make it expire now
	cache.set(googleSACertsURL, fakeResp, make(http.Header))
	clock.Sleep(time.Nanosecond) // it expires when current time is > expiration, not >=
	cert, ok = cache.get(googleSACertsURL)
	if ok || cert != nil {
		t.Fatal("cache for SA certs should be expired")
	}

	// Add an item that expires in 1 seconds
	h := make(http.Header)
	h.Set("age", "0")
	h.Set("cache-control", "public, max-age=1, must-revalidate, no-transform")
	cache.set(googleSACertsURL, fakeResp, h)
	cert, ok = cache.get(googleSACertsURL)
	if !ok || cert == nil || cert.Keys[0].Kid != "123" {
		t.Fatal("cache for SA certs have a resp")
	}
	// Wait
	clock.Sleep(2 * time.Second)
	cert, ok = cache.get(googleSACertsURL)
	if ok || cert != nil {
		t.Fatal("cache for SA certs should be expired")
	}
}
