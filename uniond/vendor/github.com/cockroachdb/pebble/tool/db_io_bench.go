// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package tool

import (
	"context"
	"fmt"
	"io"
	"math"
	"math/rand"
	"sort"
	"strconv"
	"strings"
	"sync"
	"time"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/spf13/cobra"
)

type benchIO struct {
	readableIdx int
	ofs         int64
	size        int
	// elapsed time for the IO, filled out by performIOs.
	elapsed time.Duration
}

const maxIOSize = 1024 * 1024

// runIOBench runs an IO benchmark against the current sstables of a database.
// The workload is random IO, with various IO sizes. The main goal of the
// benchmark is to establish the relationship between IO size and latency,
// especially against shared object storage.
func (d *dbT) runIOBench(cmd *cobra.Command, args []string) {
	stdout := cmd.OutOrStdout()

	ioSizes, err := parseIOSizes(d.ioSizes)
	if err != nil {
		fmt.Fprintf(stdout, "error parsing io-sizes: %s\n", err)
		return
	}

	db, err := d.openDB(args[0])
	if err != nil {
		fmt.Fprintf(stdout, "%s\n", err)
		return
	}
	defer d.closeDB(stdout, db)

	readables, err := d.openBenchTables(db)
	if err != nil {
		fmt.Fprintf(stdout, "%s\n", err)
		return
	}

	defer func() {
		for _, r := range readables {
			r.Close()
		}
	}()

	ios := genBenchIOs(stdout, readables, d.ioCount, ioSizes)

	levels := "L5,L6"
	if d.allLevels {
		levels = "all"
	}
	fmt.Fprintf(stdout, "IO count: %d  Parallelism: %d  Levels: %s\n", d.ioCount, d.ioParallelism, levels)

	var wg sync.WaitGroup
	wg.Add(d.ioParallelism)
	remainingIOs := ios
	for i := 0; i < d.ioParallelism; i++ {
		// We want to distribute the IOs among d.ioParallelism goroutines. At each
		// step, we look at the number of IOs remaining and take the average (across
		// the goroutines that are left); this deals with any rounding issues.
		n := len(remainingIOs) / (d.ioParallelism - i)
		go func(workerIdx int, ios []benchIO) {
			defer wg.Done()
			if err := performIOs(readables, ios); err != nil {
				fmt.Fprintf(stdout, "worker %d encountered error: %v", workerIdx, err)
			}
		}(i, remainingIOs[:n])
		remainingIOs = remainingIOs[n:]
	}
	wg.Wait()

	elapsed := make([]time.Duration, d.ioCount)
	for _, ioSize := range ioSizes {
		elapsed = elapsed[:0]
		for i := range ios {
			if ios[i].size == ioSize {
				elapsed = append(elapsed, ios[i].elapsed)
			}
		}
		fmt.Fprintf(stdout, "%4dKB  --  %s\n", ioSize/1024, getStats(elapsed))
	}
}

// genBenchIOs generates <count> IOs for each given size. All IOs (across all
// sizes) are in random order.
func genBenchIOs(
	stdout io.Writer, readables []objstorage.Readable, count int, sizes []int,
) []benchIO {
	// size[i] is the size of the object, in blocks of maxIOSize.
	size := make([]int, len(readables))
	// sum[i] is the sum (size[0] + ... + size[i]).
	sum := make([]int, len(readables))
	total := 0
	for i, r := range readables {
		size[i] = int(r.Size() / maxIOSize)
		total += size[i]
		sum[i] = total
	}
	fmt.Fprintf(stdout, "Opened %d objects; total size %d MB.\n", len(readables), total*maxIOSize/(1024*1024))

	// To avoid a lot of overlap between the reads, the total size should be a
	// factor larger than the size we will actually read (for the largest IO
	// size).
	const sizeFactor = 2
	if total*maxIOSize < count*sizes[len(sizes)-1]*sizeFactor {
		fmt.Fprintf(stdout, "Warning: store too small for the given IO count and sizes.\n")
	}

	// Choose how many IOs we do for each object, by selecting a random block
	// across all file blocks.
	// The choice of objects will be the same across all IO sizes.
	b := make([]int, count)
	for i := range b {
		b[i] = rand.Intn(total)
	}
	// For each b[i], find the index such that sum[idx-1] <= b < sum[idx].
	// Sorting b makes this easier: we can "merge" the sorted arrays b and sum.
	sort.Ints(b)
	rIdx := make([]int, count)
	currIdx := 0
	for i := range b {
		for b[i] >= sum[currIdx] {
			currIdx++
		}
		rIdx[i] = currIdx
	}

	res := make([]benchIO, 0, count*len(sizes))
	for _, ioSize := range sizes {
		for _, idx := range rIdx {
			// Random ioSize aligned offset.
			ofs := ioSize * rand.Intn(size[idx]*maxIOSize/ioSize)

			res = append(res, benchIO{
				readableIdx: idx,
				ofs:         int64(ofs),
				size:        ioSize,
			})
		}
	}
	rand.Shuffle(len(res), func(i, j int) {
		res[i], res[j] = res[j], res[i]
	})
	return res
}

