/*
 * Copyright 2018 Dgraph Labs, Inc. and Contributors
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
	"context"
	"errors"
	"fmt"
	"log"
	"math"
	"math/rand"
	"os"
	"strconv"
	"sync"
	"sync/atomic"
	"time"

	"github.com/spf13/cobra"

	"github.com/dgraph-io/badger/v4"
	"github.com/dgraph-io/badger/v4/pb"
	"github.com/dgraph-io/badger/v4/y"
	"github.com/dgraph-io/ristretto/z"
)

var testCmd = &cobra.Command{
	Use:   "bank",
	Short: "Run bank test on Badger.",
	Long: `
This command runs bank test on Badger, inspired by Jepsen. It creates many
accounts and moves money among them transactionally. It also reads the sum total
of all the accounts, to ensure that the total never changes.
`,
}

var bankTest = &cobra.Command{
	Use:   "test",
	Short: "Execute bank test on Badger.",
	RunE:  runTest,
}

var bankDisect = &cobra.Command{
	Use:   "disect",
	Short: "Disect the bank output.",
	Long: `
Disect the bank output BadgerDB to find the first transaction which causes
failure of the total invariant.
`,
	RunE: runDisect,
}

var (
	numGoroutines   int
	numAccounts     int
	numPrevious     int
	duration        string
	stopAll         atomic.Int32
	checkStream     bool
	checkSubscriber bool
	verbose         bool
	encryptionKey   string
)

const (
	keyPrefix         = "account:"
	initialBal uint64 = 100
)

func init() {
	RootCmd.AddCommand(testCmd)
	testCmd.AddCommand(bankTest)
	testCmd.AddCommand(bankDisect)

	testCmd.Flags().IntVarP(
		&numAccounts, "accounts", "a", 10000, "Number of accounts in the bank.")
	bankTest.Flags().IntVarP(
		&numGoroutines, "conc", "c", 16, "Number of concurrent transactions to run.")
	bankTest.Flags().StringVarP(&duration, "duration", "d", "3m", "How long to run the test.")
	bankTest.Flags().BoolVarP(&checkStream, "check_stream", "s", false,
		"If true, the test will send transactions to another badger instance via the stream "+
			"interface in order to verify that all data is streamed correctly.")
	bankTest.Flags().BoolVarP(&checkSubscriber, "check_subscriber", "w", false,
		"If true, the test will send transactions to another badger instance via the subscriber "+
			"interface in order to verify that all the data is published correctly.")
	bankTest.Flags().BoolVarP(&verbose, "verbose", "v", false,
		"If true, the test will print all the executed bank transfers to standard output. "+
			"This outputs a lot so it's best to turn it off when running the test for a while.")
	bankTest.Flags().StringVarP(&encryptionKey, "encryption-key", "e", "",
		"If it is true, badger will encrypt all the data stored on the disk.")

	bankDisect.Flags().IntVarP(&numPrevious, "previous", "p", 12,
		"Starting from the violation txn, how many previous versions to retrieve.")
	bankDisect.Flags().StringVar(&encryptionKey, "decryption-key", "",
		"If set, DB will be opened using the provided decryption key.")
}

func key(account int) []byte {
	return []byte(fmt.Sprintf("%s%s", keyPrefix, strconv.Itoa(account)))
}

func toUint64(val []byte) uint64 {
	u, err := strconv.ParseUint(string(val), 10, 64)
	y.Check(err)
	return u
}

func toSlice(bal uint64) []byte {
	return []byte(strconv.FormatUint(bal, 10))
}

func getBalance(txn *badger.Txn, account int) (uint64, error) {
	item, err := get(txn, key(account))
	if err != nil {
		return 0, err
	}

	var bal uint64
	err = item.Value(func(v []byte) error {
		bal = toUint64(v)
		return nil
	})
	return bal, err
}

func putBalance(txn *badger.Txn, account int, bal uint64) error {
	return txn.SetEntry(badger.NewEntry(key(account), toSlice(bal)))
}

func min(a, b uint64) uint64 {
	if a < b {
		return a
	}
	return b
}

var errAbandoned = errors.New("Transaction abandonded due to insufficient balance")

func moveMoney(db *badger.DB, from, to int) error {
	return db.Update(func(txn *badger.Txn) error {
		balf, err := getBalance(txn, from)
		if err != nil {
			return err
		}
		balt, err := getBalance(txn, to)
		if err != nil {
			return err
		}

		floor := min(balf, balt)
		if floor < 5 {
			return errAbandoned
		}
		// Move the money.
		balf -= 5
		balt += 5

		if err = putBalance(txn, from, balf); err != nil {
			return err
		}
		return putBalance(txn, to, balt)
	})
}

type account struct {
	Id  int
	Bal uint64
}

func diff(a, b []account) string {
	var buf bytes.Buffer
	y.AssertTruef(len(a) == len(b), "len(a)=%d. len(b)=%d\n", len(a), len(b))
	for i := range a {
		ai := a[i]
		bi := b[i]
		if ai.Id != bi.Id || ai.Bal != bi.Bal {
			buf.WriteString(fmt.Sprintf("Index: %d. Account [%+v] -> [%+v]\n", i, ai, bi))
		}
	}
	return buf.String()
}

var errFailure = errors.New("test failed due to balance mismatch")

// get function will fetch the value for the key "k" either by using the
// txn.Get API or the iterator.Seek API.
func get(txn *badger.Txn, k []byte) (*badger.Item, error) {
	if rand.Int()%2 == 0 {
		return txn.Get(k)
	}

	iopt := badger.DefaultIteratorOptions
	// PrefectValues is expensive. We don't need it here.
	iopt.PrefetchValues = false
	it := txn.NewIterator(iopt)
	defer it.Close()
	it.Seek(k)
	if it.Valid() {
		return it.Item(), nil
	}
	return nil, badger.ErrKeyNotFound
}

// seekTotal retrives the total of all accounts by seeking for each account key.
func seekTotal(txn *badger.Txn) ([]account, error) {
	expected := uint64(numAccounts) * initialBal
	var accounts []account

	var total uint64
	for i := 0; i < numAccounts; i++ {
		item, err := get(txn, key(i))
		if err != nil {
			log.Printf("Error for account: %d. err=%v. key=%q\n", i, err, key(i))
			return accounts, err
		}
		val, err := item.ValueCopy(nil)
		if err != nil {
			return accounts, err
		}
		acc := account{
			Id:  i,
			Bal: toUint64(val),
		}
		accounts = append(accounts, acc)
		total += acc.Bal
	}
	if total != expected {
		log.Printf("Balance did NOT match up. Expected: %d. Received: %d",
			expected, total)
		stopAll.Add(1)
		return accounts, errFailure
	}
	return accounts, nil
}

// Range is [lowTs, highTs).
func findFirstInvalidTxn(db *badger.DB, lowTs, highTs uint64) uint64 {
	checkAt := func(ts uint64) error {
		txn := db.NewTransactionAt(ts, false)
		_, err := seekTotal(txn)
		txn.Discard()
		return err
	}

	if highTs-lowTs < 1 {
		log.Printf("Checking at lowTs: %d\n", lowTs)
		err := checkAt(lowTs)
		if err == errFailure {
			fmt.Printf("Violation at ts: %d\n", lowTs)
			return lowTs
		} else if err != nil {
			log.Printf("Error at lowTs: %d. Err=%v\n", lowTs, err)
			return 0
		}
		fmt.Printf("No violation found at ts: %d\n", lowTs)
		return 0
	}

	midTs := (lowTs + highTs) / 2
	log.Println()
	log.Printf("Checking. low=%d. high=%d. mid=%d\n", lowTs, highTs, midTs)
	err := checkAt(midTs)
	if err == badger.ErrKeyNotFound || err == nil {
		// If no failure, move to higher ts.
		return findFirstInvalidTxn(db, midTs+1, highTs)
	}
	// Found an error.
	return findFirstInvalidTxn(db, lowTs, midTs)
}

func compareTwo(db *badger.DB, before, after uint64) {
	fmt.Printf("Comparing @ts=%d with @ts=%d\n", before, after)
	txn := db.NewTransactionAt(before, false)
	prev, err := seekTotal(txn)
	if err == errFailure {
		// pass
	} else {
		y.Check(err)
	}
	txn.Discard()

	txn = db.NewTransactionAt(after, false)
	now, err := seekTotal(txn)
	if err == errFailure {
		// pass
	} else {
		y.Check(err)
	}
	txn.Discard()

	fmt.Println(diff(prev, now))
}

func runDisect(cmd *cobra.Command, args []string) error {
	// The total did not match up. So, let's disect the DB to find the
	// transction which caused the total mismatch.
	db, err := badger.OpenManaged(badger.DefaultOptions(sstDir).
		WithValueDir(vlogDir).
		WithReadOnly(true).
		WithEncryptionKey([]byte(encryptionKey)).
		WithIndexCacheSize(1 << 30))
	if err != nil {
		return err
	}
	fmt.Println("opened db")

	var min, max uint64 = math.MaxUint64, 0
	{
		txn := db.NewTransactionAt(uint64(math.MaxUint32), false)
		iopt := badger.DefaultIteratorOptions
		iopt.AllVersions = true
		itr := txn.NewIterator(iopt)
		for itr.Rewind(); itr.Valid(); itr.Next() {
			item := itr.Item()
			if min > item.Version() {
				min = item.Version()
			}
			if max < item.Version() {
				max = item.Version()
			}
		}
		itr.Close()
		txn.Discard()
	}

	log.Printf("min=%d. max=%d\n", min, max)
	ts := findFirstInvalidTxn(db, min, max)
	fmt.Println()
	if ts == 0 {
		fmt.Println("Nothing found. Exiting.")
		return nil
	}

	for i := 0; i < numPrevious; i++ {
		compareTwo(db, ts-1-uint64(i), ts-uint64(i))
	}
	return nil
}

func runTest(cmd *cobra.Command, args []string) error {
	rand.Seed(time.Now().UnixNano())

	// Open DB
	opts := badger.DefaultOptions(sstDir).
		WithValueDir(vlogDir).
		// Do not GC any versions, because we need them for the disect.
		WithNumVersionsToKeep(int(math.MaxInt32)).
		WithBlockCacheSize(1 << 30).
		WithIndexCacheSize(1 << 30)

	if verbose {
		opts = opts.WithLoggingLevel(badger.DEBUG)
	}

	if encryptionKey != "" {
		opts = opts.WithEncryptionKey([]byte(encryptionKey))
		// The following comment is intentional as we would need the encryption key in case
		// we want to run disect tool on the directory generated by bank test tool.
		log.Printf("Using encryption key %s\n", encryptionKey)
	}
	log.Printf("Opening DB with options: %+v\n", opts)
	db, err := badger.Open(opts)
	if err != nil {
		return err
	}
	defer db.Close()

	var tmpDb *badger.DB
	var subscribeDB *badger.DB
	if checkSubscriber {
		dir, err := os.MkdirTemp("", "bank_subscribe")
		y.Check(err)

		subscribeDB, err = badger.Open(badger.DefaultOptions(dir).WithSyncWrites(false))
		if err != nil {
			return err
		}
		defer subscribeDB.Close()
	}

	if checkStream {
		dir, err := os.MkdirTemp("", "bank_stream")
		y.Check(err)

		tmpDb, err = badger.Open(badger.DefaultOptions(dir).WithSyncWrites(false))
		if err != nil {
			return err
		}
		defer tmpDb.Close()
	}

	wb := db.NewWriteBatch()
	for i := 0; i < numAccounts; i++ {
		y.Check(wb.Set(key(i), toSlice(initialBal)))
	}
	log.Println("Waiting for writes to be done...")
	y.Check(wb.Flush())

	log.Println("Bank initialization OK. Commencing test.")
	log.Printf("Running with %d accounts, and %d goroutines.\n", numAccounts, numGoroutines)
	log.Printf("Using keyPrefix: %s\n", keyPrefix)

	dur, err := time.ParseDuration(duration)
	y.Check(err)

	// startTs := time.Now()
	endTs := time.Now().Add(dur)
	var total, errors, reads atomic.Uint64

	var wg sync.WaitGroup
	wg.Add(1)
	go func() {
		defer wg.Done()
		ticker := time.NewTicker(time.Second)
		defer ticker.Stop()

		for range ticker.C {
			if stopAll.Load() > 0 {
				// Do not proceed.
				return
			}
			// log.Printf("[%6s] Total: %d. Errors: %d Reads: %d.\n",
			// 	time.Since(startTs).Round(time.Second).String(),
			// 	total.Load(),
			// 	errors.Load(),
			// 	reads.Load())
			if time.Now().After(endTs) {
				return
			}
		}
	}()

	// RW goroutines.
	for i := 0; i < numGoroutines; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()

			ticker := time.NewTicker(10 * time.Microsecond)
			defer ticker.Stop()

			for range ticker.C {
				if stopAll.Load() > 0 {
					// Do not proceed.
					return
				}
				if time.Now().After(endTs) {
					return
				}
				from := rand.Intn(numAccounts)
				to := rand.Intn(numAccounts)
				if from == to {
					continue
				}
				err := moveMoney(db, from, to)
				total.Add(1)
				if err == nil && verbose {
					log.Printf("Moved $5. %d -> %d\n", from, to)
				} else {
					errors.Add(1)
				}
			}
		}()
	}

	if checkStream {
		wg.Add(1)
		go func() {
			defer wg.Done()

			ticker := time.NewTicker(time.Second)
			defer ticker.Stop()

			for range ticker.C {
				log.Printf("Received stream\n")

				// Do not proceed.
				if stopAll.Load() > 0 || time.Now().After(endTs) {
					return
				}

				// Clean up the database receiving the stream.
				err = tmpDb.DropAll()
				y.Check(err)

				batch := tmpDb.NewWriteBatch()

				stream := db.NewStream()
				stream.Send = func(buf *z.Buffer) error {
					err := buf.SliceIterate(func(s []byte) error {
						var kv pb.KV
						if err := kv.Unmarshal(s); err != nil {
							return err
						}
						return batch.Set(kv.Key, kv.Value)
					})
					return err
				}
				y.Check(stream.Orchestrate(context.Background()))
				y.Check(batch.Flush())

				y.Check(tmpDb.View(func(txn *badger.Txn) error {
					_, err := seekTotal(txn)
					if err != nil {
						log.Printf("Error while calculating total in stream: %v", err)
					}
					return nil
				}))
			}
		}()
	}

	// RO goroutine.
	wg.Add(1)
	go func() {
		defer wg.Done()

		ticker := time.NewTicker(10 * time.Microsecond)
		defer ticker.Stop()

		for range ticker.C {
			if stopAll.Load() > 0 {
				// Do not proceed.
				return
			}
			if time.Now().After(endTs) {
				return
			}

			y.Check(db.View(func(txn *badger.Txn) error {
				_, err := seekTotal(txn)
				if err != nil {
					log.Printf("Error while calculating total: %v", err)
				} else {
					reads.Add(1)
				}
				return nil
			}))
		}
	}()

	ctx, cancel := context.WithCancel(context.Background())
	defer cancel()
	var subWg sync.WaitGroup
	if checkSubscriber {
		subWg.Add(1)
		go func() {
			defer subWg.Done()
			accountIDS := []pb.Match{}
			for i := 0; i < numAccounts; i++ {
				accountIDS = append(accountIDS, pb.Match{Prefix: key(i), IgnoreBytes: ""})
			}
			updater := func(kvs *pb.KVList) error {
				batch := subscribeDB.NewWriteBatch()
				for _, kv := range kvs.GetKv() {
					y.Check(batch.Set(kv.Key, kv.Value))
				}

				return batch.Flush()
			}
			_ = db.Subscribe(ctx, updater, accountIDS)
		}()
	}

	wg.Wait()

	if checkSubscriber {
		cancel()
		subWg.Wait()
		y.Check(subscribeDB.View(func(txn *badger.Txn) error {
			_, err := seekTotal(txn)
			if err != nil {
				log.Printf("Error while calculating subscriber DB total: %v", err)
			} else {
				reads.Add(1)
			}
			return nil
		}))
	}

	if stopAll.Load() == 0 {
		log.Println("Test OK")
		return nil
	}
	log.Println("Test FAILED")
	return fmt.Errorf("Test FAILED")
}
