// Package print provides helpers for structured output printing.
package print

import (
	"fmt"
	"io"
	"strings"
	"text/tabwriter"
)

// DefaultIndent is the default string for one level of indentation.
const DefaultIndent = "\t"

// Printer provides convenience methods for structured output printing.
// Specifically it stores any errors encountered so error checking does not have
// to be done on every print call. Also provides helpers for managing indentation.
type Printer struct {
	out     io.Writer
	level   int    // current indentation level
	indent  string // indentation string
	pending bool   // if there's a pending indentation
	err     error  // saved error from printing
}

// New builds a printer writing to w.
func New(w io.Writer) Printer {
	return Printer{
		out:    w,
		indent: DefaultIndent,
	}
}

// SetIndentString configures the string used for one level of indentation.
func (p *Printer) SetIndentString(indent string) {
	p.indent = indent
}

// Indent by one level.
func (p *Printer) Indent() {
	p.level++
}

// Dedent by one level.
func (p *Printer) Dedent() {
	p.level--
}

// Linef prints a formatted line.
func (p *Printer) Linef(format string, args ...interface{}) {
	p.Printf(format, args...)
	p.NL()
}

// NL prints a newline.
func (p *Printer) NL() {
	p.Printf("\n")
	p.pending = true
}

// Printf prints formatted output.
func (p *Printer) Printf(format string, args ...interface{}) {
	if p.err != nil {
		return
	}
	if p.pending {
		indent := strings.Repeat(p.indent, p.level)
		format = indent + format
		p.pending = false
	}
	_, err := fmt.Fprintf(p.out, format, args...)
	p.SetError(err)
}

// Error returns the first error that occurred so far, if any.
func (p *Printer) Error() error {
	return p.err
}

// SetError records a possible error.
func (p *Printer) SetError(err error) {
	if p.err == nil {
		p.err = err
	}
}

// TabWriter provides tabwriter.Writer functionality with the Printer interface.
type TabWriter struct {
	tw *tabwriter.Writer
	Printer
}

// NewTabWriter builds a TabWriter. Arguments are the same as for tabwriter.NewWriter.
func NewTabWriter(w io.Writer, minwidth, tabwidth, padding int, padchar byte, flags uint) *TabWriter {
	tw := tabwriter.NewWriter(w, minwidth, tabwidth, padding, padchar, flags)
	return &TabWriter{
		tw:      tw,
		Printer: New(tw),
	}
}

// Flush the tabwriter.
func (p *TabWriter) Flush() {
	p.SetError(p.tw.Flush())
}
