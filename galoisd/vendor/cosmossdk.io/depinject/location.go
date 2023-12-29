// Copyright (c) 2019 Uber Technologies, Inc.
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

package depinject

import (
	"fmt"
	"net/url"
	"runtime"
	"strings"
)

type Location interface {
	isLocation()
	Name() string
	fmt.Stringer
	fmt.Formatter
}

type location struct {
	name string
	pkg  string
	file string
	line int
}

func LocationFromPC(pc uintptr) Location {
	f := runtime.FuncForPC(pc)
	pkgName, funcName := splitFuncName(f.Name())
	fileName, lineNum := f.FileLine(pc)
	return &location{
		name: funcName,
		pkg:  pkgName,
		file: fileName,
		line: lineNum,
	}
}

func LocationFromCaller(skip int) Location {
	pc, _, _, _ := runtime.Caller(skip + 1)
	return LocationFromPC(pc)
}

func (f *location) isLocation() {
	panic("implement me")
}

// String returns a string representation of the function.
func (f *location) String() string {
	return fmt.Sprint(f)
}

// Name is the fully qualified function name.
func (f *location) Name() string {
	return fmt.Sprintf("%v.%v", f.pkg, f.name)
}

// Format implements fmt.Formatter for Func, printing a single-line
// representation for %v and a multi-line one for %+v.
func (f *location) Format(w fmt.State, c rune) {
	if w.Flag('+') && c == 'v' {
		// "path/to/package".MyFunction
		// 	path/to/file.go:42
		_, _ = fmt.Fprintf(w, "%v.%v", f.pkg, f.name)
		_, _ = fmt.Fprintf(w, "\n\t%v:%v", f.file, f.line)
	} else {
		// "path/to/package".MyFunction (path/to/file.go:42)
		_, _ = fmt.Fprintf(w, "%v.%v (%v:%v)", f.pkg, f.name, f.file, f.line)
	}
}

const _vendor = "/vendor/"

func splitFuncName(function string) (pname, fname string) {
	if len(function) == 0 {
		return "", ""
	}

	// We have something like "path.to/my/pkg.MyFunction". If the function is
	// a closure, it is something like, "path.to/my/pkg.MyFunction.func1".

	idx := 0

	// Everything up to the first "." after the last "/" is the package name.
	// Everything after the "." is the full function name.
	if i := strings.LastIndex(function, "/"); i >= 0 {
		idx = i
	}
	if i := strings.Index(function[idx:], "."); i >= 0 {
		idx += i
	}
	pname, fname = function[:idx], function[idx+1:]

	// The package may be vendored.
	if i := strings.Index(pname, _vendor); i > 0 {
		pname = pname[i+len(_vendor):]
	}

	// Package names are URL-encoded to avoid ambiguity in the case where the
	// package name contains ".git". Otherwise, "foo/bar.git.MyFunction" would
	// mean that "git" is the top-level function and "MyFunction" is embedded
	// inside it.
	if unescaped, err := url.QueryUnescape(pname); err == nil {
		pname = unescaped
	}

	return pname, fname
}
