//go:build !amd64
// +build !amd64

/*
 * Copyright 2020 Dgraph Labs, Inc. and Contributors
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
 */

package simd

// Search uses the Clever search to find the correct key.
func Search(xs []uint64, k uint64) int16 {
	if len(xs) < 8 || (len(xs)%8 != 0) {
		return Naive(xs, k)
	}
	var twos, pk [4]uint64
	pk[0] = k
	pk[1] = k
	pk[2] = k
	pk[3] = k
	for i := 0; i < len(xs); i += 8 {
		twos[0] = xs[i]
		twos[1] = xs[i+2]
		twos[2] = xs[i+4]
		twos[3] = xs[i+6]
		if twos[0] >= pk[0] {
			return int16(i / 2)
		}
		if twos[1] >= pk[1] {
			return int16((i + 2) / 2)
		}
		if twos[2] >= pk[2] {
			return int16((i + 4) / 2)
		}
		if twos[3] >= pk[3] {
			return int16((i + 6) / 2)
		}

	}
	return int16(len(xs) / 2)
}
