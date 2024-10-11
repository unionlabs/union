// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package pebble

import (
	"bytes"
	"fmt"
	"reflect"
	"runtime"
	"strings"
	"sync"
	"testing"
	"time"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
	"github.com/cockroachdb/redact"
	"github.com/stretchr/testify/require"
)

// Verify event listener actions, as well as expected filesystem operations.
func TestEventListener(t *testing.T) {
	if runtime.GOARCH == "386" {
		t.Skip("skipped on 32-bit due to slightly varied output")
	}
	var d *DB
	var memLog base.InMemLogger
	mem := vfs.NewMem()
	require.NoError(t, mem.MkdirAll("ext", 0755))

	datadriven.RunTest(t, "testdata/event_listener", func(t *testing.T, td *datadriven.TestData) string {
		switch td.Cmd {
		case "open":
			memLog.Reset()
			lel := MakeLoggingEventListener(&memLog)
			flushBegin, flushEnd := lel.FlushBegin, lel.FlushEnd
			lel.FlushBegin = func(info FlushInfo) {
				// Make deterministic.
				info.InputBytes = 100
				flushBegin(info)
			}
			lel.FlushEnd = func(info FlushInfo) {
				// Make deterministic.
				info.InputBytes = 100
				flushEnd(info)
			}
			opts := &Options{
				FS:                    vfs.WithLogging(mem, memLog.Infof),
				FormatMajorVersion:    internalFormatNewest,
				EventListener:         &lel,
				MaxManifestFileSize:   1,
				L0CompactionThreshold: 10,
				WALDir:                "wal",
			}
			// The table stats collector runs asynchronously and its
			// timing is less predictable. It increments nextJobID, which
			// can make these tests flaky. The TableStatsLoaded event is
			// tested separately in TestTableStats.
			opts.private.disableTableStats = true
			var err error
			d, err = Open("db", opts)
			if err != nil {
				return err.Error()
			}
			t := time.Now()
			d.timeNow = func() time.Time {
				t = t.Add(time.Second)
				return t
			}
			d.opts.private.testingAlwaysWaitForCleanup = true
			return memLog.String()

		case "close":
			memLog.Reset()
			if err := d.Close(); err != nil {
				return err.Error()
			}
			return memLog.String()

		case "flush":
			memLog.Reset()
			if err := d.Set([]byte("a"), nil, nil); err != nil {
				return err.Error()
			}
			if err := d.Flush(); err != nil {
				return err.Error()
			}
			return memLog.String()

		case "compact":
			memLog.Reset()
			if err := d.Set([]byte("a"), nil, nil); err != nil {
				return err.Error()
			}
			if err := d.Compact([]byte("a"), []byte("b"), false); err != nil {
				return err.Error()
			}
			return memLog.String()

		case "checkpoint":
			memLog.Reset()
			if err := d.Checkpoint("checkpoint"); err != nil {
				return err.Error()
			}
			return memLog.String()

		case "disable-file-deletions":
			memLog.Reset()
			d.mu.Lock()
			d.disableFileDeletions()
			d.mu.Unlock()
			return memLog.String()

		case "enable-file-deletions":
			memLog.Reset()
			func() {
				defer func() {
					if r := recover(); r != nil {
						memLog.Infof("%v", r)
					}
				}()
				d.mu.Lock()
				defer d.mu.Unlock()
				d.enableFileDeletions()
			}()
			d.TestOnlyWaitForCleaning()
			return memLog.String()

		case "ingest":
			memLog.Reset()
			f, err := mem.Create("ext/0")
			if err != nil {
				return err.Error()
			}
			w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{
				TableFormat: d.FormatMajorVersion().MaxTableFormat(),
			})
			if err := w.Add(base.MakeInternalKey([]byte("a"), 0, InternalKeyKindSet), nil); err != nil {
				return err.Error()
			}
			if err := w.Close(); err != nil {
				return err.Error()
			}
			if err := d.Ingest([]string{"ext/0"}); err != nil {
				return err.Error()
			}
			return memLog.String()

		case "ingest-flushable":
			memLog.Reset()

			// Prevent flushes during this test to ensure determinism.
			d.mu.Lock()
			d.mu.compact.flushing = true
			d.mu.Unlock()

			b := d.NewBatch()
			if err := b.Set([]byte("a"), nil, nil); err != nil {
				return err.Error()
			}
			if err := d.Apply(b, nil); err != nil {
				return err.Error()
			}
			writeTable := func(name string, key byte) error {
				f, err := mem.Create(name)
				if err != nil {
					return err
				}
				w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{
					TableFormat: d.FormatMajorVersion().MaxTableFormat(),
				})
				if err := w.Add(base.MakeInternalKey([]byte{key}, 0, InternalKeyKindSet), nil); err != nil {
					return err
				}
				if err := w.Close(); err != nil {
					return err
				}
				return nil
			}
			tableA, tableB := "ext/a", "ext/b"
			if err := writeTable(tableA, 'a'); err != nil {
				return err.Error()
			}
			if err := writeTable(tableB, 'b'); err != nil {
				return err.Error()
			}
			if err := d.Ingest([]string{tableA, tableB}); err != nil {
				return err.Error()
			}

			// Re-enable flushes, to allow the subsequent flush to proceed.
			d.mu.Lock()
			d.mu.compact.flushing = false
			d.mu.Unlock()
			if err := d.Flush(); err != nil {
				return err.Error()
			}
			return memLog.String()

		case "metrics":
			// The asynchronous loading of table stats can change metrics, so
			// wait for all the tables' stats to be loaded.
			d.mu.Lock()
			d.waitTableStats()
			d.mu.Unlock()

			return d.Metrics().StringForTests()

		case "sstables":
			var buf bytes.Buffer
			tableInfos, _ := d.SSTables()
			for i, level := range tableInfos {
				if len(level) == 0 {
					continue
				}
				fmt.Fprintf(&buf, "%d:\n", i)
				for _, m := range level {
					fmt.Fprintf(&buf, "  %d:[%s-%s]\n",
						m.FileNum, m.Smallest.UserKey, m.Largest.UserKey)
				}
			}
			return buf.String()

		default:
			return fmt.Sprintf("unknown command: %s", td.Cmd)
		}
	})
}

