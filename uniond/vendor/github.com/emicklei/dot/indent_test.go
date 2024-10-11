package dot

import (
	"bytes"
	"fmt"
	"testing"
)

func TestIndentWriter(t *testing.T) {
	b := new(bytes.Buffer)
	i := NewIndentWriter(b)
	i.WriteString("doc {")
	i.NewLineIndentWhile(func() {
		fmt.Fprint(i, "chapter {")
		i.NewLineIndentWhile(func() {
			fmt.Fprint(i, "chapter text")
		})
		i.WriteString("}")
	})
	i.WriteString("}")
	got := b.String()
	want := `doc {
	chapter {
		chapter text
	}
}`
	if got != want {
		t.Fail()
	}
}

func TestIndentWriter_IndentWhile(t *testing.T) {
	b := new(bytes.Buffer)
	i := NewIndentWriter(b)
	i.IndentWhile(func() {
		i.WriteString("[")
		i.IndentWhile(func() {
			i.WriteString("test")
		})
		i.WriteString("]")
	})
	got := b.String()
	want := `	[	test]`
	if got != want {
		t.Log(got)
		t.Fail()
	}
}
