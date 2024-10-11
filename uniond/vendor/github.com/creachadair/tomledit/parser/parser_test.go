// Copyright (C) 2022 Michael J. Fromberger. All Rights Reserved.

package parser_test

import (
	"fmt"
	"reflect"
	"strings"
	"testing"

	"github.com/creachadair/tomledit/parser"
	"github.com/google/go-cmp/cmp"
)

var (
	keyValueType = reflect.TypeOf((*parser.KeyValue)(nil))
	headingType  = reflect.TypeOf((*parser.Heading)(nil))
	commentsType = reflect.TypeOf(parser.Comments(nil))
)

func TestItems(t *testing.T) {
	type result struct {
		VType  reflect.Type
		Source string
	}
	tests := []struct {
		input string
		want  []result
	}{
		// Blanks.
		{"", nil},
		{"    ", nil},
		{"\n\n\t   \n", nil},

		// Comments.
		{"# c1\n# c2\n\n", []result{
			{commentsType, "# c1\n# c2"},
		}},

		// Key-value mappings.

		// - Weird cases.
		{`"" = true`, []result{{keyValueType, `"" = true`}}},
		{`'' = false`, []result{{keyValueType, `"" = false`}}},

		// - Basic values.
		{`x=3`, []result{{keyValueType, `x = 3`}}},
		{`x=-3.2e+19`, []result{{keyValueType, `x = -3.2e+19`}}},
		{`x=-inf`, []result{{keyValueType, `x = -inf`}}},
		{`x=true`, []result{{keyValueType, `x = true`}}},
		{`x=false`, []result{{keyValueType, `x = false`}}},
		{`x="foo"`, []result{{keyValueType, `x = "foo"`}}},
		{`x='bar'`, []result{{keyValueType, `x = 'bar'`}}},

		// - Date and time stamps.
		{`x=2021-01-06`, []result{{keyValueType, `x = 2021-01-06`}}},
		{`x=15:00:23.22`, []result{{keyValueType, `x = 15:00:23.22`}}},
		{`x=2021-01-06T15:00:23`, []result{{keyValueType, `x = 2021-01-06T15:00:23`}}},
		{`x=2021-01-06T15:00:23Z`, []result{{keyValueType, `x = 2021-01-06T15:00:23Z`}}},
		{`x=2021-01-06T15:00:23+03:30`, []result{{keyValueType, `x = 2021-01-06T15:00:23+03:30`}}},

		// - Multi-line strings.
		{`x="""baz\nquux\n"""`, []result{{keyValueType, `x = """baz\nquux\n"""`}}},
		{"x='''baz\nquux\n'''", []result{{keyValueType, "x = '''baz\nquux\n'''"}}},

		// - Arrays.
		{"x=[]", []result{{keyValueType, `x = []`}}},
		{"x=[\n5,\n\"c\",\n]\n", []result{{keyValueType, `x = [5, "c"]`}}},
		{"x = [\n# ignored\n1,2,3\n# ignored\n, ]\n", []result{{keyValueType, `x = [1, 2, 3]`}}},
		{"x = [[]] # array in array", []result{{keyValueType, `x = [[]]`}}},
		{"x = [[],[[[[],5 # ok\n]],[]],] # array in array", []result{{keyValueType, `x = [[], [[[[], 5]], []]]`}}},

		// - Inline tables.
		{"x={} # whatever, bro\n", []result{{keyValueType, `x = {}`}}},
		{"x={a=2,b=\"four\"}\n", []result{{keyValueType, `x = {a = 2, b = "four"}`}}},

		// - Compound keys.
		{`x . y = true`, []result{{keyValueType, `x.y = true`}}},
		{`a . "b c" . d='qq'`, []result{{keyValueType, `a."b c".d = 'qq'`}}},
		{`"string thing"=["ding","ding",      ] # whoa`, []result{
			{keyValueType, `"string thing" = ["ding", "ding"]`},
		}},

		// Headings.
		{`[ a . b . c ]`, []result{{headingType, `[a.b.c]`}}},
		{`[ a . '' . c ]`, []result{{headingType, `[a."".c]`}}},
		{`[ a . "b.d" . c ]`, []result{{headingType, `[a."b.d".c]`}}},
		{`[[ a . b . c ]]`, []result{{headingType, `[[a.b.c]]`}}},
	}
	for _, test := range tests {
		p := parser.New(strings.NewReader(test.input))

		items, err := p.Items()
		if err != nil {
			t.Errorf("Items: unexpected error: %v", err)
			t.Logf("Input:\n%s", test.input)
			continue
		}

		var got []result
		for _, itm := range items {
			got = append(got, result{
				VType:  reflect.TypeOf(itm),
				Source: fmt.Sprint(itm),
			})
		}

		if diff := cmp.Diff(test.want, got, cmp.Comparer(func(t1, t2 reflect.Type) bool {
			return t1 == t2
		})); diff != "" {
			t.Errorf("Items: (-want, +got)\n%s", diff)
			t.Logf("Input:\n%s", test.input)
		}
	}
}