func TestWriteStallEvents(t *testing.T) {
	const flushCount = 10
	const writeStallEnd = "write stall ending"

	testCases := []struct {
		delayFlush bool
		expected   string
	}{
		{true, "memtable count limit reached"},
		{false, "L0 file count limit exceeded"},
	}

	for _, c := range testCases {
		t.Run("", func(t *testing.T) {
			stallEnded := make(chan struct{}, 1)
			createReleased := make(chan struct{}, flushCount)
			var log base.InMemLogger
			var delayOnce sync.Once
			listener := &EventListener{
				TableCreated: func(info TableCreateInfo) {
					if c.delayFlush == (info.Reason == "flushing") {
						delayOnce.Do(func() {
							<-createReleased
						})
					}
				},
				WriteStallBegin: func(info WriteStallBeginInfo) {
					log.Infof("%s", info.String())
					createReleased <- struct{}{}
				},
				WriteStallEnd: func() {
					log.Infof("%s", writeStallEnd)
					select {
					case stallEnded <- struct{}{}:
					default:
					}
				},
			}
			d, err := Open("db", &Options{
				EventListener:               listener,
				FS:                          vfs.NewMem(),
				MemTableSize:                initialMemTableSize,
				MemTableStopWritesThreshold: 2,
				L0CompactionThreshold:       2,
				L0StopWritesThreshold:       2,
			})
			require.NoError(t, err)
			defer d.Close()

			for i := 0; i < flushCount; i++ {
				require.NoError(t, d.Set([]byte("a"), nil, NoSync))

				ch, err := d.AsyncFlush()
				require.NoError(t, err)

				// If we're delaying the flush (because we're testing for memtable
				// write stalls), we can't wait for the flush to finish as doing so
				// would deadlock. If we're not delaying the flush (because we're
				// testing for L0 write stals), we wait for the flush to finish so we
				// don't create too many memtables which would trigger a memtable write
				// stall.
				if !c.delayFlush {
					<-ch
				}
				if strings.Contains(log.String(), c.expected) {
					break
				}
			}
			<-stallEnded

			events := log.String()
			require.Contains(t, events, c.expected)
			require.Contains(t, events, writeStallEnd)
			if testing.Verbose() {
				t.Logf("\n%s", events)
			}
		})
	}
}

type redactLogger struct {
	logger Logger
}

// Infof implements the Logger.Infof interface.
func (l redactLogger) Infof(format string, args ...interface{}) {
	l.logger.Infof("%s", redact.Sprintf(format, args...).Redact())
}

// Fatalf implements the Logger.Fatalf interface.
func (l redactLogger) Fatalf(format string, args ...interface{}) {
	l.logger.Fatalf("%s", redact.Sprintf(format, args...).Redact())
}

func TestEventListenerRedact(t *testing.T) {
	// The vast majority of event listener fields logged are safe and do not
	// need to be redacted. Verify that the rare, unsafe error does appear in
	// the log redacted.
	var log base.InMemLogger
	l := MakeLoggingEventListener(redactLogger{logger: &log})
	l.WALDeleted(WALDeleteInfo{
		JobID:   5,
		FileNum: FileNum(20),
		Err:     errors.Errorf("unredacted error: %s", "unredacted string"),
	})
	require.Equal(t, "[JOB 5] WAL delete error: unredacted error: ‹×›\n", log.String())
}

func TestEventListenerEnsureDefaultsBackgroundError(t *testing.T) {
	e := EventListener{}
	e.EnsureDefaults(nil)
	e.BackgroundError(errors.New("an example error"))
}

func TestEventListenerEnsureDefaultsSetsAllCallbacks(t *testing.T) {
	e := EventListener{}
	e.EnsureDefaults(nil)
	testAllCallbacksSetInEventListener(t, e)
}

func TestMakeLoggingEventListenerSetsAllCallbacks(t *testing.T) {
	e := MakeLoggingEventListener(nil)
	testAllCallbacksSetInEventListener(t, e)
}

func TestTeeEventListenerSetsAllCallbacks(t *testing.T) {
	e := TeeEventListener(EventListener{}, EventListener{})
	testAllCallbacksSetInEventListener(t, e)
}

func testAllCallbacksSetInEventListener(t *testing.T, e EventListener) {
	t.Helper()
	v := reflect.ValueOf(e)
	for i := 0; i < v.NumField(); i++ {
		fType := v.Type().Field(i)
		fVal := v.Field(i)
		require.Equal(t, reflect.Func, fType.Type.Kind(), "unexpected non-func field: %s", fType.Name)
		require.False(t, fVal.IsNil(), "unexpected nil field: %s", fType.Name)
	}
}
