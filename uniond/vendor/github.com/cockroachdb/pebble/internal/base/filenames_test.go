// Copyright 2012 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package base

import (
	"bytes"
	"fmt"
	"os"
	"testing"

	"github.com/cockroachdb/pebble/vfs"
	"github.com/cockroachdb/redact"
	"github.com/stretchr/testify/require"
)

func TestParseFilename(t *testing.T) {
	testCases := map[string]bool{
		"000000.log":             true,
		"000000.log.zip":         false,
		"000000..log":            false,
		"a000000.log":            false,
		"abcdef.log":             false,
		"000001ldb":              false,
		"000001.sst":             true,
		"CURRENT":                true,
		"CURRaNT":                false,
		"LOCK":                   true,
		"xLOCK":                  false,
		"x.LOCK":                 false,
		"MANIFEST":               false,
		"MANIFEST123456":         false,
		"MANIFEST-":              false,
		"MANIFEST-123456":        true,
		"MANIFEST-123456.doc":    false,
		"OPTIONS":                false,
		"OPTIONS123456":          false,
		"OPTIONS-":               false,
		"OPTIONS-123456":         true,
		"OPTIONS-123456.doc":     false,
		"CURRENT.123456":         false,
		"CURRENT.dbtmp":          false,
		"CURRENT.123456.dbtmp":   true,
		"temporary.123456.dbtmp": true,
	}
	fs := vfs.NewMem()
	for tc, want := range testCases {
		_, _, got := ParseFilename(fs, fs.PathJoin("foo", tc))
		if got != want {
			t.Errorf("%q: got %v, want %v", tc, got, want)
		}
	}
}

func TestFilenameRoundTrip(t *testing.T) {
	testCases := map[FileType]bool{
		// CURRENT and LOCK files aren't numbered.
		FileTypeCurrent: false,
		FileTypeLock:    false,
		// The remaining file types are numbered.
		FileTypeLog:      true,
		FileTypeManifest: true,
		FileTypeTable:    true,
		FileTypeOptions:  true,
		FileTypeOldTemp:  true,
		FileTypeTemp:     true,
	}
	fs := vfs.NewMem()
	for fileType, numbered := range testCases {
		fileNums := []FileNum{0}
		if numbered {
			fileNums = []FileNum{0, 1, 2, 3, 10, 42, 99, 1001}
		}
		for _, fileNum := range fileNums {
			filename := MakeFilepath(fs, "foo", fileType, fileNum.DiskFileNum())
			gotFT, gotFN, gotOK := ParseFilename(fs, filename)
			if !gotOK {
				t.Errorf("could not parse %q", filename)
				continue
			}
			if gotFT != fileType || gotFN.FileNum() != fileNum {
				t.Errorf("filename=%q: got %v, %v, want %v, %v", filename, gotFT, gotFN, fileType, fileNum)
				continue
			}
		}
	}
}

type bufferFataler struct {
	buf bytes.Buffer
}

func (b *bufferFataler) Fatalf(msg string, args ...interface{}) {
	fmt.Fprintf(&b.buf, msg, args...)
}

func TestMustExist(t *testing.T) {
	err := os.ErrNotExist
	fs := vfs.Default
	var buf bufferFataler
	filename := fs.PathJoin("..", "..", "testdata", "db-stage-4", "000000.sst")

	MustExist(fs, filename, &buf, err)
	require.Equal(t, `000000.sst:
file does not exist
directory contains 10 files, 3 unknown, 1 tables, 1 logs, 1 manifests`, buf.buf.String())
}

func TestRedactFileNum(t *testing.T) {
	// Ensure that redaction never redacts file numbers.
	require.Equal(t, redact.RedactableString("000005"), redact.Sprint(FileNum(5)))
	require.Equal(t, redact.RedactableString("000005"), redact.Sprint(DiskFileNum{fn: 5}))
}