const stdExample = `# This is a TOML document.

title = "TOML Example"

[owner]
name = "Tom Preston-Werner"
dob = 1979-05-27T07:32:00-08:00 # First class dates

[database]
server = "192.168.1.1"
ports = [ 8000, 8001, 8002 ]
connection_max = 5000
enabled = true

[servers]

  # Indentation (tabs and/or spaces) is allowed but not required
  [servers.alpha]
  ip = "10.0.0.1"
  dc = "eqdc10"

  [servers.beta]
  ip = "10.0.0.2"
  dc = "eqdc10"

[clients]
data = [ ["gamma", "delta"], [1, 2] ]

# Line breaks are OK when inside arrays
hosts = [
  "alpha",
  "omega"
]
`

func mustParseItems(t *testing.T, input string) []string {
	t.Helper()
	items, err := parser.New(strings.NewReader(input)).Items()
	if err != nil {
		t.Logf("Parsing failed: unexpected error: %v", err)
		t.Logf("Input:\n%s", input)
		t.FailNow()
	}
	var got []string
	for _, item := range items {
		typ := strings.Split(fmt.Sprintf("%T", item), ".")
		got = append(got, fmt.Sprintf("%s %s", typ[len(typ)-1], item))
	}
	return got
}

func checkItems(t *testing.T, got, want []string) {
	t.Helper()
	if diff := cmp.Diff(want, got); diff != "" {
		t.Errorf("Items: (-want, +got)\n%s", diff)
		t.Logf("Input:\n%s", stdExample)
	}
}

func TestSpec(t *testing.T) {
	t.Run("README", func(t *testing.T) {
		got := mustParseItems(t, stdExample)
		checkItems(t, got, []string{
			`Comments # This is a TOML document.`,
			`KeyValue title = "TOML Example"`,
			`Heading [owner]`,
			`KeyValue name = "Tom Preston-Werner"`,
			`KeyValue dob = 1979-05-27T07:32:00-08:00`,
			`Heading [database]`,
			`KeyValue server = "192.168.1.1"`,
			`KeyValue ports = [8000, 8001, 8002]`,
			`KeyValue connection_max = 5000`,
			`KeyValue enabled = true`,
			`Heading [servers]`,
			// Comment attached to heading
			`Heading [servers.alpha]`,
			`KeyValue ip = "10.0.0.1"`,
			`KeyValue dc = "eqdc10"`,
			`Heading [servers.beta]`,
			`KeyValue ip = "10.0.0.2"`,
			`KeyValue dc = "eqdc10"`,
			`Heading [clients]`,
			`KeyValue data = [["gamma", "delta"], [1, 2]]`,
			// Comment attached to key-value
			`KeyValue hosts = ["alpha", "omega"]`,
		})
	})
	t.Run("Arrays", func(t *testing.T) {
		got := mustParseItems(t, `
integers = [ 1, 2, 3 ]
colors = [ "red", "yellow", "green" ]
nested_arrays_of_ints = [ [ 1, 2 ], [3, 4, 5] ]
nested_mixed_array = [ [ 1, 2 ], ["a", "b", "c"] ]
string_array = [ "all", 'strings', """are the same""", '''type''' ]

# Mixed-type arrays are allowed
numbers = [ 0.1, 0.2, 0.5, 1, 2, 5 ]
contributors = [
  "Foo Bar <foo@example.com>",
  { name = "Baz Qux", email = "bazqux@example.com", url = "https://example.com/bazqux" }
]`)
		checkItems(t, got, []string{
			`KeyValue integers = [1, 2, 3]`,
			`KeyValue colors = ["red", "yellow", "green"]`,
			`KeyValue nested_arrays_of_ints = [[1, 2], [3, 4, 5]]`,
			`KeyValue nested_mixed_array = [[1, 2], ["a", "b", "c"]]`,
			`KeyValue string_array = ["all", 'strings', """are the same""", '''type''']`,
			// Comment attached to key-value
			`KeyValue numbers = [0.1, 0.2, 0.5, 1, 2, 5]`,
			`KeyValue contributors = ["Foo Bar <foo@example.com>", {name = "Baz Qux", email = "bazqux@example.com", url = "https://example.com/bazqux"}]`,
		})
	})
}

