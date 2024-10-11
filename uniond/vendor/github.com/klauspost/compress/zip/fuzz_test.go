// Copyright 2021 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package zip

import (
	"bytes"
	"io"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/klauspost/compress/internal/fuzz"
)

func FuzzReader(f *testing.F) {
	testdata, err := os.ReadDir("testdata")
	if err != nil {
		f.Fatalf("failed to read testdata directory: %s", err)
	}
	for _, de := range testdata {
		if de.IsDir() {
			continue
		}
		b, err := os.ReadFile(filepath.Join("testdata", de.Name()))
		if err != nil {
			f.Fatalf("failed to read testdata: %s", err)
		}
		f.Add(b)
	}
	fuzz.AddFromZip(f, "testdata/FuzzReader-raw.zip", fuzz.TypeRaw, testing.Short())
	fuzz.AddFromZip(f, "testdata/FuzzReader-enc.zip", fuzz.TypeGoFuzz, testing.Short())

	f.Fuzz(func(t *testing.T, b []byte) {
		r, err := NewReader(bytes.NewReader(b), int64(len(b)))
		if err != nil {
			return
		}

		type file struct {
			header  *FileHeader
			content []byte
		}
		files := make([]file, 0, len(r.File))

		for i, f := range r.File {
			fr, err := f.Open()
			if err != nil {
				continue
			}
			// No more than 1MiB per file
			limit := int64(1 << 20)
			if i < 100 {
				limit = 10
			}
			content, err := io.ReadAll(io.LimitReader(fr, limit))
			if err != nil {
				continue
			}

			files = append(files, file{header: &f.FileHeader, content: content})
			if _, err := r.Open(f.Name); err != nil {
				continue
			}
		}

		// If we were unable to read anything out of the archive don't
		// bother trying to roundtrip it.
		if len(files) == 0 {
			return
		}

		w := NewWriter(io.Discard)
		for _, f := range files {
			ww, err := w.CreateHeader(f.header)
			if err != nil {
				t.Fatalf("unable to write previously parsed header: %s", err)
			}
			if !strings.HasSuffix(f.header.Name, "/") {
				if _, err := ww.Write(f.content); err != nil {
					t.Fatalf("unable to write previously parsed content: %s", err)
				}
			}
		}

		if err := w.Close(); err != nil {
			t.Fatalf("Unable to write archive: %s", err)
		}

		// TODO: We may want to check if the archive roundtrips.
	})
}
