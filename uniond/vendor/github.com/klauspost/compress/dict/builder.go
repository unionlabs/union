// Copyright 2023+ Klaus Post. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package dict

import (
	"bytes"
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"math/rand"
	"sort"
	"time"

	"github.com/klauspost/compress/s2"
	"github.com/klauspost/compress/zstd"
)

type match struct {
	hash   uint32
	n      uint32
	offset int64
}

type matchValue struct {
	value       []byte
	followBy    map[uint32]uint32
	preceededBy map[uint32]uint32
}

type Options struct {
	// MaxDictSize is the max size of the backreference dictionary.
	MaxDictSize int

	// HashBytes is the minimum length to index.
	// Must be >=4 and <=8
	HashBytes int

	// Debug output
	Output io.Writer

	// ZstdDictID is the Zstd dictionary ID to use.
	// Leave at zero to generate a random ID.
	ZstdDictID uint32

	// ZstdDictCompat will make the dictionary compatible with Zstd v1.5.5 and earlier.
	// See https://github.com/facebook/zstd/issues/3724
	ZstdDictCompat bool

	// Use the specified encoder level for Zstandard dictionaries.
	// The dictionary will be built using the specified encoder level,
	// which will reflect speed and make the dictionary tailored for that level.
	// If not set zstd.SpeedBestCompression will be used.
	ZstdLevel zstd.EncoderLevel

	outFormat int
}

const (
	formatRaw = iota
	formatZstd
	formatS2
)

// BuildZstdDict will build a Zstandard dictionary from the provided input.
func BuildZstdDict(input [][]byte, o Options) ([]byte, error) {
	o.outFormat = formatZstd
	if o.ZstdDictID == 0 {
		rng := rand.New(rand.NewSource(time.Now().UnixNano()))
		o.ZstdDictID = 32768 + uint32(rng.Int31n((1<<31)-32768))
	}
	return buildDict(input, o)
}

// BuildS2Dict will build a S2 dictionary from the provided input.
func BuildS2Dict(input [][]byte, o Options) ([]byte, error) {
	o.outFormat = formatS2
	if o.MaxDictSize > s2.MaxDictSize {
		return nil, errors.New("max dict size too large")
	}
	return buildDict(input, o)
}

// BuildRawDict will build a raw dictionary from the provided input.
// This can be used for deflate, lz4 and others.
func BuildRawDict(input [][]byte, o Options) ([]byte, error) {
	o.outFormat = formatRaw
	return buildDict(input, o)
}

