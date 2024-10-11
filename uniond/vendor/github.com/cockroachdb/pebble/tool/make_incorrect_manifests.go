// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

//go:build make_incorrect_manifests
// +build make_incorrect_manifests

// Run using: go run -tags make_incorrect_manifests ./tool/make_incorrect_manifests.go
package main

import (
	"log"

	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/record"
	"github.com/cockroachdb/pebble/vfs"
)

func writeVE(writer *record.Writer, ve *manifest.VersionEdit) {
	w, err := writer.Next()
	if err != nil {
		log.Fatal(err)
	}
	err = ve.Encode(w)
	if err != nil {
		log.Fatal(err)
	}
}

func makeManifest1() {
	fs := vfs.Default
	f, err := fs.Create("tool/testdata/MANIFEST-invalid")
	if err != nil {
		log.Fatal(err)
	}
	writer := record.NewWriter(f)
	var ve manifest.VersionEdit
	ve.ComparerName = "leveldb.BytewiseComparator"
	ve.MinUnflushedLogNum = 2
	ve.NextFileNum = 5
	ve.LastSeqNum = 20
	ve.NewFiles = []manifest.NewFileEntry{
		{Level: 6, Meta: &manifest.FileMetadata{
			FileNum: 1, SmallestSeqNum: 2, LargestSeqNum: 5}}}
	writeVE(writer, &ve)

	ve.MinUnflushedLogNum = 3
	ve.NewFiles = []manifest.NewFileEntry{
		{Level: 6, Meta: &manifest.FileMetadata{
			FileNum: 2, SmallestSeqNum: 1, LargestSeqNum: 4}}}
	writeVE(writer, &ve)

	err = writer.Close()
	if err != nil {
		log.Fatal(err)
	}
}

func main() {
	makeManifest1()
}
