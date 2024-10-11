// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build make_test_remotecat
// +build make_test_remotecat

// Run using: go run -tags make_test_remotecat ./tool/make_test_remotecat.go
package main

import (
	"log"
	"os"
	"path/filepath"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/remoteobjcat"
	"github.com/cockroachdb/pebble/vfs"
)

func main() {
	dir, err := os.MkdirTemp("", "make-test-remotecat")
	if err != nil {
		log.Fatal(err)
	}
	catalog, _, err := remoteobjcat.Open(vfs.Default, dir)
	if err != nil {
		log.Fatal(err)
	}
	if err := catalog.SetCreatorID(3); err != nil {
		log.Fatal(err)
	}

	var b remoteobjcat.Batch
	b.AddObject(remoteobjcat.RemoteObjectMetadata{
		FileNum:        base.FileNum(1).DiskFileNum(),
		FileType:       base.FileTypeTable,
		CreatorID:      3,
		CreatorFileNum: base.FileNum(1).DiskFileNum(),
		CleanupMethod:  objstorage.SharedRefTracking,
		Locator:        "foo",
	})
	if err := catalog.ApplyBatch(b); err != nil {
		log.Fatal(err)
	}
	b.Reset()
	b.AddObject(remoteobjcat.RemoteObjectMetadata{
		FileNum:        base.FileNum(2).DiskFileNum(),
		FileType:       base.FileTypeTable,
		CreatorID:      5,
		CreatorFileNum: base.FileNum(10).DiskFileNum(),
		CleanupMethod:  objstorage.SharedRefTracking,
		Locator:        "foo",
	})
	b.DeleteObject(base.FileNum(1).DiskFileNum())
	b.AddObject(remoteobjcat.RemoteObjectMetadata{
		FileNum:          base.FileNum(3).DiskFileNum(),
		FileType:         base.FileTypeTable,
		CleanupMethod:    objstorage.SharedRefTracking,
		Locator:          "bar",
		CustomObjectName: "external.sst",
	})
	if err := catalog.ApplyBatch(b); err != nil {
		log.Fatal(err)
	}
	if err := catalog.Close(); err != nil {
		log.Fatal(err)
	}
	contents, err := os.ReadFile(filepath.Join(dir, "REMOTE-OBJ-CATALOG-000001"))
	if err != nil {
		log.Fatal(err)
	}
	if err := os.WriteFile("tool/testdata/REMOTE-OBJ-CATALOG", contents, 0666); err != nil {
		log.Fatal(err)
	}
}
