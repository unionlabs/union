/*
 * Copyright 2021 Dgraph Labs, Inc. and Contributors
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

package cmd

import (
	"bytes"
	"fmt"
	"os"
	"runtime/pprof"
	"sort"
	"testing"

	"github.com/spf13/cobra"

	"github.com/dgraph-io/badger/v4"
	"github.com/dgraph-io/badger/v4/options"
	"github.com/dgraph-io/badger/v4/table"
	"github.com/dgraph-io/badger/v4/y"
)

var pickBenchCmd = &cobra.Command{
	Use:   "picktable",
	Short: "Benchmark pick tables.",
	Long:  `This command simulates pickTables used in iterators.`,
	RunE:  pickTableBench,
}

var (
	pickOpts = struct {
		readOnly   bool
		sampleSize int
		cpuprofile string
	}{}
	keys    [][]byte
	handler levelHandler
)

func init() {
	benchCmd.AddCommand(pickBenchCmd)
	pickBenchCmd.Flags().BoolVar(
		&pickOpts.readOnly, "read-only", true, "If true, DB will be opened in read only mode.")
	pickBenchCmd.Flags().IntVar(
		&pickOpts.sampleSize, "sample-size", 1000000, "Sample size of keys to be used for lookup.")
	pickBenchCmd.Flags().StringVar(
		&pickOpts.cpuprofile, "cpuprofile", "", "Write CPU profile to file.")
}

func pickTableBench(cmd *cobra.Command, args []string) error {
	opt := badger.DefaultOptions(sstDir).
		WithValueDir(vlogDir).
		WithReadOnly(pickOpts.readOnly)
	fmt.Printf("Opening badger with options = %+v\n", opt)
	db, err := badger.OpenManaged(opt)
	if err != nil {
		return y.Wrapf(err, "unable to open DB")
	}
	defer func() {
		y.Check(db.Close())
	}()

	boundaries := getBoundaries(db)
	tables := genTables(boundaries)
	defer func() {
		for _, tbl := range tables {
			if err := tbl.DecrRef(); err != nil {
				panic(err)
			}
		}
	}()
	handler.init(tables)
	keys, err = getSampleKeys(db, pickOpts.sampleSize)
	y.Check(err)
	fmt.Println("Running benchmark...")
	fmt.Println("***** BenchmarkPickTables *****")
	fmt.Println(testing.Benchmark(BenchmarkPickTables))
	fmt.Println("*******************************")
	return nil
}

func BenchmarkPickTables(b *testing.B) {
	if len(pickOpts.cpuprofile) > 0 {
		f, err := os.Create(pickOpts.cpuprofile)
		y.Check(err)
		err = pprof.StartCPUProfile(f)
		y.Check(err)
		defer pprof.StopCPUProfile()
	}
	b.ResetTimer()
	iopts := iteratorOptions{prefixIsKey: true}
	for i := 0; i < b.N; i++ {
		for _, key := range keys {
			iopts.Prefix = key
			_ = handler.pickTables(iopts)
		}
	}
}

// See badger.IteratorOptions (iterator.go)
type iteratorOptions struct {
	prefixIsKey bool   // If set, use the prefix for bloom filter lookup.
	Prefix      []byte // Only iterate over this given prefix.
	SinceTs     uint64 // Only read data that has version > SinceTs.
}

// See compareToPrefix in iterator.go
func (opt *iteratorOptions) compareToPrefix(key []byte) int {
	// We should compare key without timestamp. For example key - a[TS] might be > "aa" prefix.
	key = y.ParseKey(key)
	if len(key) > len(opt.Prefix) {
		key = key[:len(opt.Prefix)]
	}
	return bytes.Compare(key, opt.Prefix)
}

// See levelHandler in level_handler.go
type levelHandler struct {
	tables []*table.Table
}

func (s *levelHandler) init(tables []*table.Table) {
	fmt.Println("Initializing level handler...")
	s.tables = tables
}

// This implementation is based on the implementation in master branch.
func (s *levelHandler) pickTables(opt iteratorOptions) []*table.Table {
	filterTables := func(tables []*table.Table) []*table.Table {
		if opt.SinceTs > 0 {
			tmp := tables[:0]
			for _, t := range tables {
				if t.MaxVersion() < opt.SinceTs {
					continue
				}
				tmp = append(tmp, t)
			}
			tables = tmp
		}
		return tables
	}

	all := s.tables
	if len(opt.Prefix) == 0 {
		out := make([]*table.Table, len(all))
		copy(out, all)
		return filterTables(out)
	}
	sIdx := sort.Search(len(all), func(i int) bool {
		// table.Biggest >= opt.prefix
		// if opt.Prefix < table.Biggest, then surely it is not in any of the preceding tables.
		return opt.compareToPrefix(all[i].Biggest()) >= 0
	})
	if sIdx == len(all) {
		// Not found.
		return []*table.Table{}
	}

	filtered := all[sIdx:]
	if !opt.prefixIsKey {
		eIdx := sort.Search(len(filtered), func(i int) bool {
			return opt.compareToPrefix(filtered[i].Smallest()) > 0
		})
		out := make([]*table.Table, len(filtered[:eIdx]))
		copy(out, filtered[:eIdx])
		return filterTables(out)
	}

	// opt.prefixIsKey == true. This code is optimizing for opt.prefixIsKey part.
	var out []*table.Table
	// hash := y.Hash(opt.Prefix)
	for _, t := range filtered {
		// When we encounter the first table whose smallest key is higher than opt.Prefix, we can
		// stop. This is an IMPORTANT optimization, just considering how often we call
		// NewKeyIterator.
		if opt.compareToPrefix(t.Smallest()) > 0 {
			// if table.Smallest > opt.Prefix, then this and all tables after this can be ignored.
			break
		}
		out = append(out, t)
	}
	return filterTables(out)
}

// Sorts the boundaries and creates mock table out of them.
func genTables(boundaries [][]byte) []*table.Table {
	buildTable := func(k1, k2 []byte) *table.Table {
		opts := table.Options{
			ChkMode: options.NoVerification,
		}
		b := table.NewTableBuilder(opts)
		defer b.Close()
		// Add one key so that we can open this table.
		b.Add(y.KeyWithTs(k1, 1), y.ValueStruct{}, 0)
		b.Add(y.KeyWithTs(k2, 1), y.ValueStruct{}, 0)
		tab, err := table.OpenInMemoryTable(b.Finish(), 0, &opts)
		y.Check(err)
		return tab
	}

	sort.Slice(boundaries, func(i, j int) bool {
		return bytes.Compare(boundaries[i], boundaries[j]) < 0
	})
	out := make([]*table.Table, 0, len(boundaries))
	for i := range boundaries {
		var j int
		if i != 0 {
			j = i - 1
		}
		out = append(out, buildTable(boundaries[i], boundaries[j]))
	}
	fmt.Printf("Created %d mock tables.\n", len(out))
	return out
}

func getBoundaries(db *badger.DB) [][]byte {
	fmt.Println("Getting the table boundaries...")
	tables := db.Tables()
	out := make([][]byte, 0, 2*len(tables))
	for _, t := range tables {
		out = append(out, t.Left, t.Right)
	}
	return out
}
