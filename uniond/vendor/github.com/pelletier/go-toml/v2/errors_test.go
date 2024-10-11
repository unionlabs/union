package toml

import (
	"bytes"
	"errors"
	"fmt"
	"strings"
	"testing"

	"github.com/pelletier/go-toml/v2/unstable"
	"github.com/stretchr/testify/assert"
)

//nolint:funlen
func TestDecodeError(t *testing.T) {

	examples := []struct {
		desc     string
		doc      [3]string
		msg      string
		expected string
	}{
		{
			desc: "no context",
			doc:  [3]string{"", "morning", ""},
			msg:  "this is wrong",
			expected: `
1| morning
 | ~~~~~~~ this is wrong`,
		},
		{
			desc: "one line",
			doc:  [3]string{"good ", "morning", " everyone"},
			msg:  "this is wrong",
			expected: `
1| good morning everyone
 |      ~~~~~~~ this is wrong`,
		},
		{
			desc: "exactly 3 lines",
			doc: [3]string{`line1
line2
line3
before `, "highlighted", ` after
post line 1
post line 2
post line 3`},
			msg: "this is wrong",
			expected: `
1| line1
2| line2
3| line3
4| before highlighted after
 |        ~~~~~~~~~~~ this is wrong
5| post line 1
6| post line 2
7| post line 3`,
		},
		{
			desc: "more than 3 lines",
			doc: [3]string{`should not be seen1
should not be seen2
line1
line2
line3
before `, "highlighted", ` after
post line 1
post line 2
post line 3
should not be seen3
should not be seen4`},
			msg: "this is wrong",
			expected: `
3| line1
4| line2
5| line3
6| before highlighted after
 |        ~~~~~~~~~~~ this is wrong
7| post line 1
8| post line 2
9| post line 3`,
		},
		{
			desc: "more than 10 total lines",
			doc: [3]string{`should not be seen 0
should not be seen1
should not be seen2
should not be seen3
line1
line2
line3
before `, "highlighted", ` after
post line 1
post line 2
post line 3
should not be seen3
should not be seen4`},
			msg: "this is wrong",
			expected: `
 5| line1
 6| line2
 7| line3
 8| before highlighted after
  |        ~~~~~~~~~~~ this is wrong
 9| post line 1
10| post line 2
11| post line 3`,
		},
		{
			desc: "last line of more than 10",
			doc: [3]string{`should not be seen
should not be seen
should not be seen
should not be seen
should not be seen
should not be seen
should not be seen
line1
line2
line3
before `, "highlighted", ``},
			msg: "this is wrong",
			expected: `
 8| line1
 9| line2
10| line3
11| before highlighted
  |        ~~~~~~~~~~~ this is wrong
`,
		},
		{
			desc: "handle empty lines in the before/after blocks",
			doc: [3]string{
				`line1

line 2
before `, "highlighted", ` after
line 3

line 4
line 5`,
			},
			expected: `1| line1
2|
3| line 2
4| before highlighted after
 |        ~~~~~~~~~~~
5| line 3
6|
7| line 4`,
		},
		{
			desc: "handle remainder of the error line when there is only one line",
			doc:  [3]string{`P=`, `[`, `#`},
			msg:  "array is incomplete",
			expected: `1| P=[#
 |   ~ array is incomplete`,
		},
	}

	for _, e := range examples {
		e := e
		t.Run(e.desc, func(t *testing.T) {

			b := bytes.Buffer{}
			b.Write([]byte(e.doc[0]))
			start := b.Len()
			b.Write([]byte(e.doc[1]))
			end := b.Len()
			b.Write([]byte(e.doc[2]))
			doc := b.Bytes()
			hl := doc[start:end]

			err := wrapDecodeError(doc, &unstable.ParserError{
				Highlight: hl,
				Message:   e.msg,
			})

			var derr *DecodeError
			if !errors.As(err, &derr) {
				t.Errorf("error not in expected format")

				return
			}

			assert.Equal(t, strings.Trim(e.expected, "\n"), derr.String())
		})
	}
}

func TestDecodeError_Accessors(t *testing.T) {

	e := DecodeError{
		message: "foo",
		line:    1,
		column:  2,
		key:     []string{"one", "two"},
		human:   "bar",
	}
	assert.Equal(t, "toml: foo", e.Error())
	r, c := e.Position()
	assert.Equal(t, 1, r)
	assert.Equal(t, 2, c)
	assert.Equal(t, Key{"one", "two"}, e.Key())
	assert.Equal(t, "bar", e.String())
}

func ExampleDecodeError() {
	doc := `name = 123__456`

	s := map[string]interface{}{}
	err := Unmarshal([]byte(doc), &s)

	fmt.Println(err)

	var derr *DecodeError
	if errors.As(err, &derr) {
		fmt.Println(derr.String())
		row, col := derr.Position()
		fmt.Println("error occurred at row", row, "column", col)
	}
	// Output:
	// toml: number must have at least one digit between underscores
	// 1| name = 123__456
	//  |           ~~ number must have at least one digit between underscores
	// error occurred at row 1 column 11
}
