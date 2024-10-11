// Copyright 2017 The go-ethereum Authors
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

package filters

import (
	"context"
	"fmt"
	"testing"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/bitutil"
	"github.com/ethereum/go-ethereum/core/bloombits"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/node"
)

func BenchmarkBloomBits512(b *testing.B) {
	benchmarkBloomBits(b, 512)
}

func BenchmarkBloomBits1k(b *testing.B) {
	benchmarkBloomBits(b, 1024)
}

func BenchmarkBloomBits2k(b *testing.B) {
	benchmarkBloomBits(b, 2048)
}

func BenchmarkBloomBits4k(b *testing.B) {
	benchmarkBloomBits(b, 4096)
}

func BenchmarkBloomBits8k(b *testing.B) {
	benchmarkBloomBits(b, 8192)
}

func BenchmarkBloomBits16k(b *testing.B) {
	benchmarkBloomBits(b, 16384)
}

func BenchmarkBloomBits32k(b *testing.B) {
	benchmarkBloomBits(b, 32768)
}

const benchFilterCnt = 2000

func benchmarkBloomBits(b *testing.B, sectionSize uint64) {
	b.Skip("test disabled: this tests presume (and modify) an existing datadir.")
	benchDataDir := node.DefaultDataDir() + "/geth/chaindata"
	b.Log("Running bloombits benchmark   section size:", sectionSize)

	db, err := rawdb.NewLevelDBDatabase(benchDataDir, 128, 1024, "", false)
	if err != nil {
		b.Fatalf("error opening database at %v: %v", benchDataDir, err)
	}
	head := rawdb.ReadHeadBlockHash(db)
	if head == (common.Hash{}) {
		b.Fatalf("chain data not found at %v", benchDataDir)
	}

	clearBloomBits(db)
	b.Log("Generating bloombits data...")
	headNum := rawdb.ReadHeaderNumber(db, head)
	if headNum == nil || *headNum < sectionSize+512 {
		b.Fatalf("not enough blocks for running a benchmark")
	}

	start := time.Now()
	cnt := (*headNum - 512) / sectionSize
	var dataSize, compSize uint64
	for sectionIdx := uint64(0); sectionIdx < cnt; sectionIdx++ {
		bc, err := bloombits.NewGenerator(uint(sectionSize))
		if err != nil {
			b.Fatalf("failed to create generator: %v", err)
		}
		var header *types.Header
		for i := sectionIdx * sectionSize; i < (sectionIdx+1)*sectionSize; i++ {
			hash := rawdb.ReadCanonicalHash(db, i)
			if header = rawdb.ReadHeader(db, hash, i); header == nil {
				b.Fatalf("Error creating bloomBits data")
				return
			}
			bc.AddBloom(uint(i-sectionIdx*sectionSize), header.Bloom)
		}
		sectionHead := rawdb.ReadCanonicalHash(db, (sectionIdx+1)*sectionSize-1)
		for i := 0; i < types.BloomBitLength; i++ {
			data, err := bc.Bitset(uint(i))
			if err != nil {
				b.Fatalf("failed to retrieve bitset: %v", err)
			}
			comp := bitutil.CompressBytes(data)
			dataSize += uint64(len(data))
			compSize += uint64(len(comp))
			rawdb.WriteBloomBits(db, uint(i), sectionIdx, sectionHead, comp)
		}
		//if sectionIdx%50 == 0 {
		//	b.Log(" section", sectionIdx, "/", cnt)
		//}
	}

	d := time.Since(start)
	b.Log("Finished generating bloombits data")
	b.Log(" ", d, "total  ", d/time.Duration(cnt*sectionSize), "per block")
	b.Log(" data size:", dataSize, "  compressed size:", compSize, "  compression ratio:", float64(compSize)/float64(dataSize))

	b.Log("Running filter benchmarks...")
	start = time.Now()

	var (
		backend *testBackend
		sys     *FilterSystem
	)
	for i := 0; i < benchFilterCnt; i++ {
		if i%20 == 0 {
			db.Close()
			db, _ = rawdb.NewLevelDBDatabase(benchDataDir, 128, 1024, "", false)
			backend = &testBackend{db: db, sections: cnt}
			sys = NewFilterSystem(backend, Config{})
		}
		var addr common.Address
		addr[0] = byte(i)
		addr[1] = byte(i / 256)
		filter := sys.NewRangeFilter(0, int64(cnt*sectionSize-1), []common.Address{addr}, nil)
		if _, err := filter.Logs(context.Background()); err != nil {
			b.Error("filter.Logs error:", err)
		}
	}

	d = time.Since(start)
	b.Log("Finished running filter benchmarks")
	b.Log(" ", d, "total  ", d/time.Duration(benchFilterCnt), "per address", d*time.Duration(1000000)/time.Duration(benchFilterCnt*cnt*sectionSize), "per million blocks")
	db.Close()
}

//nolint:unused
func clearBloomBits(db ethdb.Database) {
	var bloomBitsPrefix = []byte("bloomBits-")
	fmt.Println("Clearing bloombits data...")
	it := db.NewIterator(bloomBitsPrefix, nil)
	for it.Next() {
		db.Delete(it.Key())
	}
	it.Release()
}

func BenchmarkNoBloomBits(b *testing.B) {
	b.Skip("test disabled: this tests presume (and modify) an existing datadir.")
	benchDataDir := node.DefaultDataDir() + "/geth/chaindata"
	b.Log("Running benchmark without bloombits")
	db, err := rawdb.NewLevelDBDatabase(benchDataDir, 128, 1024, "", false)
	if err != nil {
		b.Fatalf("error opening database at %v: %v", benchDataDir, err)
	}
	head := rawdb.ReadHeadBlockHash(db)
	if head == (common.Hash{}) {
		b.Fatalf("chain data not found at %v", benchDataDir)
	}
	headNum := rawdb.ReadHeaderNumber(db, head)

	clearBloomBits(db)

	_, sys := newTestFilterSystem(b, db, Config{})

	b.Log("Running filter benchmarks...")
	start := time.Now()
	filter := sys.NewRangeFilter(0, int64(*headNum), []common.Address{{}}, nil)
	filter.Logs(context.Background())
	d := time.Since(start)
	b.Log("Finished running filter benchmarks")
	b.Log(" ", d, "total  ", d*time.Duration(1000000)/time.Duration(*headNum+1), "per million blocks")
	db.Close()
}
