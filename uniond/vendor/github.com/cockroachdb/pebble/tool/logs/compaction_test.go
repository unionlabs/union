// Copyright 2021 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package logs

import (
	"bytes"
	"fmt"
	"math"
	"os"
	"path/filepath"
	"regexp"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/stretchr/testify/require"
)

const (
	compactionStartLine23_1 = `I211215 14:26:56.012382 51831533 3@vendor/github.com/cockroachdb/pebble/compaction.go:1845 ⋮ [n5,pebble,s6] 1216510  [JOB 284925] compacting(default) L2 [442555] (4.2 M) + L3 [445853] (8.4 M)`
	compactionStartLine     = `I211215 14:26:56.012382 51831533 3@vendor/github.com/cockroachdb/pebble/compaction.go:1845 ⋮ [n5,pebble,s6] 1216510  [JOB 284925] compacting(default) L2 [442555] (4.2MB) Score=1.01 + L3 [445853] (8.4MB) Score=0.99;  OverlappingRatio: Single 8.03, Multi 25.05;`

	compactionEndLine23_1 = `I211215 14:26:56.318543 51831533 3@vendor/github.com/cockroachdb/pebble/compaction.go:1886 ⋮ [n5,pebble,s6] 1216554  [JOB 284925] compacted(default) L2 [442555] (4.2 M) + L3 [445853] (8.4 M) -> L3 [445883 445887] (13 M), in 0.3s, output rate 42 M/s`
	compactionEndLine     = `I211215 14:26:56.318543 51831533 3@vendor/github.com/cockroachdb/pebble/compaction.go:1886 ⋮ [n5,pebble,s6] 1216554  [JOB 284925] compacted(default) L2 [442555] (4.2MB) Score=1.01 + L3 [445853] (8.4MB) Score=1.01 -> L3 [445883 445887] (13MB), in 0.3s, output rate 42MB/s`

	compactionMultiLevelStartLine = `I211215 14:26:56.318543 51831533 3@vendor/github.com/cockroachdb/pebble/compaction.go:1886 ⋮ [n5,pebble,s6] 1216554 [JOB 11] compacting(default) [multilevel] L2 [250858] (2.1MB) Score=1.09 + L3 [247985 247989 247848] (17MB) Score=0.99 + L4 [250817 250834 238396] (28MB) Score=1.00; OverlappingRatio: Single 3.77, Multi 1.46;`
	compactionMultiLevelEndline   = `I211215 14:26:56.318543 51831533 3@vendor/github.com/cockroachdb/pebble/compaction.go:1886 ⋮ [n5,pebble,s6] 1216554 [JOB 11] compacted(default) [multilevel] L2 [250858] (2.1MB) Score=1.09 + L3 [247985 247989 247848] (17MB) Score=0.99 + L4 [250817 250834 238396] (28MB) Score=1.00 -> L4 [250859 250860 250861 250862 250863] (46MB), in 0.2s (0.2s total), output rate 185MB/s`

	flushStartLine = `I211213 16:23:48.903751 21136 3@vendor/github.com/cockroachdb/pebble/event.go:599 ⋮ [n9,pebble,s8] 24 [JOB 10] flushing 2 memtables to L0`

	flushEndLine23_1 = `I211213 16:23:49.134464 21136 3@vendor/github.com/cockroachdb/pebble/event.go:603 ⋮ [n9,pebble,s8] 26 [JOB 10] flushed 2 memtables to L0 [1535806] (1.3 M), in 0.2s, output rate 5.8 M/s`
	flushEndLine     = `I211213 16:23:49.134464 21136 3@vendor/github.com/cockroachdb/pebble/event.go:603 ⋮ [n9,pebble,s8] 26 [JOB 10] flushed 2 memtables to L0 [1535806] (1.3MB), in 0.2s, output rate 5.8MB/s`

	readAmpLine23_1 = `  total     31766   188 G       -   257 G   187 G    48 K   3.6 G     744   536 G    49 K   278 G       5     2.1`
	readAmpLine     = `total |   32K 188GB     0B |     - | 257GB |   48K  187GB |   744  3.6GB |   49K  536GB | 278GB |   5  2.1`

	compactionStartNoNodeStoreLine23_1 = `I211215 14:26:56.012382 51831533 3@vendor/github.com/cockroachdb/pebble/compaction.go:1845 ⋮ [n?,pebble,s?] 1216510  [JOB 284925] compacting(default) L2 [442555] (4.2 M) + L3 [445853] (8.4 M)`
	compactionStartNoNodeStoreLine     = `I211215 14:26:56.012382 51831533 3@vendor/github.com/cockroachdb/pebble/compaction.go:1845 ⋮ [n?,pebble,s?] 1216510  [JOB 284925] compacting(default) L2 [442555] (4.2 M) + L3 [445853] (8.4MB)`
	flushStartNoNodeStoreLine          = `I211213 16:23:48.903751 21136 3@vendor/github.com/cockroachdb/pebble/event.go:599 ⋮ [n?,pebble,s?] 24 [JOB 10] flushing 2 memtables to L0`

	flushableIngestionLine23_1 = `I230831 04:13:28.824280 3780 3@pebble/event.go:685 ⋮ [n10,s10,pebble] 365  [JOB 226] flushed 6 ingested flushables L0:024334 (1.5 K) + L0:024339 (1.0 K) + L0:024335 (1.9 K) + L0:024336 (1.1 K) + L0:024337 (1.1 K) + L0:024338 (12 K) in 0.0s (0.0s total), output rate 67 M/s`
	flushableIngestionLine     = `I230831 04:13:28.824280 3780 3@pebble/event.go:685 ⋮ [n10,s10,pebble] 365  [JOB 226] flushed 6 ingested flushables L0:024334 (1.5KB) + L0:024339 (1.0KB) + L0:024335 (1.9KB) + L0:024336 (1.1KB) + L0:024337 (1.1KB) + L0:024338 (12KB) in 0.0s (0.0s total), output rate 67MB/s`
)

