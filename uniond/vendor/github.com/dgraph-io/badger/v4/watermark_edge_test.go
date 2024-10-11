/*
 * Copyright 2023 Dgraph Labs, Inc. and Contributors
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

package badger

import (
	"crypto/rand"
	"errors"
	"fmt"
	"math/big"
	"sync"
	"testing"
	"time"
)

func TestWaterMarkEdgeCase(t *testing.T) {
	const N = 1_000
	runBadgerTest(t, nil, func(t *testing.T, db *DB) {
		eg := make(chan error, N)
		defer close(eg)

		var wg sync.WaitGroup
		wg.Add(N)
		for i := 0; i < N; i++ {
			go func(j int) {
				defer wg.Done()
				if err := doWork(db, j); errors.Is(err, ErrConflict) {
					eg <- nil
				} else {
					eg <- fmt.Errorf("expected conflict not found, err: %v, i = %v", err, j)
				}
			}(i)
		}
		wg.Wait()

		for i := 0; i < N; i++ {
			if err := <-eg; err != nil {
				t.Fatal(err)
			}
		}
	})
}

func doWork(db *DB, i int) error {
	delay()

	key1 := fmt.Sprintf("v:%d:%s", i, generateRandomBytes())
	key2 := fmt.Sprintf("v:%d:%s", i, generateRandomBytes())

	tx1 := db.NewTransaction(true)
	defer tx1.Discard()
	tx2 := db.NewTransaction(true)
	defer tx2.Discard()

	getValue(tx2, key1)
	getValue(tx2, key2)
	getValue(tx1, key1)
	getValue(tx2, key1)
	setValue(tx2, key1, "value1")
	setValue(tx2, key2, "value2")

	if err := tx2.Commit(); err != nil {
		return fmt.Errorf("tx2 failed: %w (key1 = %s, key2 = %s)", err, key1, key2)
	}

	setValue(tx1, key1, "value1-second")
	getValue(tx1, key1)
	setValue(tx1, key1, "value1-third")

	delay()
	if err := tx1.Commit(); err != nil {
		return fmt.Errorf("tx1 failed: %w (key1 = %s, key2 = %s)", err, key1, key2)
	}

	return nil
}

func generateRandomBytes() []byte {
	b := make([]byte, 20)
	if _, err := rand.Read(b); err != nil {
		panic(err)
	}
	return b
}

func getValue(txn *Txn, key string) {
	if _, err := txn.Get([]byte(key)); err != nil {
		if !errors.Is(err, ErrKeyNotFound) {
			panic(err)
		}
	}
}

func setValue(txn *Txn, key, value string) {
	if err := txn.Set([]byte(key), []byte(value)); err != nil {
		panic(err)
	}
}

func delay() {
	jitter, err := rand.Int(rand.Reader, big.NewInt(100))
	if err != nil {
		panic(err)
	}
	<-time.After(time.Duration(jitter.Int64()) * time.Millisecond)
}
