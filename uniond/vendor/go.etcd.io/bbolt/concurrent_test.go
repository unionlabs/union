package bbolt_test

import (
	"bytes"
	crand "crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"io"
	mrand "math/rand"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync"
	"testing"
	"time"
	"unicode/utf8"

	"github.com/stretchr/testify/require"
	"golang.org/x/sync/errgroup"

	bolt "go.etcd.io/bbolt"
)

const (
	bucketPrefix = "bucket"
	keyPrefix    = "key"
	noopTxKey    = "%magic-no-op-key%"

	// TestConcurrentCaseDuration is used as a env variable to specify the
	// concurrent test duration.
	testConcurrentCaseDuration    = "TEST_CONCURRENT_CASE_DURATION"
	defaultConcurrentTestDuration = 30 * time.Second
)

type duration struct {
	min time.Duration
	max time.Duration
}

type bytesRange struct {
	min int
	max int
}

type operationChance struct {
	operation OperationType
	chance    int
}

type concurrentConfig struct {
	bucketCount    int
	keyCount       int
	workInterval   duration
	operationRatio []operationChance
	readInterval   duration   // only used by readOperation
	noopWriteRatio int        // only used by writeOperation
	writeBytes     bytesRange // only used by writeOperation
}

/*
TestConcurrentGenericReadAndWrite verifies:
 1. Repeatable read: a read transaction should always see the same data
    view during its lifecycle.
 2. Any data written by a writing transaction should be visible to any
    following reading transactions (with txid >= previous writing txid).
 3. The txid should never decrease.
*/
func TestConcurrentGenericReadAndWrite(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping test in short mode.")
	}

	testDuration := concurrentTestDuration(t)
	conf := concurrentConfig{
		bucketCount:  5,
		keyCount:     10000,
		workInterval: duration{},
		operationRatio: []operationChance{
			{operation: Read, chance: 60},
			{operation: Write, chance: 20},
			{operation: Delete, chance: 20},
		},
		readInterval: duration{
			min: 50 * time.Millisecond,
			max: 100 * time.Millisecond,
		},
		noopWriteRatio: 20,
		writeBytes: bytesRange{
			min: 200,
			max: 16000,
		},
	}

	testCases := []struct {
		name         string
		workerCount  int
		conf         concurrentConfig
		testDuration time.Duration
	}{
		{
			name:         "1 worker",
			workerCount:  1,
			conf:         conf,
			testDuration: testDuration,
		},
		{
			name:         "10 workers",
			workerCount:  10,
			conf:         conf,
			testDuration: testDuration,
		},
		{
			name:         "50 workers",
			workerCount:  50,
			conf:         conf,
			testDuration: testDuration,
		},
		{
			name:         "100 workers",
			workerCount:  100,
			conf:         conf,
			testDuration: testDuration,
		},
		{
			name:         "200 workers",
			workerCount:  200,
			conf:         conf,
			testDuration: testDuration,
		},
	}

	for _, tc := range testCases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			concurrentReadAndWrite(t,
				tc.workerCount,
				tc.conf,
				tc.testDuration)
		})
	}
}

func concurrentTestDuration(t *testing.T) time.Duration {
	durationInEnv := strings.ToLower(os.Getenv(testConcurrentCaseDuration))
	if durationInEnv == "" {
		t.Logf("%q not set, defaults to %s", testConcurrentCaseDuration, defaultConcurrentTestDuration)
		return defaultConcurrentTestDuration
	}

	d, err := time.ParseDuration(durationInEnv)
	if err != nil {
		t.Logf("Failed to parse %s=%s, error: %v, defaults to %s", testConcurrentCaseDuration, durationInEnv, err, defaultConcurrentTestDuration)
		return defaultConcurrentTestDuration
	}

	t.Logf("Concurrent test duration set by %s=%s", testConcurrentCaseDuration, d)
	return d
}

