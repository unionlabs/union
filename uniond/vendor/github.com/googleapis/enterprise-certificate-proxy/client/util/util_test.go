// Copyright 2022 Google LLC.
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

package util

import (
	"os"
	"testing"
)

func TestLoadSignerBinaryPath(t *testing.T) {
	path, err := LoadSignerBinaryPath("./test_data/certificate_config.json")
	if err != nil {
		t.Errorf("LoadSignerBinaryPath error: %q", err)
	}
	want := "C:/Program Files (x86)/Google/Endpoint Verification/signer.exe"
	if path != want {
		t.Errorf("Expected path is %q, got: %q", want, path)
	}
}

func TestLoadSignerBinaryPathHome(t *testing.T) {
	path, err := LoadSignerBinaryPath("./test_data/certificate_config_home_expansion.json")
	if err != nil {
		t.Errorf("LoadSignerBinaryPath error: %q", err)
	}
	want := guessHomeDir() + "/ecp/signer"
	if path != want {
		t.Errorf("Expected path is %q, got: %q", want, path)
	}
}

func TestLoadSignerBinaryPathTilde(t *testing.T) {
	path, err := LoadSignerBinaryPath("./test_data/certificate_config_tilde_expansion.json")
	if err != nil {
		t.Errorf("LoadSignerBinaryPath error: %q", err)
	}
	want := guessHomeDir() + "/ecp/signer"
	if path != want {
		t.Errorf("Expected path is %q, got: %q", want, path)
	}
}

func TestGetConfigFilePathFromEnv(t *testing.T) {
	want := "/testpath"
	os.Setenv("GOOGLE_API_CERTIFICATE_CONFIG", want)
	path := GetConfigFilePathFromEnv()
	if path != want {
		t.Errorf("Expected path is %q, got: %q", want, path)
	}
}
