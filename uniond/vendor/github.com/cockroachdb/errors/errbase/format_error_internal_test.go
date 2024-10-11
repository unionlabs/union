// Copyright 2020 The Cockroach Authors.
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

package errbase

import (
	goErr "errors"
	"fmt"
	"strings"
	"testing"

	"github.com/cockroachdb/redact"
)

type wrapMini struct {
	msg   string
	cause error
}

func (e *wrapMini) Error() string {
	return e.msg
}

func (e *wrapMini) Unwrap() error {
	return e.cause
}

type wrapElideCauses struct {
	override string
	causes   []error
}

func NewWrapElideCauses(override string, errors ...error) error {
	return &wrapElideCauses{
		override: override,
		causes:   errors,
	}
}

func (e *wrapElideCauses) Unwrap() []error {
	return e.causes
}

func (e *wrapElideCauses) SafeFormatError(p Printer) (next error) {
	p.Print(e.override)
	// Returning nil elides errors from remaining causal chain in the
	// implementation of `formatErrorInternal`.
	return nil
}

var _ SafeFormatter = &wrapElideCauses{}

func (e *wrapElideCauses) Error() string {
	b := strings.Builder{}
	b.WriteString(e.override)
	b.WriteString(": ")
	for i, ee := range e.causes {
		b.WriteString(ee.Error())
		if i < len(e.causes)-1 {
			b.WriteByte(' ')
		}
	}
	return b.String()
}

type wrapNoElideCauses struct {
	prefix string
	causes []error
}

func NewWrapNoElideCauses(prefix string, errors ...error) error {
	return &wrapNoElideCauses{
		prefix: prefix,
		causes: errors,
	}
}

func (e *wrapNoElideCauses) Unwrap() []error {
	return e.causes
}

func (e *wrapNoElideCauses) SafeFormatError(p Printer) (next error) {
	p.Print(e.prefix)
	return e.causes[0]
}

var _ SafeFormatter = &wrapNoElideCauses{}

func (e *wrapNoElideCauses) Error() string {
	b := strings.Builder{}
	b.WriteString(e.prefix)
	b.WriteString(": ")
	for i, ee := range e.causes {
		b.WriteString(ee.Error())
		if i < len(e.causes)-1 {
			b.WriteByte(' ')
		}
	}
	return b.String()
}