func TestCompactionLogs_Regex(t *testing.T) {
	tcs := []struct {
		name    string
		re      *regexp.Regexp
		line    string
		matches map[int]string
	}{
		{
			name: "compaction start - sentinel",
			re:   sentinelPattern,
			line: compactionStartLine23_1,
			matches: map[int]string{
				sentinelPatternPrefixIdx: "compact",
				sentinelPatternSuffixIdx: "ing",
			},
		},
		{
			name: "compaction start - sentinel",
			re:   sentinelPattern,
			line: compactionStartLine,
			matches: map[int]string{
				sentinelPatternPrefixIdx: "compact",
				sentinelPatternSuffixIdx: "ing",
			},
		},
		{
			name: "compaction end - sentinel",
			re:   sentinelPattern,
			line: compactionEndLine23_1,
			matches: map[int]string{
				sentinelPatternPrefixIdx: "compact",
				sentinelPatternSuffixIdx: "ed",
			},
		},
		{
			name: "compaction end - sentinel",
			re:   sentinelPattern,
			line: compactionEndLine,
			matches: map[int]string{
				sentinelPatternPrefixIdx: "compact",
				sentinelPatternSuffixIdx: "ed",
			},
		},
		{
			name: "flush start - sentinel",
			re:   sentinelPattern,
			line: flushStartLine,
			matches: map[int]string{
				sentinelPatternPrefixIdx: "flush",
				sentinelPatternSuffixIdx: "ing",
			},
		},
		{
			name: "flush end - sentinel",
			re:   sentinelPattern,
			line: flushEndLine23_1,
			matches: map[int]string{
				sentinelPatternPrefixIdx: "flush",
				sentinelPatternSuffixIdx: "ed",
			},
		},
		{
			name: "flush end - sentinel",
			re:   sentinelPattern,
			line: flushEndLine,
			matches: map[int]string{
				sentinelPatternPrefixIdx: "flush",
				sentinelPatternSuffixIdx: "ed",
			},
		},
		{
			name: "compaction start",
			re:   compactionPattern,
			line: compactionStartLine23_1,
			matches: map[int]string{
				compactionPatternJobIdx:    "284925",
				compactionPatternSuffixIdx: "ing",
				compactionPatternTypeIdx:   "default",
				compactionPatternFromIdx:   "2",
				compactionPatternToIdx:     "3",
				compactionPatternLevels:    "L2 [442555] (4.2 M) + L3 [445853] (8.4 M)",
			},
		},
		{
			name: "compaction start",
			re:   compactionPattern,
			line: compactionStartLine,
			matches: map[int]string{
				compactionPatternJobIdx:    "284925",
				compactionPatternSuffixIdx: "ing",
				compactionPatternTypeIdx:   "default",
				compactionPatternFromIdx:   "2",
				compactionPatternToIdx:     "3",
				compactionPatternLevels:    "L2 [442555] (4.2MB) Score=1.01 + L3 [445853] (8.4MB) Score=0.99",
			},
		},
		{
			name: "compaction start multilevel",
			re:   compactionPattern,
			line: compactionMultiLevelStartLine,
			matches: map[int]string{
				compactionPatternJobIdx:    "11",
				compactionPatternSuffixIdx: "ing",
				compactionPatternTypeIdx:   "default",
				compactionPatternFromIdx:   "2",
				compactionPatternToIdx:     "4",
				compactionPatternLevels:    "L2 [250858] (2.1MB) Score=1.09 + L3 [247985 247989 247848] (17MB) Score=0.99 + L4 [250817 250834 238396] (28MB) Score=1.00",
			},
		},
		{
			name: "compaction start - no node / store",
			re:   compactionPattern,
			line: compactionStartNoNodeStoreLine,
			matches: map[int]string{
				compactionPatternJobIdx:    "284925",
				compactionPatternSuffixIdx: "ing",
				compactionPatternTypeIdx:   "default",
				compactionPatternFromIdx:   "2",
				compactionPatternToIdx:     "3",
			},
		},
		{
			name: "compaction end",
			re:   compactionPattern,
			line: compactionEndLine,
			matches: map[int]string{
				compactionPatternJobIdx:    "284925",
				compactionPatternSuffixIdx: "ed",
				compactionPatternTypeIdx:   "default",
				compactionPatternFromIdx:   "2",
				compactionPatternToIdx:     "3",
				compactionPatternBytesIdx:  "13MB",
			},
		},
		{
			name: "compaction end",
			re:   compactionPattern,
			line: compactionMultiLevelEndline,
			matches: map[int]string{
				compactionPatternJobIdx:    "11",
				compactionPatternSuffixIdx: "ed",
				compactionPatternTypeIdx:   "default",
				compactionPatternFromIdx:   "2",
				compactionPatternToIdx:     "4",
				compactionPatternBytesIdx:  "46MB",
			},
		},
		{
			name: "flush start",
			re:   flushPattern,
			line: flushStartLine,
			matches: map[int]string{
				flushPatternJobIdx:    "10",
				flushPatternSuffixIdx: "ing",
				flushPatternBytesIdx:  "",
			},
		},
		{
			name: "flush start - no node / store",
			re:   flushPattern,
			line: flushStartNoNodeStoreLine,
			matches: map[int]string{
				flushPatternJobIdx:    "10",
				flushPatternSuffixIdx: "ing",
				flushPatternBytesIdx:  "",
			},
		},
		{
			name: "flush end",
			re:   flushPattern,
			line: flushEndLine,
			matches: map[int]string{
				flushPatternJobIdx:    "10",
				flushPatternSuffixIdx: "ed",
				flushPatternBytesIdx:  "1.3MB",
			},
		},
		{
			name: "read amp suffix",
			re:   readAmpPattern,
			line: readAmpLine,
			matches: map[int]string{
				readAmpPatternValueIdx: "5",
			},
		},
		{
			name: "read amp suffix 23.1",
			re:   readAmpPattern,
			line: readAmpLine23_1,
			matches: map[int]string{
				readAmpPatternValueIdx: "5",
			},
		},
		{
			name: "ingestion during flush job 23.1",
			re:   flushableIngestedPattern,
			line: flushableIngestionLine23_1,
			matches: map[int]string{
				flushableIngestedPatternJobIdx: "226",
			},
		},
		{
			name: "ingestion during flush 23.1",
			re:   ingestedFilePattern,
			line: flushableIngestionLine23_1,
			matches: map[int]string{
				// Just looking at the first match for these.
				ingestedFilePatternLevelIdx: "0",
				ingestedFilePatternFileIdx:  "024334",
				ingestedFilePatternBytesIdx: "1.5 K",
			},
		},
		{
			name: "ingestion during flush job",
			re:   flushableIngestedPattern,
			line: flushableIngestionLine,
			matches: map[int]string{
				flushableIngestedPatternJobIdx: "226",
			},
		},
		{
			name: "ingestion during flush",
			re:   ingestedFilePattern,
			line: flushableIngestionLine,
			matches: map[int]string{
				// Just looking at the first match for these.
				ingestedFilePatternLevelIdx: "0",
				ingestedFilePatternFileIdx:  "024334",
				ingestedFilePatternBytesIdx: "1.5KB",
			},
		},
	}

	for _, tc := range tcs {
		t.Run(tc.name, func(t *testing.T) {
			matches := tc.re.FindStringSubmatch(tc.line)
			require.NotNil(t, matches)
			for idx, want := range tc.matches {
				require.Equal(t, want, matches[idx])
			}
		})
	}
}

