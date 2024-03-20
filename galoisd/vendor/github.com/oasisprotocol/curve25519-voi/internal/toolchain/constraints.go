// Copyright (c) 2021 Oasis Labs Inc. All rights reserved.
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
// 1. Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//
// 2. Redistributions in binary form must reproduce the above copyright
// notice, this list of conditions and the following disclaimer in the
// documentation and/or other materials provided with the distribution.
//
// 3. Neither the name of the copyright holder nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS
// IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED
// TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A
// PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED
// TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

// Package toolchain enforces the minimum supported toolchain.
package toolchain

var (
	// This is enforced so that I can consolidate build constraints
	// instead of keeping track of exactly when each 64-bit target got
	// support for SSA doing the right thing for bits.Add64/bits.Mul64.
	//
	// If you absolutely must get this working on older Go versions,
	// the 64-bit codepath is safe (and performant) as follows:
	//
	//  * 1.12 - amd64 (all other targets INSECURE due to vartime fallback)
	//  * 1.13 - arm64, ppcle, ppc64
	//  * 1.14 - s390x
	//
	//  * riscv64 became fast during 1.19, not shipped yet
	//
	// Last updated: Go 1.19 (src/cmd/compile/internal/ssagen/ssa.go)
	_ = __SOFTWARE_REQUIRES_GO_VERSION_1_18__

	// gccgo doesn't support Go's assembly dialect, and I don't want to
	// have to special case that with more build constraints either.
	_ = __SOFTWARE_REQUIRES_GC_COMPILER__
)
