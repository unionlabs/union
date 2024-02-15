// Minio Cloud Storage, (C) 2021 Minio, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//

package sha256

import (
	"bytes"
	"io/ioutil"
	"runtime"

	"github.com/klauspost/cpuid/v2"
)

var (
	hasIntelSha = runtime.GOARCH == "amd64" && cpuid.CPU.Supports(cpuid.SHA, cpuid.SSSE3, cpuid.SSE4)
	hasAvx512   = cpuid.CPU.Supports(cpuid.AVX512F, cpuid.AVX512DQ, cpuid.AVX512BW, cpuid.AVX512VL)
)

func hasArmSha2() bool {
	if cpuid.CPU.Has(cpuid.SHA2) {
		return true
	}
	if runtime.GOARCH != "arm64" || runtime.GOOS != "linux" {
		return false
	}

	// Fall back to hacky cpuinfo parsing...
	const procCPUInfo = "/proc/cpuinfo"

	// Feature to check for.
	const sha256Feature = "sha2"

	cpuInfo, err := ioutil.ReadFile(procCPUInfo)
	if err != nil {
		return false
	}
	return bytes.Contains(cpuInfo, []byte(sha256Feature))
}