// TestFormatErrorInternal attempts to highlight some idiosyncrasies of
// the error formatting especially when used with multi-cause error
// structures. Comments on specific cases below outline some gaps that
// still require formatting tweaks.
func TestFormatErrorInternal(t *testing.T) {
	tests := []struct {
		name            string
		err             error
		expectedSimple  string
		expectedVerbose string
	}{
		{
			name:           "single wrapper",
			err:            fmt.Errorf("%w", fmt.Errorf("a%w", goErr.New("b"))),
			expectedSimple: "ab",
			expectedVerbose: `ab
(1)
Wraps: (2) ab
Wraps: (3) b
Error types: (1) *fmt.wrapError (2) *fmt.wrapError (3) *errors.errorString`,
		},
		{
			name:           "simple multi-wrapper",
			err:            goErr.Join(goErr.New("a"), goErr.New("b")),
			expectedSimple: "a\nb",
			// TODO(davidh): verbose test case should have line break
			// between `a` and `b` on second line.
			expectedVerbose: `a
(1) ab
Wraps: (2) b
Wraps: (3) a
Error types: (1) *errors.joinError (2) *errors.errorString (3) *errors.errorString`,
		},
		{
			name: "multi-wrapper with custom formatter and partial elide",
			err: NewWrapNoElideCauses("A",
				NewWrapNoElideCauses("C", goErr.New("3"), goErr.New("4")),
				NewWrapElideCauses("B", goErr.New("1"), goErr.New("2")),
			),
			expectedSimple: `A: B: C: 4: 3`, // 1 and 2 omitted because they are elided.
			expectedVerbose: `A: B: C: 4: 3
(1) A
Wraps: (2) B
└─ Wraps: (3) 2
└─ Wraps: (4) 1
Wraps: (5) C
└─ Wraps: (6) 4
└─ Wraps: (7) 3
Error types: (1) *errbase.wrapNoElideCauses (2) *errbase.wrapElideCauses (3) *errors.errorString (4) *errors.errorString (5) *errbase.wrapNoElideCauses (6) *errors.errorString (7) *errors.errorString`,
		},
		{
			name: "multi-wrapper with custom formatter and no elide",
			// All errors in this example omit eliding their children.
			err: NewWrapNoElideCauses("A",
				NewWrapNoElideCauses("B", goErr.New("1"), goErr.New("2")),
				NewWrapNoElideCauses("C", goErr.New("3"), goErr.New("4")),
			),
			expectedSimple: `A: C: 4: 3: B: 2: 1`,
			expectedVerbose: `A: C: 4: 3: B: 2: 1
(1) A
Wraps: (2) C
└─ Wraps: (3) 4
└─ Wraps: (4) 3
Wraps: (5) B
└─ Wraps: (6) 2
└─ Wraps: (7) 1
Error types: (1) *errbase.wrapNoElideCauses (2) *errbase.wrapNoElideCauses (3) *errors.errorString (4) *errors.errorString (5) *errbase.wrapNoElideCauses (6) *errors.errorString (7) *errors.errorString`,
		},
		{
			name:           "simple multi-line error",
			err:            goErr.New("a\nb\nc\nd"),
			expectedSimple: "a\nb\nc\nd",
			// TODO(davidh): verbose test case should preserve all 3
			// linebreaks in original error.
			expectedVerbose: `a
(1) ab
  |
  | c
  | d
Error types: (1) *errors.errorString`,
		},
		{
			name: "two-level multi-wrapper",
			err: goErr.Join(
				goErr.Join(goErr.New("a"), goErr.New("b")),
				goErr.Join(goErr.New("c"), goErr.New("d")),
			),
			expectedSimple: "a\nb\nc\nd",
			// TODO(davidh): verbose output should preserve line breaks after (1)
			// and also after (2) and (5) in `c\nd` and `a\nb`.
			expectedVerbose: `a
(1) ab
  |
  | c
  | d
Wraps: (2) cd
└─ Wraps: (3) d
└─ Wraps: (4) c
Wraps: (5) ab
└─ Wraps: (6) b
└─ Wraps: (7) a
Error types: (1) *errors.joinError (2) *errors.joinError (3) *errors.errorString (4) *errors.errorString (5) *errors.joinError (6) *errors.errorString (7) *errors.errorString`,
		},
		{
			name: "simple multi-wrapper with single-cause chains inside",
			err: goErr.Join(
				fmt.Errorf("a%w", goErr.New("b")),
				fmt.Errorf("c%w", goErr.New("d")),
			),
			expectedSimple: "ab\ncd",
			expectedVerbose: `ab
(1) ab
  | cd
Wraps: (2) cd
└─ Wraps: (3) d
Wraps: (4) ab
└─ Wraps: (5) b
Error types: (1) *errors.joinError (2) *fmt.wrapError (3) *errors.errorString (4) *fmt.wrapError (5) *errors.errorString`,
		},
		{
			name: "multi-cause wrapper with single-cause chains inside",
			err: goErr.Join(
				fmt.Errorf("a%w", fmt.Errorf("b%w", fmt.Errorf("c%w", goErr.New("d")))),
				fmt.Errorf("e%w", fmt.Errorf("f%w", fmt.Errorf("g%w", goErr.New("h")))),
			),
			expectedSimple: `abcd
efgh`,
			expectedVerbose: `abcd
(1) abcd
  | efgh
Wraps: (2) efgh
└─ Wraps: (3) fgh
  └─ Wraps: (4) gh
    └─ Wraps: (5) h
Wraps: (6) abcd
└─ Wraps: (7) bcd
  └─ Wraps: (8) cd
    └─ Wraps: (9) d
Error types: (1) *errors.joinError (2) *fmt.wrapError (3) *fmt.wrapError (4) *fmt.wrapError (5) *errors.errorString (6) *fmt.wrapError (7) *fmt.wrapError (8) *fmt.wrapError (9) *errors.errorString`},
		{
			name: "single cause chain with multi-cause wrapper inside with single-cause chains inside",
			err: fmt.Errorf(
				"prefix1: %w",
				fmt.Errorf(
					"prefix2: %w",
					goErr.Join(
						fmt.Errorf("a%w", fmt.Errorf("b%w", fmt.Errorf("c%w", goErr.New("d")))),
						fmt.Errorf("e%w", fmt.Errorf("f%w", fmt.Errorf("g%w", goErr.New("h")))),
					))),
			expectedSimple: `prefix1: prefix2: abcd
efgh`,
			expectedVerbose: `prefix1: prefix2: abcd
(1) prefix1
Wraps: (2) prefix2
Wraps: (3) abcd
  | efgh
  └─ Wraps: (4) efgh
    └─ Wraps: (5) fgh
      └─ Wraps: (6) gh
        └─ Wraps: (7) h
  └─ Wraps: (8) abcd
    └─ Wraps: (9) bcd
      └─ Wraps: (10) cd
        └─ Wraps: (11) d
Error types: (1) *fmt.wrapError (2) *fmt.wrapError (3) *errors.joinError (4) *fmt.wrapError (5) *fmt.wrapError (6) *fmt.wrapError (7) *errors.errorString (8) *fmt.wrapError (9) *fmt.wrapError (10) *fmt.wrapError (11) *errors.errorString`,
		},
		{
			name:           "test wrapMini elides cause error string",
			err:            &wrapMini{"whoa: d", goErr.New("d")},
			expectedSimple: "whoa: d",
			expectedVerbose: `whoa: d
(1) whoa
Wraps: (2) d
Error types: (1) *errbase.wrapMini (2) *errors.errorString`,
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			fe := Formattable(tt.err)
			s := fmt.Sprintf("%s", fe)
			if s != tt.expectedSimple {
				t.Errorf("\nexpected: \n%s\nbut got:\n%s\n", tt.expectedSimple, s)
			}
			s = fmt.Sprintf("%+v", fe)
			if s != tt.expectedVerbose {
				t.Errorf("\nexpected: \n%s\nbut got:\n%s\n", tt.expectedVerbose, s)
			}
		})
	}
}

