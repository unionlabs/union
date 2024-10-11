// Copyright 2019 The Cockroach Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied. See the License for the specific language governing
// permissions and limitations under the License.

package fmttests

import (
	"context"
	goErr "errors"
	"fmt"
	"os"
	"path/filepath"
	"regexp"
	"strings"
	"testing"

	"github.com/cockroachdb/errors/errbase"
	"github.com/cockroachdb/errors/errutil"
	"github.com/cockroachdb/errors/testutils"
	"github.com/cockroachdb/redact"
	"github.com/gogo/protobuf/proto"
	pkgErr "github.com/pkg/errors"
)

func TestFormatViaRedact(t *testing.T) {
	tt := testutils.T{t}

	sm := string(redact.StartMarker())
	em := string(redact.EndMarker())

	var nilErr error
	err := errutil.Newf("hello %s %v", "world", nilErr)

	tt.CheckEqual(string(redact.Sprintf("%v", err)), `hello `+sm+`world`+em+` <nil>`)
	tt.CheckEqual(string(redact.Sprintf("%v", errbase.Formattable(err))), `hello `+sm+`world`+em+` <nil>`)

	err = goErr.New("hello")
	expected := sm + `hello` + em + `
(1) ` + sm + `hello` + em + `
Error types: (1) *errors.errorString`
	tt.CheckEqual(string(redact.Sprintf("%+v", err)), expected)

	expected = sm + `hello` + em + `
(1)
Wraps: (2) ` + sm + `hello` + em + `
Error types: (1) *errbase.errorFormatter (2) *errors.errorString`
	tt.CheckEqual(string(redact.Sprintf("%+v", errbase.Formattable(err))), expected)

	// Regression test.
	f := &fmtWrap{err}
	tt.CheckEqual(string(redact.Sprintf("%v", f)), sm+`hello`+em)
	expected = sm + `hello` + em + `
` + sm + `(1) hello` + em + `
` + sm + `Error types: (1) *errors.errorString` + em
	tt.CheckEqual(string(redact.Sprintf("%+v", f)), expected)

	// Regression test 2.
	f2 := &fmter{}
	tt.CheckEqual(string(redact.Sprintf("%v", f2)), sm+`hello`+em)
	tt.CheckEqual(string(redact.Sprintf("%+v", f2)), sm+`hello`+em)

	// Another regression test.
	// https://github.com/cockroachdb/redact/issues/12
	var buf redact.StringBuilder
	buf.Printf("safe %v", "unsafe")
	e := errutil.Newf("%v", buf)
	tt.CheckEqual(string(redact.Sprint(e)), `safe `+sm+`unsafe`+em)
}

type fmtWrap struct {
	err error
}

// Format implements the fmt.Formatter interface.
func (ef *fmtWrap) Format(s fmt.State, verb rune) { errbase.FormatError(ef.err, s, verb) }

type fmter struct{}

// Format implements the fmt.Formatter interface.
func (ef *fmter) Format(s fmt.State, verb rune) { _, _ = s.Write([]byte("hello")) }