func TestCompactionLogs(t *testing.T) {
	dir := t.TempDir()
	datadriven.Walk(t, "testdata", func(t *testing.T, path string) {
		var c *logEventCollector
		var logNum int
		resetFn := func() {
			c = newEventCollector()
			logNum = 0
		}
		resetFn()
		datadriven.RunTest(t, path, func(t *testing.T, td *datadriven.TestData) string {
			switch td.Cmd {
			case "log":
				basename := fmt.Sprintf("%d.log", logNum)
				logNum++
				f, err := os.Create(filepath.Join(dir, basename))
				if err != nil {
					panic(err)
				}
				if _, err = f.WriteString(td.Input); err != nil {
					panic(err)
				}
				_ = f.Close()

				if err = parseLog(f.Name(), c); err != nil {
					return err.Error()
				}
				return basename

			case "summarize":
				window := 1 * time.Minute
				longRunning := time.Duration(math.MaxInt64)

				var err error
				for _, cmdArg := range td.CmdArgs {
					switch cmdArg.Key {
					case "window":
						window, err = time.ParseDuration(cmdArg.Vals[0])
						if err != nil {
							panic(errors.Newf("could not parse window: %s", err))
						}

					case "long-running":
						longRunning, err = time.ParseDuration(cmdArg.Vals[0])
						if err != nil {
							panic(errors.Newf("could not parse long-running: %s", err))
						}

					default:
						panic(errors.Newf("unknown arg %q", cmdArg.Key))
					}
				}

				a := newAggregator(window, longRunning, c.events, c.readAmps)
				windows := a.aggregate()

				var b bytes.Buffer
				for _, w := range windows {
					b.WriteString(w.String())
				}

				return b.String()

			case "reset":
				resetFn()
				return ""

			default:
				return fmt.Sprintf("unknown command %q", td.Cmd)
			}
		})
	})
}

func TestParseInputBytes(t *testing.T) {
	testCases := []struct {
		s    string
		want uint64
	}{
		// Test both 23.1 and current formats.
		{
			"(10 B)",
			10,
		},
		{
			"(10B)",
			10,
		},
		{
			"(10 M)",
			10 << 20,
		},
		{
			"(10MB)",
			10 << 20,
		},
		{
			"(10 M) + (20 M)",
			30 << 20,
		},
		{
			"(10MB) + (20MB)",
			30 << 20,
		},
		{
			"(10 M) + (20 M) + (30 M)",
			60 << 20,
		},
		{
			"(10MB) + (20MB) + (30MB)",
			60 << 20,
		},
		{
			"foo",
			0,
		},
	}
	for _, tc := range testCases {
		t.Run(tc.s, func(t *testing.T) {
			got, err := sumInputBytes(tc.s)
			require.NoError(t, err)
			require.Equal(t, tc.want, got)
		})
	}
}