func concurrentReadAndWrite(t *testing.T,
	workerCount int,
	conf concurrentConfig,
	testDuration time.Duration) {

	t.Log("Preparing db.")
	db := mustCreateDB(t, nil)
	defer db.Close()
	err := db.Update(func(tx *bolt.Tx) error {
		for i := 0; i < conf.bucketCount; i++ {
			if _, err := tx.CreateBucketIfNotExists(bucketName(i)); err != nil {
				return err
			}
		}
		return nil
	})
	require.NoError(t, err)

	var records historyRecords
	// t.Failed() returns false during panicking. We need to forcibly
	// save data on panicking.
	// Refer to: https://github.com/golang/go/issues/49929
	panicked := true
	defer func() {
		t.Log("Save data if failed.")
		saveDataIfFailed(t, db, records, panicked)
	}()

	t.Log("Starting workers.")
	records = runWorkers(t,
		db,
		workerCount,
		conf,
		testDuration)

	t.Log("Analyzing the history records.")
	if err := validateSequential(records); err != nil {
		t.Errorf("The history records are not sequential:\n %v", err)
	}

	t.Log("Checking database consistency.")
	if err := checkConsistency(t, db); err != nil {
		t.Errorf("The data isn't consistency: %v", err)
	}

	panicked = false
	// TODO (ahrtr):
	//   1. intentionally inject a random failpoint.
}

// mustCreateDB is created in place of `btesting.MustCreateDB`, and it's
// only supposed to be used by the concurrent test case. The purpose is
// to ensure the test case can be executed on old branches or versions,
// e.g. `release-1.3` or `1.3.[5-7]`.
func mustCreateDB(t *testing.T, o *bolt.Options) *bolt.DB {
	f := filepath.Join(t.TempDir(), "db")

	return mustOpenDB(t, f, o)
}

func mustReOpenDB(t *testing.T, db *bolt.DB, o *bolt.Options) *bolt.DB {
	f := db.Path()

	t.Logf("Closing bbolt DB at: %s", f)
	err := db.Close()
	require.NoError(t, err)

	return mustOpenDB(t, f, o)
}

func mustOpenDB(t *testing.T, dbPath string, o *bolt.Options) *bolt.DB {
	t.Logf("Opening bbolt DB at: %s", dbPath)
	if o == nil {
		o = bolt.DefaultOptions
	}

	freelistType := bolt.FreelistArrayType
	if env := os.Getenv("TEST_FREELIST_TYPE"); env == string(bolt.FreelistMapType) {
		freelistType = bolt.FreelistMapType
	}

	o.FreelistType = freelistType

	db, err := bolt.Open(dbPath, 0600, o)
	require.NoError(t, err)

	return db
}

func checkConsistency(t *testing.T, db *bolt.DB) error {
	return db.View(func(tx *bolt.Tx) error {
		cnt := 0
		for err := range tx.Check() {
			t.Errorf("Consistency error: %v", err)
			cnt++
		}
		if cnt > 0 {
			return fmt.Errorf("%d consistency errors found", cnt)
		}
		return nil
	})
}

/*
*********************************************************
Data structures and functions/methods for running concurrent
workers, which execute different operations, including `Read`,
`Write` and `Delete`.
*********************************************************
*/
func runWorkers(t *testing.T,
	db *bolt.DB,
	workerCount int,
	conf concurrentConfig,
	testDuration time.Duration) historyRecords {
	stopCh := make(chan struct{}, 1)
	errCh := make(chan error, workerCount)

	var mu sync.Mutex
	var rs historyRecords

	g := new(errgroup.Group)
	for i := 0; i < workerCount; i++ {
		w := &worker{
			id: i,
			db: db,

			conf: conf,

			errCh:  errCh,
			stopCh: stopCh,
			t:      t,
		}
		g.Go(func() error {
			wrs, err := runWorker(t, w, errCh)
			mu.Lock()
			rs = append(rs, wrs...)
			mu.Unlock()
			return err
		})
	}

	t.Logf("Keep all workers running for about %s.", testDuration)
	select {
	case <-time.After(testDuration):
	case <-errCh:
	}

	close(stopCh)
	t.Log("Waiting for all workers to finish.")
	if err := g.Wait(); err != nil {
		t.Errorf("Received error: %v", err)
	}

	return rs
}

func runWorker(t *testing.T, w *worker, errCh chan error) (historyRecords, error) {
	rs, err := w.run()
	if len(rs) > 0 && err == nil {
		if terr := validateIncrementalTxid(rs); terr != nil {
			txidErr := fmt.Errorf("[%s]: %w", w.name(), terr)
			t.Error(txidErr)
			errCh <- txidErr
			return rs, txidErr
		}
	}
	return rs, err
}

type worker struct {
	id int
	db *bolt.DB

	conf concurrentConfig

	errCh  chan error
	stopCh chan struct{}

	t *testing.T
}

func (w *worker) name() string {
	return fmt.Sprintf("worker-%d", w.id)
}

