//go:build !noasm && !appengine && gc
// +build !noasm,!appengine,gc

/*
 * Minio Cloud Storage, (C) 2017 Minio, Inc.
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

package sha256

import (
	"bytes"
	"encoding/binary"
	"encoding/hex"
	"fmt"
	"hash"
	"reflect"
	"sync"
	"testing"
)

func TestGoldenAVX512(t *testing.T) {

	if !hasAvx512 {
		t.SkipNow()
		return
	}

	server := NewAvx512Server()
	h512 := NewAvx512(server)

	for _, g := range golden {
		h512.Reset()
		h512.Write([]byte(g.in))
		digest := h512.Sum([]byte{})
		s := fmt.Sprintf("%x", digest)
		if !reflect.DeepEqual(digest, g.out[:]) {
			t.Fatalf("Sum256 function: sha256(%s) = %s want %s", g.in, s, hex.EncodeToString(g.out[:]))
		}
	}
}

func createInputs(size int) [16][]byte {
	input := [16][]byte{}
	for i := 0; i < 16; i++ {
		input[i] = make([]byte, size)
	}
	return input
}

func initDigests() *[512]byte {
	digests := [512]byte{}
	for i := 0; i < 16; i++ {
		binary.LittleEndian.PutUint32(digests[(i+0*16)*4:], init0)
		binary.LittleEndian.PutUint32(digests[(i+1*16)*4:], init1)
		binary.LittleEndian.PutUint32(digests[(i+2*16)*4:], init2)
		binary.LittleEndian.PutUint32(digests[(i+3*16)*4:], init3)
		binary.LittleEndian.PutUint32(digests[(i+4*16)*4:], init4)
		binary.LittleEndian.PutUint32(digests[(i+5*16)*4:], init5)
		binary.LittleEndian.PutUint32(digests[(i+6*16)*4:], init6)
		binary.LittleEndian.PutUint32(digests[(i+7*16)*4:], init7)
	}
	return &digests
}

func testSha256Avx512(t *testing.T, offset, padding int) [16][]byte {

	if !hasAvx512 {
		t.SkipNow()
		return [16][]byte{}
	}

	l := uint(len(golden[offset].in))
	extraBlock := uint(0)
	if padding == 0 {
		extraBlock += 9
	} else {
		extraBlock += 64
	}
	input := createInputs(int(l + extraBlock))
	for i := 0; i < 16; i++ {
		copy(input[i], golden[offset+i].in)
		input[i][l] = 0x80
		copy(input[i][l+1:], bytes.Repeat([]byte{0}, padding))

		// Length in bits.
		len := uint64(l)
		len <<= 3
		for ii := uint(0); ii < 8; ii++ {
			input[i][l+1+uint(padding)+ii] = byte(len >> (56 - 8*ii))
		}
	}
	mask := make([]uint64, len(input[0])>>6)
	for m := range mask {
		mask[m] = 0xffff
	}
	output := blockAvx512(initDigests(), input, mask)
	for i := 0; i < 16; i++ {
		if bytes.Compare(output[i][:], golden[offset+i].out[:]) != 0 {
			t.Fatalf("Sum256 function: sha256(%s) = %s want %s", golden[offset+i].in, hex.EncodeToString(output[i][:]), hex.EncodeToString(golden[offset+i].out[:]))
		}
	}
	return input
}

func TestAvx512_1Block(t *testing.T)  { testSha256Avx512(t, 31, 0) }
func TestAvx512_3Blocks(t *testing.T) { testSha256Avx512(t, 47, 55) }

func TestAvx512_MixedBlocks(t *testing.T) {

	if !hasAvx512 {
		t.SkipNow()
		return
	}

	inputSingleBlock := testSha256Avx512(t, 31, 0)
	inputMultiBlock := testSha256Avx512(t, 47, 55)

	input := [16][]byte{}

	for i := range input {
		if i%2 == 0 {
			input[i] = inputMultiBlock[i]
		} else {
			input[i] = inputSingleBlock[i]
		}
	}

	mask := [3]uint64{0xffff, 0x5555, 0x5555}
	output := blockAvx512(initDigests(), input, mask[:])
	var offset int
	for i := 0; i < len(output); i++ {
		if i%2 == 0 {
			offset = 47
		} else {
			offset = 31
		}
		if bytes.Compare(output[i][:], golden[offset+i].out[:]) != 0 {
			t.Fatalf("Sum256 function: sha256(%s) = %s want %s", golden[offset+i].in, hex.EncodeToString(output[i][:]), hex.EncodeToString(golden[offset+i].out[:]))
		}
	}
}

func TestAvx512_MixedWithNilBlocks(t *testing.T) {

	if !hasAvx512 {
		t.SkipNow()
		return
	}

	inputSingleBlock := testSha256Avx512(t, 31, 0)
	inputMultiBlock := testSha256Avx512(t, 47, 55)

	input := [16][]byte{}

	for i := range input {
		if i%3 == 0 {
			input[i] = inputMultiBlock[i]
		} else if i%3 == 1 {
			input[i] = inputSingleBlock[i]
		} else {
			input[i] = nil
		}
	}

	mask := [3]uint64{0xb6db, 0x9249, 0x9249}
	output := blockAvx512(initDigests(), input, mask[:])
	var offset int
	for i := 0; i < len(output); i++ {
		if i%3 == 2 { // for nil inputs
			initvec := [32]byte{0x6a, 0x09, 0xe6, 0x67, 0xbb, 0x67, 0xae, 0x85,
				0x3c, 0x6e, 0xf3, 0x72, 0xa5, 0x4f, 0xf5, 0x3a,
				0x51, 0x0e, 0x52, 0x7f, 0x9b, 0x05, 0x68, 0x8c,
				0x1f, 0x83, 0xd9, 0xab, 0x5b, 0xe0, 0xcd, 0x19}
			if bytes.Compare(output[i][:], initvec[:]) != 0 {
				t.Fatalf("Sum256 function: sha256 for nil vector = %s want %s", hex.EncodeToString(output[i][:]), hex.EncodeToString(initvec[:]))
			}
			continue
		}
		if i%3 == 0 {
			offset = 47
		} else {
			offset = 31
		}
		if bytes.Compare(output[i][:], golden[offset+i].out[:]) != 0 {
			t.Fatalf("Sum256 function: sha256(%s) = %s want %s", golden[offset+i].in, hex.EncodeToString(output[i][:]), hex.EncodeToString(golden[offset+i].out[:]))
		}
	}
}

func TestAvx512Server(t *testing.T) {

	if !hasAvx512 {
		t.SkipNow()
		return
	}

	const offset = 31 + 16
	server := NewAvx512Server()

	// First block of 64 bytes
	for i := 0; i < 16; i++ {
		input := make([]byte, 64)
		copy(input, golden[offset+i].in)
		server.Write(uint64(Avx512ServerUID+i), input)
	}

	// Second block of 64 bytes
	for i := 0; i < 16; i++ {
		input := make([]byte, 64)
		copy(input, golden[offset+i].in[64:])
		server.Write(uint64(Avx512ServerUID+i), input)
	}

	wg := sync.WaitGroup{}
	wg.Add(16)

	// Third and final block
	for i := 0; i < 16; i++ {
		input := make([]byte, 64)
		input[0] = 0x80
		copy(input[1:], bytes.Repeat([]byte{0}, 63-8))

		// Length in bits.
		len := uint64(128)
		len <<= 3
		for ii := uint(0); ii < 8; ii++ {
			input[63-8+1+ii] = byte(len >> (56 - 8*ii))
		}
		go func(i int, uid uint64, input []byte) {
			output := server.Sum(uid, input)
			if bytes.Compare(output[:], golden[offset+i].out[:]) != 0 {
				t.Fatalf("Sum256 function: sha256(%s) = %s want %s", golden[offset+i].in, hex.EncodeToString(output[:]), hex.EncodeToString(golden[offset+i].out[:]))
			}
			wg.Done()
		}(i, uint64(Avx512ServerUID+i), input)
	}

	wg.Wait()
}

func TestAvx512Digest(t *testing.T) {

	if !hasAvx512 {
		t.SkipNow()
		return
	}

	server := NewAvx512Server()

	const tests = 16
	h512 := [16]hash.Hash{}
	for i := 0; i < tests; i++ {
		h512[i] = NewAvx512(server)
	}

	const offset = 31 + 16
	for i := 0; i < tests; i++ {
		input := make([]byte, 64)
		copy(input, golden[offset+i].in)
		h512[i].Write(input)
	}
	for i := 0; i < tests; i++ {
		input := make([]byte, 64)
		copy(input, golden[offset+i].in[64:])
		h512[i].Write(input)
	}
	for i := 0; i < tests; i++ {
		output := h512[i].Sum([]byte{})
		if bytes.Compare(output[:], golden[offset+i].out[:]) != 0 {
			t.Fatalf("Sum256 function: sha256(%s) = %s want %s", golden[offset+i].in, hex.EncodeToString(output[:]), hex.EncodeToString(golden[offset+i].out[:]))
		}
	}
}

func benchmarkAvx512SingleCore(h512 []hash.Hash, body []byte) {

	for i := 0; i < len(h512); i++ {
		h512[i].Write(body)
	}
	for i := 0; i < len(h512); i++ {
		_ = h512[i].Sum([]byte{})
	}
}

func benchmarkAvx512(b *testing.B, size int) {

	if !hasAvx512 {
		b.SkipNow()
		return
	}

	server := NewAvx512Server()

	const tests = 16
	body := make([]byte, size)

	b.SetBytes(int64(len(body) * tests))
	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		h512 := make([]hash.Hash, tests)
		for i := 0; i < tests; i++ {
			h512[i] = NewAvx512(server)
		}

		benchmarkAvx512SingleCore(h512, body)
	}
}

func BenchmarkAvx512_05M(b *testing.B) { benchmarkAvx512(b, 512*1024) }
func BenchmarkAvx512_1M(b *testing.B)  { benchmarkAvx512(b, 1*1024*1024) }
func BenchmarkAvx512_5M(b *testing.B)  { benchmarkAvx512(b, 5*1024*1024) }
func BenchmarkAvx512_10M(b *testing.B) { benchmarkAvx512(b, 10*1024*1024) }

func benchmarkAvx512MultiCore(b *testing.B, size, cores int) {

	if !hasAvx512 {
		b.SkipNow()
		return
	}

	servers := make([]*Avx512Server, cores)
	for c := 0; c < cores; c++ {
		servers[c] = NewAvx512Server()
	}

	const tests = 16

	body := make([]byte, size)

	h512 := make([]hash.Hash, tests*cores)
	for i := 0; i < tests*cores; i++ {
		h512[i] = NewAvx512(servers[i>>4])
	}

	b.SetBytes(int64(size * 16 * cores))
	b.ResetTimer()

	var wg sync.WaitGroup

	for i := 0; i < b.N; i++ {
		wg.Add(cores)
		for c := 0; c < cores; c++ {
			go func(c int) { benchmarkAvx512SingleCore(h512[c*tests:(c+1)*tests], body); wg.Done() }(c)
		}
		wg.Wait()
	}
}

func BenchmarkAvx512_5M_2Cores(b *testing.B) { benchmarkAvx512MultiCore(b, 5*1024*1024, 2) }
func BenchmarkAvx512_5M_4Cores(b *testing.B) { benchmarkAvx512MultiCore(b, 5*1024*1024, 4) }
func BenchmarkAvx512_5M_6Cores(b *testing.B) { benchmarkAvx512MultiCore(b, 5*1024*1024, 6) }

type maskTest struct {
	in  [16]int
	out [16]maskRounds
}

var goldenMask = []maskTest{
	{[16]int{0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0}, [16]maskRounds{}},
	{[16]int{64, 0, 64, 0, 64, 0, 64, 0, 64, 0, 64, 0, 64, 0, 64, 0}, [16]maskRounds{{0x5555, 1}}},
	{[16]int{0, 64, 0, 64, 0, 64, 0, 64, 0, 64, 0, 64, 0, 64, 0, 64}, [16]maskRounds{{0xaaaa, 1}}},
	{[16]int{64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64}, [16]maskRounds{{0xffff, 1}}},
	{[16]int{128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128}, [16]maskRounds{{0xffff, 2}}},
	{[16]int{64, 128, 64, 128, 64, 128, 64, 128, 64, 128, 64, 128, 64, 128, 64, 128}, [16]maskRounds{{0xffff, 1}, {0xaaaa, 1}}},
	{[16]int{128, 64, 128, 64, 128, 64, 128, 64, 128, 64, 128, 64, 128, 64, 128, 64}, [16]maskRounds{{0xffff, 1}, {0x5555, 1}}},
	{[16]int{64, 192, 64, 192, 64, 192, 64, 192, 64, 192, 64, 192, 64, 192, 64, 192}, [16]maskRounds{{0xffff, 1}, {0xaaaa, 2}}},
	//
	//  >= 64   0110=6          1011=b          1101=d           0110=6
	//  >=128   0100=4          0010=2          1001=9           0100=4
	{[16]int{0, 64, 128, 0, 64, 128, 0, 64, 128, 0, 64, 128, 0, 64, 128, 0}, [16]maskRounds{{0x6db6, 1}, {0x4924, 1}}},
	{[16]int{1 * 64, 2 * 64, 3 * 64, 4 * 64, 5 * 64, 6 * 64, 7 * 64, 8 * 64, 9 * 64, 10 * 64, 11 * 64, 12 * 64, 13 * 64, 14 * 64, 15 * 64, 16 * 64},
		[16]maskRounds{{0xffff, 1}, {0xfffe, 1}, {0xfffc, 1}, {0xfff8, 1}, {0xfff0, 1}, {0xffe0, 1}, {0xffc0, 1}, {0xff80, 1},
			{0xff00, 1}, {0xfe00, 1}, {0xfc00, 1}, {0xf800, 1}, {0xf000, 1}, {0xe000, 1}, {0xc000, 1}, {0x8000, 1}}},
	{[16]int{2 * 64, 1 * 64, 3 * 64, 4 * 64, 5 * 64, 6 * 64, 7 * 64, 8 * 64, 9 * 64, 10 * 64, 11 * 64, 12 * 64, 13 * 64, 14 * 64, 15 * 64, 16 * 64},
		[16]maskRounds{{0xffff, 1}, {0xfffd, 1}, {0xfffc, 1}, {0xfff8, 1}, {0xfff0, 1}, {0xffe0, 1}, {0xffc0, 1}, {0xff80, 1},
			{0xff00, 1}, {0xfe00, 1}, {0xfc00, 1}, {0xf800, 1}, {0xf000, 1}, {0xe000, 1}, {0xc000, 1}, {0x8000, 1}}},
	{[16]int{10 * 64, 20 * 64, 30 * 64, 40 * 64, 50 * 64, 60 * 64, 70 * 64, 80 * 64, 90 * 64, 100 * 64, 110 * 64, 120 * 64, 130 * 64, 140 * 64, 150 * 64, 160 * 64},
		[16]maskRounds{{0xffff, 10}, {0xfffe, 10}, {0xfffc, 10}, {0xfff8, 10}, {0xfff0, 10}, {0xffe0, 10}, {0xffc0, 10}, {0xff80, 10},
			{0xff00, 10}, {0xfe00, 10}, {0xfc00, 10}, {0xf800, 10}, {0xf000, 10}, {0xe000, 10}, {0xc000, 10}, {0x8000, 10}}},
	{[16]int{10 * 64, 19 * 64, 27 * 64, 34 * 64, 40 * 64, 45 * 64, 49 * 64, 52 * 64, 54 * 64, 55 * 64, 57 * 64, 60 * 64, 64 * 64, 69 * 64, 75 * 64, 82 * 64},
		[16]maskRounds{{0xffff, 10}, {0xfffe, 9}, {0xfffc, 8}, {0xfff8, 7}, {0xfff0, 6}, {0xffe0, 5}, {0xffc0, 4}, {0xff80, 3},
			{0xff00, 2}, {0xfe00, 1}, {0xfc00, 2}, {0xf800, 3}, {0xf000, 4}, {0xe000, 5}, {0xc000, 6}, {0x8000, 7}}},
}

func TestMaskGen(t *testing.T) {
	input := [16][]byte{}
	for gcase, g := range goldenMask {
		for i, l := range g.in {
			buf := make([]byte, l)
			input[i] = buf[:]
		}

		mr := genMask(input)

		if !reflect.DeepEqual(mr, g.out) {
			t.Fatalf("case %d: got %04x\n                    want %04x", gcase, mr, g.out)
		}
	}
}
