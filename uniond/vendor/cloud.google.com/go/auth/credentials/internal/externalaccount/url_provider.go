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

package externalaccount

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"

	"cloud.google.com/go/auth/internal"
	"cloud.google.com/go/auth/internal/credsfile"
)

const (
	fileTypeText             = "text"
	fileTypeJSON             = "json"
	urlProviderType          = "url"
	programmaticProviderType = "programmatic"
)

type urlSubjectProvider struct {
	URL     string
	Headers map[string]string
	Format  *credsfile.Format
	Client  *http.Client
}

func (sp *urlSubjectProvider) subjectToken(ctx context.Context) (string, error) {
	req, err := http.NewRequestWithContext(ctx, "GET", sp.URL, nil)
	if err != nil {
		return "", fmt.Errorf("credentials: HTTP request for URL-sourced credential failed: %w", err)
	}

	for key, val := range sp.Headers {
		req.Header.Add(key, val)
	}
	resp, err := sp.Client.Do(req)
	if err != nil {
		return "", fmt.Errorf("credentials: invalid response when retrieving subject token: %w", err)
	}
	defer resp.Body.Close()

	respBody, err := internal.ReadAll(resp.Body)
	if err != nil {
		return "", fmt.Errorf("credentials: invalid body in subject token URL query: %w", err)
	}
	if c := resp.StatusCode; c < http.StatusOK || c >= http.StatusMultipleChoices {
		return "", fmt.Errorf("credentials: status code %d: %s", c, respBody)
	}

	if sp.Format == nil {
		return string(respBody), nil
	}
	switch sp.Format.Type {
	case "json":
		jsonData := make(map[string]interface{})
		err = json.Unmarshal(respBody, &jsonData)
		if err != nil {
			return "", fmt.Errorf("credentials: failed to unmarshal subject token file: %w", err)
		}
		val, ok := jsonData[sp.Format.SubjectTokenFieldName]
		if !ok {
			return "", errors.New("credentials: provided subject_token_field_name not found in credentials")
		}
		token, ok := val.(string)
		if !ok {
			return "", errors.New("credentials: improperly formatted subject token")
		}
		return token, nil
	case fileTypeText:
		return string(respBody), nil
	default:
		return "", errors.New("credentials: invalid credential_source file format type: " + sp.Format.Type)
	}
}

func (sp *urlSubjectProvider) providerType() string {
	return urlProviderType
}
