// Copyright (C) 2022 Michael J. Fromberger. All Rights Reserved.

package scanner_test

import (
	"io"
	"strings"
	"testing"

	"github.com/creachadair/tomledit/scanner"
	"github.com/google/go-cmp/cmp"
)

func TestScanner(t *testing.T) {
	type result struct {
		// N.B. Fields exported to simplify cmp usage.
		Tok  scanner.Token
		Text string
	}
	tests := []struct {
		input string
		want  []result
	}{
		{"", nil},
		{"   \t   ", nil},
		{"  \n  \n\t", []result{{scanner.Newline, ""}, {scanner.Newline, ""}}},

		{"# complete comment\n", []result{{scanner.Comment, "# complete comment"}}},
		{"# EOF comment", []result{{scanner.Comment, "# EOF comment"}}},

		{`0`, []result{{scanner.Integer, "0"}}},
		{`100`, []result{{scanner.Integer, "100"}}},
		{`-256_512`, []result{{scanner.Integer, "-256_512"}}},
		{`0x0`, []result{{scanner.Integer, "0x0"}}},
		{`0b0`, []result{{scanner.Integer, "0b0"}}},
		{`0o0`, []result{{scanner.Integer, "0o0"}}},

		{`-0.6e-15`, []result{{scanner.Float, "-0.6e-15"}}},
		{`inf +inf -inf`, []result{{scanner.Float, "inf"}, {scanner.Float, "+inf"}, {scanner.Float, "-inf"}}},
		{`nan`, []result{{scanner.Float, "nan"}}},

		{`"" ''`, []result{{scanner.String, `""`}, {scanner.LString, `''`}}},
		{`"\"\\\""`, []result{{scanner.String, `"\"\\\""`}}},
		{`"""foo\nbar"""`, []result{{scanner.MString, `"""foo\nbar"""`}}},
		{`'''foo'''`, []result{{scanner.MLString, `'''foo'''`}}},
		{`"""\
I am a man of \
constant sorrow.
"""`, []result{{scanner.MString, "\"\"\"\\\nI am a man of \\\nconstant sorrow.\n\"\"\""}}},
		{"'''\nI've seen trouble\nall my days.\n\n'''", []result{
			{scanner.MLString, "'''\nI've seen trouble\nall my days.\n\n'''"},
		}},
		{`"""Here are fifteen quotation marks: ""\"""\"""\"""\"""\"."""`, []result{
			{scanner.MString, `"""Here are fifteen quotation marks: ""\"""\"""\"""\"""\"."""`},
		}},

		{`[table] [[array]]`, []result{
			{scanner.LBracket, "["}, {scanner.Word, "table"}, {scanner.RBracket, "]"},
			{scanner.LBracket, "["}, {scanner.LBracket, "["}, {scanner.Word, "array"}, {scanner.RBracket, "]"}, {scanner.RBracket, "]"},
		}},

		{`a."b c".d`, []result{
			{scanner.Word, "a"},
			{scanner.Dot, "."},
			{scanner.String, `"b c"`},
			{scanner.Dot, "."},
			{scanner.Word, "d"},
		}},

		{`1985-03-04 # ok`, []result{{scanner.LocalDate, "1985-03-04"}, {scanner.Comment, "# ok"}}},
		{`1987-09-12`, []result{{scanner.LocalDate, "1987-09-12"}}},
		{`1985-07-19qqq`, []result{{scanner.Word, "1985-07-19qqq"}}},
		{`1999-12-31T23:59:59.99999Z 2000-01-01T00:00:00+01:00`, []result{
			{scanner.DateTime, "1999-12-31T23:59:59.99999Z"},
			{scanner.DateTime, "2000-01-01T00:00:00+01:00"},
		}},
		{`2020-11-02T18:30:01.9001 2021-01-06T17:25:00 2021-03-04t01:01:33`, []result{
			{scanner.LocalDateTime, "2020-11-02T18:30:01.9001"},
			{scanner.LocalDateTime, "2021-01-06T17:25:00"},
			{scanner.LocalDateTime, "2021-03-04t01:01:33"},
		}},
		{`1985-07-26 15:23:04.155 01:01:05`, []result{
			{scanner.LocalDateTime, "1985-07-26 15:23:04.155"},
			{scanner.LocalTime, "01:01:05"},
		}},

		{`[ foo."bar" ]
baz = "quux"
frob = 2021-12-01
`, []result{
			{scanner.LBracket, "["}, {scanner.Word, "foo"}, {scanner.Dot, "."},
			{scanner.String, `"bar"`}, {scanner.RBracket, "]"}, {scanner.Newline, ""},
			{scanner.Word, "baz"}, {scanner.Equal, "="}, {scanner.String, `"quux"`}, {scanner.Newline, ""},
			{scanner.Word, "frob"}, {scanner.Equal, "="}, {scanner.LocalDate, "2021-12-01"}, {scanner.Newline, ""},
		}},

		{`1 +2 -3.2 +6e-9, {two=three}, "four"`, []result{
			{scanner.Integer, "1"}, {scanner.Integer, "+2"},
			{scanner.Float, "-3.2"}, {scanner.Float, "+6e-9"}, {scanner.Comma, ","},
			{scanner.LInline, "{"},
			{scanner.Word, "two"}, {scanner.Equal, "="}, {scanner.Word, "three"},
			{scanner.RInline, "}"}, {scanner.Comma, ","},
			{scanner.String, `"four"`},
		}},
	}

	for _, test := range tests {
		var got []result
		s := scanner.New(strings.NewReader(test.input))
		for s.Next() == nil {
			got = append(got, result{
				Tok:  s.Token(),
				Text: string(s.Text()),
			})
		}
		if s.Err() != io.EOF {
			t.Errorf("Next failed: %v", s.Err())
		}
		if diff := cmp.Diff(test.want, got); diff != "" {
			t.Errorf("Input: %#q\nTokens: (-want, +got)\n%s", test.input, diff)
		}
	}
}

