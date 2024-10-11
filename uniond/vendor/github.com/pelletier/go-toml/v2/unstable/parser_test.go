package unstable

import (
	"fmt"
	"strconv"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestParser_AST_Numbers(t *testing.T) {
	examples := []struct {
		desc  string
		input string
		kind  Kind
		err   bool
	}{
		{
			desc:  "integer just digits",
			input: `1234`,
			kind:  Integer,
		},
		{
			desc:  "integer zero",
			input: `0`,
			kind:  Integer,
		},
		{
			desc:  "integer sign",
			input: `+99`,
			kind:  Integer,
		},
		{
			desc:  "integer hex uppercase",
			input: `0xDEADBEEF`,
			kind:  Integer,
		},
		{
			desc:  "integer hex lowercase",
			input: `0xdead_beef`,
			kind:  Integer,
		},
		{
			desc:  "integer octal",
			input: `0o01234567`,
			kind:  Integer,
		},
		{
			desc:  "integer binary",
			input: `0b11010110`,
			kind:  Integer,
		},
		{
			desc:  "float zero",
			input: `0.0`,
			kind:  Float,
		},
		{
			desc:  "float positive zero",
			input: `+0.0`,
			kind:  Float,
		},
		{
			desc:  "float negative zero",
			input: `-0.0`,
			kind:  Float,
		},
		{
			desc:  "float pi",
			input: `3.1415`,
			kind:  Float,
		},
		{
			desc:  "float negative",
			input: `-0.01`,
			kind:  Float,
		},
		{
			desc:  "float signed exponent",
			input: `5e+22`,
			kind:  Float,
		},
		{
			desc:  "float exponent lowercase",
			input: `1e06`,
			kind:  Float,
		},
		{
			desc:  "float exponent uppercase",
			input: `-2E-2`,
			kind:  Float,
		},
		{
			desc:  "float fractional with exponent",
			input: `6.626e-34`,
			kind:  Float,
		},
		{
			desc:  "float underscores",
			input: `224_617.445_991_228`,
			kind:  Float,
		},
		{
			desc:  "inf",
			input: `inf`,
			kind:  Float,
		},
		{
			desc:  "inf negative",
			input: `-inf`,
			kind:  Float,
		},
		{
			desc:  "inf positive",
			input: `+inf`,
			kind:  Float,
		},
		{
			desc:  "nan",
			input: `nan`,
			kind:  Float,
		},
		{
			desc:  "nan negative",
			input: `-nan`,
			kind:  Float,
		},
		{
			desc:  "nan positive",
			input: `+nan`,
			kind:  Float,
		},
	}

	for _, e := range examples {
		e := e
		t.Run(e.desc, func(t *testing.T) {
			p := Parser{}
			p.Reset([]byte(`A = ` + e.input))
			p.NextExpression()
			err := p.Error()
			if e.err {
				require.Error(t, err)
			} else {
				require.NoError(t, err)

				expected := astNode{
					Kind: KeyValue,
					Children: []astNode{
						{Kind: e.kind, Data: []byte(e.input)},
						{Kind: Key, Data: []byte(`A`)},
					},
				}
				compareNode(t, expected, p.Expression())
			}
		})
	}
}

type (
	astNode struct {
		Kind     Kind
		Data     []byte
		Children []astNode
	}
)

func compareNode(t *testing.T, e astNode, n *Node) {
	t.Helper()
	require.Equal(t, e.Kind, n.Kind)
	require.Equal(t, e.Data, n.Data)

	compareIterator(t, e.Children, n.Children())
}

func compareIterator(t *testing.T, expected []astNode, actual Iterator) {
	t.Helper()
	idx := 0

	for actual.Next() {
		n := actual.Node()

		if idx >= len(expected) {
			t.Fatal("extra child in actual tree")
		}
		e := expected[idx]

		compareNode(t, e, n)

		idx++
	}

	if idx < len(expected) {
		t.Fatal("missing children in actual", "idx =", idx, "expected =", len(expected))
	}
}

//nolint:funlen
func TestParser_AST(t *testing.T) {
	examples := []struct {
		desc  string
		input string
		ast   astNode
		err   bool
	}{
		{
			desc:  "simple string assignment",
			input: `A = "hello"`,
			ast: astNode{
				Kind: KeyValue,
				Children: []astNode{
					{
						Kind: String,
						Data: []byte(`hello`),
					},
					{
						Kind: Key,
						Data: []byte(`A`),
					},
				},
			},
		},
		{
			desc:  "simple bool assignment",
			input: `A = true`,
			ast: astNode{
				Kind: KeyValue,
				Children: []astNode{
					{
						Kind: Bool,
						Data: []byte(`true`),
					},
					{
						Kind: Key,
						Data: []byte(`A`),
					},
				},
			},
		},
		{
			desc:  "array of strings",
			input: `A = ["hello", ["world", "again"]]`,
			ast: astNode{
				Kind: KeyValue,
				Children: []astNode{
					{
						Kind: Array,
						Children: []astNode{
							{
								Kind: String,
								Data: []byte(`hello`),
							},
							{
								Kind: Array,
								Children: []astNode{
									{
										Kind: String,
										Data: []byte(`world`),
									},
									{
										Kind: String,
										Data: []byte(`again`),
									},
								},
							},
						},
					},
					{
						Kind: Key,
						Data: []byte(`A`),
					},
				},
			},
		},
		{
			desc:  "array of arrays of strings",
			input: `A = ["hello", "world"]`,
			ast: astNode{
				Kind: KeyValue,
				Children: []astNode{
					{
						Kind: Array,
						Children: []astNode{
							{
								Kind: String,
								Data: []byte(`hello`),
							},
							{
								Kind: String,
								Data: []byte(`world`),
							},
						},
					},
					{
						Kind: Key,
						Data: []byte(`A`),
					},
				},
			},
		},
		{
			desc:  "inline table",
			input: `name = { first = "Tom", last = "Preston-Werner" }`,
			ast: astNode{
				Kind: KeyValue,
				Children: []astNode{
					{
						Kind: InlineTable,
						Children: []astNode{
							{
								Kind: KeyValue,
								Children: []astNode{
									{Kind: String, Data: []byte(`Tom`)},
									{Kind: Key, Data: []byte(`first`)},
								},
							},
							{
								Kind: KeyValue,
								Children: []astNode{
									{Kind: String, Data: []byte(`Preston-Werner`)},
									{Kind: Key, Data: []byte(`last`)},
								},
							},
						},
					},
					{
						Kind: Key,
						Data: []byte(`name`),
					},
				},
			},
		},
	}

	for _, e := range examples {
		e := e
		t.Run(e.desc, func(t *testing.T) {
			p := Parser{}
			p.Reset([]byte(e.input))
			p.NextExpression()
			err := p.Error()
			if e.err {
				require.Error(t, err)
			} else {
				require.NoError(t, err)
				compareNode(t, e.ast, p.Expression())
			}
		})
	}
}

func BenchmarkParseBasicStringWithUnicode(b *testing.B) {
	p := &Parser{}
	b.Run("4", func(b *testing.B) {
		input := []byte(`"\u1234\u5678\u9ABC\u1234\u5678\u9ABC"`)
		b.ReportAllocs()
		b.SetBytes(int64(len(input)))

		for i := 0; i < b.N; i++ {
			p.parseBasicString(input)
		}
	})
	b.Run("8", func(b *testing.B) {
		input := []byte(`"\u12345678\u9ABCDEF0\u12345678\u9ABCDEF0"`)
		b.ReportAllocs()
		b.SetBytes(int64(len(input)))

		for i := 0; i < b.N; i++ {
			p.parseBasicString(input)
		}
	})
}

func BenchmarkParseBasicStringsEasy(b *testing.B) {
	p := &Parser{}

	for _, size := range []int{1, 4, 8, 16, 21} {
		b.Run(strconv.Itoa(size), func(b *testing.B) {
			input := []byte(`"` + strings.Repeat("A", size) + `"`)

			b.ReportAllocs()
			b.SetBytes(int64(len(input)))

			for i := 0; i < b.N; i++ {
				p.parseBasicString(input)
			}
		})
	}
}

func TestParser_AST_DateTimes(t *testing.T) {
	examples := []struct {
		desc  string
		input string
		kind  Kind
		err   bool
	}{
		{
			desc:  "offset-date-time with delim 'T' and UTC offset",
			input: `2021-07-21T12:08:05Z`,
			kind:  DateTime,
		},
		{
			desc:  "offset-date-time with space delim and +8hours offset",
			input: `2021-07-21 12:08:05+08:00`,
			kind:  DateTime,
		},
		{
			desc:  "local-date-time with nano second",
			input: `2021-07-21T12:08:05.666666666`,
			kind:  LocalDateTime,
		},
		{
			desc:  "local-date-time",
			input: `2021-07-21T12:08:05`,
			kind:  LocalDateTime,
		},
		{
			desc:  "local-date",
			input: `2021-07-21`,
			kind:  LocalDate,
		},
	}

	for _, e := range examples {
		e := e
		t.Run(e.desc, func(t *testing.T) {
			p := Parser{}
			p.Reset([]byte(`A = ` + e.input))
			p.NextExpression()
			err := p.Error()
			if e.err {
				require.Error(t, err)
			} else {
				require.NoError(t, err)

				expected := astNode{
					Kind: KeyValue,
					Children: []astNode{
						{Kind: e.kind, Data: []byte(e.input)},
						{Kind: Key, Data: []byte(`A`)},
					},
				}
				compareNode(t, expected, p.Expression())
			}
		})
	}
}

// This example demonstrates how to parse a TOML document and preserving
// comments.  Comments are stored in the AST as Comment nodes. This example
// displays the structure of the full AST generated by the parser using the
// following structure:
//
//  1. Each root-level expression is separated by three dashes.
//  2. Bytes associated to a node are displayed in square brackets.
//  3. Siblings have the same indentation.
//  4. Children of a node are indented one level.
func ExampleParser_comments() {
	doc := `# Top of the document comment.
# Optional, any amount of lines.

# Above table.
[table] # Next to table.
# Above simple value.
key = "value" # Next to simple value.
# Below simple value.

# Some comment alone.

# Multiple comments, on multiple lines.

# Above inline table.
name = { first = "Tom", last = "Preston-Werner" } # Next to inline table.
# Below inline table.

# Above array.
array = [ 1, 2, 3 ] # Next to one-line array.
# Below array.

# Above multi-line array.
key5 = [ # Next to start of inline array.
  # Second line before array content.
  1, # Next to first element.
  # After first element.
  # Before second element.
  2,
  3, # Next to last element
  # After last element.
] # Next to end of array.
# Below multi-line array.

# Before array table.
[[products]] # Next to array table.
# After array table.
`

	var printGeneric func(*Parser, int, *Node)
	printGeneric = func(p *Parser, indent int, e *Node) {
		if e == nil {
			return
		}
		s := p.Shape(e.Raw)
		x := fmt.Sprintf("%d:%d->%d:%d (%d->%d)", s.Start.Line, s.Start.Column, s.End.Line, s.End.Column, s.Start.Offset, s.End.Offset)
		fmt.Printf("%-25s | %s%s [%s]\n", x, strings.Repeat("  ", indent), e.Kind, e.Data)
		printGeneric(p, indent+1, e.Child())
		printGeneric(p, indent, e.Next())
	}

	printTree := func(p *Parser) {
		for p.NextExpression() {
			e := p.Expression()
			fmt.Println("---")
			printGeneric(p, 0, e)
		}
		if err := p.Error(); err != nil {
			panic(err)
		}
	}

	p := &Parser{
		KeepComments: true,
	}
	p.Reset([]byte(doc))
	printTree(p)

	// Output:
	// ---
	// 1:1->1:31 (0->30)         | Comment [# Top of the document comment.]
	// ---
	// 2:1->2:33 (31->63)        | Comment [# Optional, any amount of lines.]
	// ---
	// 4:1->4:15 (65->79)        | Comment [# Above table.]
	// ---
	// 1:1->1:1 (0->0)           | Table []
	// 5:2->5:7 (81->86)         |   Key [table]
	// 5:9->5:25 (88->104)       | Comment [# Next to table.]
	// ---
	// 6:1->6:22 (105->126)      | Comment [# Above simple value.]
	// ---
	// 1:1->1:1 (0->0)           | KeyValue []
	// 7:7->7:14 (133->140)      |   String [value]
	// 7:1->7:4 (127->130)       |   Key [key]
	// 7:15->7:38 (141->164)     | Comment [# Next to simple value.]
	// ---
	// 8:1->8:22 (165->186)      | Comment [# Below simple value.]
	// ---
	// 10:1->10:22 (188->209)    | Comment [# Some comment alone.]
	// ---
	// 12:1->12:40 (211->250)    | Comment [# Multiple comments, on multiple lines.]
	// ---
	// 14:1->14:22 (252->273)    | Comment [# Above inline table.]
	// ---
	// 1:1->1:1 (0->0)           | KeyValue []
	// 15:8->15:9 (281->282)     |   InlineTable []
	// 1:1->1:1 (0->0)           |     KeyValue []
	// 15:18->15:23 (291->296)   |       String [Tom]
	// 15:10->15:15 (283->288)   |       Key [first]
	// 1:1->1:1 (0->0)           |     KeyValue []
	// 15:32->15:48 (305->321)   |       String [Preston-Werner]
	// 15:25->15:29 (298->302)   |       Key [last]
	// 15:1->15:5 (274->278)     |   Key [name]
	// 15:51->15:74 (324->347)   | Comment [# Next to inline table.]
	// ---
	// 16:1->16:22 (348->369)    | Comment [# Below inline table.]
	// ---
	// 18:1->18:15 (371->385)    | Comment [# Above array.]
	// ---
	// 1:1->1:1 (0->0)           | KeyValue []
	// 1:1->1:1 (0->0)           |   Array []
	// 19:11->19:12 (396->397)   |     Integer [1]
	// 19:14->19:15 (399->400)   |     Integer [2]
	// 19:17->19:18 (402->403)   |     Integer [3]
	// 19:1->19:6 (386->391)     |   Key [array]
	// 19:21->19:46 (406->431)   | Comment [# Next to one-line array.]
	// ---
	// 20:1->20:15 (432->446)    | Comment [# Below array.]
	// ---
	// 22:1->22:26 (448->473)    | Comment [# Above multi-line array.]
	// ---
	// 1:1->1:1 (0->0)           | KeyValue []
	// 1:1->1:1 (0->0)           |   Array []
	// 23:10->23:42 (483->515)   |     Comment [# Next to start of inline array.]
	// 24:3->24:38 (518->553)    |       Comment [# Second line before array content.]
	// 25:3->25:4 (556->557)     |     Integer [1]
	// 25:6->25:30 (559->583)    |     Comment [# Next to first element.]
	// 26:3->26:25 (586->608)    |       Comment [# After first element.]
	// 27:3->27:27 (611->635)    |       Comment [# Before second element.]
	// 28:3->28:4 (638->639)     |     Integer [2]
	// 29:3->29:4 (643->644)     |     Integer [3]
	// 29:6->29:28 (646->668)    |     Comment [# Next to last element]
	// 30:3->30:24 (671->692)    |       Comment [# After last element.]
	// 23:1->23:5 (474->478)     |   Key [key5]
	// 31:3->31:26 (695->718)    | Comment [# Next to end of array.]
	// ---
	// 32:1->32:26 (719->744)    | Comment [# Below multi-line array.]
	// ---
	// 34:1->34:22 (746->767)    | Comment [# Before array table.]
	// ---
	// 1:1->1:1 (0->0)           | ArrayTable []
	// 35:3->35:11 (770->778)    |   Key [products]
	// 35:14->35:36 (781->803)   | Comment [# Next to array table.]
	// ---
	// 36:1->36:21 (804->824)    | Comment [# After array table.]
}

func ExampleParser() {
	doc := `
	hello = "world"
	value = 42
	`
	p := Parser{}
	p.Reset([]byte(doc))
	for p.NextExpression() {
		e := p.Expression()
		fmt.Printf("Expression: %s\n", e.Kind)
		value := e.Value()
		it := e.Key()
		k := it.Node() // shortcut: we know there is no dotted key in the example
		fmt.Printf("%s -> (%s) %s\n", k.Data, value.Kind, value.Data)
	}

	// Output:
	// Expression: KeyValue
	// hello -> (String) world
	// Expression: KeyValue
	// value -> (Integer) 42
}
