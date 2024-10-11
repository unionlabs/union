// Copyright 2021 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Package detect is used find information from the environment.
package detect

import (
	"context"
	"errors"
	"fmt"
	"os"

	"golang.org/x/oauth2/google"
	"google.golang.org/api/option"
	"google.golang.org/api/transport"
)

const (
	projectIDSentinel = "*detect-project-id*"
	envProjectID      = "GOOGLE_CLOUD_PROJECT"
)

var (
	adcLookupFunc func(context.Context, ...option.ClientOption) (*google.Credentials, error) = transport.Creds
	envLookupFunc func(string) string                                                        = os.Getenv
)

// ProjectID tries to detect the project ID from the environment if the sentinel
// value, "*detect-project-id*", is sent. It looks in the following order:
//  1. GOOGLE_CLOUD_PROJECT envvar
//  2. ADC creds.ProjectID
//  3. A static value if the environment is emulated.
func ProjectID(ctx context.Context, projectID string, emulatorEnvVar string, opts ...option.ClientOption) (string, error) {
	if projectID != projectIDSentinel {
		return projectID, nil
	}
	// 1. Try a well known environment variable
	if id := envLookupFunc(envProjectID); id != "" {
		return id, nil
	}
	// 2. Try ADC
	creds, err := adcLookupFunc(ctx, opts...)
	if err != nil {
		return "", fmt.Errorf("fetching creds: %v", err)
	}
	// 3. If ADC does not work, and the environment is emulated, return a const value.
	if creds.ProjectID == "" && emulatorEnvVar != "" && envLookupFunc(emulatorEnvVar) != "" {
		return "emulated-project", nil
	}
	// 4. If 1-3 don't work, error out
	if creds.ProjectID == "" {
		return "", errors.New("unable to detect projectID, please refer to docs for DetectProjectID")
	}
	// Success from ADC
	return creds.ProjectID, nil
}