func buildDict(input [][]byte, o Options) ([]byte, error) {
	matches := make(map[uint32]uint32)
	offsets := make(map[uint32]int64)
	var total uint64

	wantLen := o.MaxDictSize
	hashBytes := o.HashBytes
	if len(input) == 0 {
		return nil, fmt.Errorf("no input provided")
	}
	if hashBytes < 4 || hashBytes > 8 {
		return nil, fmt.Errorf("HashBytes must be >= 4 and <= 8")
	}
	println := func(args ...interface{}) {
		if o.Output != nil {
			fmt.Fprintln(o.Output, args...)
		}
	}
	printf := func(s string, args ...interface{}) {
		if o.Output != nil {
			fmt.Fprintf(o.Output, s, args...)
		}
	}
	found := make(map[uint32]struct{})
	for i, b := range input {
		for k := range found {
			delete(found, k)
		}
		for i := range b {
			rem := b[i:]
			if len(rem) < 8 {
				break
			}
			h := hashLen(binary.LittleEndian.Uint64(rem), 32, uint8(hashBytes))
			if _, ok := found[h]; ok {
				// Only count first occurrence
				continue
			}
			matches[h]++
			offsets[h] += int64(i)
			total++
			found[h] = struct{}{}
		}
		printf("\r input %d indexed...", i)
	}
	threshold := uint32(total / uint64(len(matches)))
	println("\nTotal", total, "match", len(matches), "avg", threshold)
	sorted := make([]match, 0, len(matches)/2)
	for k, v := range matches {
		if v <= threshold {
			continue
		}
		sorted = append(sorted, match{hash: k, n: v, offset: offsets[k]})
	}
	sort.Slice(sorted, func(i, j int) bool {
		if true {
			// Group very similar counts together and emit low offsets first.
			// This will keep together strings that are very similar.
			deltaN := int(sorted[i].n) - int(sorted[j].n)
			if deltaN < 0 {
				deltaN = -deltaN
			}
			if uint32(deltaN) < sorted[i].n/32 {
				return sorted[i].offset < sorted[j].offset
			}
		} else {
			if sorted[i].n == sorted[j].n {
				return sorted[i].offset < sorted[j].offset
			}
		}
		return sorted[i].n > sorted[j].n
	})
	println("Sorted len:", len(sorted))
	if len(sorted) > wantLen {
		sorted = sorted[:wantLen]
	}
	lowestOcc := sorted[len(sorted)-1].n
	println("Cropped len:", len(sorted), "Lowest occurrence:", lowestOcc)

	wantMatches := make(map[uint32]uint32, len(sorted))
	for _, v := range sorted {
		wantMatches[v.hash] = v.n
	}

	output := make(map[uint32]matchValue, len(sorted))
	var remainCnt [256]int
	var remainTotal int
	var firstOffsets []int
	for i, b := range input {
		for i := range b {
			rem := b[i:]
			if len(rem) < 8 {
				break
			}
			var prev []byte
			if i > hashBytes {
				prev = b[i-hashBytes:]
			}

			h := hashLen(binary.LittleEndian.Uint64(rem), 32, uint8(hashBytes))
			if _, ok := wantMatches[h]; !ok {
				remainCnt[rem[0]]++
				remainTotal++
				continue
			}
			mv := output[h]
			if len(mv.value) == 0 {
				var tmp = make([]byte, hashBytes)
				copy(tmp[:], rem)
				mv.value = tmp[:]
			}
			if mv.followBy == nil {
				mv.followBy = make(map[uint32]uint32, 4)
				mv.preceededBy = make(map[uint32]uint32, 4)
			}
			if len(rem) > hashBytes+8 {
				// Check if we should add next as well.
				hNext := hashLen(binary.LittleEndian.Uint64(rem[hashBytes:]), 32, uint8(hashBytes))
				if _, ok := wantMatches[hNext]; ok {
					mv.followBy[hNext]++
				}
			}
			if len(prev) >= 8 {
				// Check if we should prev next as well.
				hPrev := hashLen(binary.LittleEndian.Uint64(prev), 32, uint8(hashBytes))
				if _, ok := wantMatches[hPrev]; ok {
					mv.preceededBy[hPrev]++
				}
			}
			output[h] = mv
		}
		printf("\rinput %d re-indexed...", i)
	}
	println("")
	dst := make([][]byte, 0, wantLen/hashBytes)
	added := 0
	const printUntil = 500
	for i, e := range sorted {
		if added > o.MaxDictSize {
			println("Ending. Next Occurrence:", e.n)
			break
		}
		m, ok := output[e.hash]
		if !ok {
			// Already added
			continue
		}
		wantLen := e.n / uint32(hashBytes) / 4
		if wantLen <= lowestOcc {
			wantLen = lowestOcc
		}

		var tmp = make([]byte, 0, hashBytes*2)
		{
			sortedPrev := make([]match, 0, len(m.followBy))
			for k, v := range m.preceededBy {
				if _, ok := output[k]; v < wantLen || !ok {
					continue
				}
				sortedPrev = append(sortedPrev, match{
					hash: k,
					n:    v,
				})
			}
			if len(sortedPrev) > 0 {
				sort.Slice(sortedPrev, func(i, j int) bool {
					return sortedPrev[i].n > sortedPrev[j].n
				})
				bestPrev := output[sortedPrev[0].hash]
				tmp = append(tmp, bestPrev.value...)
			}
		}
		tmp = append(tmp, m.value...)
		delete(output, e.hash)

		sortedFollow := make([]match, 0, len(m.followBy))
		for {
			var nh uint32 // Next hash
			stopAfter := false
			{
				sortedFollow = sortedFollow[:0]
				for k, v := range m.followBy {
					if _, ok := output[k]; !ok {
						continue
					}
					sortedFollow = append(sortedFollow, match{
						hash:   k,
						n:      v,
						offset: offsets[k],
					})
				}
				if len(sortedFollow) == 0 {
					// Step back
					// Extremely small impact, but helps longer hashes a bit.
					const stepBack = 2
					if stepBack > 0 && len(tmp) >= hashBytes+stepBack {
						var t8 [8]byte
						copy(t8[:], tmp[len(tmp)-hashBytes-stepBack:])
						m, ok = output[hashLen(binary.LittleEndian.Uint64(t8[:]), 32, uint8(hashBytes))]
						if ok && len(m.followBy) > 0 {
							found := []byte(nil)
							for k := range m.followBy {
								v, ok := output[k]
								if !ok {
									continue
								}
								found = v.value
								break
							}
							if found != nil {
								tmp = tmp[:len(tmp)-stepBack]
								printf("Step back: %q +  %q\n", string(tmp), string(found))
								continue
							}
						}
						break
					} else {
						if i < printUntil {
							printf("FOLLOW: none after %q\n", string(m.value))
						}
					}
					break
				}
				sort.Slice(sortedFollow, func(i, j int) bool {
					if sortedFollow[i].n == sortedFollow[j].n {
						return sortedFollow[i].offset > sortedFollow[j].offset
					}
					return sortedFollow[i].n > sortedFollow[j].n
				})
				nh = sortedFollow[0].hash
				stopAfter = sortedFollow[0].n < wantLen
				if stopAfter && i < printUntil {
					printf("FOLLOW: %d < %d after %q. Stopping after this.\n", sortedFollow[0].n, wantLen, string(m.value))
				}
			}
			m, ok = output[nh]
			if !ok {
				break
			}
			if len(tmp) > 0 {
				// Delete all hashes that are in the current string to avoid stuttering.
				var toDel [16 + 8]byte
				copy(toDel[:], tmp[len(tmp)-hashBytes:])
				copy(toDel[hashBytes:], m.value)
				for i := range toDel[:hashBytes*2] {
					delete(output, hashLen(binary.LittleEndian.Uint64(toDel[i:]), 32, uint8(hashBytes)))
				}
			}
			tmp = append(tmp, m.value...)
			//delete(output, nh)
			if stopAfter {
				// Last entry was no significant.
				break
			}
		}
		if i < printUntil {
			printf("ENTRY %d: %q (%d occurrences, cutoff %d)\n", i, string(tmp), e.n, wantLen)
		}
		// Delete substrings already added.
		if len(tmp) > hashBytes {
			for j := range tmp[:len(tmp)-hashBytes+1] {
				var t8 [8]byte
				copy(t8[:], tmp[j:])
				if i < printUntil {
					//printf("* POST DELETE %q\n", string(t8[:hashBytes]))
				}
				delete(output, hashLen(binary.LittleEndian.Uint64(t8[:]), 32, uint8(hashBytes)))
			}
		}
		dst = append(dst, tmp)
		added += len(tmp)
		// Find offsets
		// TODO: This can be better if done as a global search.
		if len(firstOffsets) < 3 {
			if len(tmp) > 16 {
				tmp = tmp[:16]
			}
			offCnt := make(map[int]int, len(input))
			// Find first offsets
			for _, b := range input {
				off := bytes.Index(b, tmp)
				if off == -1 {
					continue
				}
				offCnt[off]++
			}
			for _, off := range firstOffsets {
				// Very unlikely, but we deleted it just in case
				delete(offCnt, off-added)
			}
			maxCnt := 0
			maxOffset := 0
			for k, v := range offCnt {
				if v == maxCnt && k > maxOffset {
					// Prefer the longer offset on ties , since it is more expensive to encode
					maxCnt = v
					maxOffset = k
					continue
				}

				if v > maxCnt {
					maxCnt = v
					maxOffset = k
				}
			}
			if maxCnt > 1 {
				firstOffsets = append(firstOffsets, maxOffset+added)
				println(" - Offset:", len(firstOffsets), "at", maxOffset+added, "count:", maxCnt, "total added:", added, "src index", maxOffset)
			}
		}
	}
	out := bytes.NewBuffer(nil)
	written := 0
	for i, toWrite := range dst {
		if len(toWrite)+written > wantLen {
			toWrite = toWrite[:wantLen-written]
		}
		dst[i] = toWrite
		written += len(toWrite)
		if written >= wantLen {
			dst = dst[:i+1]
			break
		}
	}
	// Write in reverse order.
	for i := range dst {
		toWrite := dst[len(dst)-i-1]
		out.Write(toWrite)
	}
	if o.outFormat == formatRaw {
		return out.Bytes(), nil
	}

	if o.outFormat == formatS2 {
		dOff := 0
		dBytes := out.Bytes()
		if len(dBytes) > s2.MaxDictSize {
			dBytes = dBytes[:s2.MaxDictSize]
		}
		for _, off := range firstOffsets {
			myOff := len(dBytes) - off
			if myOff < 0 || myOff > s2.MaxDictSrcOffset {
				continue
			}
			dOff = myOff
		}

		dict := s2.MakeDictManual(dBytes, uint16(dOff))
		if dict == nil {
			return nil, fmt.Errorf("unable to create s2 dictionary")
		}
		return dict.Bytes(), nil
	}

	offsetsZstd := [3]int{1, 4, 8}
	for i, off := range firstOffsets {
		if i >= 3 || off == 0 || off >= out.Len() {
			break
		}
		offsetsZstd[i] = off
	}
	println("\nCompressing. Offsets:", offsetsZstd)
	return zstd.BuildDict(zstd.BuildDictOptions{
		ID:         o.ZstdDictID,
		Contents:   input,
		History:    out.Bytes(),
		Offsets:    offsetsZstd,
		CompatV155: o.ZstdDictCompat,
		Level:      o.ZstdLevel,
		DebugOut:   o.Output,
	})
}

