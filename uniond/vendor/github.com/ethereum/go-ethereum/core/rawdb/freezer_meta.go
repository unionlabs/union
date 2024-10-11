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
	"io"
	"os"

	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/rlp"
)

const freezerVersion = 1 // The initial version tag of freezer table metadata

// freezerTableMeta wraps all the metadata of the freezer table.
type freezerTableMeta struct {
	// Version is the versioning descriptor of the freezer table.
	Version uint16

	// VirtualTail indicates how many items have been marked as deleted.
	// Its value is equal to the number of items removed from the table
	// plus the number of items hidden in the table, so it should never
	// be lower than the "actual tail".
	VirtualTail uint64
}

// newMetadata initializes the metadata object with the given virtual tail.
func newMetadata(tail uint64) *freezerTableMeta {
	return &freezerTableMeta{
		Version:     freezerVersion,
		VirtualTail: tail,
	}
}

// readMetadata reads the metadata of the freezer table from the
// given metadata file.
func readMetadata(file *os.File) (*freezerTableMeta, error) {
	_, err := file.Seek(0, io.SeekStart)
	if err != nil {
		return nil, err
	}
	var meta freezerTableMeta
	if err := rlp.Decode(file, &meta); err != nil {
		return nil, err
	}
	return &meta, nil
}

// writeMetadata writes the metadata of the freezer table into the
// given metadata file.
func writeMetadata(file *os.File, meta *freezerTableMeta) error {
	_, err := file.Seek(0, io.SeekStart)
	if err != nil {
		return err
	}
	return rlp.Encode(file, meta)
}

// loadMetadata loads the metadata from the given metadata file.
// Initializes the metadata file with the given "actual tail" if
// it's empty.
func loadMetadata(file *os.File, tail uint64) (*freezerTableMeta, error) {
	stat, err := file.Stat()
	if err != nil {
		return nil, err
	}
	// Write the metadata with the given actual tail into metadata file
	// if it's non-existent. There are two possible scenarios here:
	// - the freezer table is empty
	// - the freezer table is legacy
	// In both cases, write the meta into the file with the actual tail
	// as the virtual tail.
	if stat.Size() == 0 {
		m := newMetadata(tail)
		if err := writeMetadata(file, m); err != nil {
			return nil, err
		}
		return m, nil
	}
	m, err := readMetadata(file)
	if err != nil {
		return nil, err
	}
	// Update the virtual tail with the given actual tail if it's even
	// lower than it. Theoretically it shouldn't happen at all, print
	// a warning here.
	if m.VirtualTail < tail {
		log.Warn("Updated virtual tail", "have", m.VirtualTail, "now", tail)
		m.VirtualTail = tail
		if err := writeMetadata(file, m); err != nil {
			return nil, err
		}
	}
	return m, nil
}
