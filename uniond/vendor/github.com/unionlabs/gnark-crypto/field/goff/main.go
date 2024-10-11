// Copyright 2020 ConsenSys Software Inc.
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

// Package goff (go finite field) is a library that generates fast field arithmetic code for a given modulus.
//
// Generated code is optimized for x86 (amd64) targets, and most methods do not allocate memory on the heap.
//
// Example usage:
//
//	goff -m 0xffffffff00000001 -o ./goldilocks/ -p goldilocks -e Element
//
// # Warning
//
// The generated code has not been audited for all moduli (only bn254 and bls12-381) and is provided as-is. In particular, there is no security guarantees such as constant time implementation or side-channel attack resistance.
package main

import "github.com/consensys/gnark-crypto/field/goff/cmd"

func main() {
	cmd.Execute()
}
