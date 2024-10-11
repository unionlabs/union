// Copyright 2019 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package storage

import (
	"context"
	"fmt"
	"io"
	"testing"
)

// TestStorage tests a Storage implementation.
func TestStorage(t *testing.T, ctx context.Context, storage Storage) {
	s := storage

	// Insert records.
	err := s.ReadWrite(ctx, func(ctx context.Context, tx Transaction) error {
		for i := 0; i < 10; i++ {
			err := tx.BufferWrites([]Write{
				{Key: fmt.Sprint(i), Value: fmt.Sprint(-i)},
				{Key: fmt.Sprint(1000 + i), Value: fmt.Sprint(-1000 - i)},
			})
			if err != nil {
				t.Fatal(err)
			}
		}
		return nil
	})
	if err != nil {
		t.Fatal(err)
	}

	// Read the records back.
	testRead := func() {
		err := s.ReadOnly(ctx, func(ctx context.Context, tx Transaction) error {
			for i := int64(0); i < 1010; i++ {
				if i == 10 {
					i = 1000
				}
				val, err := tx.ReadValue(ctx, fmt.Sprint(i))
				if err != nil {
					t.Fatalf("reading %v: %v", i, err)
				}
				if want := fmt.Sprint(-i); val != want {
					t.Fatalf("ReadValue %v = %q, want %v", i, val, want)
				}
			}
			return nil
		})
		if err != nil {
			t.Fatal(err)
		}
	}
	testRead()

	// Buffered writes in failed transaction should not be applied.
	err = s.ReadWrite(ctx, func(ctx context.Context, tx Transaction) error {
		tx.BufferWrites([]Write{
			{Key: fmt.Sprint(0), Value: ""},          // delete
			{Key: fmt.Sprint(1), Value: "overwrite"}, // overwrite
		})
		if err != nil {
			t.Fatal(err)
		}
		return io.ErrUnexpectedEOF
	})
	if err != io.ErrUnexpectedEOF {
		t.Fatalf("ReadWrite returned %v, want ErrUnexpectedEOF", err)
	}

	// All same values should still be there.
	testRead()
}