func TestPrintEntry(t *testing.T) {
	b := func(s string) []byte { return []byte(s) }

	testCases := []struct {
		entry formatEntry
		exp   string
	}{
		{formatEntry{}, ""},
		{formatEntry{head: b("abc")}, " abc"},
		{formatEntry{head: b("abc\nxyz")}, " abc\nxyz"},
		{formatEntry{details: b("def")}, " def"},
		{formatEntry{details: b("def\nxyz")}, " def\nxyz"},
		{formatEntry{head: b("abc"), details: b("def")}, " abcdef"},
		{formatEntry{head: b("abc\nxyz"), details: b("def")}, " abc\nxyzdef"},
		{formatEntry{head: b("abc"), details: b("def\n  | xyz")}, " abcdef\n  | xyz"},
		{formatEntry{head: b("abc\nxyz"), details: b("def\n  | xyz")}, " abc\nxyzdef\n  | xyz"},
	}

	for _, tc := range testCases {
		s := state{}
		s.printEntry(tc.entry)
		if s.finalBuf.String() != tc.exp {
			t.Errorf("%s: expected %q, got %q", tc.entry, tc.exp, s.finalBuf.String())
		}
	}
}

func TestFormatSingleLineOutput(t *testing.T) {
	b := func(s string) []byte { return []byte(s) }
	testCases := []struct {
		entries []formatEntry
		exp     string
	}{
		{[]formatEntry{{}}, ``},
		{[]formatEntry{{head: b(`a`)}}, `a`},
		{[]formatEntry{{head: b(`a`)}, {head: b(`b`)}, {head: b(`c`)}}, `c: b: a`},
		{[]formatEntry{{}, {head: b(`b`)}}, `b`},
		{[]formatEntry{{head: b(`a`)}, {}}, `a`},
		{[]formatEntry{{head: b(`a`)}, {}, {head: b(`c`)}}, `c: a`},
		{[]formatEntry{{head: b(`a`), elideShort: true}, {head: b(`b`)}}, `b`},
		{[]formatEntry{{head: b("abc\ndef")}, {head: b("ghi\nklm")}}, "ghi\nklm: abc\ndef"},
	}

	for _, tc := range testCases {
		s := state{entries: tc.entries}
		s.formatSingleLineOutput()
		if s.finalBuf.String() != tc.exp {
			t.Errorf("%s: expected %q, got %q", tc.entries, tc.exp, s.finalBuf.String())
		}
	}
}

