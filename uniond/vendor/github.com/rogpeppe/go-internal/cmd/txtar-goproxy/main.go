// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// The txtar-goproxy command runs a Go module proxy from a txtar module
// directory (as manipulated by the txtar-addmod command).
//
// This allows interactive experimentation with the set of proxied modules.
// For example:
//
//	cd cmd/go
//	go test -proxy=localhost:1234 &
//	export GOPROXY=http://localhost:1234/mod
//
// and then run go commands as usual.
package main

import (
	"flag"
	"fmt"
	"log"
	"os"

	"github.com/rogpeppe/go-internal/goproxytest"
)

var proxyAddr = flag.String("addr", "", "run proxy on this network address")

func usage() {
	fmt.Fprintf(os.Stderr, "usage: txtar-goproxy [flags] dir\n")
	flag.PrintDefaults()
	os.Exit(2)
}

func main() {
	flag.Usage = usage
	flag.Parse()
	if flag.NArg() != 1 {
		usage()
	}
	srv, err := goproxytest.NewServer(flag.Arg(0), *proxyAddr)
	if err != nil {
		log.Fatal(err)
	}
	fmt.Printf("export GOPROXY=%s\n", srv.URL)
	select {}
}