func TestFormat(t *testing.T) {
	tt := testutils.T{t}

	const woo = `woo`
	const waa = `waa`
	const mwoo = "woo\nother"
	const waawoo = `waa: woo`
	const wuuwaawoo = `wuu: waa: woo`
	testCases := []struct {
		name          string
		err           error
		expFmtSimple  string
		expFmtVerbose string
		// We're expecting the result of printing an error with %q
		// to be the quoted version of %s in the general case.
		// However, the implementation of (*pkgErr.withMessage).Format()
		// gets this wrong, so we need a separate ref string for this
		// specific case.
		expFmtQuote string
	}{
		{"nofmt wrap in + nofmt wrap out",
			&werrNoFmt{&werrNoFmt{&errFmt{woo}, waa}, "wuu"},
			wuuwaawoo, wuuwaawoo, ``},

		{"nofmt wrap in + fmd-old wrap out",
			&werrFmto{&werrNoFmt{&errFmt{woo}, waa}, "wuu"},
			wuuwaawoo, `
waa: woo
-- this is wuu's
multi-line payload (fmt)`, ``,
		},

		{"nofmt wrap in + fmt wrap out",
			&werrFmt{&werrNoFmt{&errFmt{woo}, waa}, "wuu"},
			wuuwaawoo, `
wuu: waa: woo
(1) wuu
  | -- this is wuu's
  | multi-line wrapper payload
Wraps: (2) waa
Wraps: (3) woo
  | -- this is woo's
  | multi-line leaf payload
Error types: (1) *fmttests.werrFmt (2) *fmttests.werrNoFmt (3) *fmttests.errFmt`, ``,
		},

		{"fmt-old wrap in + nofmt wrap out",
			&werrNoFmt{&werrFmto{&errFmt{woo}, waa}, "wuu"},
			wuuwaawoo, wuuwaawoo, ``},

		{"fmt-old wrap in + fmd-old wrap out",
			&werrFmto{&werrFmto{&errFmt{woo}, waa}, "wuu"},
			wuuwaawoo, `
woo
(1) woo
  | -- this is woo's
  | multi-line leaf payload
Error types: (1) *fmttests.errFmt
-- this is waa's
multi-line payload (fmt)
-- this is wuu's
multi-line payload (fmt)`, ``,
		},

		{"fmt-old wrap in + fmt wrap out",
			&werrFmt{&werrFmto{&errFmt{woo}, waa}, "wuu"},
			wuuwaawoo, `
wuu: waa: woo
(1) wuu
  | -- this is wuu's
  | multi-line wrapper payload
Wraps: (2) waa
Wraps: (3) woo
  | -- this is woo's
  | multi-line leaf payload
Error types: (1) *fmttests.werrFmt (2) *fmttests.werrFmto (3) *fmttests.errFmt`, ``,
		},

		{"fmt wrap in + nofmt wrap out",
			&werrNoFmt{&werrFmt{&errFmt{woo}, waa}, "wuu"},
			wuuwaawoo, wuuwaawoo, ``},

		{"fmt wrap in + fmd-old wrap out",
			&werrFmto{&werrFmt{&errFmt{woo}, waa}, "wuu"},
			wuuwaawoo, `
waa: woo
(1) waa
  | -- this is waa's
  | multi-line wrapper payload
Wraps: (2) woo
  | -- this is woo's
  | multi-line leaf payload
Error types: (1) *fmttests.werrFmt (2) *fmttests.errFmt
-- this is wuu's
multi-line payload (fmt)`, ``,
		},

		{"fmt wrap in + fmt wrap out",
			&werrFmt{&werrFmt{&errFmt{woo}, waa}, "wuu"},
			wuuwaawoo, `
wuu: waa: woo
(1) wuu
  | -- this is wuu's
  | multi-line wrapper payload
Wraps: (2) waa
  | -- this is waa's
  | multi-line wrapper payload
Wraps: (3) woo
  | -- this is woo's
  | multi-line leaf payload
Error types: (1) *fmttests.werrFmt (2) *fmttests.werrFmt (3) *fmttests.errFmt`, ``,
		},

		// Compatibility with github.com/pkg/errors.

		{"fmt wrap + pkg msg + fmt leaf",
			&werrFmt{pkgErr.WithMessage(&errFmt{woo}, waa), "wuu"},
			wuuwaawoo, `
wuu: waa: woo
(1) wuu
  | -- this is wuu's
  | multi-line wrapper payload
Wraps: (2) waa
Wraps: (3) woo
  | -- this is woo's
  | multi-line leaf payload
Error types: (1) *fmttests.werrFmt (2) *errors.withMessage (3) *fmttests.errFmt`, ``,
		},

		{"fmt wrap + pkg msg1 + pkg.msg2 + fmt leaf",
			&werrFmt{
				pkgErr.WithMessage(
					pkgErr.WithMessage(
						&errFmt{woo}, "waa2"),
					"waa1"),
				"wuu"},
			`wuu: waa1: waa2: woo`, `
wuu: waa1: waa2: woo
(1) wuu
  | -- this is wuu's
  | multi-line wrapper payload
Wraps: (2) waa1
Wraps: (3) waa2
Wraps: (4) woo
  | -- this is woo's
  | multi-line leaf payload
Error types: (1) *fmttests.werrFmt (2) *errors.withMessage (3) *errors.withMessage (4) *fmttests.errFmt`, ``,
		},

		{"fmt wrap + pkg stack + fmt leaf",
			&werrFmt{pkgErr.WithStack(&errFmt{woo}), waa},
			waawoo, `
waa: woo
(1) waa
  | -- this is waa's
  | multi-line wrapper payload
Wraps: (2)
  -- stack trace:
  | github.com/cockroachdb/errors/fmttests.TestFormat
  | <tab><path>:<lineno>
  | testing.tRunner
  | <tab><path>:<lineno>
  | runtime.goexit
  | <tab><path>:<lineno>
Wraps: (3) woo
  | -- this is woo's
  | multi-line leaf payload
Error types: (1) *fmttests.werrFmt (2) *errors.withStack (3) *fmttests.errFmt`, ``,
		},

		{"delegating wrap + pkg stack + fmt leaf",
			&werrDelegate{pkgErr.WithStack(&errFmt{woo}), "prefix"},
			"prefix: woo", `
prefix: woo
(1) prefix
  | -- multi-line
  | wrapper payload
Wraps: (2)
  -- stack trace:
  | github.com/cockroachdb/errors/fmttests.TestFormat
  | <tab><path>:<lineno>
  | testing.tRunner
  | <tab><path>:<lineno>
  | runtime.goexit
  | <tab><path>:<lineno>
Wraps: (3) woo
  | -- this is woo's
  | multi-line leaf payload
Error types: (1) *fmttests.werrDelegate (2) *errors.withStack (3) *fmttests.errFmt`, ``,
		},

		{"empty wrap + pkg stack + fmt leaf",
			&werrEmpty{pkgErr.WithStack(&errFmt{woo})},
			woo, `
woo
(1)
Wraps: (2)
  -- stack trace:
  | github.com/cockroachdb/errors/fmttests.TestFormat
  | <tab><path>:<lineno>
  | testing.tRunner
  | <tab><path>:<lineno>
  | runtime.goexit
  | <tab><path>:<lineno>
Wraps: (3) woo
  | -- this is woo's
  | multi-line leaf payload
Error types: (1) *fmttests.werrEmpty (2) *errors.withStack (3) *fmttests.errFmt`, ``,
		},

		{"empty delegate wrap + pkg stack + fmt leaf",
			&werrDelegateEmpty{pkgErr.WithStack(&errFmt{woo})},
			woo, `
woo
(1)
Wraps: (2)
  -- stack trace:
  | github.com/cockroachdb/errors/fmttests.TestFormat
  | <tab><path>:<lineno>
  | testing.tRunner
  | <tab><path>:<lineno>
  | runtime.goexit
  | <tab><path>:<lineno>
Wraps: (3) woo
  | -- this is woo's
  | multi-line leaf payload
Error types: (1) *fmttests.werrDelegateEmpty (2) *errors.withStack (3) *fmttests.errFmt`, ``,
		},

		{"safeformatter leaf",
			&errSafeFormat{msg: "world"},
			`safe world`, `
safe world
(1) safe world
Error types: (1) *fmttests.errSafeFormat`, ``},
	}

	for _, test := range testCases {
		tt.Run(test.name, func(tt testutils.T) {
			err := test.err

			// %s is simple formatting
			tt.CheckStringEqual(fmt.Sprintf("%s", err), test.expFmtSimple)

			// %v is simple formatting too, for compatibility with the past.
			tt.CheckStringEqual(fmt.Sprintf("%v", err), test.expFmtSimple)

			// %q is always like %s but quotes the result.
			ref := test.expFmtQuote
			if ref == "" {
				ref = fmt.Sprintf("%q", test.expFmtSimple)
			}
			tt.CheckStringEqual(fmt.Sprintf("%q", err), ref)

			// %+v is the verbose mode.
			refV := strings.TrimPrefix(test.expFmtVerbose, "\n")
			spv := fmtClean(fmt.Sprintf("%+v", err))
			tt.CheckStringEqual(spv, refV)
		})
	}
}