func (w *worker) run() (historyRecords, error) {
	var rs historyRecords
	for {
		select {
		case <-w.stopCh:
			w.t.Logf("%q finished.", w.name())
			return rs, nil
		default:
		}

		op := w.pickOperation()
		bucket, key := w.pickBucket(), w.pickKey()
		rec, err := executeOperation(op, w.db, bucket, key, w.conf)
		if err != nil {
			readErr := fmt.Errorf("[%s: %s]: %w", w.name(), op, err)
			w.t.Error(readErr)
			w.errCh <- readErr
			return rs, readErr
		}

		rs = append(rs, rec)
		if w.conf.workInterval != (duration{}) {
			time.Sleep(randomDurationInRange(w.conf.workInterval.min, w.conf.workInterval.max))
		}
	}
}

func (w *worker) pickBucket() []byte {
	return bucketName(mrand.Intn(w.conf.bucketCount))
}

func bucketName(index int) []byte {
	bucket := fmt.Sprintf("%s_%d", bucketPrefix, index)
	return []byte(bucket)
}

func (w *worker) pickKey() []byte {
	key := fmt.Sprintf("%s_%d", keyPrefix, mrand.Intn(w.conf.keyCount))
	return []byte(key)
}

func (w *worker) pickOperation() OperationType {
	sum := 0
	for _, op := range w.conf.operationRatio {
		sum += op.chance
	}
	roll := mrand.Int() % sum
	for _, op := range w.conf.operationRatio {
		if roll < op.chance {
			return op.operation
		}
		roll -= op.chance
	}
	panic("unexpected")
}

func executeOperation(op OperationType, db *bolt.DB, bucket []byte, key []byte, conf concurrentConfig) (historyRecord, error) {
	switch op {
	case Read:
		return executeRead(db, bucket, key, conf.readInterval)
	case Write:
		return executeWrite(db, bucket, key, conf.writeBytes, conf.noopWriteRatio)
	case Delete:
		return executeDelete(db, bucket, key)
	default:
		panic(fmt.Sprintf("unexpected operation type: %s", op))
	}
}

func executeRead(db *bolt.DB, bucket []byte, key []byte, readInterval duration) (historyRecord, error) {
	var rec historyRecord
	err := db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket(bucket)

		initialVal := b.Get(key)
		time.Sleep(randomDurationInRange(readInterval.min, readInterval.max))
		val := b.Get(key)

		if !bytes.Equal(initialVal, val) {
			return fmt.Errorf("read different values for the same key (%q), value1: %q, value2: %q",
				string(key), formatBytes(initialVal), formatBytes(val))
		}

		clonedVal := make([]byte, len(val))
		copy(clonedVal, val)

		rec = historyRecord{
			OperationType: Read,
			Bucket:        string(bucket),
			Key:           string(key),
			Value:         clonedVal,
			Txid:          tx.ID(),
		}

		return nil
	})

	return rec, err
}

func executeWrite(db *bolt.DB, bucket []byte, key []byte, writeBytes bytesRange, noopWriteRatio int) (historyRecord, error) {
	var rec historyRecord

	err := db.Update(func(tx *bolt.Tx) error {
		if mrand.Intn(100) < noopWriteRatio {
			// A no-op write transaction has two consequences:
			//    1. The txid increases by 1;
			//    2. Two meta pages point to the same root page.
			rec = historyRecord{
				OperationType: Write,
				Bucket:        string(bucket),
				Key:           noopTxKey,
				Value:         nil,
				Txid:          tx.ID(),
			}
			return nil
		}

		b := tx.Bucket(bucket)

		valueBytes := randomIntInRange(writeBytes.min, writeBytes.max)
		v := make([]byte, valueBytes)
		if _, cErr := crand.Read(v); cErr != nil {
			return cErr
		}

		putErr := b.Put(key, v)
		if putErr == nil {
			rec = historyRecord{
				OperationType: Write,
				Bucket:        string(bucket),
				Key:           string(key),
				Value:         v,
				Txid:          tx.ID(),
			}
		}

		return putErr
	})

	return rec, err
}

func executeDelete(db *bolt.DB, bucket []byte, key []byte) (historyRecord, error) {
	var rec historyRecord

	err := db.Update(func(tx *bolt.Tx) error {
		b := tx.Bucket(bucket)

		deleteErr := b.Delete(key)
		if deleteErr == nil {
			rec = historyRecord{
				OperationType: Delete,
				Bucket:        string(bucket),
				Key:           string(key),
				Txid:          tx.ID(),
			}
		}

		return deleteErr
	})

	return rec, err
}

