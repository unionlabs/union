// Copyright 2022 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
	"fmt"
	"math"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/cache"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
	"golang.org/x/exp/rand"
)

func TestExternalIterator(t *testing.T) {
	mem := vfs.NewMem()
	o := &Options{
		FS:                 mem,
		Comparer:           testkeys.Comparer,
		FormatMajorVersion: FormatRangeKeys,
	}
	o.EnsureDefaults()
	d, err := Open("", o)
	require.NoError(t, err)
	defer func() { require.NoError(t, d.Close()) }()

	datadriven.RunTest(t, "testdata/external_iterator", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "reset":
			mem = vfs.NewMem()
			return ""
		case "build":
			if err := runBuildCmd(td, d, mem); err != nil {
				return err.Error()
			}
			return ""
		case "iter":
			opts := IterOptions{KeyTypes: IterKeyTypePointsAndRanges}
			var externalIterOpts []ExternalIterOption
			var files [][]sstable.ReadableFile
			for _, arg := range td.CmdArgs {
				switch arg.Key {
				case "fwd-only":
					externalIterOpts = append(externalIterOpts, ExternalIterForwardOnly{})
				case "mask-suffix":
					opts.RangeKeyMasking.Suffix = []byte(arg.Vals[0])
				case "lower":
					opts.LowerBound = []byte(arg.Vals[0])
				case "upper":
					opts.UpperBound = []byte(arg.Vals[0])
				case "files":
					for _, v := range arg.Vals {
						f, err := mem.Open(v)
						require.NoError(t, err)
						files = append(files, []sstable.ReadableFile{f})
					}
				}
			}
			it, err := NewExternalIter(o, &opts, files, externalIterOpts...)
			require.NoError(t, err)
			return runIterCmd(td, it, true /* close iter */)
		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func TestSimpleLevelIter(t *testing.T) {
	mem := vfs.NewMem()
	o := &Options{
		FS:                 mem,
		Comparer:           testkeys.Comparer,
		FormatMajorVersion: FormatRangeKeys,
	}
	o.EnsureDefaults()
	d, err := Open("", o)
	require.NoError(t, err)
	defer func() { require.NoError(t, d.Close()) }()

	datadriven.RunTest(t, "testdata/simple_level_iter", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "reset":
			mem = vfs.NewMem()
			return ""
		case "build":
			if err := runBuildCmd(td, d, mem); err != nil {
				return err.Error()
			}
			return ""
		case "iter":
			var files []sstable.ReadableFile
			var filenames []string
			td.ScanArgs(t, "files", &filenames)
			for _, name := range filenames {
				f, err := mem.Open(name)
				require.NoError(t, err)
				files = append(files, f)
			}
			readers, err := openExternalTables(o, files, 0, o.MakeReaderOptions())
			require.NoError(t, err)
			defer func() {
				for i := range readers {
					_ = readers[i].Close()
				}
			}()
			var internalIters []internalIterator
			for i := range readers {
				iter, err := readers[i].NewIter(nil, nil)
				require.NoError(t, err)
				internalIters = append(internalIters, iter)
			}
			it := &simpleLevelIter{cmp: o.Comparer.Compare, iters: internalIters}
			it.init(IterOptions{})

			response := runInternalIterCmd(t, td, it)
			require.NoError(t, it.Close())
			return response
		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func TestSimpleIterError(t *testing.T) {
	s := simpleLevelIter{cmp: DefaultComparer.Compare, iters: []internalIterator{&errorIter{err: errors.New("injected")}}}
	s.init(IterOptions{})
	defer s.Close()

	iterKey, _ := s.First()
	require.Nil(t, iterKey)
	require.Error(t, s.Error())
}

func TestIterRandomizedMaybeFilteredKeys(t *testing.T) {
	mem := vfs.NewMem()

	seed := *seed
	if seed == 0 {
		seed = uint64(time.Now().UnixNano())
		t.Logf("seed: %d", seed)
	}
	rng := rand.New(rand.NewSource(seed))
	numKeys := 100 + rng.Intn(5000)
	// The block property filter will exclude keys with suffixes [0, tsSeparator-1].
	// We use the first "part" of the keyspace below to write keys >= tsSeparator,
	// and the second part to write keys < tsSeparator. Successive parts (if any)
	// will contain keys at random before or after the separator.
	tsSeparator := 10 + rng.Int63n(5000)
	const keyLen = 5

	// We split the keyspace into logical "parts" which are disjoint slices of the
	// keyspace. That is, the keyspace a-z could be comprised of parts {a-k, l-z}.
	// We rely on this partitioning when generating timestamps to give us some
	// predictable clustering of timestamps in sstable blocks, however it is not
	// strictly necessary for this test.
	alpha := testkeys.Alpha(keyLen)
	numParts := rng.Intn(3) + 2
	blockSize := 16 + rng.Intn(64)

	c := cache.New(128 << 20)
	defer c.Unref()

	for fileIdx, twoLevelIndex := range []bool{false, true} {
		t.Run(fmt.Sprintf("twoLevelIndex=%v", twoLevelIndex), func(t *testing.T) {
			keys := make([][]byte, 0, numKeys)

			filename := fmt.Sprintf("test-%d", fileIdx)
			f0, err := mem.Create(filename)
			require.NoError(t, err)

			indexBlockSize := 4096
			if twoLevelIndex {
				indexBlockSize = 1
			}
			w := sstable.NewWriter(objstorageprovider.NewFileWritable(f0), sstable.WriterOptions{
				BlockSize:      blockSize,
				Comparer:       testkeys.Comparer,
				IndexBlockSize: indexBlockSize,
				TableFormat:    sstable.TableFormatPebblev2,
				BlockPropertyCollectors: []func() BlockPropertyCollector{
					func() BlockPropertyCollector {
						return sstable.NewTestKeysBlockPropertyCollector()
					},
				},
			})
			buf := make([]byte, alpha.MaxLen()+testkeys.MaxSuffixLen)
			valBuf := make([]byte, 20)
			keyIdx := int64(0)
			for i := 0; i < numParts; i++ {
				// The first two parts of the keyspace are special. The first one has
				// all keys with timestamps greater than tsSeparator, while the second
				// one has all keys with timestamps less than tsSeparator. Any additional
				// keys could have timestamps at random before or after the tsSeparator.
				maxKeysPerPart := numKeys / numParts
				for j := 0; j < maxKeysPerPart; j++ {
					var ts int64
					if i == 0 {
						ts = rng.Int63n(5000) + tsSeparator
					} else if i == 1 {
						ts = rng.Int63n(tsSeparator)
					} else {
						ts = rng.Int63n(tsSeparator + 5000)
					}
					n := testkeys.WriteKeyAt(buf, alpha, keyIdx*alpha.Count()/int64(numKeys), ts)
					keys = append(keys, append([]byte(nil), buf[:n]...))
					randStr(valBuf, rng)
					require.NoError(t, w.Set(buf[:n], valBuf))
					keyIdx++
				}
			}
			require.NoError(t, w.Close())

			// Re-open that filename for reading.
			f1, err := mem.Open(filename)
			require.NoError(t, err)

			readable, err := sstable.NewSimpleReadable(f1)
			require.NoError(t, err)

			r, err := sstable.NewReader(readable, sstable.ReaderOptions{
				Cache:    c,
				Comparer: testkeys.Comparer,
			})
			require.NoError(t, err)
			defer r.Close()

			filter := sstable.NewTestKeysBlockPropertyFilter(uint64(tsSeparator), math.MaxUint64)
			filterer, err := sstable.IntersectsTable([]BlockPropertyFilter{filter}, nil, r.Properties.UserProperties)
			require.NoError(t, err)
			require.NotNil(t, filterer)

			var iter sstable.Iterator
			iter, err = r.NewIterWithBlockPropertyFilters(
				nil, nil, filterer, false /* useFilterBlock */, nil, /* stats */
				sstable.TrivialReaderProvider{Reader: r})
			require.NoError(t, err)
			defer iter.Close()
			var lastSeekKey, lowerBound, upperBound []byte
			narrowBoundsMode := false

			for i := 0; i < 10000; i++ {
				if rng.Intn(8) == 0 {
					// Toggle narrow bounds mode.
					if narrowBoundsMode {
						// Reset bounds.
						lowerBound, upperBound = nil, nil
						iter.SetBounds(nil /* lower */, nil /* upper */)
					}
					narrowBoundsMode = !narrowBoundsMode
				}
				keyIdx := rng.Intn(len(keys))
				seekKey := keys[keyIdx]
				if narrowBoundsMode {
					// Case 1: We just entered narrow bounds mode, and both bounds
					// are nil. Set a lower/upper bound.
					//
					// Case 2: The seek key is outside our last bounds.
					//
					// In either case, pick a narrow range of keys to set bounds on,
					// let's say keys[keyIdx-5] and keys[keyIdx+5], before doing our
					// seek operation. Picking narrow bounds increases the chance of
					// monotonic bound changes.
					cmp := testkeys.Comparer.Compare
					case1 := lowerBound == nil && upperBound == nil
					case2 := (lowerBound != nil && cmp(lowerBound, seekKey) > 0) || (upperBound != nil && cmp(upperBound, seekKey) <= 0)
					if case1 || case2 {
						lowerBound = nil
						if keyIdx-5 >= 0 {
							lowerBound = keys[keyIdx-5]
						}
						upperBound = nil
						if keyIdx+5 < len(keys) {
							upperBound = keys[keyIdx+5]
						}
						iter.SetBounds(lowerBound, upperBound)
					}
					// Case 3: The current seek key is within the previously-set bounds.
					// No need to change bounds.
				}
				flags := base.SeekGEFlagsNone
				if lastSeekKey != nil && bytes.Compare(seekKey, lastSeekKey) > 0 {
					flags = flags.EnableTrySeekUsingNext()
				}
				lastSeekKey = append(lastSeekKey[:0], seekKey...)

				newKey, _ := iter.SeekGE(seekKey, flags)
				if newKey == nil || !bytes.Equal(newKey.UserKey, seekKey) {
					// We skipped some keys. Check if maybeFilteredKeys is true.
					formattedNewKey := "<nil>"
					if newKey != nil {
						formattedNewKey = fmt.Sprintf("%s", testkeys.Comparer.FormatKey(newKey.UserKey))
					}
					require.True(t, iter.MaybeFilteredKeys(), "seeked for key = %s, got key = %s indicating block property filtering but MaybeFilteredKeys = false", testkeys.Comparer.FormatKey(seekKey), formattedNewKey)
				}
			}
		})
	}
}

func BenchmarkExternalIter_NonOverlapping_SeekNextScan(b *testing.B) {
	ks := testkeys.Alpha(6)
	opts := (&Options{}).EnsureDefaults()
	iterOpts := &IterOptions{
		KeyTypes: IterKeyTypePointsAndRanges,
	}
	writeOpts := opts.MakeWriterOptions(6, sstable.TableFormatPebblev2)

	for _, keyCount := range []int{100, 10_000, 100_000} {
		b.Run(fmt.Sprintf("keys=%d", keyCount), func(b *testing.B) {
			for _, fileCount := range []int{1, 10, 100} {
				b.Run(fmt.Sprintf("files=%d", fileCount), func(b *testing.B) {
					var fs vfs.FS = vfs.NewMem()
					filenames := make([]string, fileCount)
					var keys [][]byte
					for i := 0; i < fileCount; i++ {
						filename := fmt.Sprintf("%03d.sst", i)
						wf, err := fs.Create(filename)
						require.NoError(b, err)
						w := sstable.NewWriter(objstorageprovider.NewFileWritable(wf), writeOpts)
						for j := 0; j < keyCount/fileCount; j++ {
							key := testkeys.Key(ks, int64(len(keys)))
							keys = append(keys, key)
							require.NoError(b, w.Set(key, key))
						}
						require.NoError(b, w.Close())
						filenames[i] = filename
					}

					for _, forwardOnly := range []bool{false, true} {
						b.Run(fmt.Sprintf("forward-only=%t", forwardOnly), func(b *testing.B) {
							var externalIterOpts []ExternalIterOption
							if forwardOnly {
								externalIterOpts = append(externalIterOpts, ExternalIterForwardOnly{})
							}

							for i := 0; i < b.N; i++ {
								func() {
									files := make([][]sstable.ReadableFile, fileCount)
									for i := 0; i < fileCount; i++ {
										f, err := fs.Open(filenames[i])
										require.NoError(b, err)
										files[i] = []sstable.ReadableFile{f}
									}

									it, err := NewExternalIter(opts, iterOpts, files, externalIterOpts...)
									require.NoError(b, err)
									defer it.Close()

									for k := 0; k+1 < len(keys); k += 2 {
										if !it.SeekGE(keys[k]) {
											b.Fatalf("key %q not found", keys[k])
										}
										if !it.Next() {
											b.Fatalf("key %q not found", keys[k+1])
										}
										if !bytes.Equal(it.Key(), keys[k+1]) {
											b.Fatalf("expected key %q, found %q", keys[k+1], it.Key())
										}
									}
								}()
							}
						})
					}
				})
			}
		})
	}
}
