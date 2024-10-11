// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package storage defines storage interfaces for and a basic implementation of a checksum database.
package storage

import "context"

// A Storage is a transaction key-value storage system.
type Storage interface {
	// ReadOnly runs f in a read-only transaction.
	// It is equivalent to ReadWrite except that the
	// transaction's BufferWrite method will fail unconditionally.
	// (The implementation may be able to optimize the
	// transaction if it knows at the start that no writes will happen.)
	ReadOnly(ctx context.Context, f func(context.Context, Transaction) error) error

	// ReadWrite runs f in a read-write transaction.
	// If f returns an error, the transaction aborts and returns that error.
	// If f returns nil, the transaction attempts to commit and then return nil.
	// Otherwise it tries again. Note that f may be called multiple times and that
	// the result only describes the effect of the final call to f.
	// The caller must take care not to use any state computed during
	// earlier calls to f, or even the last call to f when an error is returned.
	ReadWrite(ctx context.Context, f func(context.Context, Transaction) error) error
}

// A Transaction provides read and write operations within a transaction,
// as executed by [Storage]'s ReadOnly or ReadWrite methods.
type Transaction interface {
	// ReadValue reads the value associated with a single key.
	// If there is no value associated with that key, ReadKey returns an empty value.
	// An error is only returned for problems accessing the storage.
	ReadValue(ctx context.Context, key string) (value string, err error)

	// ReadValues reads the values associated with the given keys.
	// If there is no value stored for a given key, ReadValues returns an empty value for that key.
	// An error is only returned for problems accessing the storage.
	ReadValues(ctx context.Context, keys []string) (values []string, err error)

	// BufferWrites buffers the given writes,
	// to be applied at the end of the transaction.
	// BufferWrites panics if this is a ReadOnly transaction.
	// It returns an error if it detects any other problems.
	// The behavior of multiple writes buffered using the same key
	// is undefined: it may return an error or not.
	BufferWrites(writes []Write) error
}

// A Write is a single change to be applied at the end of a read-write transaction.
// A Write with an empty value deletes the value associated with the given key.
type Write struct {
	Key   string
	Value string
}
