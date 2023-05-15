/*
 * Copyright 2019 Dgraph Labs, Inc. and Contributors
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

package y

import (
	"hash/crc32"

	"github.com/dgraph-io/badger/v2/pb"

	"github.com/cespare/xxhash"
	"github.com/pkg/errors"
)

// ErrChecksumMismatch is returned at checksum mismatch.
var ErrChecksumMismatch = errors.New("checksum mismatch")

// CalculateChecksum calculates checksum for data using ct checksum type.
func CalculateChecksum(data []byte, ct pb.Checksum_Algorithm) uint64 {
	switch ct {
	case pb.Checksum_CRC32C:
		return uint64(crc32.Checksum(data, CastagnoliCrcTable))
	case pb.Checksum_XXHash64:
		return xxhash.Sum64(data)
	default:
		panic("checksum type not supported")
	}
}

// VerifyChecksum validates the checksum for the data against the given expected checksum.
func VerifyChecksum(data []byte, expected *pb.Checksum) error {
	actual := CalculateChecksum(data, expected.Algo)
	if actual != expected.Sum {
		return Wrapf(ErrChecksumMismatch, "actual: %d, expected: %d", actual, expected.Sum)
	}
	return nil
}
