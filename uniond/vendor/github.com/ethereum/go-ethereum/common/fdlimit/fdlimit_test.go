// Copyright 2016 The go-ethereum Authors
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

package fdlimit

import (
	"testing"
)

// TestFileDescriptorLimits simply tests whether the file descriptor allowance
// per this process can be retrieved.
func TestFileDescriptorLimits(t *testing.T) {
	target := 4096
	hardlimit, err := Maximum()
	if err != nil {
		t.Fatal(err)
	}
	if hardlimit < target {
		t.Skipf("system limit is less than desired test target: %d < %d", hardlimit, target)
	}

	if limit, err := Current(); err != nil || limit <= 0 {
		t.Fatalf("failed to retrieve file descriptor limit (%d): %v", limit, err)
	}
	if _, err := Raise(uint64(target)); err != nil {
		t.Fatalf("failed to raise file allowance")
	}
	if limit, err := Current(); err != nil || limit < target {
		t.Fatalf("failed to retrieve raised descriptor limit (have %v, want %v): %v", limit, target, err)
	}
}