func TestHelperForErrorf(t *testing.T) {
	origErr := goErr.New("small\nuniverse")
	s, e := redact.HelperForErrorf("hello %s", origErr)
	if actual, expected := string(s), "hello ‹small›\n‹universe›"; actual != expected {
		t.Errorf("expected %q, got %q", expected, actual)
	}
	if e != nil {
		t.Errorf("expected no error, got %v", e)
	}

	s, e = redact.HelperForErrorf("hello %w", origErr)
	if actual, expected := string(s), "hello ‹small›\n‹universe›"; actual != expected {
		t.Errorf("expected %q, got %q", expected, actual)
	}
	if e != origErr {
		t.Errorf("expected error %v, got %v (%T)", origErr, e, e)
	}
}

func fmtClean(spv string) string {
	spv = fileref.ReplaceAllString(spv, "<path>:<lineno>")
	spv = libref.ReplaceAllString(spv, "<path>")
	spv = stackref.ReplaceAllString(spv, `&stack{...}`)
	spv = hexref.ReplaceAllString(spv, "0xAAAABBBB")
	spv = strings.ReplaceAll(spv, "\t", "<tab>")
	spv = funcNN.ReplaceAllString(spv, `...funcNN...`)

	// When running the tests with a Go version before 1.16,
	// the reference test output wrt fs.PathError will not match what the
	// Go runtime thinks (os.PathError was a separate type).
	//
	// In that case, we translate the test output as per the old
	// runtime into the shape expected in 1.16 or later.
	spv = fakeGo116(spv)

	return spv
}

