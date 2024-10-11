// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package storage

import (
	"context"
	"errors"
	"math/rand"
	"sync"
)

// Mem is an in-memory implementation of [Storage].
// It is meant for tests and does not store any data to persistent storage.
//
// The zero value is an empty Mem ready for use.
type Mem struct {
	mu    sync.RWMutex
	table map[string]string
}

// A memTx is a transaction in a Mem.
type memTx struct {
	m      *Mem
	writes []Write
}

// errRetry is an internal sentinel indicating that the transaction should be retried.
// It is never returned to the caller.
var errRetry = errors.New("retry")

// ReadOnly runs f in a read-only transaction.
func (m *Mem) ReadOnly(ctx context.Context, f func(context.Context, Transaction) error) error {
	tx := &memTx{m: m}
	for {
		err := func() error {
			m.mu.Lock()
			defer m.mu.Unlock()

			if err := f(ctx, tx); err != nil {
				return err
			}
			// Spurious retry with 10% probability.
			if rand.Intn(10) == 0 {
				return errRetry
			}
			return nil
		}()
		if err != errRetry {
			return err
		}
	}
}

// ReadWrite runs f in a read-write transaction.
func (m *Mem) ReadWrite(ctx context.Context, f func(context.Context, Transaction) error) error {
	tx := &memTx{m: m}
	for {
		err := func() error {
			m.mu.Lock()
			defer m.mu.Unlock()

			tx.writes = []Write{}
			if err := f(ctx, tx); err != nil {
				return err
			}
			// Spurious retry with 10% probability.
			if rand.Intn(10) == 0 {
				return errRetry
			}
			if m.table == nil {
				m.table = make(map[string]string)
			}
			for _, w := range tx.writes {
				if w.Value == "" {
					delete(m.table, w.Key)
				} else {
					m.table[w.Key] = w.Value
				}
			}
			return nil
		}()
		if err != errRetry {
			return err
		}
	}
}

// ReadValues returns the values associated with the given keys.
func (tx *memTx) ReadValues(ctx context.Context, keys []string) ([]string, error) {
	vals := make([]string, len(keys))
	for i, key := range keys {
		vals[i] = tx.m.table[key]
	}
	return vals, nil
}

// ReadValue returns the value associated with the single key.
func (tx *memTx) ReadValue(ctx context.Context, key string) (string, error) {
	return tx.m.table[key], nil
}

// BufferWrites buffers a list of writes to be applied
// to the table when the transaction commits.
// The changes are not visible to reads within the transaction.
// The map argument is not used after the call returns.
func (tx *memTx) BufferWrites(list []Write) error {
	if tx.writes == nil {
		panic("BufferWrite on read-only transaction")
	}
	tx.writes = append(tx.writes, list...)
	return nil
}
