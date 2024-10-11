// Copyright (c) 2021 Uber Technologies, Inc.
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

package multierr_test

import (
	"fmt"
	"log"
	"os"
	"path/filepath"

	"go.uber.org/multierr"
)

func ExampleAppendInvoke() {
	if err := run(); err != nil {
		log.Fatal(err)
	}
}

func run() (err error) {
	dir, err := os.MkdirTemp("", "multierr")
	// We create a temporary directory and defer its deletion when this
	// function returns.
	//
	// If we failed to delete the temporary directory, we append its
	// failure into the returned value with multierr.AppendInvoke.
	//
	// This uses a custom invoker that we implement below.
	defer multierr.AppendInvoke(&err, RemoveAll(dir))

	path := filepath.Join(dir, "example.txt")
	f, err := os.Create(path)
	if err != nil {
		return err
	}
	// Similarly, we defer closing the open file when the function returns,
	// and appends its failure, if any, into the returned error.
	//
	// This uses the multierr.Close invoker included in multierr.
	defer multierr.AppendInvoke(&err, multierr.Close(f))

	if _, err := fmt.Fprintln(f, "hello"); err != nil {
		return err
	}

	return nil
}

// RemoveAll is a multierr.Invoker that deletes the provided directory and all
// of its contents.
type RemoveAll string

func (r RemoveAll) Invoke() error {
	return os.RemoveAll(string(r))
}
