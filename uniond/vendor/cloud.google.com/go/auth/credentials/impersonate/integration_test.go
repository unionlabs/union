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

package impersonate_test

import (
	"context"
	"flag"
	"fmt"
	"log"
	"math/rand"
	"os"
	"testing"
	"time"

	"cloud.google.com/go/auth"
	"cloud.google.com/go/auth/credentials"
	"cloud.google.com/go/auth/credentials/idtoken"
	"cloud.google.com/go/auth/credentials/impersonate"
	"cloud.google.com/go/auth/internal/credsfile"
	"cloud.google.com/go/auth/internal/testutil"
	"cloud.google.com/go/auth/internal/testutil/testgcs"
)

const (
	envProjectID   = "GCLOUD_TESTS_GOLANG_PROJECT_ID"
	envReaderCreds = "GCLOUD_TESTS_IMPERSONATE_READER_KEY"
	envReaderEmail = "GCLOUD_TESTS_IMPERSONATE_READER_EMAIL"
	envWriterEmail = "GCLOUD_TESTS_IMPERSONATE_WRITER_EMAIL"
)

var (
	baseKeyFile   string
	readerKeyFile string
	readerEmail   string
	writerEmail   string
	projectID     string
	random        *rand.Rand
)

func TestMain(m *testing.M) {
	flag.Parse()
	random = rand.New(rand.NewSource(time.Now().UnixNano()))
	baseKeyFile = os.Getenv(credsfile.GoogleAppCredsEnvVar)
	projectID = os.Getenv(envProjectID)
	readerKeyFile = os.Getenv(envReaderCreds)
	readerEmail = os.Getenv(envReaderEmail)
	writerEmail = os.Getenv(envWriterEmail)

	if !testing.Short() && (baseKeyFile == "" ||
		readerKeyFile == "" ||
		readerEmail == "" ||
		writerEmail == "" ||
		projectID == "") {
		log.Println("required environment variable not set, skipping")
		os.Exit(0)
	}

	os.Exit(m.Run())
}

func TestNewCredentialsIntegration(t *testing.T) {
	testutil.IntegrationTestCheck(t)
	tests := []struct {
		name            string
		baseKeyFile     string
		delegates       []string
		useDefaultCreds bool
	}{
		{
			name:        "SA -> SA",
			baseKeyFile: readerKeyFile,
		},
		{
			name:            "SA -> SA (Default)",
			baseKeyFile:     readerKeyFile,
			useDefaultCreds: true,
		},
		{
			name:        "SA -> Delegate -> SA",
			baseKeyFile: baseKeyFile,
			delegates:   []string{readerEmail},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ctx := context.Background()
			var baseCreds *auth.Credentials
			if !tt.useDefaultCreds {
				var err error
				baseCreds, err = credentials.DetectDefault(&credentials.DetectOptions{
					Scopes:          []string{"https://www.googleapis.com/auth/cloud-platform"},
					CredentialsFile: tt.baseKeyFile,
				})
				if err != nil {
					t.Fatalf("credentials.DetectDefault() = %v", err)
				}
			}

			opts := &impersonate.CredentialsOptions{
				TargetPrincipal: writerEmail,
				Scopes:          []string{"https://www.googleapis.com/auth/devstorage.full_control"},
				Delegates:       tt.delegates,
			}
			if !tt.useDefaultCreds {
				opts.Credentials = baseCreds
			}
			creds, err := impersonate.NewCredentials(opts)
			if err != nil {
				t.Fatalf("failed to create ts: %v", err)
			}
			client := testgcs.NewClient(creds)
			bucketName := fmt.Sprintf("%s-impersonate-test-%d", projectID, random.Int63())
			if err := client.CreateBucket(ctx, projectID, bucketName); err != nil {
				t.Fatalf("error creating bucket: %v", err)
			}
			if err := client.DeleteBucket(ctx, bucketName); err != nil {
				t.Fatalf("unable to cleanup bucket %q: %v", bucketName, err)
			}
		})
	}
}

func TestNewIDTokenCredentialsIntegration(t *testing.T) {
	testutil.IntegrationTestCheck(t)

	ctx := context.Background()
	tests := []struct {
		name            string
		baseKeyFile     string
		delegates       []string
		useDefaultCreds bool
	}{
		{
			name:        "SA -> SA",
			baseKeyFile: readerKeyFile,
		},
		{
			name:            "SA -> SA (Default)",
			useDefaultCreds: true,
		},
		{
			name:        "SA -> Delegate -> SA",
			baseKeyFile: baseKeyFile,
			delegates:   []string{readerEmail},
		},
	}

	for _, tt := range tests {
		name := tt.name
		t.Run(name, func(t *testing.T) {
			var baseCreds *auth.Credentials
			if !tt.useDefaultCreds {
				var err error
				baseCreds, err = credentials.DetectDefault(&credentials.DetectOptions{
					Scopes:          []string{"https://www.googleapis.com/auth/cloud-platform"},
					CredentialsFile: tt.baseKeyFile,
				})
				if err != nil {
					t.Fatalf("credentials.DetectDefault() = %v", err)
				}
			}
			aud := "http://example.com/"
			opts := &impersonate.IDTokenOptions{
				TargetPrincipal: writerEmail,
				Audience:        aud,
				Delegates:       tt.delegates,
				IncludeEmail:    true,
			}
			if !tt.useDefaultCreds {
				opts.Credentials = baseCreds
			}
			creds, err := impersonate.NewIDTokenCredentials(opts)
			if err != nil {
				t.Fatalf("failed to create ts: %v", err)
			}
			tok, err := creds.Token(ctx)
			if err != nil {
				t.Fatalf("unable to retrieve Token: %v", err)
			}
			validTok, err := idtoken.Validate(ctx, tok.Value, aud)
			if err != nil {
				t.Fatalf("token validation failed: %v", err)
			}
			if validTok.Audience != aud {
				t.Fatalf("got %q, want %q", validTok.Audience, aud)
			}
			if validTok.Claims["email"] != writerEmail {
				t.Fatalf("got %q, want %q", validTok.Claims["email"], writerEmail)
			}
		})
	}
}