func TestPrintEntryRedactable(t *testing.T) {
	sm := string(redact.StartMarker())
	em := string(redact.EndMarker())
	esc := string(redact.EscapeMarkers(redact.StartMarker()))
	b := func(s string) []byte { return []byte(s) }
	q := func(s string) string { return sm + s + em }

	testCases := []struct {
		entry formatEntry
		exp   string
	}{
		// If the entries are not redactable, they may contain arbitrary
		// characters; they get enclosed in redaction markers in the final output.
		{formatEntry{}, ""},
		{formatEntry{head: b("abc")}, " " + q("abc")},
		{formatEntry{head: b("abc\nxyz")}, " " + q("abc") + "\n" + q("xyz")},
		{formatEntry{details: b("def")}, " " + q("def")},
		{formatEntry{details: b("def\nxyz")}, " " + q("def") + "\n" + q("xyz")},
		{formatEntry{head: b("abc"), details: b("def")}, " " + q("abc") + q("def")},
		{formatEntry{head: b("abc\nxyz"), details: b("def")}, " " + q("abc") + "\n" + q("xyz") + q("def")},
		{formatEntry{head: b("abc"), details: b("def\n  | xyz")}, " " + q("abc") + q("def") + "\n" + q("  | xyz")},
		{formatEntry{head: b("abc\nxyz"), details: b("def\n  | xyz")}, " " + q("abc") + "\n" + q("xyz") + q("def") + "\n" + q("  | xyz")},
		// If there were markers in the entry, they get escaped in the output.
		{formatEntry{head: b("abc" + em + sm), details: b("def" + em + sm)}, " " + q("abc"+esc+esc) + q("def"+esc+esc)},

		// If the entries are redactable, then whatever characters they contain
		// are assumed safe and copied as-is to the final output.
		{formatEntry{redactable: true}, ""},
		{formatEntry{redactable: true, head: b("abc")}, " abc"},
		{formatEntry{redactable: true, head: b("abc\nxyz")}, " abc\nxyz"},
		{formatEntry{redactable: true, details: b("def")}, " def"},
		{formatEntry{redactable: true, details: b("def\nxyz")}, " def\nxyz"},
		{formatEntry{redactable: true, head: b("abc"), details: b("def")}, " abcdef"},
		{formatEntry{redactable: true, head: b("abc\nxyz"), details: b("def")}, " abc\nxyzdef"},
		{formatEntry{redactable: true, head: b("abc"), details: b("def\n  | xyz")}, " abcdef\n  | xyz"},
		{formatEntry{redactable: true, head: b("abc\nxyz"), details: b("def\n  | xyz")}, " abc\nxyzdef\n  | xyz"},
		// Entry already contains some markers.
		{formatEntry{redactable: true, head: b("a " + q("bc")), details: b("d " + q("ef"))}, " a " + q("bc") + "d " + q("ef")},
	}

	for _, tc := range testCases {
		s := state{redactableOutput: true}
		s.printEntry(tc.entry)
		if s.finalBuf.String() != tc.exp {
			t.Errorf("%s: expected %q, got %q", tc.entry, tc.exp, s.finalBuf.String())
		}
	}
}

