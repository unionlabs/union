// Copyright 2014-2022 Ulrich Kunitz. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package xz_test

import (
	"bufio"
	"fmt"
	"io"
	"log"
	"os"

	"github.com/ulikunitz/xz"
)

func ExampleReader() {
	f, err := os.Open("fox.xz")
	if err != nil {
		log.Fatalf("os.Open(%q) error %s", "fox.xz", err)
	}
	defer f.Close()
	r, err := xz.NewReader(bufio.NewReader(f))
	if err != nil {
		log.Fatalf("xz.NewReader(f) error %s", err)
	}
	if _, err = io.Copy(os.Stdout, r); err != nil {
		log.Fatalf("io.Copy error %s", err)
	}
	// Output:
	// The quick brown fox jumps over the lazy dog.
}

func ExampleWriter() {
	f, err := os.Create("example.xz")
	if err != nil {
		log.Fatalf("os.Open(%q) error %s", "example.xz", err)
	}
	defer f.Close()
	w, err := xz.NewWriter(f)
	if err != nil {
		log.Fatalf("xz.NewWriter(f) error %s", err)
	}
	defer w.Close()
	_, err = fmt.Fprintln(w, "The brown fox jumps over the lazy dog.")
	if err != nil {
		log.Fatalf("fmt.Fprintln error %s", err)
	}
	if err = w.Close(); err != nil {
		log.Fatalf("w.Close() error %s", err)
	}
	// Output:
}
