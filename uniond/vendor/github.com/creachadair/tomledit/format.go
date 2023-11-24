// Copyright (C) 2022 Michael J. Fromberger. All Rights Reserved.

package tomledit

import (
	"fmt"
	"io"
	"strings"

	"github.com/creachadair/tomledit/parser"
	"github.com/creachadair/tomledit/scanner"
)

// Format formats the specified document with default options.
func Format(w io.Writer, doc *Document) error {
	var out Formatter
	return out.Format(w, doc)
}

// Formatter defines options for formatting a TOML document.  The zero value is
// ready for use with default options (of which there are presently none).
type Formatter struct{}

func (f Formatter) Format(w io.Writer, doc *Document) error {
	var all []parser.Item
	if doc.Global != nil {
		all = append(all, doc.Global.Items...)
	}
	for i, s := range doc.Sections {
		if len(s.TableName()) == 0 {
			return fmt.Errorf("section at offset %d has no heading", i)
		}
		all = append(all, s.Heading)
		all = append(all, s.Items...)
	}
	return f.indent(all, w, "")
}

func (f Formatter) indent(items []parser.Item, w io.Writer, prefix string) error {
	for i, item := range items {
		// If the current item wants extra space, or the previous item was a
		// block comment, inject a newline prior to rendering the value.  The
		// second case is necessary so the block comment doesn't attach itself to
		// the current item if we read the input back.
		if i > 0 && (wantsBlank(item) || isComment(items[i-1])) {
			fmt.Fprintln(w)
		}
		if err := f.indentItem(item, w, prefix); err != nil {
			return err
		}
	}
	return nil
}

func (f Formatter) indentItem(item parser.Item, w io.Writer, prefix string) error {
	switch t := item.(type) {
	case parser.Comments:
		for _, line := range t.Clean() {
			fmt.Fprint(w, prefix, line, "\n")
		}

	case *parser.Heading:
		if err := f.indentItem(t.Block, w, prefix); err != nil {
			return err
		}
		fmt.Fprint(w, prefix, t) // handles brackets
		if t.Trailer != "" {
			fmt.Fprint(w, "  ", parser.CleanTrailer(t.Trailer))
		}
		fmt.Fprintln(w)

	case *parser.KeyValue:
		if err := f.indentItem(t.Block, w, prefix); err != nil {
			return err
		}
		fmt.Fprint(w, prefix, t.Name, " = ")

		// N.B. Do not pre-indent the RHS of a key-value mapping.
		if err := f.indentDatum(t.Value.X, w, prefix); err != nil {
			return err
		}

		if t.Value.Trailer != "" {
			fmt.Fprint(w, "  ", parser.CleanTrailer(t.Value.Trailer))
		}
		fmt.Fprintln(w)

	default:
		return fmt.Errorf("invalid item type %T", item)

	}
	return nil
}

func (f Formatter) indentArrayItem(item parser.ArrayItem, w io.Writer, prefix string) error {
	switch t := item.(type) {
	case parser.Comments:
		for _, line := range t.Clean() {
			fmt.Fprint(w, prefix, line, "\n")
		}

	case parser.Value:
		// N.B. Plain values only occur in arrays, and in that case we handle the
		// trailing comments separately.
		return f.indentDatum(t.X, w, prefix)

	default:
		return fmt.Errorf("invalid array item type %T", item)

	}
	return nil
}

func (f Formatter) indentDatum(datum parser.Datum, w io.Writer, prefix string) error {
	switch t := datum.(type) {
	case parser.Array:
		return f.indentArray(t, w, prefix)
	case parser.Inline:
		return f.indentInline(t, w, prefix)
	}
	fmt.Fprint(w, prefix, datum.String())
	return nil
}

func (f Formatter) indentArray(array parser.Array, w io.Writer, prefix string) error {
	if len(array) == 0 {
		fmt.Fprint(w, prefix, "[]")
		return nil
	}

	// Array items can only be values or comments. If an array contains any
	// comments, or any of the values is itself a multi-line string, or a
	// non-empty array or inline table, format this array with indentation.
	if shouldIndentArray(array) {
		fmt.Fprint(w, prefix, "[\n")
		for _, elt := range array {
			if err := f.indentArrayItem(elt, w, prefix+"  "); err != nil {
				return err
			}
			if v, ok := elt.(parser.Value); ok {
				fmt.Fprint(w, ",")
				if v.Trailer != "" {
					fmt.Fprint(w, "  ", parser.CleanTrailer(v.Trailer))
				}
			}
			fmt.Fprintln(w)
		}
		fmt.Fprint(w, prefix, "]")
		return nil
	}

	// Reaching here, we know there are no comments or compound values, so we
	// can just format everything plainly.
	elts := make([]string, len(array))
	for i, elt := range array {
		elts[i] = fmt.Sprint(elt)
	}
	fmt.Fprint(w, prefix, "[", strings.Join(elts, ", "), "]")
	return nil
}

func (f Formatter) indentInline(inline parser.Inline, w io.Writer, prefix string) error {
	if len(inline) == 0 {
		fmt.Fprint(w, prefix, "{}")
		return nil
	}

	// The key-value mappings in an inline table cannot have their own comments
	// or newlines at the top level, but may have them inside string literals or
	// compound values.
	fmt.Fprint(w, prefix, "{")
	for i, elt := range inline {
		fmt.Fprint(w, prefix, elt.Name, " = ")
		if err := f.indentDatum(elt.Value.X, w, prefix); err != nil {
			return err
		}
		if i+1 < len(inline) {
			fmt.Fprint(w, ", ")
		}
	}
	fmt.Fprint(w, prefix, "}")
	return nil
}

func shouldIndentArray(array parser.Array) bool {
	for _, elt := range array {
		switch t := elt.(type) {
		case parser.Value:
			if t.Trailer != "" || isInteresting(t.X) {
				return true
			}
		case parser.Comments:
			if len(t) != 0 {
				return true
			}
		}
	}
	return false
}

func isInteresting(datum parser.Datum) bool {
	switch t := datum.(type) {
	case parser.Array:
		return len(t) != 0
	case parser.Inline:
		return len(t) != 0
	case parser.Token:
		return t.Type == scanner.MString || t.Type == scanner.MLString
	default:
		return false
	}
}

func wantsBlank(item parser.Item) bool {
	switch t := item.(type) {
	case parser.Comments:
		return len(t) != 0
	case *parser.Heading:
		return true
	case *parser.KeyValue:
		return len(t.Block) != 0
	}
	return false
}

func isComment(item parser.Item) bool {
	_, ok := item.(parser.Comments)
	return ok
}