func TestEscape(t *testing.T) {
	tests := []struct {
		input, want string
		multi       bool
	}{
		{"", "", false},
		{"", "", true},
		{"  ", "  ", false},
		{"  ", "  ", true},
		{" \t \r", " \\t \\r", false},
		{" \t \r", " \\t \\r", true},

		{"no escape", "no escape", false},
		{"no escape", "no escape", true},

		{"a\tb", "a\\tb", false},
		{"a\tb", "a\\tb", true},
		{"ab\n", "ab\\n", false},
		{"ab\n", "ab\n", true},
		{"a\\tb", "a\\\\tb", false},
		{"a\\tb", "a\\\\tb", true},

		{"\a\v\n\t", "\\u0007\\u000b\\n\\t", false},
		{"\a\v\n\t", "\\u0007\\u000b\n\\t", true},

		{"\ta\nb\nc", "\\ta\\nb\\nc", false},
		{"\ta\nb\nc", "\\ta\nb\nc", true},

		{"😍🐈\n", "😍🐈\\n", false},
		{"😍🐈\n", "😍🐈\n", true},
	}
	for _, test := range tests {
		var got string
		if test.multi {
			got = string(scanner.EscapeMultiline(test.input))
		} else {
			got = string(scanner.Escape(test.input))
		}
		if got != test.want {
			t.Errorf("Escape %#q multi=%v: got %#q, want %#q", test.input, test.multi, got, test.want)
		}
	}
}

func TestUnescape(t *testing.T) {
	tests := []struct {
		input, want string
	}{
		{"", ""},
		{"Nothing here need be done", "Nothing here need be done"},

		{`a \t\r\nb \u0009`, "a \t\r\nb \t"}, // escaped whitespace
		{`a \u0007 b`, "a \a b"},             // escaped control
		{`a \"b\" c`, `a "b" c`},             // escaped quotes
		{`\u0113`, "\u0113"},                 // short Unicode escape
		{`\U0001F60D`, "\U0001f60d"},         // long Unicode escape
		{"a \\\nb c", "a \\\nb c"},           // newline at EOL
	}
	for _, test := range tests {
		bits, err := scanner.Unescape([]byte(test.input))
		if err != nil {
			t.Errorf("Unescape %#q: %v", test.input, err)
			continue
		}

		if got := string(bits); got != test.want {
			t.Errorf("Unescape %#q: got %#q, want %#q", test.input, got, test.want)
		}
	}
}

func TestUnescapeErrors(t *testing.T) {
	const badEscape = "incomplete escape sequence"
	const badUnicode = "incomplete Unicode escape"
	tests := []struct {
		input, want string
	}{
		{`\`, badEscape},
		{`a b c\`, badEscape},
		{`\uXY`, badUnicode},
		{`\u01x`, badUnicode},
		{`\U`, badUnicode},
		{`\U0113`, badUnicode},
		{`\U01132fc`, badUnicode},
	}
	for _, test := range tests {
		got, err := scanner.Unescape([]byte(test.input))
		if err == nil || !strings.Contains(err.Error(), test.want) {
			t.Errorf("Unescape %#q: got (%q, %v), wanted %v", test.input, string(got), err, test.want)
		}
	}
}
