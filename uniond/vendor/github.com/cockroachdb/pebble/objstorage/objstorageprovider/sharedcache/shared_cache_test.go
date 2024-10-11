package sharedcache_test

import (
	"bytes"
	"context"
	"fmt"
	"strconv"
	"sync"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/sharedcache"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/stretchr/testify/require"
	"golang.org/x/exp/rand"
)

func TestSharedCache(t *testing.T) {
	ctx := context.Background()

	datadriven.Walk(t, "testdata/cache", func(t *testing.T, path string) {
		var log base.InMemLogger
		fs := vfs.WithLogging(vfs.NewMem(), func(fmt string, args ...interface{}) {
			log.Infof("<local fs> "+fmt, args...)
		})

		provider, err := objstorageprovider.Open(objstorageprovider.DefaultSettings(fs, ""))
		require.NoError(t, err)

		var cache *sharedcache.Cache
		defer func() {
			if cache != nil {
				cache.Close()
			}
		}()

		var objData []byte
		datadriven.RunTest(t, path, func(t *testing.T, d *datadriven.TestData) string {
			log.Reset()
			switch d.Cmd {
			case "init":
				blockSize := parseBytesArg(t, d, "block-size", 32*1024)
				shardingBlockSize := parseBytesArg(t, d, "sharding-block-size", 1024*1024)
				numShards := parseBytesArg(t, d, "num-shards", 32)
				size := parseBytesArg(t, d, "size", numShards*shardingBlockSize)
				if size%(numShards*shardingBlockSize) != 0 {
					d.Fatalf(t, "size (%d) must be a multiple of numShards (%d) * shardingBlockSize(%d)",
						size, numShards, shardingBlockSize,
					)
				}
				cache, err = sharedcache.Open(
					fs, base.DefaultLogger, "", blockSize, int64(shardingBlockSize), int64(size), numShards,
				)
				require.NoError(t, err)
				return fmt.Sprintf("initialized with block-size=%d size=%d num-shards=%d", blockSize, size, numShards)

			case "write":
				size := mustParseBytesArg(t, d, "size")

				writable, _, err := provider.Create(ctx, base.FileTypeTable, base.FileNum(1).DiskFileNum(), objstorage.CreateOptions{})
				require.NoError(t, err)
				defer writable.Finish()

				// With invariants on, Write will modify its input buffer.
				objData = make([]byte, size)
				wrote := make([]byte, size)
				for i := 0; i < size; i++ {
					objData[i] = byte(i)
					wrote[i] = byte(i)
				}
				err = writable.Write(wrote)
				// Writing a file is test setup, and it always is expected to succeed, so we assert
				// within the test, rather than returning n and/or err here.
				require.NoError(t, err)

				return ""
			case "read", "read-for-compaction":
				missesBefore := cache.Metrics().ReadsWithPartialHit + cache.Metrics().ReadsWithNoHit
				offset := mustParseBytesArg(t, d, "offset")
				size := mustParseBytesArg(t, d, "size")

				readable, err := provider.OpenForReading(ctx, base.FileTypeTable, base.FileNum(1).DiskFileNum(), objstorage.OpenOptions{})
				require.NoError(t, err)
				defer readable.Close()

				got := make([]byte, size)
				flags := sharedcache.ReadFlags{
					ReadOnly: d.Cmd == "read-for-compaction",
				}
				err = cache.ReadAt(ctx, base.FileNum(1).DiskFileNum(), got, int64(offset), readable, readable.Size(), flags)
				// We always expect cache.ReadAt to succeed.
				require.NoError(t, err)
				// It is easier to assert this condition programmatically, rather than returning
				// got, which may be very large.
				require.True(t, bytes.Equal(objData[int(offset):int(offset)+size], got), "incorrect data returned")

				// In order to ensure we get a hit on the next read, we must wait for writing to
				// the cache to complete.
				cache.WaitForWritesToComplete()

				// TODO(josh): Not tracing out filesystem activity here, since logging_fs.go
				// doesn't trace calls to ReadAt or WriteAt. We should consider changing this.
				missesAfter := cache.Metrics().ReadsWithPartialHit + cache.Metrics().ReadsWithNoHit
				return fmt.Sprintf("misses=%d", missesAfter-missesBefore)
			default:
				d.Fatalf(t, "unknown command %s", d.Cmd)
				return ""
			}
		})
	})
}

