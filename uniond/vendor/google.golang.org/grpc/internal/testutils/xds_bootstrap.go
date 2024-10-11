/*
 *
 * Copyright 2024 gRPC authors.
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

package testutils

import (
	"os"
	"testing"

	"google.golang.org/grpc/internal/envconfig"
)

// CreateBootstrapFileForTesting creates a temporary file with the provided
// bootstrap contents, and updates the bootstrap environment variable to point
// to this file.
//
// Registers a cleanup function on the provided testing.T, that deletes the
// temporary file and resets the bootstrap environment variable.
func CreateBootstrapFileForTesting(t *testing.T, bootstrapContents []byte) {
	t.Helper()

	f, err := os.CreateTemp("", "test_xds_bootstrap_*")
	if err != nil {
		t.Fatalf("Failed to created bootstrap file: %v", err)
	}

	if err := os.WriteFile(f.Name(), bootstrapContents, 0644); err != nil {
		t.Fatalf("Failed to created bootstrap file: %v", err)
	}

	origBootstrapFileName := envconfig.XDSBootstrapFileName
	envconfig.XDSBootstrapFileName = f.Name()
	t.Cleanup(func() {
		os.Remove(f.Name())
		envconfig.XDSBootstrapFileName = origBootstrapFileName
	})
}
