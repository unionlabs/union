// Copyright 2014 Will Fitzgerald. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// This file tests bit sets

package bitset

import (
	"bytes"
	"math/rand"
	"testing"
)

func BenchmarkSet(b *testing.B) {
	b.StopTimer()
	r := rand.New(rand.NewSource(0))
	sz := 100000
	s := New(uint(sz))
	b.StartTimer()
	for i := 0; i < b.N; i++ {
		s.Set(uint(r.Int31n(int32(sz))))
	}
}

func BenchmarkGetTest(b *testing.B) {
	b.StopTimer()
	r := rand.New(rand.NewSource(0))
	sz := 100000
	s := New(uint(sz))
	b.StartTimer()
	for i := 0; i < b.N; i++ {
		s.Test(uint(r.Int31n(int32(sz))))
	}
}

func BenchmarkSetExpand(b *testing.B) {
	b.StopTimer()
	sz := uint(100000)
	b.StartTimer()
	for i := 0; i < b.N; i++ {
		var s BitSet
		s.Set(sz)
	}
}

// go test -bench=Count
func BenchmarkCount(b *testing.B) {
	b.StopTimer()
	s := New(100000)
	for i := 0; i < 100000; i += 100 {
		s.Set(uint(i))
	}
	b.StartTimer()
	for i := 0; i < b.N; i++ {
		s.Count()
	}
}

// go test -bench=Iterate
func BenchmarkIterate(b *testing.B) {
	b.StopTimer()
	s := New(10000)
	for i := 0; i < 10000; i += 3 {
		s.Set(uint(i))
	}
	b.StartTimer()
	for j := 0; j < b.N; j++ {
		c := uint(0)
		for i, e := s.NextSet(0); e; i, e = s.NextSet(i + 1) {
			c++
		}
	}
}

// go test -bench=SparseIterate
func BenchmarkSparseIterate(b *testing.B) {
	b.StopTimer()
	s := New(100000)
	for i := 0; i < 100000; i += 30 {
		s.Set(uint(i))
	}
	b.StartTimer()
	for j := 0; j < b.N; j++ {
		c := uint(0)
		for i, e := s.NextSet(0); e; i, e = s.NextSet(i + 1) {
			c++
		}
	}
}

// go test -bench=LemireCreate
// see http://lemire.me/blog/2016/09/22/swift-versus-java-the-bitset-performance-test/
func BenchmarkLemireCreate(b *testing.B) {
	for i := 0; i < b.N; i++ {
		bitmap := New(0) // we force dynamic memory allocation
		for v := uint(0); v <= 100000000; v += 100 {
			bitmap.Set(v)
		}
	}
}

// go test -bench=LemireCount
// see http://lemire.me/blog/2016/09/22/swift-versus-java-the-bitset-performance-test/
func BenchmarkLemireCount(b *testing.B) {
	bitmap := New(100000000)
	for v := uint(0); v <= 100000000; v += 100 {
		bitmap.Set(v)
	}
	b.ResetTimer()
	sum := uint(0)
	for i := 0; i < b.N; i++ {
		sum += bitmap.Count()
	}
	if sum == 0 { // added just to fool ineffassign
		return
	}
}

// go test -bench=LemireIterate
// see http://lemire.me/blog/2016/09/22/swift-versus-java-the-bitset-performance-test/
func BenchmarkLemireIterate(b *testing.B) {
	bitmap := New(100000000)
	for v := uint(0); v <= 100000000; v += 100 {
		bitmap.Set(v)
	}
	b.ResetTimer()
	sum := uint(0)
	for i := 0; i < b.N; i++ {
		for j, e := bitmap.NextSet(0); e; j, e = bitmap.NextSet(j + 1) {
			sum++
		}
	}
	if sum == 0 { // added just to fool ineffassign
		return
	}
}

// go test -bench=LemireIterateb
// see http://lemire.me/blog/2016/09/22/swift-versus-java-the-bitset-performance-test/
func BenchmarkLemireIterateb(b *testing.B) {
	bitmap := New(100000000)
	for v := uint(0); v <= 100000000; v += 100 {
		bitmap.Set(v)
	}
	b.ResetTimer()
	sum := uint(0)
	for i := 0; i < b.N; i++ {
		for j, e := bitmap.NextSet(0); e; j, e = bitmap.NextSet(j + 1) {
			sum += j
		}
	}

	if sum == 0 { // added just to fool ineffassign
		return
	}
}

