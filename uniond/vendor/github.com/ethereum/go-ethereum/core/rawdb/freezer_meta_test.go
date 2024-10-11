// Copyright 2022 The go-ethereum Authors
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

package rawdb

import (
	"os"
	"testing"
)

func TestReadWriteFreezerTableMeta(t *testing.T) {
	f, err := os.CreateTemp(os.TempDir(), "*")
	if err != nil {
		t.Fatalf("Failed to create file %v", err)
	}
	err = writeMetadata(f, newMetadata(100))
	if err != nil {
		t.Fatalf("Failed to write metadata %v", err)
	}
	meta, err := readMetadata(f)
	if err != nil {
		t.Fatalf("Failed to read metadata %v", err)
	}
	if meta.Version != freezerVersion {
		t.Fatalf("Unexpected version field")
	}
	if meta.VirtualTail != uint64(100) {
		t.Fatalf("Unexpected virtual tail field")
	}
}

func TestInitializeFreezerTableMeta(t *testing.T) {
	f, err := os.CreateTemp(os.TempDir(), "*")
	if err != nil {
		t.Fatalf("Failed to create file %v", err)
	}
	meta, err := loadMetadata(f, uint64(100))
	if err != nil {
		t.Fatalf("Failed to read metadata %v", err)
	}
	if meta.Version != freezerVersion {
		t.Fatalf("Unexpected version field")
	}
	if meta.VirtualTail != uint64(100) {
		t.Fatalf("Unexpected virtual tail field")
	}
}
