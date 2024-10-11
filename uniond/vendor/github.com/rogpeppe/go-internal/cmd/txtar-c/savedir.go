// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// The txtar-c command archives a directory tree as a txtar archive printed to standard output.
//
// Usage:
//
//	txtar-c /path/to/dir >saved.txtar
//
// See https://godoc.org/golang.org/x/tools/txtar for details of the format
// and how to parse a txtar file.
package main

import (
	"bytes"
	stdflag "flag"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"strings"
	"unicode/utf8"

	"github.com/rogpeppe/go-internal/txtar"
)

var flag = stdflag.NewFlagSet(os.Args[0], stdflag.ContinueOnError)

func usage() {
	fmt.Fprintf(os.Stderr, "usage: txtar-c dir >saved.txtar\n")
	flag.PrintDefaults()
}

var (
	quoteFlag = flag.Bool("quote", false, "quote files that contain txtar file markers instead of failing")
	allFlag   = flag.Bool("a", false, "include dot files too")
)

func main() {
	os.Exit(main1())
}

func main1() int {
	flag.Usage = usage
	if flag.Parse(os.Args[1:]) != nil {
		return 2
	}
	if flag.NArg() != 1 {
		usage()
		return 2
	}

	log.SetPrefix("txtar-c: ")
	log.SetFlags(0)

	dir := flag.Arg(0)

	a := new(txtar.Archive)
	dir = filepath.Clean(dir)
	if err := filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
		if err != nil {
			return err
		}
		if path == dir {
			return nil
		}
		name := info.Name()
		if strings.HasPrefix(name, ".") && !*allFlag {
			if info.IsDir() {
				return filepath.SkipDir
			}
			return nil
		}
		if !info.Mode().IsRegular() {
			return nil
		}
		data, err := os.ReadFile(path)
		if err != nil {
			return err
		}
		if !utf8.Valid(data) {
			log.Printf("%s: ignoring file with invalid UTF-8 data", path)
			return nil
		}
		if len(data) > 0 && !bytes.HasSuffix(data, []byte("\n")) {
			log.Printf("%s: adding final newline", path)
			data = append(data, '\n')
		}
		filename := strings.TrimPrefix(path, dir+string(filepath.Separator))
		if txtar.NeedsQuote(data) {
			if !*quoteFlag {
				log.Printf("%s: ignoring file with txtar marker in", path)
				return nil
			}
			data, err = txtar.Quote(data)
			if err != nil {
				log.Printf("%s: ignoring unquotable file: %v", path, err)
				return nil
			}
			a.Comment = append(a.Comment, []byte("unquote "+filename+"\n")...)
		}
		a.Files = append(a.Files, txtar.File{
			Name: filepath.ToSlash(filename),
			Data: data,
		})
		return nil
	}); err != nil {
		log.Fatal(err)
	}

	data := txtar.Format(a)
	os.Stdout.Write(data)

	return 0
}
