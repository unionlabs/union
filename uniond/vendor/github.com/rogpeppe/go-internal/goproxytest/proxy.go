// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

/*
Package goproxytest serves Go modules from a proxy server designed to run on
localhost during tests, both to make tests avoid requiring specific network
servers and also to make them significantly faster.

Each module archive is either a file named path_vers.txtar or path_vers.txt, or
a directory named path_vers, where slashes in path have been replaced with underscores.
The archive or directory must contain two files ".info" and ".mod", to be served as
the info and mod files in the proxy protocol (see
https://research.swtch.com/vgo-module).  The remaining files are served as the
content of the module zip file.  The path@vers prefix required of files in the
zip file is added automatically by the proxy: the files in the archive have
names without the prefix, like plain "go.mod", "x.go", and so on.

See ../cmd/txtar-addmod and ../cmd/txtar-c for tools generate txtar
files, although it's fine to write them by hand.
*/
package goproxytest

import (
	"archive/zip"
	"bytes"
	"cmp"
	"encoding/json"
	"fmt"
	"io/fs"
	"log"
	"net"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"golang.org/x/mod/module"
	"golang.org/x/mod/semver"
	"golang.org/x/tools/txtar"

	"github.com/rogpeppe/go-internal/par"
)

type Server struct {
	server       *http.Server
	URL          string
	dir          string
	logf         func(string, ...any)
	modList      []module.Version
	zipCache     par.Cache
	archiveCache par.Cache
}

// NewTestServer is a wrapper around [NewServer] for use in Go tests.
// Failure to start the server stops the test via [testing.TB.Fatalf],
// all server logs go through [testing.TB.Logf],
// and the server is closed when the test finishes via [testing.TB.Cleanup].
func NewTestServer(tb testing.TB, dir, addr string) *Server {
	srv, err := newServer(dir, addr, tb.Logf)
	if err != nil {
		tb.Fatalf("cannot start Go proxy: %v", err)
	}
	tb.Cleanup(srv.Close)
	return srv
}

// NewServer starts the Go module proxy listening on the given
// network address. It serves modules taken from the given directory
// name. If addr is empty, it will listen on an arbitrary
// localhost port. If dir is empty, "testmod" will be used.
//
// The returned Server should be closed after use.
func NewServer(dir, addr string) (*Server, error) {
	return newServer(dir, addr, log.Printf)
}

func newServer(dir, addr string, logf func(string, ...any)) (*Server, error) {
	addr = cmp.Or(addr, "localhost:0")
	dir = cmp.Or(dir, "testmod")
	srv := Server{dir: dir, logf: logf}
	if err := srv.readModList(); err != nil {
		return nil, fmt.Errorf("cannot read modules: %v", err)
	}
	l, err := net.Listen("tcp", addr)
	if err != nil {
		return nil, fmt.Errorf("cannot listen on %q: %v", addr, err)
	}
	srv.server = &http.Server{
		Handler: http.HandlerFunc(srv.handler),
	}
	addr = l.Addr().String()
	srv.URL = "http://" + addr + "/mod"
	go func() {
		if err := srv.server.Serve(l); err != nil && err != http.ErrServerClosed {
			srv.logf("go proxy: http.Serve: %v", err)
		}
	}()
	return &srv, nil
}

// Close shuts down the proxy.
func (srv *Server) Close() {
	srv.server.Close()
}

func (srv *Server) readModList() error {
	entries, err := os.ReadDir(srv.dir)
	if err != nil {
		return err
	}
	for _, entry := range entries {
		name := entry.Name()
		switch {
		case strings.HasSuffix(name, ".txt"):
			name = strings.TrimSuffix(name, ".txt")
		case strings.HasSuffix(name, ".txtar"):
			name = strings.TrimSuffix(name, ".txtar")
		case entry.IsDir():
		default:
			continue
		}
		i := strings.LastIndex(name, "_v")
		if i < 0 {
			continue
		}
		encPath := strings.ReplaceAll(name[:i], "_", "/")
		path, err := module.UnescapePath(encPath)
		if err != nil {
			return fmt.Errorf("cannot decode module path in %q: %v", name, err)
		}
		encVers := name[i+1:]
		vers, err := module.UnescapeVersion(encVers)
		if err != nil {
			return fmt.Errorf("cannot decode module version in %q: %v", name, err)
		}
		srv.modList = append(srv.modList, module.Version{Path: path, Version: vers})
	}
	return nil
}

