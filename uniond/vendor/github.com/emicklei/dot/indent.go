package dot

import (
	"fmt"
	"io"
)

// IndentWriter decorates an io.Writer to insert leading TAB \t character per line
type IndentWriter struct {
	level  int
	writer io.Writer
}

// NewIndentWriter returns a new IndentWriter with indent level 0.
func NewIndentWriter(w io.Writer) *IndentWriter {
	return &IndentWriter{level: 0, writer: w}
}

// Indent raises the level and writes the extra \t (TAB) character.
func (i *IndentWriter) Indent() {
	i.level++
	fmt.Fprint(i.writer, "\t")
}

// BackIndent drops the level with one.
func (i *IndentWriter) BackIndent() {
	i.level--
}

// IndentWhile call the blocks after an indent and will restore that indent afterward.
func (i *IndentWriter) IndentWhile(block func()) {
	i.Indent()
	block()
	i.BackIndent()
}

// NewLineIndentWhile is a variation of IndentWhile that produces extra newlines.
func (i *IndentWriter) NewLineIndentWhile(block func()) {
	i.NewLine()
	i.Indent()
	block()
	i.BackIndent()
	i.NewLine()
}

// NewLine writes the new line and a number of tab \t characters that matches the level count.
func (i *IndentWriter) NewLine() {
	fmt.Fprint(i.writer, "\n")
	for j := 0; j < i.level; j++ {
		fmt.Fprint(i.writer, "\t")
	}
}

// Write makes it an io.Writer
func (i *IndentWriter) Write(data []byte) (n int, err error) {
	return i.writer.Write(data)
}

// WriteString is a convenient Write.
func (i *IndentWriter) WriteString(s string) (n int, err error) {
	fmt.Fprint(i.writer, s)
	return len(s), nil
}
