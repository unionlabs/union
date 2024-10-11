package sstable

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"math/rand"
	"strconv"
	"testing"

	"github.com/cockroachdb/pebble/bloom"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/stretchr/testify/require"
)

func TestRewriteSuffixProps(t *testing.T) {
	from, to := []byte("_212"), []byte("_646")
	for format := TableFormatPebblev2; format <= TableFormatMax; format++ {
		t.Run(format.String(), func(t *testing.T) {
			wOpts := WriterOptions{
				FilterPolicy: bloom.FilterPolicy(10),
				Comparer:     test4bSuffixComparer,
				TablePropertyCollectors: []func() TablePropertyCollector{
					intSuffixTablePropCollectorFn("ts3", 3), intSuffixTablePropCollectorFn("ts2", 2),
				},
				BlockPropertyCollectors: []func() BlockPropertyCollector{
					keyCountCollectorFn("count"),
					intSuffixIntervalCollectorFn("bp3", 3),
					intSuffixIntervalCollectorFn("bp2", 2),
					intSuffixIntervalCollectorFn("bp1", 1),
				},
				TableFormat: format,
			}
			if format >= TableFormatPebblev4 {
				wOpts.IsStrictObsolete = true
			}

			const keyCount = 1e5
			const rangeKeyCount = 100
			// Setup our test SST.
			sst := make4bSuffixTestSST(t, wOpts, []byte(from), keyCount, rangeKeyCount)

			expectedProps := make(map[string]string)
			expectedProps["ts2.min"] = "46"
			expectedProps["ts2.max"] = "46"
			expectedProps["ts3.min"] = "646"
			expectedProps["ts3.max"] = "646"

			// Also expect to see the aggregated block properties with their updated value
			// at the correct (new) shortIDs. Seeing the rolled up value here is almost an
			// end-to-end test since we only fed them each block during rewrite.
			expectedProps["count"] = string(append([]byte{1}, strconv.Itoa(keyCount+rangeKeyCount)...))
			expectedProps["bp2"] = string(interval{46, 47}.encode([]byte{2}))
			expectedProps["bp3"] = string(interval{646, 647}.encode([]byte{0}))

			// Swap the order of two of the props so they have new shortIDs, and remove
			// one. rwOpts inherits the IsStrictObsolete value from wOpts.
			rwOpts := wOpts
			if rand.Intn(2) != 0 {
				rwOpts.TableFormat = TableFormatPebblev2
				rwOpts.IsStrictObsolete = false
				t.Log("table format set to TableFormatPebblev2")
			}
			fmt.Printf("from format %s, to format %s\n", format.String(), rwOpts.TableFormat.String())
			rwOpts.BlockPropertyCollectors = rwOpts.BlockPropertyCollectors[:3]
			rwOpts.BlockPropertyCollectors[0], rwOpts.BlockPropertyCollectors[1] = rwOpts.BlockPropertyCollectors[1], rwOpts.BlockPropertyCollectors[0]

			// Rewrite the SST using updated options and check the returned props.
			readerOpts := ReaderOptions{
				Comparer: test4bSuffixComparer,
				Filters:  map[string]base.FilterPolicy{wOpts.FilterPolicy.Name(): wOpts.FilterPolicy},
			}
			r, err := NewMemReader(sst, readerOpts)
			require.NoError(t, err)
			defer r.Close()

			var sstBytes [2][]byte
			adjustPropsForEffectiveFormat := func(effectiveFormat TableFormat) {
				if effectiveFormat == TableFormatPebblev4 {
					expectedProps["obsolete-key"] = string([]byte{3})
				} else {
					delete(expectedProps, "obsolete-key")
				}
			}
			for i, byBlocks := range []bool{false, true} {
				t.Run(fmt.Sprintf("byBlocks=%v", byBlocks), func(t *testing.T) {
					rewrittenSST := &memFile{}
					if byBlocks {
						_, rewriteFormat, err := rewriteKeySuffixesInBlocks(
							r, rewrittenSST, rwOpts, from, to, 8)
						// rewriteFormat is equal to the original format, since
						// rwOpts.TableFormat is ignored.
						require.Equal(t, wOpts.TableFormat, rewriteFormat)
						require.NoError(t, err)
						adjustPropsForEffectiveFormat(rewriteFormat)
					} else {
						_, err := RewriteKeySuffixesViaWriter(r, rewrittenSST, rwOpts, from, to)
						require.NoError(t, err)
						adjustPropsForEffectiveFormat(rwOpts.TableFormat)
					}

					sstBytes[i] = rewrittenSST.Data()
					// Check that a reader on the rewritten STT has the expected props.
					rRewritten, err := NewMemReader(rewrittenSST.Data(), readerOpts)
					require.NoError(t, err)
					defer rRewritten.Close()
					require.Equal(t, expectedProps, rRewritten.Properties.UserProperties)
					require.False(t, rRewritten.Properties.IsStrictObsolete)

					// Compare the block level props from the data blocks in the layout,
					// only if we did not do a rewrite from one format to another. If the
					// format changes, the block boundaries change slightly.
					if !byBlocks && wOpts.TableFormat != rwOpts.TableFormat {
						return
					}
					layout, err := r.Layout()
					require.NoError(t, err)
					newLayout, err := rRewritten.Layout()
					require.NoError(t, err)

					ival := interval{}
					for i := range layout.Data {
						oldProps := make([][]byte, len(wOpts.BlockPropertyCollectors))
						oldDecoder := blockPropertiesDecoder{layout.Data[i].Props}
						for !oldDecoder.done() {
							id, val, err := oldDecoder.next()
							require.NoError(t, err)
							oldProps[id] = val
						}
						newProps := make([][]byte, len(rwOpts.BlockPropertyCollectors))
						newDecoder := blockPropertiesDecoder{newLayout.Data[i].Props}
						for !newDecoder.done() {
							id, val, err := newDecoder.next()
							require.NoError(t, err)
							if int(id) < len(newProps) {
								newProps[id] = val
							}
						}
						require.Equal(t, oldProps[0], newProps[1])
						ival.decode(newProps[0])
						require.Equal(t, interval{646, 647}, ival)
						ival.decode(newProps[2])
						require.Equal(t, interval{46, 47}, ival)
					}
				})
			}
			if wOpts.TableFormat == rwOpts.TableFormat {
				// Both methods of rewriting should produce the same result.
				require.Equal(t, sstBytes[0], sstBytes[1])
			}
		})
	}
}