func TestFormatSingleLineOutputRedactable(t *testing.T) {
	sm := string(redact.StartMarker())
	em := string(redact.EndMarker())
	// 	esc := string(redact.EscapeMarkers(redact.StartMarker()))
	b := func(s string) []byte { return []byte(s) }
	q := func(s string) string { return sm + s + em }

	testCases := []struct {
		entries []formatEntry
		exp     string
	}{
		// If the entries are not redactable, then whatever characters they contain
		// get enclosed within redaction markers.
		{[]formatEntry{{}}, ``},
		{[]formatEntry{{head: b(`a`)}}, q(`a`)},
		{[]formatEntry{{head: b(`a`)}, {head: b(`b`)}, {head: b(`c`)}}, q(`c`) + ": " + q(`b`) + ": " + q(`a`)},
		{[]formatEntry{{}, {head: b(`b`)}}, q(`b`)},
		{[]formatEntry{{head: b(`a`)}, {}}, q(`a`)},
		{[]formatEntry{{head: b(`a`)}, {}, {head: b(`c`)}}, q(`c`) + ": " + q(`a`)},
		{[]formatEntry{{head: b(`a`), elideShort: true}, {head: b(`b`)}}, q(`b`)},
		{[]formatEntry{{head: b("abc\ndef")}, {head: b("ghi\nklm")}}, q("ghi") + "\n" + q("klm") + ": " + q("abc") + "\n" + q("def")},

		// If some entries are redactable but not others, then
		// only those that are redactable are passed through.
		{[]formatEntry{{redactable: true}}, ``},
		{[]formatEntry{{redactable: true, head: b(`a`)}}, `a`},
		{[]formatEntry{{redactable: true, head: b(`a`)}, {head: b(`b`)}, {redactable: true, head: b(`c`)}}, `c: ` + q(`b`) + `: a`},

		{[]formatEntry{{redactable: true}, {head: b(`b`)}}, q(`b`)},
		{[]formatEntry{{}, {redactable: true, head: b(`b`)}}, `b`},
		{[]formatEntry{{redactable: true, head: b(`a`)}, {}}, `a`},
		{[]formatEntry{{head: b(`a`)}, {redactable: true}}, q(`a`)},

		{[]formatEntry{{head: b(`a`)}, {}, {head: b(`c`)}}, q(`c`) + `: ` + q(`a`)},
		{[]formatEntry{{head: b(`a`)}, {redactable: true}, {head: b(`c`)}}, q(`c`) + `: ` + q(`a`)},
		{[]formatEntry{{head: b(`a`), elideShort: true, redactable: true}, {head: b(`b`)}}, q(`b`)},
		{[]formatEntry{{redactable: true, head: b("abc\ndef")}, {head: b("ghi\nklm")}}, q("ghi") + "\n" + q("klm") + ": abc\ndef"},
		{[]formatEntry{{head: b("abc\ndef")}, {redactable: true, head: b("ghi\nklm")}}, "ghi\nklm: " + q("abc") + "\n" + q("def")},
		// Entry already contains some markers.
		{[]formatEntry{{redactable: true, head: b(`a` + q(" b"))}, {redactable: true, head: b(`c ` + q("d"))}}, `c ` + q(`d`) + `: a` + q(` b`)},
	}

	for _, tc := range testCases {
		s := state{entries: tc.entries, redactableOutput: true}
		s.formatSingleLineOutput()
		if s.finalBuf.String() != tc.exp {
			t.Errorf("%s: expected %q, got %q", tc.entries, tc.exp, s.finalBuf.String())
		}
	}
}