func TestSharedCacheRandomized(t *testing.T) {
	ctx := context.Background()

	var log base.InMemLogger
	fs := vfs.WithLogging(vfs.NewMem(), func(fmt string, args ...interface{}) {
		log.Infof("<local fs> "+fmt, args...)
	})

	provider, err := objstorageprovider.Open(objstorageprovider.DefaultSettings(fs, ""))
	require.NoError(t, err)

	seed := uint64(time.Now().UnixNano())
	fmt.Printf("seed: %v\n", seed)
	rand.Seed(seed)

	helper := func(
		blockSize int,
		shardingBlockSize int64) func(t *testing.T) {
		return func(t *testing.T) {
			for _, concurrentReads := range []bool{false, true} {
				t.Run(fmt.Sprintf("concurrentReads=%v", concurrentReads), func(t *testing.T) {
					maxShards := 32
					if invariants.RaceEnabled {
						maxShards = 8
					}
					numShards := rand.Intn(maxShards) + 1
					cacheSize := shardingBlockSize * int64(numShards) // minimum allowed cache size

					cache, err := sharedcache.Open(fs, base.DefaultLogger, "", blockSize, shardingBlockSize, cacheSize, numShards)
					require.NoError(t, err)
					defer cache.Close()

					writable, _, err := provider.Create(ctx, base.FileTypeTable, base.FileNum(1).DiskFileNum(), objstorage.CreateOptions{})
					require.NoError(t, err)

					// With invariants on, Write will modify its input buffer.
					// If size == 0, we can see panics below, so force a nonzero size.
					size := rand.Int63n(cacheSize-1) + 1
					objData := make([]byte, size)
					wrote := make([]byte, size)
					for i := 0; i < int(size); i++ {
						objData[i] = byte(i)
						wrote[i] = byte(i)
					}

					require.NoError(t, writable.Write(wrote))
					require.NoError(t, writable.Finish())

					readable, err := provider.OpenForReading(ctx, base.FileTypeTable, base.FileNum(1).DiskFileNum(), objstorage.OpenOptions{})
					require.NoError(t, err)
					defer readable.Close()

					const numDistinctReads = 100
					wg := sync.WaitGroup{}
					for i := 0; i < numDistinctReads; i++ {
						wg.Add(1)
						go func() {
							defer wg.Done()
							offset := rand.Int63n(size)

							got := make([]byte, size-offset)
							err := cache.ReadAt(ctx, base.FileNum(1).DiskFileNum(), got, offset, readable, readable.Size(), sharedcache.ReadFlags{})
							require.NoError(t, err)
							require.Equal(t, objData[int(offset):], got)

							got = make([]byte, size-offset)
							err = cache.ReadAt(ctx, base.FileNum(1).DiskFileNum(), got, offset, readable, readable.Size(), sharedcache.ReadFlags{})
							require.NoError(t, err)
							require.Equal(t, objData[int(offset):], got)
						}()
						// If concurrent reads, only wait 50% of loop iterations on average.
						if concurrentReads && rand.Intn(2) == 0 {
							wg.Wait()
						}
						if !concurrentReads {
							wg.Wait()
						}
					}
					wg.Wait()
				})
			}
		}
	}
	t.Run("32 KB block size", helper(32*1024, 1024*1024))
	t.Run("1 MB block size", helper(1024*1024, 1024*1024))

	if !invariants.RaceEnabled {
		for i := 0; i < 5; i++ {
			exp := rand.Intn(11) + 10   // [10, 20]
			randomBlockSize := 1 << exp // [1 KB, 1 MB]

			factor := rand.Intn(4) + 1                                 // [1, 4]
			randomShardingBlockSize := int64(randomBlockSize * factor) // [1 KB, 4 MB]

			t.Run("random block and sharding block size", helper(randomBlockSize, randomShardingBlockSize))
		}
	}
}

// parseBytesArg parses an optional argument that specifies a byte size; if the
// argument is not specified the default value is used. K/M/G suffixes are
// supported.
func parseBytesArg(t testing.TB, d *datadriven.TestData, argName string, defaultValue int) int {
	res, ok := tryParseBytesArg(t, d, argName)
	if !ok {
		return defaultValue
	}
	return res
}

// parseBytesArg parses a mandatory argument that specifies a byte size; K/M/G
// suffixes are supported.
func mustParseBytesArg(t testing.TB, d *datadriven.TestData, argName string) int {
	res, ok := tryParseBytesArg(t, d, argName)
	if !ok {
		t.Fatalf("argument '%s' missing", argName)
	}
	return res
}

func tryParseBytesArg(t testing.TB, d *datadriven.TestData, argName string) (val int, ok bool) {
	arg, ok := d.Arg(argName)
	if !ok {
		return 0, false
	}
	if len(arg.Vals) != 1 {
		t.Fatalf("expected 1 value for '%s'", argName)
	}
	v := arg.Vals[0]
	factor := 1
	switch v[len(v)-1] {
	case 'k', 'K':
		factor = 1024
	case 'm', 'M':
		factor = 1024 * 1024
	case 'g', 'G':
		factor = 1024 * 1024 * 1024
	}
	if factor > 1 {
		v = v[:len(v)-1]
	}
	res, err := strconv.Atoi(v)
	if err != nil {
		t.Fatalf("could not parse value '%s' for '%s'", arg.Vals[0], argName)
	}

	return res * factor, true
}
