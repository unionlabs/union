package btesting

import (
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/require"

	bolt "go.etcd.io/bbolt"
)

var statsFlag = flag.Bool("stats", false, "show performance stats")

const (
	// TestFreelistType is used as an env variable for test to indicate the backend type.
	TestFreelistType = "TEST_FREELIST_TYPE"
	// TestEnableStrictMode is used to enable strict check by default after opening each DB.
	TestEnableStrictMode = "TEST_ENABLE_STRICT_MODE"
)

// DB is a test wrapper for bolt.DB.
type DB struct {
	*bolt.DB
	f string
	o *bolt.Options
	t testing.TB
}

// MustCreateDB returns a new, open DB at a temporary location.
func MustCreateDB(t testing.TB) *DB {
	return MustCreateDBWithOption(t, nil)
}

// MustCreateDBWithOption returns a new, open DB at a temporary location with given options.
func MustCreateDBWithOption(t testing.TB, o *bolt.Options) *DB {
	f := filepath.Join(t.TempDir(), "db")
	return MustOpenDBWithOption(t, f, o)
}

func MustOpenDBWithOption(t testing.TB, f string, o *bolt.Options) *DB {
	t.Logf("Opening bbolt DB at: %s", f)
	if o == nil {
		o = bolt.DefaultOptions
	}

	freelistType := bolt.FreelistArrayType
	if env := os.Getenv(TestFreelistType); env == string(bolt.FreelistMapType) {
		freelistType = bolt.FreelistMapType
	}

	o.FreelistType = freelistType

	db, err := bolt.Open(f, 0600, o)
	require.NoError(t, err)
	resDB := &DB{
		DB: db,
		f:  f,
		o:  o,
		t:  t,
	}
	resDB.strictModeEnabledDefault()
	t.Cleanup(resDB.PostTestCleanup)
	return resDB
}

func (db *DB) PostTestCleanup() {
	// Check database consistency after every test.
	if db.DB != nil {
		db.MustCheck()
		db.MustClose()
	}
}

// Close closes the database but does NOT delete the underlying file.
func (db *DB) Close() error {
	if db.DB != nil {
		// Log statistics.
		if *statsFlag {
			db.PrintStats()
		}
		db.t.Logf("Closing bbolt DB at: %s", db.f)
		err := db.DB.Close()
		if err != nil {
			return err
		}
		db.DB = nil
	}
	return nil
}

// MustClose closes the database but does NOT delete the underlying file.
func (db *DB) MustClose() {
	err := db.Close()
	require.NoError(db.t, err)
}

func (db *DB) MustDeleteFile() {
	err := os.Remove(db.Path())
	require.NoError(db.t, err)
}

func (db *DB) SetOptions(o *bolt.Options) {
	db.o = o
}

// MustReopen reopen the database. Panic on error.
func (db *DB) MustReopen() {
	if db.DB != nil {
		panic("Please call Close() before MustReopen()")
	}
	db.t.Logf("Reopening bbolt DB at: %s", db.f)
	indb, err := bolt.Open(db.Path(), 0600, db.o)
	require.NoError(db.t, err)
	db.DB = indb
	db.strictModeEnabledDefault()
}

// MustCheck runs a consistency check on the database and panics if any errors are found.
func (db *DB) MustCheck() {
	err := db.View(func(tx *bolt.Tx) error {
		// Collect all the errors.
		var errors []error
		for err := range tx.Check() {
			errors = append(errors, err)
			if len(errors) > 10 {
				break
			}
		}

		// If errors occurred, copy the DB and print the errors.
		if len(errors) > 0 {
			var path = filepath.Join(db.t.TempDir(), "db.backup")
			err := tx.CopyFile(path, 0600)
			require.NoError(db.t, err)

			// Print errors.
			fmt.Print("\n\n")
			fmt.Printf("consistency check failed (%d errors)\n", len(errors))
			for _, err := range errors {
				fmt.Println(err)
			}
			fmt.Println("")
			fmt.Println("db saved to:")
			fmt.Println(path)
			fmt.Print("\n\n")
			os.Exit(-1)
		}

		return nil
	})
	require.NoError(db.t, err)
}

// Fill - fills the DB using numTx transactions and numKeysPerTx.
func (db *DB) Fill(bucket []byte, numTx int, numKeysPerTx int,
	keyGen func(tx int, key int) []byte,
	valueGen func(tx int, key int) []byte) error {
	for tr := 0; tr < numTx; tr++ {
		err := db.Update(func(tx *bolt.Tx) error {
			b, _ := tx.CreateBucketIfNotExists(bucket)
			for i := 0; i < numKeysPerTx; i++ {
				if err := b.Put(keyGen(tr, i), valueGen(tr, i)); err != nil {
					return err
				}
			}
			return nil
		})
		if err != nil {
			return err
		}
	}
	return nil
}

func (db *DB) Path() string {
	return db.f
}

// CopyTempFile copies a database to a temporary file.
func (db *DB) CopyTempFile() {
	path := filepath.Join(db.t.TempDir(), "db.copy")
	err := db.View(func(tx *bolt.Tx) error {
		return tx.CopyFile(path, 0600)
	})
	require.NoError(db.t, err)
	fmt.Println("db copied to: ", path)
}

// PrintStats prints the database stats
func (db *DB) PrintStats() {
	var stats = db.Stats()
	fmt.Printf("[db] %-20s %-20s %-20s\n",
		fmt.Sprintf("pg(%d/%d)", stats.TxStats.GetPageCount(), stats.TxStats.GetPageAlloc()),
		fmt.Sprintf("cur(%d)", stats.TxStats.GetCursorCount()),
		fmt.Sprintf("node(%d/%d)", stats.TxStats.GetNodeCount(), stats.TxStats.GetNodeDeref()),
	)
	fmt.Printf("     %-20s %-20s %-20s\n",
		fmt.Sprintf("rebal(%d/%v)", stats.TxStats.GetRebalance(), truncDuration(stats.TxStats.GetRebalanceTime())),
		fmt.Sprintf("spill(%d/%v)", stats.TxStats.GetSpill(), truncDuration(stats.TxStats.GetSpillTime())),
		fmt.Sprintf("w(%d/%v)", stats.TxStats.GetWrite(), truncDuration(stats.TxStats.GetWriteTime())),
	)
}

func truncDuration(d time.Duration) string {
	return regexp.MustCompile(`^(\d+)(\.\d+)`).ReplaceAllString(d.String(), "$1")
}

func (db *DB) strictModeEnabledDefault() {
	strictModeEnabled := strings.ToLower(os.Getenv(TestEnableStrictMode))
	db.StrictMode = strictModeEnabled == "true"
}

func (db *DB) ForceDisableStrictMode() {
	db.StrictMode = false
}
