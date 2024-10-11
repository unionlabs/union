// Copyright 2020 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

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
	dummyResp := &certResponse{
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
	cache.set(googleSACertsURL, dummyResp, make(http.Header))
	clock.Sleep(time.Nanosecond) // it expires when current time is > expiration, not >=
	cert, ok = cache.get(googleSACertsURL)
	if ok || cert != nil {
		t.Fatal("cache for SA certs should be expired")
	}

	// Add an item that expires in 1 seconds
	h := make(http.Header)
	h.Set("age", "0")
	h.Set("cache-control", "public, max-age=1, must-revalidate, no-transform")
	cache.set(googleSACertsURL, dummyResp, h)
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
