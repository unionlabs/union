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

package bls

import "testing"

func FuzzCrossPairing(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzzCrossPairing(data)
	})
}

func FuzzCrossG1Add(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzzCrossG1Add(data)
	})
}

func FuzzCrossG2Add(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzzCrossG2Add(data)
	})
}

func FuzzCrossG1MultiExp(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzzCrossG1MultiExp(data)
	})
}

func FuzzG1Add(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzz(blsG1Add, data)
	})
}

func FuzzG1Mul(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzz(blsG1Mul, data)
	})
}

func FuzzG1MultiExp(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzz(blsG1MultiExp, data)
	})
}

func FuzzG2Add(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzz(blsG2Add, data)
	})
}

func FuzzG2Mul(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzz(blsG2Mul, data)
	})
}

func FuzzG2MultiExp(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzz(blsG2MultiExp, data)
	})
}

func FuzzPairing(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzz(blsPairing, data)
	})
}

func FuzzMapG1(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzz(blsMapG1, data)
	})
}

func FuzzMapG2(f *testing.F) {
	f.Fuzz(func(t *testing.T, data []byte) {
		fuzz(blsMapG2, data)
	})
}