func TestParseKey(t *testing.T) {
	t.Run("Good", func(t *testing.T) {
		const input = ` a . "b.c d" . e`
		key, err := parser.ParseKey(input)
		if err != nil {
			t.Fatalf("ParseKey(%q): %v", input, err)
		}
		if diff := cmp.Diff(parser.Key{"a", "b.c d", "e"}, key); diff != "" {
			t.Errorf("Incorrect key result: (+want, -got)\n%s", diff)
		}
	})

	t.Run("Bad", func(t *testing.T) {
		for _, in := range []string{"", "  ", `#nope`, `.garbage`, `extra stuff`} {
			key, err := parser.ParseKey(in)
			if err == nil {
				t.Errorf("ParseKey(%q): got %v, wanted error", in, key)
			}
		}
	})
}

func TestKeyCompare(t *testing.T) {
	tests := []struct {
		lhs, rhs   string
		eq, before bool
	}{
		{"a", "a", true, false},
		{"b", "a", false, false},
		{"a", "b", false, true},

		{"a.b", "a.b", true, false},
		{"a.c", "a.b", false, false},
		{"a.b", "a.c", false, true},

		{"a.b.a", "a", false, false},
		{"a.b.a", "a.c", false, true},
		{"a.b.a", "a.c.a", false, true},
		{"a.c.a", "a.b.a", false, false},

		{"a.b.c.d", "a.b.c.e", false, true},
		{"b.b.c.d", "a.b.c.e", false, false},
		{"a.a.c.d", "a.b.c.e", false, true},
		{"a.c.c.d", "a.b.c.e", false, false},
	}
	for i, test := range tests {
		// Safety check on the test cases.
		if test.eq && test.before {
			t.Fatalf("Invalid test case %d: eq and before both true: %+v", i, test)
		}

		lhs := mustParseKey(t, test.lhs)
		rhs := mustParseKey(t, test.rhs)

		if got := lhs.Equals(rhs); got != test.eq {
			t.Errorf("(%q).Equals(%q): got %v, want %v", lhs, rhs, got, test.eq)
		}
		if got := lhs.Before(rhs); got != test.before {
			t.Errorf("(%q).Before(%q): got %v, want %v", lhs, rhs, got, test.before)
		}
	}
}

func TestParseValue(t *testing.T) {
	tests := []struct {
		input, want, comment string
	}{
		{"17 # wilkommen\n", "17", "# wilkommen"},
		{"  false   ", "false", ""},
		{"'beagle breath'", `'beagle breath'`, ""},
		{" \t\t true\t\n", "true", ""},
		{"'''read\nthe\tdocs\n'''", "'''read\nthe\tdocs\n'''", ""},
		{"\t[\n\t   \n\n]\n", "[]", ""},
		{"[0,\n1,\n2\n] # bienvenue\n", "[0, 1, 2]", "# bienvenue"},
		{"{  } # welcome", "{}", "# welcome"},
		{"{ all . 'we' . are = 42 }", "{all.we.are = 42}", ""},
	}

	for _, test := range tests {
		v, err := parser.ParseValue(test.input)
		if err != nil {
			t.Errorf("ParseValue(%#q): unexpected error: %v", test.input, err)
			continue
		}
		got := v.String()
		if got != test.want {
			t.Errorf("ParseValue(%#q):\n got %q\nwant %q", test.input, got, test.want)
		}
		if v.Trailer != test.comment {
			t.Errorf("ParseValue(%#q): got comment %q, want %q", test.input, v.Trailer, test.comment)
		}
	}
}

func TestCleanTrailer(t *testing.T) {
	tests := []struct {
		input, want string
	}{
		{"", "#"},
		{"  \t ", "#"},
		{"#", "#"},
		{" # ", "#"},
		{"abc", "# abc"},
		{"# abc", "# abc"},
		{"  abc  ", "# abc"},
		{"  # abc  ", "# abc"},
		{"abc\ndef", "# abc def"},
		{"# abc\ndef ghi", "# abc def ghi"},
	}

	for _, test := range tests {
		got := parser.CleanTrailer(test.input)
		if got != test.want {
			t.Errorf("CleanTrailer(%q):\ngot:  %s\nwant: %s", test.input, got, test.want)
		}
	}
}

func mustParseKey(t *testing.T, s string) parser.Key {
	t.Helper()

	k, err := parser.ParseKey(s)
	if err != nil {
		t.Fatalf("ParseKey %q: %v", s, err)
	}
	return k
}
