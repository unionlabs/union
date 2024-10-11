// Copyright 2023 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package snap

import (
	"testing"

	"github.com/ethereum/go-ethereum/eth/protocols/snap"
)

func FuzzARange(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		doFuzz(data, &snap.GetAccountRangePacket{}, snap.GetAccountRangeMsg)
	})
}

func FuzzSRange(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		doFuzz(data, &snap.GetStorageRangesPacket{}, snap.GetStorageRangesMsg)
	})
}

func FuzzByteCodes(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		doFuzz(data, &snap.GetByteCodesPacket{}, snap.GetByteCodesMsg)
	})
}

func FuzzTrieNodes(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		doFuzz(data, &snap.GetTrieNodesPacket{}, snap.GetTrieNodesMsg)
	})
}