// go test -bench=BenchmarkLemireIterateManyb
// see http://lemire.me/blog/2016/09/22/swift-versus-java-the-bitset-performance-test/
func BenchmarkLemireIterateManyb(b *testing.B) {
	bitmap := New(100000000)
	for v := uint(0); v <= 100000000; v += 100 {
		bitmap.Set(v)
	}
	buffer := make([]uint, 256)
	b.ResetTimer()
	sum := uint(0)
	for i := 0; i < b.N; i++ {
		j := uint(0)
		j, buffer = bitmap.NextSetMany(j, buffer)
		for ; len(buffer) > 0; j, buffer = bitmap.NextSetMany(j, buffer) {
			for k := range buffer {
				sum += buffer[k]
			}
			j++
		}
	}

	if sum == 0 { // added just to fool ineffassign
		return
	}
}

func setRnd(bits []uint64, halfings int) {
	var rnd = rand.NewSource(0).(rand.Source64)
	for i := range bits {
		bits[i] = 0xFFFFFFFFFFFFFFFF
		for j := 0; j < halfings; j++ {
			bits[i] &= rnd.Uint64()
		}
	}
}

// go test -bench=BenchmarkFlorianUekermannIterateMany
func BenchmarkFlorianUekermannIterateMany(b *testing.B) {
	var input = make([]uint64, 68)
	setRnd(input, 4)
	var bitmap = From(input)
	buffer := make([]uint, 256)
	b.ResetTimer()
	var checksum = uint(0)
	for i := 0; i < b.N; i++ {
		var last, batch = bitmap.NextSetMany(0, buffer)
		for len(batch) > 0 {
			for _, idx := range batch {
				checksum += idx
			}
			last, batch = bitmap.NextSetMany(last+1, batch)
		}
	}
	if checksum == 0 { // added just to fool ineffassign
		return
	}
}

func BenchmarkFlorianUekermannIterateManyReg(b *testing.B) {
	var input = make([]uint64, 68)
	setRnd(input, 4)
	var bitmap = From(input)
	b.ResetTimer()
	var checksum = uint(0)
	for i := 0; i < b.N; i++ {
		for j, e := bitmap.NextSet(0); e; j, e = bitmap.NextSet(j + 1) {
			checksum += j
		}
	}
	if checksum == 0 { // added just to fool ineffassign
		return
	}
}

// function provided by FlorianUekermann
func good(set []uint64) (checksum uint) {
	for wordIdx, word := range set {
		var wordIdx = uint(wordIdx * 64)
		for word != 0 {
			var bitIdx = uint(trailingZeroes64(word))
			word ^= 1 << bitIdx
			var index = wordIdx + bitIdx
			checksum += index
		}
	}
	return checksum
}

func BenchmarkFlorianUekermannIterateManyComp(b *testing.B) {
	var input = make([]uint64, 68)
	setRnd(input, 4)
	b.ResetTimer()
	var checksum = uint(0)
	for i := 0; i < b.N; i++ {
		checksum += good(input)
	}
	if checksum == 0 { // added just to fool ineffassign
		return
	}
}

/////// Mid density

// go test -bench=BenchmarkFlorianUekermannLowDensityIterateMany
func BenchmarkFlorianUekermannLowDensityIterateMany(b *testing.B) {
	var input = make([]uint64, 1000000)
	var rnd = rand.NewSource(0).(rand.Source64)
	for i := 0; i < 50000; i++ {
		input[rnd.Uint64()%1000000] = 1
	}
	var bitmap = From(input)
	buffer := make([]uint, 256)
	b.ResetTimer()
	var sum = uint(0)
	for i := 0; i < b.N; i++ {
		j := uint(0)
		j, buffer = bitmap.NextSetMany(j, buffer)
		for ; len(buffer) > 0; j, buffer = bitmap.NextSetMany(j, buffer) {
			for k := range buffer {
				sum += buffer[k]
			}
			j++
		}
	}
	if sum == 0 { // added just to fool ineffassign
		return
	}
}

func BenchmarkFlorianUekermannLowDensityIterateManyReg(b *testing.B) {
	var input = make([]uint64, 1000000)
	var rnd = rand.NewSource(0).(rand.Source64)
	for i := 0; i < 50000; i++ {
		input[rnd.Uint64()%1000000] = 1
	}
	var bitmap = From(input)
	b.ResetTimer()
	var checksum = uint(0)
	for i := 0; i < b.N; i++ {
		for j, e := bitmap.NextSet(0); e; j, e = bitmap.NextSet(j + 1) {
			checksum += j
		}
	}
	if checksum == 0 { // added just to fool ineffassign
		return
	}
}

func BenchmarkFlorianUekermannLowDensityIterateManyComp(b *testing.B) {
	var input = make([]uint64, 1000000)
	var rnd = rand.NewSource(0).(rand.Source64)
	for i := 0; i < 50000; i++ {
		input[rnd.Uint64()%1000000] = 1
	}
	b.ResetTimer()
	var checksum = uint(0)
	for i := 0; i < b.N; i++ {
		checksum += good(input)
	}
	if checksum == 0 { // added just to fool ineffassign
		return
	}
}

/////// Mid density

