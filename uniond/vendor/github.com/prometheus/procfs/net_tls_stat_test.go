// Copyright 2023 Prometheus Team
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package procfs

import (
	"testing"
)

func TestTLSStat(t *testing.T) {
	tlsStats, err := getProcFixtures(t).NewTLSStat()
	if err != nil {
		t.Fatal(err)
	}

	for _, test := range []struct {
		name string
		want int
		got  int
	}{
		{name: "TLSCurrTxSw", want: 5, got: tlsStats.TLSCurrTxSw},
		{name: "TLSCurrRxSw", want: 5, got: tlsStats.TLSCurrRxSw},
		{name: "TLSCurrTxDevice", want: 0, got: tlsStats.TLSCurrTxDevice},
		{name: "TLSCurrRxDevice", want: 0, got: tlsStats.TLSCurrRxDevice},
		{name: "TLSTxSw", want: 8711, got: tlsStats.TLSTxSw},
		{name: "TLSTxSw", want: 8711, got: tlsStats.TLSRxSw},
		{name: "TLSTxDevice", want: 0, got: tlsStats.TLSTxDevice},
		{name: "TLSRxDevice", want: 0, got: tlsStats.TLSRxDevice},
		{name: "TLSDecryptError", want: 13, got: tlsStats.TLSDecryptError},
		{name: "TLSRxDeviceResync", want: 0, got: tlsStats.TLSRxDeviceResync},
		{name: "TLSDecryptRetry", want: 0, got: tlsStats.TLSDecryptRetry},
		{name: "TLSRxNoPadViolation", want: 0, got: tlsStats.TLSRxNoPadViolation},
	} {
		if test.want != test.got {
			t.Errorf("Want %s %d, have %d", test.name, test.want, test.got)
		}
	}
}