// handler serves the Go module proxy protocol.
// See the proxy section of https://research.swtch.com/vgo-module.
func (srv *Server) handler(w http.ResponseWriter, r *http.Request) {
	if !strings.HasPrefix(r.URL.Path, "/mod/") {
		http.NotFound(w, r)
		return
	}
	path := strings.TrimPrefix(r.URL.Path, "/mod/")
	i := strings.Index(path, "/@v/")
	if i < 0 {
		http.NotFound(w, r)
		return
	}
	enc, file := path[:i], path[i+len("/@v/"):]
	path, err := module.UnescapePath(enc)
	if err != nil {
		srv.logf("go proxy_test: %v\n", err)
		http.NotFound(w, r)
		return
	}
	if file == "list" {
		n := 0
		for _, m := range srv.modList {
			if m.Path == path && !isPseudoVersion(m.Version) {
				if err := module.Check(m.Path, m.Version); err == nil {
					fmt.Fprintf(w, "%s\n", m.Version)
					n++
				}
			}
		}
		if n == 0 {
			http.NotFound(w, r)
		}
		return
	}

	i = strings.LastIndex(file, ".")
	if i < 0 {
		http.NotFound(w, r)
		return
	}
	encVers, ext := file[:i], file[i+1:]
	vers, err := module.UnescapeVersion(encVers)
	if err != nil {
		srv.logf("go proxy_test: %v\n", err)
		http.NotFound(w, r)
		return
	}

	if allHex(vers) {
		var best string
		// Convert commit hash (only) to known version.
		// Use latest version in semver priority, to match similar logic
		// in the repo-based module server (see modfetch.(*codeRepo).convert).
		for _, m := range srv.modList {
			if m.Path == path && semver.Compare(best, m.Version) < 0 {
				var hash string
				if isPseudoVersion(m.Version) {
					hash = m.Version[strings.LastIndex(m.Version, "-")+1:]
				} else {
					hash = srv.findHash(m)
				}
				if strings.HasPrefix(hash, vers) || strings.HasPrefix(vers, hash) {
					best = m.Version
				}
			}
		}
		if best != "" {
			vers = best
		}
	}

	a := srv.readArchive(path, vers)
	if a == nil {
		// As of https://go-review.googlesource.com/c/go/+/189517, cmd/go
		// resolves all paths. i.e. for github.com/hello/world, cmd/go attempts
		// to resolve github.com, github.com/hello and github.com/hello/world.
		// cmd/go expects a 404/410 response if there is nothing there. Hence we
		// cannot return with a 500.
		srv.logf("go proxy: no archive %s %s\n", path, vers)
		http.NotFound(w, r)
		return
	}

	switch ext {
	case "info", "mod":
		want := "." + ext
		for _, f := range a.Files {
			if f.Name == want {
				w.Write(f.Data)
				return
			}
		}

	case "zip":
		type cached struct {
			zip []byte
			err error
		}
		c := srv.zipCache.Do(a, func() interface{} {
			var buf bytes.Buffer
			z := zip.NewWriter(&buf)
			for _, f := range a.Files {
				if strings.HasPrefix(f.Name, ".") {
					continue
				}
				zf, err := z.Create(path + "@" + vers + "/" + f.Name)
				if err != nil {
					return cached{nil, err}
				}
				if _, err := zf.Write(f.Data); err != nil {
					return cached{nil, err}
				}
			}
			if err := z.Close(); err != nil {
				return cached{nil, err}
			}
			return cached{buf.Bytes(), nil}
		}).(cached)

		if c.err != nil {
			srv.logf("go proxy: %v\n", c.err)
			http.Error(w, c.err.Error(), 500)
			return
		}
		w.Write(c.zip)
		return

	}
	http.NotFound(w, r)
}

func (srv *Server) findHash(m module.Version) string {
	a := srv.readArchive(m.Path, m.Version)
	if a == nil {
		return ""
	}
	var data []byte
	for _, f := range a.Files {
		if f.Name == ".info" {
			data = f.Data
			break
		}
	}
	var info struct{ Short string }
	json.Unmarshal(data, &info)
	return info.Short
}

func (srv *Server) readArchive(path, vers string) *txtar.Archive {
	enc, err := module.EscapePath(path)
	if err != nil {
		srv.logf("go proxy: %v\n", err)
		return nil
	}
	encVers, err := module.EscapeVersion(vers)
	if err != nil {
		srv.logf("go proxy: %v\n", err)
		return nil
	}

	prefix := strings.ReplaceAll(enc, "/", "_")
	name := filepath.Join(srv.dir, prefix+"_"+encVers)
	txtName := name + ".txt"
	txtarName := name + ".txtar"
	a := srv.archiveCache.Do(name, func() interface{} {
		a, err := txtar.ParseFile(txtarName)
		if os.IsNotExist(err) {
			// fall back to trying with the .txt extension
			a, err = txtar.ParseFile(txtName)
		}
		if os.IsNotExist(err) {
			// fall back to trying a directory
			a = new(txtar.Archive)

			err = filepath.WalkDir(name, func(path string, entry fs.DirEntry, err error) error {
				if err != nil {
					return err
				}
				if path == name && !entry.IsDir() {
					return fmt.Errorf("expected a directory root")
				}
				if entry.IsDir() {
					return nil
				}
				arpath := filepath.ToSlash(strings.TrimPrefix(path, name+string(os.PathSeparator)))
				data, err := os.ReadFile(path)
				if err != nil {
					return err
				}
				a.Files = append(a.Files, txtar.File{
					Name: arpath,
					Data: data,
				})
				return nil
			})
		}
		if err != nil {
			if !os.IsNotExist(err) {
				srv.logf("go proxy: %v\n", err)
			}
			a = nil
		}
		return a
	}).(*txtar.Archive)
	return a
}
