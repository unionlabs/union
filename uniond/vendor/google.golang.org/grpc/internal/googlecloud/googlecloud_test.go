/*
 *
 * Copyright 2021 gRPC authors.
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

package googlecloud

import (
	"testing"
)

func TestIsRunningOnGCE(t *testing.T) {
	for _, tc := range []struct {
		description      string
		testOS           string
		testManufacturer string
		out              bool
	}{
		// Linux tests.
		{"linux: not a GCP platform", "linux", "not GCP", false},
		{"Linux: GCP platform (Google)", "linux", "Google", true},
		{"Linux: GCP platform (Google Compute Engine)", "linux", "Google Compute Engine", true},
		{"Linux: GCP platform (Google Compute Engine) with extra spaces", "linux", "  Google Compute Engine        ", true},
		// Windows tests.
		{"windows: not a GCP platform", "windows", "not GCP", false},
		{"windows: GCP platform (Google)", "windows", "Google", true},
		{"windows: GCP platform (Google) with extra spaces", "windows", "  Google     ", true},
	} {
		if got, want := isRunningOnGCE([]byte(tc.testManufacturer), tc.testOS), tc.out; got != want {
			t.Errorf("%v: isRunningOnGCE()=%v, want %v", tc.description, got, want)
		}
	}
}