func randomDurationInRange(min, max time.Duration) time.Duration {
	d := int64(max) - int64(min)
	d = int64(mrand.Intn(int(d))) + int64(min)
	return time.Duration(d)
}

func randomIntInRange(min, max int) int {
	return mrand.Intn(max-min) + min
}

func formatBytes(val []byte) string {
	if utf8.ValidString(string(val)) {
		return string(val)
	}

	return hex.EncodeToString(val)
}

/*
*********************************************************
Functions for persisting test data, including db file
and operation history
*********************************************************
*/
func saveDataIfFailed(t *testing.T, db *bolt.DB, rs historyRecords, force bool) {
	if t.Failed() || force {
		t.Log("Saving data...")
		dbPath := db.Path()
		if err := db.Close(); err != nil {
			t.Errorf("Failed to close db: %v", err)
		}
		backupPath := testResultsDirectory(t)
		backupDB(t, dbPath, backupPath)
		persistHistoryRecords(t, rs, backupPath)
	}
}

func backupDB(t *testing.T, srcPath string, dstPath string) {
	targetFile := filepath.Join(dstPath, "db.bak")
	t.Logf("Saving the DB file to %s", targetFile)
	err := copyFile(srcPath, targetFile)
	require.NoError(t, err)
	t.Logf("DB file saved to %s", targetFile)
}

func copyFile(srcPath, dstPath string) error {
	// Ensure source file exists.
	_, err := os.Stat(srcPath)
	if os.IsNotExist(err) {
		return fmt.Errorf("source file %q not found", srcPath)
	} else if err != nil {
		return err
	}

	// Ensure output file not exist.
	_, err = os.Stat(dstPath)
	if err == nil {
		return fmt.Errorf("output file %q already exists", dstPath)
	} else if !os.IsNotExist(err) {
		return err
	}

	srcDB, err := os.Open(srcPath)
	if err != nil {
		return fmt.Errorf("failed to open source file %q: %w", srcPath, err)
	}
	defer srcDB.Close()
	dstDB, err := os.Create(dstPath)
	if err != nil {
		return fmt.Errorf("failed to create output file %q: %w", dstPath, err)
	}
	defer dstDB.Close()
	written, err := io.Copy(dstDB, srcDB)
	if err != nil {
		return fmt.Errorf("failed to copy database file from %q to %q: %w", srcPath, dstPath, err)
	}

	srcFi, err := srcDB.Stat()
	if err != nil {
		return fmt.Errorf("failed to get source file info %q: %w", srcPath, err)
	}
	initialSize := srcFi.Size()
	if initialSize != written {
		return fmt.Errorf("the byte copied (%q: %d) isn't equal to the initial db size (%q: %d)", dstPath, written, srcPath, initialSize)
	}

	return nil
}

func persistHistoryRecords(t *testing.T, rs historyRecords, path string) {
	recordFilePath := filepath.Join(path, "history_records.json")
	t.Logf("Saving history records to %s", recordFilePath)
	recordFile, err := os.OpenFile(recordFilePath, os.O_RDWR|os.O_CREATE|os.O_TRUNC, 0755)
	require.NoError(t, err)
	defer recordFile.Close()
	encoder := json.NewEncoder(recordFile)
	for _, rec := range rs {
		err := encoder.Encode(rec)
		require.NoError(t, err)
	}
}

func testResultsDirectory(t *testing.T) string {
	resultsDirectory, ok := os.LookupEnv("RESULTS_DIR")
	var err error
	if !ok {
		resultsDirectory, err = os.MkdirTemp("", "*.db")
		require.NoError(t, err)
	}
	resultsDirectory, err = filepath.Abs(resultsDirectory)
	require.NoError(t, err)

	path, err := filepath.Abs(filepath.Join(resultsDirectory, strings.ReplaceAll(t.Name(), "/", "_")))
	require.NoError(t, err)

	err = os.RemoveAll(path)
	require.NoError(t, err)

	err = os.MkdirAll(path, 0700)
	require.NoError(t, err)

	return path
}

/*
*********************************************************
Data structures and functions for analyzing history records
*********************************************************
*/
type OperationType string

const (
	Read   OperationType = "read"
	Write  OperationType = "write"
	Delete OperationType = "delete"
)

type historyRecord struct {
	OperationType OperationType `json:"operationType,omitempty"`
	Txid          int           `json:"txid,omitempty"`
	Bucket        string        `json:"bucket,omitempty"`
	Key           string        `json:"key,omitempty"`
	Value         []byte        `json:"value,omitempty"`
}

