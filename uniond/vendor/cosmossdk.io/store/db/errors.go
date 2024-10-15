package db

import "errors"

var (
	// ErrKeyEmpty is returned when a key is empty.
	ErrKeyEmpty = errors.New("key empty")

	// ErrBatchClosed is returned when a closed or written batch is used.
	ErrBatchClosed = errors.New("batch has been written or closed")

	// ErrValueNil is returned when attempting to set a nil value.
	ErrValueNil = errors.New("value nil")
)