var funcNN = regexp.MustCompile(`(?m)((\.\.func\d+| func\d+\(\)|\(func\d+\))"?$)|(\.\.func\d+\\n)`)

var stackref = regexp.MustCompile(`(&(?:errors\.stack|withstack\.stack)\{[^}]*\})`)
var fileref = regexp.MustCompile(`(` +
	// Any path ending with .{go,s}:NNN:
	`[a-zA-Z0-9\._/@-]+\.(?:go|s):\d+` +
	`)`)
var libroot = func() string {
	wd, err := os.Getwd()
	if err != nil {
		panic(err)
	}
	return filepath.Clean(filepath.Join(wd, ".."))
}()
var libref = regexp.MustCompile(
	`((` +
		// Any path starting with the errors library root directory:
		libroot +
		`|` +
		// Any path containing the error library:
		`(/[a-zA-Z0-9\._/@-]+)+` +
		`/github.com/cockroachdb/errors` +
		`)` +
		// Followed by some directory components:
		`(/[a-zA-Z0-9\._/@-]+)*` +
		`)`)
var hexref = regexp.MustCompile(`(0x[a-f0-9]{4,})`)

// errNoFmt does neither implement Format() nor FormatError().
type errNoFmt struct{ msg string }

func (e *errNoFmt) Error() string { return e.msg }

// werrNoFmt does like errNoFmt. This is used to check
// that FormatError() knows about "dumb" error wrappers.
type werrNoFmt struct {
	cause error
	msg   string
}

func (e *werrNoFmt) Error() string { return fmt.Sprintf("%s: %v", e.msg, e.cause) }
func (e *werrNoFmt) Unwrap() error { return e.cause }

// errFmto has just a Format() method that does everything, and does
// not know about neither FormatError() nor errors.Formatter.
// This is the "old style" support for formatting, e.g. used
// in github.com/pkg/errors.
type errFmto struct{ msg string }

func (e *errFmto) Error() string { return e.msg }
func (e *errFmto) Format(s fmt.State, verb rune) {
	switch verb {
	case 'v':
		if s.Flag('+') {
			fmt.Fprint(s, e.msg)
			fmt.Fprintf(s, "\n-- this is %s's\nmulti-line payload", e.msg)
			return
		}
		fallthrough
	default:
		fmt.Fprintf(s, fmt.Sprintf("%%%s%c", flags(s), verb), e.msg)
	}
}