// memFile is a file-like struct that buffers all data written to it in memory.
// Implements the objstorage.Writable interface.
type memFile struct {
	buf bytes.Buffer
}

var _ objstorage.Writable = (*memFile)(nil)

// Finish is part of the objstorage.Writable interface.
func (*memFile) Finish() error {
	return nil
}

// Abort is part of the objstorage.Writable interface.
func (*memFile) Abort() {}

// Write is part of the objstorage.Writable interface.
func (f *memFile) Write(p []byte) error {
	_, err := f.buf.Write(p)
	return err
}

// Data returns the in-memory buffer behind this MemFile.
func (f *memFile) Data() []byte {
	return f.buf.Bytes()
}

func make4bSuffixTestSST(
	t testing.TB, writerOpts WriterOptions, suffix []byte, keys int, rangeKeys int,
) []byte {
	key := make([]byte, 28)
	endKey := make([]byte, 24)
	copy(key[24:], suffix)

	f := &memFile{}
	w := NewWriter(f, writerOpts)
	for i := 0; i < keys; i++ {
		binary.BigEndian.PutUint64(key[:8], 123) // 16-byte shared prefix
		binary.BigEndian.PutUint64(key[8:16], 456)
		binary.BigEndian.PutUint64(key[16:], uint64(i))
		err := w.AddWithForceObsolete(
			base.MakeInternalKey(key, 0, InternalKeyKindSet), key, false)
		if err != nil {
			t.Fatal(err)
		}
	}
	for i := 0; i < rangeKeys; i++ {
		binary.BigEndian.PutUint64(key[:8], 123) // 16-byte shared prefix
		binary.BigEndian.PutUint64(key[8:16], 456)
		binary.BigEndian.PutUint64(key[16:], uint64(i))
		binary.BigEndian.PutUint64(endKey[:8], 123) // 16-byte shared prefix
		binary.BigEndian.PutUint64(endKey[8:16], 456)
		binary.BigEndian.PutUint64(endKey[16:], uint64(i+1))
		if err := w.RangeKeySet(key[:24], endKey[:24], suffix, key); err != nil {
			t.Fatal(err)
		}
	}
	if err := w.Close(); err != nil {
		t.Fatal(err)
	}

	return f.buf.Bytes()
}

func BenchmarkRewriteSST(b *testing.B) {
	from, to := []byte("_123"), []byte("_456")
	writerOpts := WriterOptions{
		FilterPolicy: bloom.FilterPolicy(10),
		Comparer:     test4bSuffixComparer,
		TableFormat:  TableFormatPebblev2,
	}

	sizes := []int{100, 10000, 1e6}
	compressions := []Compression{NoCompression, SnappyCompression}

	files := make([][]*Reader, len(compressions))

	for comp := range compressions {
		files[comp] = make([]*Reader, len(sizes))

		for size := range sizes {
			writerOpts.Compression = compressions[comp]
			sst := make4bSuffixTestSST(b, writerOpts, from, sizes[size], 0 /* rangeKeys */)
			r, err := NewMemReader(sst, ReaderOptions{
				Comparer: test4bSuffixComparer,
				Filters:  map[string]base.FilterPolicy{writerOpts.FilterPolicy.Name(): writerOpts.FilterPolicy},
			})
			if err != nil {
				b.Fatal(err)
			}
			files[comp][size] = r
		}
	}

	b.ResetTimer()
	for comp := range compressions {
		b.Run(compressions[comp].String(), func(b *testing.B) {
			for sz := range sizes {
				r := files[comp][sz]
				b.Run(fmt.Sprintf("keys=%d", sizes[sz]), func(b *testing.B) {
					b.Run("ReaderWriterLoop", func(b *testing.B) {
						b.SetBytes(r.readable.Size())
						for i := 0; i < b.N; i++ {
							if _, err := RewriteKeySuffixesViaWriter(r, &discardFile{}, writerOpts, from, to); err != nil {
								b.Fatal(err)
							}
						}
					})
					for _, concurrency := range []int{1, 2, 4, 8, 16} {
						b.Run(fmt.Sprintf("RewriteKeySuffixes,concurrency=%d", concurrency), func(b *testing.B) {
							b.SetBytes(r.readable.Size())
							for i := 0; i < b.N; i++ {
								if _, _, err := rewriteKeySuffixesInBlocks(r, &discardFile{}, writerOpts, []byte("_123"), []byte("_456"), concurrency); err != nil {
									b.Fatal(err)
								}
							}
						})
					}
				})
			}
		})
	}
}
