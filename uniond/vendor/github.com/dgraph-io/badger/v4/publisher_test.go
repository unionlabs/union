/*
 * Copyright 2019 Dgraph Labs, Inc. and Contributors
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
	"context"
	"fmt"
	"runtime"
	"sync"
	"sync/atomic"
	"testing"

	"github.com/pkg/errors"
	"github.com/stretchr/testify/require"

	"github.com/dgraph-io/badger/v4/pb"
)

// This test will result in deadlock for commits before this.
// Exiting this test gracefully will be the proof that the
// publisher is no longer stuck in deadlock.
func TestPublisherDeadlock(t *testing.T) {
	runBadgerTest(t, nil, func(t *testing.T, db *DB) {
		var subWg sync.WaitGroup
		subWg.Add(1)

		var firstUpdate sync.WaitGroup
		firstUpdate.Add(1)

		var allUpdatesDone sync.WaitGroup
		allUpdatesDone.Add(1)
		var subDone sync.WaitGroup
		subDone.Add(1)
		go func() {
			subWg.Done()
			match := pb.Match{Prefix: []byte("ke"), IgnoreBytes: ""}
			err := db.Subscribe(context.Background(), func(kvs *pb.KVList) error {
				firstUpdate.Done()
				// Before exiting Subscribe process, we will wait until each of the
				// 1110 updates (defined below) have been completed.
				allUpdatesDone.Wait()
				return errors.New("error returned")
			}, []pb.Match{match})
			require.Error(t, err, errors.New("error returned"))
			subDone.Done()
		}()
		subWg.Wait()
		go func() {
			err := db.Update(func(txn *Txn) error {
				e := NewEntry([]byte(fmt.Sprintf("key%d", 0)), []byte(fmt.Sprintf("value%d", 0)))
				return txn.SetEntry(e)
			})
			require.NoError(t, err)
		}()

		firstUpdate.Wait()
		var req atomic.Int64
		for i := 1; i < 1110; i++ {
			go func(i int) {
				err := db.Update(func(txn *Txn) error {
					e := NewEntry([]byte(fmt.Sprintf("key%d", i)), []byte(fmt.Sprintf("value%d", i)))
					return txn.SetEntry(e)
				})
				require.NoError(t, err)
				req.Add(1)
			}(i)
		}
		for {
			if req.Load() == 1109 {
				break
			}
			// FYI: This does the same as "thread.yield()" from other languages.
			//      In other words, it tells the go-routine scheduler to switch
			//      to another go-routine. This is strongly preferred over
			//      time.Sleep(...).
			runtime.Gosched()
		}
		// Free up the subscriber, which is waiting for updates to finish.
		allUpdatesDone.Done()
		// Exit when the subscription process has been exited.
		subDone.Wait()
	})
}

func TestPublisherOrdering(t *testing.T) {
	runBadgerTest(t, nil, func(t *testing.T, db *DB) {
		order := []string{}
		var wg sync.WaitGroup
		wg.Add(1)
		var subWg sync.WaitGroup
		subWg.Add(1)
		go func() {
			subWg.Done()
			updates := 0
			match := pb.Match{Prefix: []byte("ke"), IgnoreBytes: ""}
			err := db.Subscribe(context.Background(), func(kvs *pb.KVList) error {
				updates += len(kvs.GetKv())
				for _, kv := range kvs.GetKv() {
					order = append(order, string(kv.Value))
				}
				if updates == 5 {
					wg.Done()
				}
				return nil
			}, []pb.Match{match})
			if err != nil {
				require.Equal(t, err.Error(), context.Canceled.Error())
			}
		}()
		subWg.Wait()
		for i := 0; i < 5; i++ {
			require.NoError(t, db.Update(func(txn *Txn) error {
				e := NewEntry([]byte(fmt.Sprintf("key%d", i)), []byte(fmt.Sprintf("value%d", i)))
				return txn.SetEntry(e)
			}))
		}
		wg.Wait()
		for i := 0; i < 5; i++ {
			require.Equal(t, fmt.Sprintf("value%d", i), order[i])
		}
	})
}

func TestMultiplePrefix(t *testing.T) {
	runBadgerTest(t, nil, func(t *testing.T, db *DB) {
		var wg sync.WaitGroup
		wg.Add(1)
		var subWg sync.WaitGroup
		subWg.Add(1)
		go func() {
			subWg.Done()
			updates := 0
			match1 := pb.Match{Prefix: []byte("ke"), IgnoreBytes: ""}
			match2 := pb.Match{Prefix: []byte("hel"), IgnoreBytes: ""}
			err := db.Subscribe(context.Background(), func(kvs *pb.KVList) error {
				updates += len(kvs.GetKv())
				for _, kv := range kvs.GetKv() {
					if string(kv.Key) == "key" {
						require.Equal(t, string(kv.Value), "value")
					} else {
						require.Equal(t, string(kv.Value), "badger")
					}
				}
				if updates == 2 {
					wg.Done()
				}
				return nil
			}, []pb.Match{match1, match2})
			if err != nil {
				require.Equal(t, err.Error(), context.Canceled.Error())
			}
		}()
		subWg.Wait()
		require.NoError(t, db.Update(func(txn *Txn) error {
			return txn.SetEntry(NewEntry([]byte("key"), []byte("value")))
		}))
		require.NoError(t, db.Update(func(txn *Txn) error {
			return txn.SetEntry(NewEntry([]byte("hello"), []byte("badger")))
		}))
		wg.Wait()
	})
}
