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
	"bytes"
	"os"
	"testing"
)

func TestCopyFrom(t *testing.T) {
	var (
		content = []byte{0x1, 0x2, 0x3, 0x4, 0x5, 0x6, 0x7, 0x8}
		prefix  = []byte{0x9, 0xa, 0xb, 0xc, 0xd, 0xf}
	)
	var cases = []struct {
		src, dest   string
		offset      uint64
		writePrefix bool
	}{
		{"foo", "bar", 0, false},
		{"foo", "bar", 1, false},
		{"foo", "bar", 8, false},
		{"foo", "foo", 0, false},
		{"foo", "foo", 1, false},
		{"foo", "foo", 8, false},
		{"foo", "bar", 0, true},
		{"foo", "bar", 1, true},
		{"foo", "bar", 8, true},
	}
	for _, c := range cases {
		os.WriteFile(c.src, content, 0600)

		if err := copyFrom(c.src, c.dest, c.offset, func(f *os.File) error {
			if !c.writePrefix {
				return nil
			}
			f.Write(prefix)
			return nil
		}); err != nil {
			os.Remove(c.src)
			t.Fatalf("Failed to copy %v", err)
		}

		blob, err := os.ReadFile(c.dest)
		if err != nil {
			os.Remove(c.src)
			os.Remove(c.dest)
			t.Fatalf("Failed to read %v", err)
		}
		want := content[c.offset:]
		if c.writePrefix {
			want = append(prefix, want...)
		}
		if !bytes.Equal(blob, want) {
			t.Fatal("Unexpected value")
		}
		os.Remove(c.src)
		os.Remove(c.dest)
	}
}