const (
	prime3bytes = 506832829
	prime4bytes = 2654435761
	prime5bytes = 889523592379
	prime6bytes = 227718039650203
	prime7bytes = 58295818150454627
	prime8bytes = 0xcf1bbcdcb7a56463
)

// hashLen returns a hash of the lowest l bytes of u for a size size of h bytes.
// l must be >=4 and <=8. Any other value will return hash for 4 bytes.
// h should always be <32.
// Preferably h and l should be a constant.
// LENGTH 4 is passed straight through
func hashLen(u uint64, hashLog, mls uint8) uint32 {
	switch mls {
	case 5:
		return hash5(u, hashLog)
	case 6:
		return hash6(u, hashLog)
	case 7:
		return hash7(u, hashLog)
	case 8:
		return hash8(u, hashLog)
	default:
		return uint32(u)
	}
}

// hash3 returns the hash of the lower 3 bytes of u to fit in a hash table with h bits.
// Preferably h should be a constant and should always be <32.
func hash3(u uint32, h uint8) uint32 {
	return ((u << (32 - 24)) * prime3bytes) >> ((32 - h) & 31)
}

// hash4 returns the hash of u to fit in a hash table with h bits.
// Preferably h should be a constant and should always be <32.
func hash4(u uint32, h uint8) uint32 {
	return (u * prime4bytes) >> ((32 - h) & 31)
}