type historyRecords []historyRecord

func (rs historyRecords) Len() int {
	return len(rs)
}

func (rs historyRecords) Less(i, j int) bool {
	// Sorted by (bucket, key) firstly: all records in the same
	// (bucket, key) are grouped together.
	bucketCmp := strings.Compare(rs[i].Bucket, rs[j].Bucket)
	if bucketCmp != 0 {
		return bucketCmp < 0
	}
	keyCmp := strings.Compare(rs[i].Key, rs[j].Key)
	if keyCmp != 0 {
		return keyCmp < 0
	}

	// Sorted by txid
	if rs[i].Txid != rs[j].Txid {
		return rs[i].Txid < rs[j].Txid
	}

	// Sorted by operation type: put `Read` after other operation types
	// if they operate on the same (bucket, key) and have the same txid.
	if rs[i].OperationType == Read {
		return false
	}

	return true
}

func (rs historyRecords) Swap(i, j int) {
	rs[i], rs[j] = rs[j], rs[i]
}

func validateIncrementalTxid(rs historyRecords) error {
	lastTxid := rs[0].Txid

	for i := 1; i < len(rs); i++ {
		if (rs[i].OperationType == Read && rs[i].Txid < lastTxid) || (rs[i].OperationType != Read && rs[i].Txid <= lastTxid) {
			return fmt.Errorf("detected non-incremental txid(%d, %d) in %s mode", lastTxid, rs[i].Txid, rs[i].OperationType)
		}
		lastTxid = rs[i].Txid
	}

	return nil
}

func validateSequential(rs historyRecords) error {
	sort.Sort(rs)

	type bucketAndKey struct {
		bucket string
		key    string
	}
	lastWriteKeyValueMap := make(map[bucketAndKey]*historyRecord)

	for _, rec := range rs {
		bk := bucketAndKey{
			bucket: rec.Bucket,
			key:    rec.Key,
		}
		if v, ok := lastWriteKeyValueMap[bk]; ok {
			if rec.OperationType == Write {
				v.Txid = rec.Txid
				if rec.Key != noopTxKey {
					v.Value = rec.Value
				}
			} else if rec.OperationType == Delete {
				delete(lastWriteKeyValueMap, bk)
			} else {
				if !bytes.Equal(v.Value, rec.Value) {
					return fmt.Errorf("readOperation[txid: %d, bucket: %s, key: %s] read %x, \nbut writer[txid: %d] wrote %x",
						rec.Txid, rec.Bucket, rec.Key, rec.Value, v.Txid, v.Value)
				}
			}
		} else {
			if rec.OperationType == Write && rec.Key != noopTxKey {
				lastWriteKeyValueMap[bk] = &historyRecord{
					OperationType: Write,
					Bucket:        rec.Bucket,
					Key:           rec.Key,
					Value:         rec.Value,
					Txid:          rec.Txid,
				}
			} else if rec.OperationType == Read {
				if len(rec.Value) != 0 {
					return fmt.Errorf("expected the first readOperation[txid: %d, bucket: %s, key: %s] read nil, \nbut got %x",
						rec.Txid, rec.Bucket, rec.Key, rec.Value)
				}
			}
		}
	}

	return nil
}