// go test -bench=BenchmarkFlorianUekermannMidDensityIterateMany
func BenchmarkFlorianUekermannMidDensityIterateMany(b *testing.B) {
	var input = make([]uint64, 1000000)
	var rnd = rand.NewSource(0).(rand.Source64)
	for i := 0; i < 3000000; i++ {
		input[rnd.Uint64()%1000000] |= uint64(1) << (rnd.Uint64() % 64)
	}
	var bitmap = From(input)
	buffer := make([]uint, 256)
	b.ResetTimer()
	sum := uint(0)
	for i := 0; i < b.N; i++ {
		j := uint(0)
		j, buffer = bitmap.NextSetMany(j, buffer)
		for ; len(buffer) > 0; j, buffer = bitmap.NextSetMany(j, buffer) {
			for k := range buffer {
				sum += buffer[k]
			}
			j++
		}
	}

	if sum == 0 { // added just to fool ineffassign
		return
	}
}

func BenchmarkFlorianUekermannMidDensityIterateManyReg(b *testing.B) {
	var input = make([]uint64, 1000000)
	var rnd = rand.NewSource(0).(rand.Source64)
	for i := 0; i < 3000000; i++ {
		input[rnd.Uint64()%1000000] |= uint64(1) << (rnd.Uint64() % 64)
	}
	var bitmap = From(input)
	b.ResetTimer()
	var checksum = uint(0)
	for i := 0; i < b.N; i++ {
		for j, e := bitmap.NextSet(0); e; j, e = bitmap.NextSet(j + 1) {
			checksum += j
		}
	}
	if checksum == 0 { // added just to fool ineffassign
		return
	}
}

func BenchmarkFlorianUekermannMidDensityIterateManyComp(b *testing.B) {
	var input = make([]uint64, 1000000)
	var rnd = rand.NewSource(0).(rand.Source64)
	for i := 0; i < 3000000; i++ {
		input[rnd.Uint64()%1000000] |= uint64(1) << (rnd.Uint64() % 64)
	}
	b.ResetTimer()
	var checksum = uint(0)
	for i := 0; i < b.N; i++ {
		checksum += good(input)
	}
	if checksum == 0 { // added just to fool ineffassign
		return
	}
}

////////// High density

func BenchmarkFlorianUekermannMidStrongDensityIterateMany(b *testing.B) {
	var input = make([]uint64, 1000000)
	var rnd = rand.NewSource(0).(rand.Source64)
	for i := 0; i < 20000000; i++ {
		input[rnd.Uint64()%1000000] |= uint64(1) << (rnd.Uint64() % 64)
	}
	var bitmap = From(input)
	buffer := make([]uint, 256)
	b.ResetTimer()
	sum := uint(0)
	for i := 0; i < b.N; i++ {
		j := uint(0)
		j, buffer = bitmap.NextSetMany(j, buffer)
		for ; len(buffer) > 0; j, buffer = bitmap.NextSetMany(j, buffer) {
			for k := range buffer {
				sum += buffer[k]
			}
			j++
		}
	}

	if sum == 0 { // added just to fool ineffassign
		return
	}
}

func BenchmarkFlorianUekermannMidStrongDensityIterateManyReg(b *testing.B) {
	var input = make([]uint64, 1000000)
	var rnd = rand.NewSource(0).(rand.Source64)
	for i := 0; i < 20000000; i++ {
		input[rnd.Uint64()%1000000] |= uint64(1) << (rnd.Uint64() % 64)
	}
	var bitmap = From(input)
	b.ResetTimer()
	var checksum = uint(0)
	for i := 0; i < b.N; i++ {
		for j, e := bitmap.NextSet(0); e; j, e = bitmap.NextSet(j + 1) {
			checksum += j
		}
	}
	if checksum == 0 { // added just to fool ineffassign
		return
	}
}

func BenchmarkFlorianUekermannMidStrongDensityIterateManyComp(b *testing.B) {
	var input = make([]uint64, 1000000)
	var rnd = rand.NewSource(0).(rand.Source64)
	for i := 0; i < 20000000; i++ {
		input[rnd.Uint64()%1000000] |= uint64(1) << (rnd.Uint64() % 64)
	}
	b.ResetTimer()
	var checksum = uint(0)
	for i := 0; i < b.N; i++ {
		checksum += good(input)
	}
	if checksum == 0 { // added just to fool ineffassign
		return
	}
}

func BenchmarkBitsetReadWrite(b *testing.B) {
	s := New(100000)
	for i := 0; i < 100000; i += 100 {
		s.Set(uint(i))
	}
	buffer := bytes.Buffer{}
	temp := New(100000)
	b.ResetTimer()
	for i := 0; i < b.N; i++ {
		s.WriteTo(&buffer)
		temp.ReadFrom(&buffer)
		buffer.Reset()
	}
}
