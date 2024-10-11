// Copyright 2021 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package impersonate_test

import (
	"context"
	"flag"
	"fmt"
	"math/rand"
	"os"
	"testing"
	"time"

	admin "google.golang.org/api/admin/directory/v1"
	"google.golang.org/api/idtoken"
	"google.golang.org/api/impersonate"
	"google.golang.org/api/option"

	"google.golang.org/api/storage/v1"
)

var (
	baseKeyFile   string
	readerKeyFile string
	readerEmail   string
	writerEmail   string
	projectID     string
	domain        string
	domainAdmin   string
)

func TestMain(m *testing.M) {
	flag.Parse()
	rand.Seed(time.Now().UnixNano())
	baseKeyFile = os.Getenv("GOOGLE_APPLICATION_CREDENTIALS")
	projectID = os.Getenv("GOOGLE_CLOUD_PROJECT")
	readerKeyFile = os.Getenv("GCLOUD_TESTS_IMPERSONATE_READER_KEY")
	readerEmail = os.Getenv("GCLOUD_TESTS_IMPERSONATE_READER_EMAIL")
	writerEmail = os.Getenv("GCLOUD_TESTS_IMPERSONATE_WRITER_EMAIL")
	domain = os.Getenv("GCLOUD_TESTS_IMPERSONATE_DOMAIN")
	domainAdmin = os.Getenv("GCLOUD_TESTS_IMPERSONATE_DOMAIN_ADMIN")

	os.Exit(m.Run())
}

func validateEnvVars(t *testing.T) {
	t.Helper()
	if baseKeyFile == "" ||
		readerKeyFile == "" ||
		readerEmail == "" ||
		writerEmail == "" ||
		projectID == "" {
		t.Skip("required environment variable not set, skipping")
	}
}

func TestCredentialsTokenSourceIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test")
	}
	validateEnvVars(t)

	ctx := context.Background()
	tests := []struct {
		name        string
		baseKeyFile string
		delegates   []string
	}{
		{
			name:        "SA -> SA",
			baseKeyFile: readerKeyFile,
		},
		{
			name:        "SA -> Delegate -> SA",
			baseKeyFile: baseKeyFile,
			delegates:   []string{readerEmail},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ts, err := impersonate.CredentialsTokenSource(ctx,
				impersonate.CredentialsConfig{
					TargetPrincipal: writerEmail,
					Scopes:          []string{"https://www.googleapis.com/auth/devstorage.full_control"},
					Delegates:       tt.delegates,
				},
				option.WithCredentialsFile(tt.baseKeyFile),
			)
			if err != nil {
				t.Fatalf("failed to create ts: %v", err)
			}
			svc, err := storage.NewService(ctx, option.WithTokenSource(ts))
			if err != nil {
				t.Fatalf("failed to create client: %v", err)
			}
			bucketName := fmt.Sprintf("%s-%d", projectID, rand.Int63())
			if _, err := svc.Buckets.Insert(projectID, &storage.Bucket{
				Name: bucketName,
			}).Do(); err != nil {
				t.Fatalf("error creating bucket: %v", err)
			}
			if err := svc.Buckets.Delete(bucketName).Do(); err != nil {
				t.Fatalf("unable to cleanup bucket %q: %v", bucketName, err)
			}
		})
	}
}

func TestIDTokenSourceIntegration(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping integration test")
	}
	validateEnvVars(t)

	ctx := context.Background()
	tests := []struct {
		name        string
		baseKeyFile string
		delegates   []string
	}{
		{
			name:        "SA -> SA",
			baseKeyFile: readerKeyFile,
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
			aud := "http://example.com/"
			ts, err := impersonate.IDTokenSource(ctx,
				impersonate.IDTokenConfig{
					TargetPrincipal: writerEmail,
					Audience:        aud,
					Delegates:       tt.delegates,
					IncludeEmail:    true,
				},
				option.WithCredentialsFile(tt.baseKeyFile),
			)
			if err != nil {
				t.Fatalf("failed to create ts: %v", err)
			}
			tok, err := ts.Token()
			if err != nil {
				t.Fatalf("unable to retrieve Token: %v", err)
			}
			validTok, err := idtoken.Validate(context.Background(), tok.AccessToken, aud)
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

func TestTokenSourceIntegration_user(t *testing.T) {
	t.Skip("https://github.com/googleapis/google-api-go-client/issues/948")
	if testing.Short() {
		t.Skip("skipping integration test")
	}
	validateEnvVars(t)
	ctx := context.Background()
	tests := []struct {
		name        string
		baseKeyFile string
		delegates   []string
	}{
		{
			name:        "SA -> SA",
			baseKeyFile: readerKeyFile,
		},
		{
			name:        "SA -> Delegate -> SA",
			baseKeyFile: baseKeyFile,
			delegates:   []string{readerEmail},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			ts, err := impersonate.CredentialsTokenSource(ctx,
				impersonate.CredentialsConfig{
					TargetPrincipal: writerEmail,
					Delegates:       tt.delegates,
					Scopes:          []string{"https://www.googleapis.com/auth/admin.directory.user", "https://www.googleapis.com/auth/admin.directory.group"},
					Subject:         domainAdmin,
				},
				option.WithCredentialsFile(baseKeyFile),
			)
			if err != nil {
				t.Fatalf("failed to create ts: %v", err)
			}
			svc, err := admin.NewService(ctx, option.WithTokenSource(ts))
			if err != nil {
				t.Fatalf("failed to create client: %v", err)
			}
			if _, err := svc.Users.List().Domain(domain).Do(); err != nil {
				t.Fatalf("failed to list users: %v", err)
			}
		})
	}
}
