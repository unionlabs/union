// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// The txtar-addmod command adds a module as a txtar archive to the a testdata module directory
// as understood by the goproxytest package (see https://godoc.org/github.com/rogpeppe/go-internal/goproxytest).
//
// Usage:
//
//	txtar-addmod dir path@version...
//
// where dir is the directory to add the module to.
//
// In general, it's intended to be used only for very small modules - we do not want to check
// very large files into testdata/mod.
//
// It is acceptable to edit the archive afterward to remove or shorten files.
package main

import (
	"bytes"
	"flag"
	"fmt"
	"log"
	"os"
	"os/exec"
	"path/filepath"
	"strings"

	"golang.org/x/mod/module"
	"golang.org/x/tools/txtar"
)

func usage() {
	fmt.Fprintf(os.Stderr, "usage: txtar-addmod dir path@version...\n")
	flag.PrintDefaults()

	fmt.Fprintf(os.Stderr, `
The txtar-addmod command adds a module as a txtar archive to the
testdata module directory as understood by the goproxytest package
(see https://godoc.org/github.com/rogpeppe/go-internal/goproxytest).

The dir argument names to directory to add the module to. If dir is "-",
the result will instead be written to the standard output in a form
suitable for embedding directly into a testscript txtar file, with each
file prefixed with the ".gomodproxy" directory.

In general, txtar-addmod is intended to be used only for very small
modules - we do not want to check very large files into testdata/mod.

It is acceptable to edit the archive afterward to remove or shorten files.
`)
	os.Exit(2)
}

var tmpdir string

func fatalf(format string, args ...interface{}) {
	os.RemoveAll(tmpdir)
	log.Fatalf(format, args...)
}

const goCmd = "go"

func main() {
	os.Exit(main1())
}

var allFiles = flag.Bool("all", false, "include all source files")

func main1() int {
	flag.Usage = usage
	flag.Parse()
	if flag.NArg() < 2 {
		usage()
	}
	targetDir := flag.Arg(0)
	modules := flag.Args()[1:]

	log.SetPrefix("txtar-addmod: ")
	log.SetFlags(0)

	var err error
	tmpdir, err = os.MkdirTemp("", "txtar-addmod-")
	if err != nil {
		log.Fatal(err)
	}

	run := func(command string, args ...string) string {
		cmd := exec.Command(command, args...)
		cmd.Dir = tmpdir
		var stderr bytes.Buffer
		cmd.Stderr = &stderr
		out, err := cmd.Output()
		if err != nil {
			fatalf("%s %s: %v\n%s", command, strings.Join(args, " "), err, stderr.Bytes())
		}
		return string(out)
	}

	gopath := strings.TrimSpace(run("go", "env", "GOPATH"))
	if gopath == "" {
		fatalf("cannot find GOPATH")
	}

	exitCode := 0
	for _, arg := range modules {
		if err := os.WriteFile(filepath.Join(tmpdir, "go.mod"), []byte("module m\n"), 0o666); err != nil {
			fatalf("%v", err)
		}
		run(goCmd, "get", "-d", arg)
		path := arg
		if i := strings.Index(path, "@"); i >= 0 {
			path = path[:i]
		}
		out := run(goCmd, "list", "-m", "-f={{.Path}} {{.Version}} {{.Dir}}", path)
		f := strings.Fields(out)
		if len(f) != 3 {
			log.Printf("go list -m %s: unexpected output %q", arg, out)
			exitCode = 1
			continue
		}
		path, vers, dir := f[0], f[1], f[2]

		encpath, err := module.EscapePath(path)
		if err != nil {
			log.Printf("failed to encode path %q: %v", path, err)
			continue
		}
		path = encpath

		mod, err := os.ReadFile(filepath.Join(gopath, "pkg/mod/cache/download", path, "@v", vers+".mod"))
		if err != nil {
			log.Printf("%s: %v", arg, err)
			exitCode = 1
			continue
		}
		info, err := os.ReadFile(filepath.Join(gopath, "pkg/mod/cache/download", path, "@v", vers+".info"))
		if err != nil {
			log.Printf("%s: %v", arg, err)
			exitCode = 1
			continue
		}

		a := new(txtar.Archive)
		title := arg
		if !strings.Contains(arg, "@") {
			title += "@" + vers
		}
		dir = filepath.Clean(dir)
		modDir := strings.ReplaceAll(path, "/", "_") + "_" + vers
		filePrefix := ""
		if targetDir == "-" {
			filePrefix = ".gomodproxy/" + modDir + "/"
		} else {
			// No comment if we're writing to stdout.
			a.Comment = []byte(fmt.Sprintf("module %s\n\n", title))
		}
		a.Files = []txtar.File{
			{Name: filePrefix + ".mod", Data: mod},
			{Name: filePrefix + ".info", Data: info},
		}
		err = filepath.Walk(dir, func(path string, info os.FileInfo, err error) error {
			if err != nil {
				return err
			}
			if !info.Mode().IsRegular() {
				return nil
			}
			// TODO: skip dirs like "testdata" or "_foo" unless -all
			// is given?
			name := info.Name()
			switch {
			case *allFiles:
			case name == "go.mod":
			case strings.HasSuffix(name, ".go"):
			default:
				// the name is not in the whitelist, and we're
				// not including all files via -all
				return nil
			}
			data, err := os.ReadFile(path)
			if err != nil {
				return err
			}
			a.Files = append(a.Files, txtar.File{
				Name: filePrefix + strings.TrimPrefix(path, dir+string(filepath.Separator)),
				Data: data,
			})
			return nil
		})
		if err != nil {
			log.Printf("%s: %v", arg, err)
			exitCode = 1
			continue
		}

		data := txtar.Format(a)
		if targetDir == "-" {
			if _, err := os.Stdout.Write(data); err != nil {
				log.Printf("cannot write output: %v", err)
				exitCode = 1
				break
			}
		} else {
			if err := os.WriteFile(filepath.Join(targetDir, modDir+".txtar"), data, 0o666); err != nil {
				log.Printf("%s: %v", arg, err)
				exitCode = 1
				continue
			}
		}
	}
	os.RemoveAll(tmpdir)
	return exitCode
}
