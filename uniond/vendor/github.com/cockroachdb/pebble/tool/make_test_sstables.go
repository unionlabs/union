// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build make_test_sstables
// +build make_test_sstables

// Run using: go run -tags make_test_sstables ./tool/make_test_sstables.go
package main

import (
	"log"

	"github.com/cockroachdb/pebble/internal/private"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/cockroachdb/pebble/vfs"
)

func makeOutOfOrder() {
	fs := vfs.Default
	f, err := fs.Create("tool/testdata/out-of-order.sst")
	if err != nil {
		log.Fatal(err)
	}
	w := sstable.NewWriter(objstorageprovider.NewFileWritable(f), sstable.WriterOptions{})
	private.SSTableWriterDisableKeyOrderChecks(w)

	set := func(key string) {
		if err := w.Set([]byte(key), nil); err != nil {
			log.Fatal(err)
		}
	}

	set("a")
	set("c")
	set("b")

	if err := w.Close(); err != nil {
		log.Fatal(err)
	}
}

func main() {
	makeOutOfOrder()
}
