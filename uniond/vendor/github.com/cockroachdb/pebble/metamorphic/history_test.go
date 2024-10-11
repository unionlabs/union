// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package metamorphic

import (
	"bytes"
	"fmt"
	"regexp"
	"strings"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/pmezard/go-difflib/difflib"
	"github.com/stretchr/testify/require"
)

func TestHistoryLogger(t *testing.T) {
	var buf bytes.Buffer
	h := newHistory(nil, &buf)
	h.Infof("hello\nworld\n")
	h.Fatalf("hello\n\nworld")

	expected := `// INFO: hello
// INFO: world
// FATAL: hello
// FATAL: 
// FATAL: world
`
	if actual := buf.String(); expected != actual {
		t.Fatalf("expected\n%s\nbut found\n%s", expected, actual)
	}
}

func TestHistoryFail(t *testing.T) {
	var buf bytes.Buffer
	h := newHistory(regexp.MustCompile("foo"), &buf)
	h.Recordf(1, "bar")
	require.NoError(t, h.Error())
	h.Recordf(2, "foo bar")
	require.EqualError(t, h.Error(), `failure regexp "foo" matched output: foo bar #2`)
}

func TestReorderHistory(t *testing.T) {
	datadriven.RunTest(t, "testdata/reorder_history", func(t *testing.T, d *datadriven.TestData) string {
		switch d.Cmd {
		case "reorder":
			lines := difflib.SplitLines(string(d.Input))
			lines = reorderHistory(lines)
			return strings.Join(lines, "")
		default:
			return fmt.Sprintf("unknown command: %s", d.Cmd)
		}
	})
}