// errFmtoDelegate is like errFmto but the Error() method delegates to
// Format().
type errFmtoDelegate struct{ msg string }

func (e *errFmtoDelegate) Error() string { return fmt.Sprintf("%v", e) }
func (e *errFmtoDelegate) Format(s fmt.State, verb rune) {
	switch verb {
	case 'v':
		if s.Flag('+') {
			fmt.Fprint(s, e.msg)
			fmt.Fprintf(s, "\n-- this is %s's\nmulti-line payload", e.msg)
			return
		}
		fallthrough
	default:
		fmt.Fprintf(s, fmt.Sprintf("%%%s%c", flags(s), verb), e.msg)
	}
}

// werrFmto is like errFmto but is a wrapper.
type werrFmto struct {
	cause error
	msg   string
}

func (e *werrFmto) Error() string { return fmt.Sprintf("%s: %v", e.msg, e.cause) }
func (e *werrFmto) Unwrap() error { return e.cause }
func (e *werrFmto) Format(s fmt.State, verb rune) {
	switch verb {
	case 'v':
		if s.Flag('+') {
			fmt.Fprintf(s, "%+v", e.cause)
			fmt.Fprintf(s, "\n-- this is %s's\nmulti-line payload (fmt)", e.msg)
			return
		}
		fallthrough
	default:
		fmt.Fprintf(s, fmt.Sprintf("%%%s%c", flags(s), verb), e.Error())
	}
}

// werrFmtoDelegate is like errFmtoDelegate but is a wrapper.
type werrFmtoDelegate struct {
	cause error
	msg   string
}

func (e *werrFmtoDelegate) Error() string { return fmt.Sprintf("%v", e) }
func (e *werrFmtoDelegate) Unwrap() error { return e.cause }
func (e *werrFmtoDelegate) Format(s fmt.State, verb rune) {
	switch verb {
	case 'v':
		if s.Flag('+') {
			fmt.Fprintf(s, "%+v", e.cause)
			fmt.Fprintf(s, "\n-- this is %s's\nmulti-line wrapper payload", e.msg)
			return
		}
		fallthrough
	default:
		fmts := fmt.Sprintf("%%%s%c", flags(s), verb)
		fmt.Fprintf(s, fmts, e.msg+": "+e.cause.Error())
	}
}

// errFmtp implements Format() that forwards to FormatError(),
// but does not implement errors.Formatter. It is used
// to check that FormatError() does the right thing.
type errFmtp struct {
	msg string
}

func (e *errFmtp) Error() string                 { return e.msg }
func (e *errFmtp) Format(s fmt.State, verb rune) { errbase.FormatError(e, s, verb) }

// werrFmtp is like errFmtp but is a wrapper.
type werrFmtp struct {
	cause error
	msg   string
}

func (e *werrFmtp) Error() string                 { return fmt.Sprintf("%s: %v", e.msg, e.cause) }
func (e *werrFmtp) Unwrap() error                 { return e.cause }
func (e *werrFmtp) Format(s fmt.State, verb rune) { errbase.FormatError(e, s, verb) }

// errFmt has both Format() and FormatError(),
// and demonstrates the common case of "rich" errors.
type errFmt struct{ msg string }

func (e *errFmt) Error() string                 { return e.msg }
func (e *errFmt) Format(s fmt.State, verb rune) { errbase.FormatError(e, s, verb) }
func (e *errFmt) FormatError(p errbase.Printer) error {
	p.Print(e.msg)
	if p.Detail() {
		p.Printf("-- this is %s's\nmulti-line leaf payload", e.msg)
	}
	return nil
}

// werrFmt is like errFmt but is a wrapper.
type werrFmt struct {
	cause error
	msg   string
}

func (e *werrFmt) Error() string                 { return fmt.Sprintf("%s: %v", e.msg, e.cause) }
func (e *werrFmt) Unwrap() error                 { return e.cause }
func (e *werrFmt) Format(s fmt.State, verb rune) { errbase.FormatError(e, s, verb) }
func (e *werrFmt) FormatError(p errbase.Printer) error {
	p.Print(e.msg)
	if p.Detail() {
		p.Printf("-- this is %s's\nmulti-line wrapper payload", e.msg)
	}
	return e.cause
}

