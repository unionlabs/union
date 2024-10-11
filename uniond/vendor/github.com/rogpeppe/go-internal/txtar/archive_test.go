// Copyright 2018 The Go Authors. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package txtar

import (
	"bytes"
	"fmt"
	"reflect"
	"testing"
)

var tests = []struct {
	name   string
	text   string
	parsed *Archive
}{
	// General test
	{
		name: "basic",
		text: `comment1
comment2
-- file1 --
File 1 text.
-- foo ---
More file 1 text.
-- file 2 --
File 2 text.
-- empty --
-- noNL --
hello world`,
		parsed: &Archive{
			Comment: []byte("comment1\ncomment2\n"),
			Files: []File{
				{"file1", []byte("File 1 text.\n-- foo ---\nMore file 1 text.\n")},
				{"file 2", []byte("File 2 text.\n")},
				{"empty", []byte{}},
				{"noNL", []byte("hello world\n")},
			},
		},
	},
	// Test CRLF input
	{
		name: "basicCRLF",
		text: "blah\r\n-- hello --\r\nhello\r\n",
		parsed: &Archive{
			Comment: []byte("blah\r\n"),
			Files: []File{
				{"hello", []byte("hello\r\n")},
			},
		},
	},
}

func Test(t *testing.T) {
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			a := Parse([]byte(tt.text))
			if !reflect.DeepEqual(a, tt.parsed) {
				t.Fatalf("Parse: wrong output:\nhave:\n%s\nwant:\n%s", shortArchive(a), shortArchive(tt.parsed))
			}
			text := Format(a)
			a = Parse(text)
			if !reflect.DeepEqual(a, tt.parsed) {
				t.Fatalf("Parse after Format: wrong output:\nhave:\n%s\nwant:\n%s", shortArchive(a), shortArchive(tt.parsed))
			}
		})
	}
}

func shortArchive(a *Archive) string {
	var buf bytes.Buffer
	fmt.Fprintf(&buf, "comment: %q\n", a.Comment)
	for _, f := range a.Files {
		fmt.Fprintf(&buf, "file %q: %q\n", f.Name, f.Data)
	}
	return buf.String()
}

func TestWrite(t *testing.T) {
	td := t.TempDir()

	good := &Archive{Files: []File{File{Name: "good.txt"}}}
	if err := Write(good, td); err != nil {
		t.Fatalf("expected no error; got %v", err)
	}

	badRel := &Archive{Files: []File{File{Name: "../bad.txt"}}}
	want := `"../bad.txt": outside parent directory`
	if err := Write(badRel, td); err == nil || err.Error() != want {
		t.Fatalf("expected %v; got %v", want, err)
	}

	badAbs := &Archive{Files: []File{File{Name: "/bad.txt"}}}
	want = `"/bad.txt": outside parent directory`
	if err := Write(badAbs, td); err == nil || err.Error() != want {
		t.Fatalf("expected %v; got %v", want, err)
	}
}

var unquoteErrorTests = []struct {
	testName    string
	data        string
	expectError string
}{{
	testName:    "no final newline",
	data:        ">hello",
	expectError: `data does not appear to be quoted`,
}, {
	testName:    "no initial >",
	data:        "hello\n",
	expectError: `data does not appear to be quoted`,
}}

func TestUnquote(t *testing.T) {
	for _, test := range unquoteErrorTests {
		t.Run(test.testName, func(t *testing.T) {
			_, err := Unquote([]byte(test.data))
			if err == nil {
				t.Fatalf("unexpected success")
			}
			if err.Error() != test.expectError {
				t.Fatalf("unexpected error; got %q want %q", err, test.expectError)
			}
		})
	}
}

var quoteTests = []struct {
	testName    string
	data        string
	expect      string
	expectError string
}{{
	testName: "empty",
	data:     "",
	expect:   "",
}, {
	testName: "one line",
	data:     "foo\n",
	expect:   ">foo\n",
}, {
	testName: "several lines",
	data:     "foo\nbar\n-- baz --\n",
	expect:   ">foo\n>bar\n>-- baz --\n",
}, {
	testName:    "bad data",
	data:        "foo\xff\n",
	expectError: `data contains non-UTF-8 characters`,
}, {
	testName:    "no final newline",
	data:        "foo",
	expectError: `data has no final newline`,
}}

func TestQuote(t *testing.T) {
	for _, test := range quoteTests {
		t.Run(test.testName, func(t *testing.T) {
			got, err := Quote([]byte(test.data))
			if test.expectError != "" {
				if err == nil {
					t.Fatalf("unexpected success")
				}
				if err.Error() != test.expectError {
					t.Fatalf("unexpected error; got %q want %q", err, test.expectError)
				}
				return
			}
			if err != nil {
				t.Fatalf("quote error: %v", err)
			}
			if string(got) != test.expect {
				t.Fatalf("unexpected result; got %q want %q", got, test.expect)
			}
			orig, err := Unquote(got)
			if err != nil {
				t.Fatal(err)
			}
			if string(orig) != test.data {
				t.Fatalf("round trip failed; got %q want %q", orig, test.data)
			}
		})
	}
}