// hash4x64 returns the hash of the lowest 4 bytes of u to fit in a hash table with h bits.
// Preferably h should be a constant and should always be <32.
func hash4x64(u uint64, h uint8) uint32 {
	return (uint32(u) * prime4bytes) >> ((32 - h) & 31)
}

// hash5 returns the hash of the lowest 5 bytes of u to fit in a hash table with h bits.
// Preferably h should be a constant and should always be <64.
func hash5(u uint64, h uint8) uint32 {
	return uint32(((u << (64 - 40)) * prime5bytes) >> ((64 - h) & 63))
}

// hash6 returns the hash of the lowest 6 bytes of u to fit in a hash table with h bits.
// Preferably h should be a constant and should always be <64.
func hash6(u uint64, h uint8) uint32 {
	return uint32(((u << (64 - 48)) * prime6bytes) >> ((64 - h) & 63))
}

// hash7 returns the hash of the lowest 7 bytes of u to fit in a hash table with h bits.
// Preferably h should be a constant and should always be <64.
func hash7(u uint64, h uint8) uint32 {
	return uint32(((u << (64 - 56)) * prime7bytes) >> ((64 - h) & 63))
}

// hash8 returns the hash of u to fit in a hash table with h bits.
// Preferably h should be a constant and should always be <64.
func hash8(u uint64, h uint8) uint32 {
	return uint32((u * prime8bytes) >> ((64 - h) & 63))
}
