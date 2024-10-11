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

// Package testgcs is a light GCS client used for testings to avoid pulling in
// dependencies.
package testgcs

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"

	"cloud.google.com/go/auth"
	"cloud.google.com/go/auth/httptransport"
	"cloud.google.com/go/auth/internal"
)

// Client is a lightweight GCS client for testing.
type Client struct {
	client *http.Client
}

// NewClient creates a [Client] using the provided
// [cloud.google.com/go/auth.TokenProvider] for authentication.
func NewClient(tp auth.TokenProvider) *Client {
	client := internal.CloneDefaultClient()
	httptransport.AddAuthorizationMiddleware(client, auth.NewCredentials(&auth.CredentialsOptions{
		TokenProvider: tp,
	}))
	return &Client{
		client: client,
	}
}

// CreateBucket creates the specified bucket.
func (c *Client) CreateBucket(ctx context.Context, projectID, bucket string) error {
	var bucketRequest struct {
		Name string `json:"name,omitempty"`
	}
	bucketRequest.Name = bucket
	b, err := json.Marshal(bucketRequest)
	if err != nil {
		return err
	}

	req, err := http.NewRequest("POST", fmt.Sprintf("https://storage.googleapis.com/storage/v1/b?project=%s", projectID), bytes.NewReader(b))
	if err != nil {
		return err
	}
	req.Header.Add("Content-Type", "application/json")
	resp, err := c.client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode > 299 {
		errBody, err := io.ReadAll(resp.Body)
		if err != nil {
			return err
		}
		return fmt.Errorf("%s", errBody)
	}
	return nil
}

// DeleteBucket deletes the specified bucket.
func (c *Client) DeleteBucket(ctx context.Context, bucket string) error {
	req, err := http.NewRequest("DELETE", fmt.Sprintf("https://storage.googleapis.com/storage/v1/b/%s", bucket), nil)
	if err != nil {
		return err
	}
	resp, err := c.client.Do(req)
	if err != nil {
		return err
	}
	defer resp.Body.Close()
	if resp.StatusCode < 200 || resp.StatusCode > 299 {
		errBody, err := io.ReadAll(resp.Body)
		if err != nil {
			return err
		}
		return fmt.Errorf("%s", errBody)
	}
	return nil
}

// DownloadObject returns an [http.Response] who's body can be consumed to
// read the contents of an object.
func (c *Client) DownloadObject(ctx context.Context, bucket, object string) (*http.Response, error) {
	req, err := http.NewRequest("GET", fmt.Sprintf("https://storage.googleapis.com/storage/v1/b/%s/o/%s", bucket, object), nil)
	if err != nil {
		return nil, err
	}
	req.Header.Add("alt", "media")
	resp, err := c.client.Do(req)
	if err != nil {
		return nil, err
	}
	if resp.StatusCode < 200 || resp.StatusCode > 299 {
		errBody, err := io.ReadAll(resp.Body)
		if err != nil {
			return nil, err
		}
		return nil, fmt.Errorf("%s", errBody)
	}
	return resp, nil
}
