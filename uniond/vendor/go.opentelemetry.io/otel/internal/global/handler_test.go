// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

package global

import (
	"bytes"
	"errors"
	"log"
	"os"
	"strings"
	"testing"
)

func TestErrDelegator(t *testing.T) {
	buf := new(bytes.Buffer)
	log.Default().SetOutput(buf)
	t.Cleanup(func() { log.Default().SetOutput(os.Stderr) })

	e := &ErrDelegator{}

	err := errors.New("testing")
	e.Handle(err)

	got := buf.String()
	if !strings.Contains(got, err.Error()) {
		t.Error("default handler did not log")
	}
	buf.Reset()

	var gotErr error
	e.setDelegate(fnErrHandler(func(e error) { gotErr = e }))
	e.Handle(err)

	if buf.String() != "" {
		t.Error("delegate not set")
	} else if !errors.Is(gotErr, err) {
		t.Error("error not passed to delegate")
	}
}
