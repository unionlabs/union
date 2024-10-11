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
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"
	"strings"
	"sync"
	"time"
)

type cachingClient struct {
	client *http.Client

	// clock optionally specifies a func to return the current time.
	// If nil, time.Now is used.
	clock func() time.Time

	mu    sync.Mutex
	certs map[string]*cachedResponse
}

func newCachingClient(client *http.Client) *cachingClient {
	return &cachingClient{
		client: client,
		certs:  make(map[string]*cachedResponse, 2),
	}
}

type cachedResponse struct {
	resp *certResponse
	exp  time.Time
}

func (c *cachingClient) getCert(ctx context.Context, url string) (*certResponse, error) {
	if response, ok := c.get(url); ok {
		return response, nil
	}
	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		return nil, err
	}
	req = req.WithContext(ctx)
	resp, err := c.client.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("idtoken: unable to retrieve cert, got status code %d", resp.StatusCode)
	}

	certResp := &certResponse{}
	if err := json.NewDecoder(resp.Body).Decode(certResp); err != nil {
		return nil, err

	}
	c.set(url, certResp, resp.Header)
	return certResp, nil
}

func (c *cachingClient) now() time.Time {
	if c.clock != nil {
		return c.clock()
	}
	return time.Now()
}

func (c *cachingClient) get(url string) (*certResponse, bool) {
	c.mu.Lock()
	defer c.mu.Unlock()
	cachedResp, ok := c.certs[url]
	if !ok {
		return nil, false
	}
	if c.now().After(cachedResp.exp) {
		return nil, false
	}
	return cachedResp.resp, true
}

func (c *cachingClient) set(url string, resp *certResponse, headers http.Header) {
	exp := c.calculateExpireTime(headers)
	c.mu.Lock()
	c.certs[url] = &cachedResponse{resp: resp, exp: exp}
	c.mu.Unlock()
}

// calculateExpireTime will determine the expire time for the cache based on
// HTTP headers. If there is any difficulty reading the headers the fallback is
// to set the cache to expire now.
func (c *cachingClient) calculateExpireTime(headers http.Header) time.Time {
	var maxAge int
	cc := strings.Split(headers.Get("cache-control"), ",")
	for _, v := range cc {
		if strings.Contains(v, "max-age") {
			ss := strings.Split(v, "=")
			if len(ss) < 2 {
				return c.now()
			}
			ma, err := strconv.Atoi(ss[1])
			if err != nil {
				return c.now()
			}
			maxAge = ma
		}
	}
	a := headers.Get("age")
	if a == "" {
		return c.now().Add(time.Duration(maxAge) * time.Second)
	}
	age, err := strconv.Atoi(a)
	if err != nil {
		return c.now()
	}
	return c.now().Add(time.Duration(maxAge-age) * time.Second)
}
