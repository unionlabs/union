// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// The txtar-x command extracts a txtar archive to a filesystem.
//
// Usage:
//
//	txtar-x [-C root-dir] saved.txt
//
// See https://godoc.org/golang.org/x/tools/txtar for details of the format
// and how to parse a txtar file.
package main

import (
	"flag"
	"fmt"
	"io"
	"log"
	"os"

	"github.com/rogpeppe/go-internal/txtar"
)

var (
	extractDir = flag.String("C", ".", "directory to extract files into")
)

func usage() {
	fmt.Fprintf(os.Stderr, "usage: txtar-x [flags] [file]\n")
	flag.PrintDefaults()
}

func main() {
	os.Exit(main1())
}

func main1() int {
	flag.Usage = usage
	flag.Parse()
	if flag.NArg() > 1 {
		usage()
		return 2
	}
	log.SetPrefix("txtar-x: ")
	log.SetFlags(0)

	var a *txtar.Archive
	if flag.NArg() == 0 {
		data, err := io.ReadAll(os.Stdin)
		if err != nil {
			log.Printf("cannot read stdin: %v", err)
			return 1
		}
		a = txtar.Parse(data)
	} else {
		a1, err := txtar.ParseFile(flag.Arg(0))
		if err != nil {
			log.Print(err)
			return 1
		}
		a = a1
	}
	if err := txtar.Write(a, *extractDir); err != nil {
		log.Print(err)
		return 1
	}
	return 0
}