func flags(s fmt.State) string {
	flags := ""
	if s.Flag('#') {
		flags += "#"
	}
	if s.Flag('+') {
		flags += "+"
	}
	if s.Flag(' ') {
		flags += " "
	}
	return flags
}

// TestDelegateProtocol checks that there is no infinite recursion
// when Error() delegates its behavior to FormatError().
func TestDelegateProtocol(t *testing.T) {
	tt := testutils.T{t}

	var err error
	err = &werrDelegate{&errNoFmt{"woo"}, "prefix"}
	tt.CheckStringEqual(fmt.Sprintf("%v", err), "prefix: woo")

	err = &werrDelegateNoPrefix{&errNoFmt{"woo"}}
	tt.CheckStringEqual(fmt.Sprintf("%v", err), "woo")
}

// werrDelegate delegates its Error() behavior to FormatError().
type werrDelegate struct {
	wrapped error
	msg     string
}

var _ fmt.Formatter = (*werrDelegate)(nil)
var _ errbase.Formatter = (*werrDelegate)(nil)

func (e *werrDelegate) Error() string                 { return fmt.Sprintf("%v", e) }
func (e *werrDelegate) Cause() error                  { return e.wrapped }
func (e *werrDelegate) Format(s fmt.State, verb rune) { errbase.FormatError(e, s, verb) }
func (e *werrDelegate) FormatError(p errbase.Printer) error {
	p.Print(e.msg)
	if p.Detail() {
		p.Print("-- multi-line\nwrapper payload")
	}
	return e.wrapped
}

// werrDelegateNoPrefix delegates its Error() behavior to FormatError()
// via fmt.Format, has no prefix of its own in its short message
// but has a detail field.
type werrDelegateNoPrefix struct {
	wrapped error
}

var _ errbase.Formatter = (*werrDelegateNoPrefix)(nil)
var _ fmt.Formatter = (*werrDelegateNoPrefix)(nil)

func (e *werrDelegateNoPrefix) Error() string                 { return fmt.Sprintf("%v", e) }
func (e *werrDelegateNoPrefix) Cause() error                  { return e.wrapped }
func (e *werrDelegateNoPrefix) Format(s fmt.State, verb rune) { errbase.FormatError(e, s, verb) }
func (e *werrDelegateNoPrefix) FormatError(p errbase.Printer) error {
	if p.Detail() {
		p.Print("detail")
	}
	return e.wrapped
}

// werrDelegateEmpty implements Error via fmt.Formatter using FormatError,
// and has no message nor detail of its own.
type werrDelegateEmpty struct {
	wrapped error
}

var _ errbase.Formatter = (*werrDelegateEmpty)(nil)
var _ fmt.Formatter = (*werrDelegateEmpty)(nil)

func (e *werrDelegateEmpty) Error() string                 { return fmt.Sprintf("%v", e) }
func (e *werrDelegateEmpty) Cause() error                  { return e.wrapped }
func (e *werrDelegateEmpty) Format(s fmt.State, verb rune) { errbase.FormatError(e, s, verb) }
func (e *werrDelegateEmpty) FormatError(p errbase.Printer) error {
	return e.wrapped
}

// werrEmpty has no message of its own.
type werrEmpty struct {
	wrapped error
}

var _ error = (*werrEmpty)(nil)
var _ fmt.Formatter = (*werrEmpty)(nil)

func (e *werrEmpty) Error() string                 { return e.wrapped.Error() }
func (e *werrEmpty) Cause() error                  { return e.wrapped }
func (e *werrEmpty) Format(s fmt.State, verb rune) { errbase.FormatError(e, s, verb) }

// werrWithElidedClause overrides its cause's Error() from its own
// short message.
type werrWithElidedCause struct {
	wrapped error
	msg     string
}

