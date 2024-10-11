// Copyright 2020 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build integration
// +build integration

package impersonate

import (
	"context"
	"fmt"
	"math/rand"
	"os"
	"testing"
	"time"

	"google.golang.org/api/option"
	"google.golang.org/api/storage/v1"
)

var (
	// envReaderCredentialFile points to a service account that is a "Service
	// Account Token Creator" on envReaderSA.
	envBaseSACredentialFile = "API_GO_CLIENT_IMPERSONATE_BASE"
	// envUserCredentialFile points to a user credential that is a "Service
	// Account Token Creator" on envReaderSA.
	envUserCredentialFile = "API_GO_CLIENT_IMPERSONATE_USER"
	// envReaderCredentialFile points to a service account that is a "Storage
	// Object Reader" and is a "Service Account Token Creator" on envWriterSA.
	envReaderCredentialFile = "API_GO_CLIENT_IMPERSONATE_READER"
	// envReaderSA is the name of the reader service account.
	envReaderSA = "API_GO_CLIENT_IMPERSONATE_READER_SA"
	// envWriterSA is the name of the writer service account. This service
	// account has been granted roles/serviceusage.serviceUsageConsumer.
	envWriterSA = "API_GO_CLIENT_IMPERSONATE_WRITER_SA"
	// envProjectID is a project that hosts a GCS bucket.
	envProjectID = "GOOGLE_CLOUD_PROJECT"
)

func init() {
	rand.Seed(time.Now().UnixNano())
}

func TestImpersonatedCredentials(t *testing.T) {
	ctx := context.Background()
	projID := os.Getenv(envProjectID)
	writerSA := os.Getenv(envWriterSA)
	tests := []struct {
		name           string
		baseSALocation string
		delgates       []string
	}{
		{
			name:           "SA -> SA",
			baseSALocation: os.Getenv(envReaderCredentialFile),
			delgates:       []string{},
		},
		{
			name:           "SA -> Delegate -> SA",
			baseSALocation: os.Getenv(envBaseSACredentialFile),
			delgates:       []string{os.Getenv(envReaderSA)},
		},
		{
			name:           "User Credential -> Delegate -> SA",
			baseSALocation: os.Getenv(envUserCredentialFile),
			delgates:       []string{os.Getenv(envReaderSA)},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			svc, err := storage.NewService(ctx,
				option.WithCredentialsFile(tt.baseSALocation),
				option.ImpersonateCredentials(writerSA, tt.delgates...),
			)
			if err != nil {
				t.Fatalf("failed to create client: %v", err)
			}
			bucketName := fmt.Sprintf("%s-%d", projID, rand.Int63())
			if _, err := svc.Buckets.Insert(projID, &storage.Bucket{
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