/*
TestConcurrentRepeatableRead verifies repeatable read. The case
intentionally creates a scenario that read and write transactions
are interleaved. It performs several writing operations after starting
each long-running read transaction to ensure it has a larger txid
than previous read transaction. It verifies that bbolt correctly
releases free pages, and will not pollute (e.g. prematurely release)
any pages which are still being used by any read transaction.
*/
func TestConcurrentRepeatableRead(t *testing.T) {
	if testing.Short() {
		t.Skip("skipping test in short mode.")
	}

	testCases := []struct {
		name           string
		noFreelistSync bool
		freelistType   bolt.FreelistType
	}{
		// [array] freelist
		{
			name:           "sync array freelist",
			noFreelistSync: false,
			freelistType:   bolt.FreelistArrayType,
		},
		{
			name:           "not sync array freelist",
			noFreelistSync: true,
			freelistType:   bolt.FreelistArrayType,
		},
		// [map] freelist
		{
			name:           "sync map freelist",
			noFreelistSync: false,
			freelistType:   bolt.FreelistMapType,
		},
		{
			name:           "not sync map freelist",
			noFreelistSync: true,
			freelistType:   bolt.FreelistMapType,
		},
	}

	for _, tc := range testCases {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {

			t.Log("Preparing db.")
			var (
				bucket = []byte("data")
				key    = []byte("mykey")

				option = &bolt.Options{
					PageSize:       4096,
					NoFreelistSync: tc.noFreelistSync,
					FreelistType:   tc.freelistType,
				}
			)

			db := mustCreateDB(t, option)
			defer func() {
				// The db will be reopened later, so put `db.Close()` in a function
				// to avoid premature evaluation of `db`. Note that the execution
				// of a deferred function is deferred to the moment the surrounding
				// function returns, but the function value and parameters to the
				// call are evaluated as usual and saved anew.
				db.Close()
			}()

			// Create lots of K/V to allocate some pages
			err := db.Update(func(tx *bolt.Tx) error {
				b, err := tx.CreateBucketIfNotExists(bucket)
				if err != nil {
					return err
				}
				for i := 0; i < 1000; i++ {
					k := fmt.Sprintf("key_%d", i)
					if err := b.Put([]byte(k), make([]byte, 1024)); err != nil {
						return err
					}
				}
				return nil
			})
			require.NoError(t, err)

			// Remove all K/V to create some free pages
			err = db.Update(func(tx *bolt.Tx) error {
				b := tx.Bucket(bucket)
				for i := 0; i < 1000; i++ {
					k := fmt.Sprintf("key_%d", i)
					if err := b.Delete([]byte(k)); err != nil {
						return err
					}
				}
				return b.Put(key, []byte("randomValue"))
			})
			require.NoError(t, err)

			// bbolt will not release free pages directly after committing
			// a writing transaction; instead all pages freed are putting
			// into a pending list. Accordingly, the free pages might not
			// be able to be reused by following writing transactions. So
			// we reopen the db to completely release all free pages.
			db = mustReOpenDB(t, db, option)

			var (
				wg                     sync.WaitGroup
				longRunningReaderCount = 10
				stopCh                 = make(chan struct{})
				errCh                  = make(chan error, longRunningReaderCount)
				readInterval           = duration{5 * time.Millisecond, 10 * time.Millisecond}

				writeOperationCountInBetween = 5
				writeBytes                   = bytesRange{10, 20}

				testDuration = 10 * time.Second
			)

			for i := 0; i < longRunningReaderCount; i++ {
				readWorkerName := fmt.Sprintf("reader_%d", i)
				t.Logf("Starting long running read operation: %s", readWorkerName)
				wg.Add(1)
				go func() {
					defer wg.Done()
					rErr := executeLongRunningRead(t, readWorkerName, db, bucket, key, readInterval, stopCh)
					if rErr != nil {
						errCh <- rErr
					}
				}()
				time.Sleep(500 * time.Millisecond)

				t.Logf("Perform %d write operations after starting a long running read operation", writeOperationCountInBetween)
				for j := 0; j < writeOperationCountInBetween; j++ {
					_, err := executeWrite(db, bucket, key, writeBytes, 0)
					require.NoError(t, err)
				}
			}

			t.Log("Perform lots of write operations to check whether the long running read operations will read dirty data")
			wg.Add(1)
			go func() {
				defer wg.Done()
				cnt := longRunningReaderCount * writeOperationCountInBetween
				for i := 0; i < cnt; i++ {
					select {
					case <-stopCh:
						return
					default:
					}
					_, err := executeWrite(db, bucket, key, writeBytes, 0)
					require.NoError(t, err)
				}
			}()

			t.Log("Waiting for result")
			select {
			case err := <-errCh:
				close(stopCh)
				t.Errorf("Detected dirty read: %v", err)
			case <-time.After(testDuration):
				close(stopCh)
			}

			wg.Wait()
		})
	}
}

func executeLongRunningRead(t *testing.T, name string, db *bolt.DB, bucket []byte, key []byte, readInterval duration, stopCh chan struct{}) error {
	err := db.View(func(tx *bolt.Tx) error {
		b := tx.Bucket(bucket)

		initialVal := b.Get(key)

		for {
			select {
			case <-stopCh:
				t.Logf("%q finished.", name)
				return nil
			default:
			}

			time.Sleep(randomDurationInRange(readInterval.min, readInterval.max))
			val := b.Get(key)

			if !bytes.Equal(initialVal, val) {
				dirtyReadErr := fmt.Errorf("read different values for the same key (%q), value1: %q, value2: %q",
					string(key), formatBytes(initialVal), formatBytes(val))
				return dirtyReadErr
			}
		}
	})

	return err
}