func (e *werrWithElidedCause) Error() string                 { return fmt.Sprintf("%v", e) }
func (e *werrWithElidedCause) Cause() error                  { return e.wrapped }
func (e *werrWithElidedCause) Format(s fmt.State, verb rune) { errbase.FormatError(e, s, verb) }
func (e *werrWithElidedCause) FormatError(p errbase.Printer) error {
	p.Print(e.msg)
	return nil
}

// Preserve the type over the network, otherwise the opaqueWrapper
// takes over, and that does not respect the elision.
func encodeWithElidedCause(
	_ context.Context, err error,
) (prefix string, _ []string, _ proto.Message, _ errbase.MessageType) {
	m := err.(*werrWithElidedCause)
	return m.msg, nil, nil, errbase.FullMessage
}

func decodeWithElidedCause(
	_ context.Context, cause error, msg string, _ []string, _ proto.Message,
) error {
	return &werrWithElidedCause{cause, msg}
}

func init() {
	errbase.RegisterWrapperEncoderWithMessageType(errbase.GetTypeKey(&werrWithElidedCause{}), encodeWithElidedCause)
	errbase.RegisterWrapperDecoder(errbase.GetTypeKey(&werrWithElidedCause{}), decodeWithElidedCause)
}

type werrMigrated struct {
	cause error
}

func (w *werrMigrated) Error() string                 { return w.cause.Error() }
func (w *werrMigrated) Cause() error                  { return w.cause }
func (w *werrMigrated) Format(s fmt.State, verb rune) { errbase.FormatError(w, s, verb) }

func init() {
	errbase.RegisterTypeMigration("some/previous/path", "prevpkg.prevType", (*werrMigrated)(nil))
}

type errSafeFormat struct {
	msg string
}

func (e *errSafeFormat) Error() string                 { return fmt.Sprint(e) }
func (e *errSafeFormat) Format(s fmt.State, verb rune) { errbase.FormatError(e, s, verb) }
func (e *errSafeFormat) SafeFormatError(p errbase.Printer) (next error) {
	p.Printf("safe %s", e.msg)
	return nil
}

type werrSafeFormat struct {
	cause error
	msg   string
}

func (w *werrSafeFormat) Cause() error                  { return w.cause }
func (w *werrSafeFormat) Error() string                 { return fmt.Sprint(w) }
func (w *werrSafeFormat) Format(s fmt.State, verb rune) { errbase.FormatError(w, s, verb) }
func (w *werrSafeFormat) SafeFormatError(p errbase.Printer) (next error) {
	p.Printf("safe %s", w.msg)
	return w.cause
}

type errMultiCause struct {
	causes []error
	msg    string
	elide  bool
}

func newMultiCause(msg string, elide bool, causes ...error) *errMultiCause {
	return &errMultiCause{
		causes: causes,
		msg:    msg,
		elide:  elide,
	}
}

func (e *errMultiCause) Error() string                 { return fmt.Sprint(e) }
func (e *errMultiCause) Format(s fmt.State, verb rune) { errbase.FormatError(e, s, verb) }
func (e *errMultiCause) SafeFormatError(p errbase.Printer) (next error) {
	p.Printf("%s", e.msg)
	if e.elide {
		return nil
	} else {
		return e.causes[0]
	}
}
func (e *errMultiCause) Unwrap() []error { return e.causes }

func init() {
	errbase.RegisterMultiCauseEncoder(errbase.GetTypeKey(&errMultiCause{}), encodeWithMultiCause)
	errbase.RegisterMultiCauseDecoder(errbase.GetTypeKey(&errMultiCause{}), decodeWithMultiCause)
}

func encodeWithMultiCause(
	_ context.Context, err error,
) (string, []string, proto.Message) {
	m := err.(*errMultiCause)
	if m.elide {
		return m.msg, []string{"elide"}, nil
	} else {
		return m.msg, nil, nil
	}
}

func decodeWithMultiCause(
	_ context.Context, causes []error, msg string, details []string, _ proto.Message,
) error {
	elide := false
	if len(details) == 1 && details[0] == "elide" {
		elide = true
	}
	return &errMultiCause{causes, msg, elide}
}