// openBenchTables opens the sstables for the benchmark and returns them as a
// list of Readables.
//
// By default, only L5/L6 sstables are used; all levels are used if the
// allLevels flag is set.
//
// Note that only sstables that are at least maxIOSize (1MB) are used.
func (d *dbT) openBenchTables(db *pebble.DB) ([]objstorage.Readable, error) {
	tables, err := db.SSTables()
	if err != nil {
		return nil, err
	}
	startLevel := 5
	if d.allLevels {
		startLevel = 0
	}

	var nums []base.DiskFileNum
	numsMap := make(map[base.DiskFileNum]struct{})
	for l := startLevel; l < len(tables); l++ {
		for _, t := range tables[l] {
			n := t.BackingSSTNum.DiskFileNum()
			if _, ok := numsMap[n]; !ok {
				nums = append(nums, n)
				numsMap[n] = struct{}{}
			}
		}
	}

	p := db.ObjProvider()
	var res []objstorage.Readable
	for _, n := range nums {
		r, err := p.OpenForReading(context.Background(), base.FileTypeTable, n, objstorage.OpenOptions{})
		if err != nil {
			for _, r := range res {
				_ = r.Close()
			}
			return nil, err
		}
		if r.Size() < maxIOSize {
			_ = r.Close()
			continue
		}
		res = append(res, r)
	}
	if len(res) == 0 {
		return nil, errors.Errorf("no sstables (with size at least %d)", maxIOSize)
	}

	return res, nil
}

// parseIOSizes parses a comma-separated list of IO sizes, in KB.
func parseIOSizes(sizes string) ([]int, error) {
	var res []int
	for _, s := range strings.Split(sizes, ",") {
		n, err := strconv.Atoi(s)
		if err != nil {
			return nil, err
		}
		ioSize := n * 1024
		if ioSize > maxIOSize {
			return nil, errors.Errorf("IO sizes over %d not supported", maxIOSize)
		}
		if maxIOSize%ioSize != 0 {
			return nil, errors.Errorf("IO size must be a divisor of %d", maxIOSize)
		}
		res = append(res, ioSize)
	}
	if len(res) == 0 {
		return nil, errors.Errorf("no IO sizes specified")
	}
	sort.Ints(res)
	return res, nil
}

// performIOs performs the given list of IOs and populates the elapsed fields.
func performIOs(readables []objstorage.Readable, ios []benchIO) error {
	ctx := context.Background()
	rh := make([]objstorage.ReadHandle, len(readables))
	for i := range rh {
		rh[i] = readables[i].NewReadHandle(ctx)
	}
	defer func() {
		for i := range rh {
			rh[i].Close()
		}
	}()

	buf := make([]byte, maxIOSize)
	startTime := time.Now()
	var firstErr error
	var nOtherErrs int
	for i := range ios {
		if err := rh[ios[i].readableIdx].ReadAt(ctx, buf[:ios[i].size], ios[i].ofs); err != nil {
			if firstErr == nil {
				firstErr = err
			} else {
				nOtherErrs++
			}
		}
		endTime := time.Now()
		ios[i].elapsed = endTime.Sub(startTime)
		startTime = endTime
	}
	if nOtherErrs > 0 {
		return errors.Errorf("%v; plus %d more errors", firstErr, nOtherErrs)
	}
	return firstErr
}

// getStats calculates various statistics given a list of elapsed times.
func getStats(d []time.Duration) string {
	sort.Slice(d, func(i, j int) bool { return d[i] < d[j] })

	factor := 1.0 / float64(len(d))
	var mean float64
	for i := range d {
		mean += float64(d[i]) * factor
	}
	var variance float64
	for i := range d {
		delta := float64(d[i]) - mean
		variance += delta * delta * factor
	}

	toStr := func(d time.Duration) string {
		if d < 10*time.Millisecond {
			return fmt.Sprintf("%1.2fms", float64(d)/float64(time.Millisecond))
		}
		if d < 100*time.Millisecond {
			return fmt.Sprintf("%2.1fms", float64(d)/float64(time.Millisecond))
		}
		return fmt.Sprintf("%4dms", d/time.Millisecond)
	}

	return fmt.Sprintf(
		"avg %s   stddev %s   p10 %s   p50 %s   p90 %s   p95 %s   p99 %s",
		toStr(time.Duration(mean)),
		toStr(time.Duration(math.Sqrt(variance))),
		toStr(d[len(d)*10/100]),
		toStr(d[len(d)*50/100]),
		toStr(d[len(d)*90/100]),
		toStr(d[len(d)*95/100]),
		toStr(d[len(d)*99/100]),
	)
}
