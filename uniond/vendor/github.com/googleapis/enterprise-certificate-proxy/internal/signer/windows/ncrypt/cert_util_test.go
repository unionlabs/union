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

//go:build windows
// +build windows

package ncrypt

import (
	"testing"
)

func TestCredProviderNotSupported(t *testing.T) {
	_, err := Cred("issuer", "store", "unsupported_provider")
	if err == nil {
		t.Errorf("Expected error, but got nil.")
	}
	want := "provider must be local_machine or current_user"
	if err.Error() != want {
		t.Errorf("Expected error is %q, got: %q", want, err.Error())
	}
}
