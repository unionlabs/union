// Copyright (C) 2022 Michael J. Fromberger. All Rights Reserved.

package parser

import (
	"fmt"
	"io"
	"strings"

	"github.com/creachadair/tomledit/scanner"
)

// An Item is an element in a TOML document. The concrete type of an Item is
// one of Comments, Heading, or KeyValue.
type Item interface{ isItem() }

// Comments is an Item that represents a block of comments.  Each entry
// represents a single comment line.
//
// Comment lines parsed from a source file include the comment marker at the
// beginning.  When constructing comments programmatically, the comment marker
// is optional; one will be prepended if it does not exist.
type Comments []string

func (Comments) isItem()      {}
func (Comments) isArrayItem() {}

func (c Comments) String() string {
	return strings.Join(c.Clean(), "\n")
}

// Clean returns a copy of c in which each line has been "cleaned", removing
// leading and trailing whitespace and prepending a "#" marker to each line
// that does not already have one. If a line includes embedded newlines, it is
// split into multiple lines, each of which is cleaned.
func (c Comments) Clean() []string {
	// Preallocate space for as many lines as original entries, but if there are
	// embedded newlines this may expand beyond the original allocation.  Hence,
	// the use of append here is intentional.
	out := make([]string, 0, len(c))
	for _, s := range c {
		for _, line := range strings.Split(strings.TrimSpace(s), "\n") {
			clean := strings.TrimSpace(line)
			if clean == "" {
				clean = "#" // skip trailing space
			} else if !strings.HasPrefix(clean, "#") {
				clean = "# " + clean
			}
			out = append(out, clean)
		}
	}
	return out
}

// CleanTrailer returns a copy of s that is suitable for use as a line-ending
// comment. It removes leading and trailing whitespace and prepends a "#"
// marker if necessary.  If s contains newlines, they are converted to spaces.
func CleanTrailer(s string) string {
	clean := strings.ReplaceAll(strings.TrimSpace(s), "\n", " ")
	if clean == "" {
		return "#"
	} else if !strings.HasPrefix(clean, "#") {
		return "# " + clean
	}
	return clean
}

// Heading is an Item that represents a table section heading.
type Heading struct {
	Block   Comments // a block comment before the heading (empty if none)
	Trailer string   // a trailing line comment after the heading (empty if none)
	IsArray bool     // whether this table is part of a table array
	Name    Key      // the name of the table
	Line    int      // the input line where the heading was defined (1-based)
}

func (Heading) isItem() {}

func (h *Heading) String() string {
	if h == nil {
		return "(empty)"
	} else if h.IsArray {
		return fmt.Sprintf("[[%s]]", h.Name)
	}
	return fmt.Sprintf("[%s]", h.Name)
}

// KeyValue is an Item that represents a key-value definition.
type KeyValue struct {
	Block Comments // a block comment before the key-value pair (empty if none)
	Name  Key
	Value Value
	Line  int // the input line where the key-value was defined (1-based)
}

func (KeyValue) isItem() {}

func (kv *KeyValue) String() string {
	if kv == nil {
		return ""
	}
	return fmt.Sprintf("%s = %s", kv.Name, kv.Value)
}

// A Key represents a dotted compound name.
type Key []string

// ParseKey parses s as a TOML key.
func ParseKey(s string) (Key, error) {
	p := New(strings.NewReader(s))
	if _, err := p.require(); err != nil {
		return nil, err
	}
	key, _, err := p.parseKey()
	if err != nil {
		return nil, err
	} else if p.sc.Err() != io.EOF {
		return key, fmt.Errorf("at %s: extra input after key", p.sc.Location().First)
	}
	return key, nil
}

// Equals reports whether k and k2 are equal.
func (k Key) Equals(k2 Key) bool {
	return k.IsPrefixOf(k2) && len(k) == len(k2)
}

// Before reports whether k is lexicographically prior to k2.
func (k Key) Before(k2 Key) bool {
	i, j := 0, 0
	for i < len(k) && j < len(k2) {
		if k[i] < k2[j] {
			return true
		} else if k[i] > k2[j] {
			return false
		}
		i++
		j++
	}
	return i == len(k) && j < len(k2)
}

// IsPrefixOf reports whether k is a prefix of k2.
func (k Key) IsPrefixOf(k2 Key) bool {
	if len(k) > len(k2) {
		return false
	}
	for i, elt := range k {
		if elt != k2[i] {
			return false
		}
	}
	return true
}

func (k Key) String() string {
	ss := make([]string, len(k))
	for i, word := range k {
		if scanner.IsWord(word) && word != "" {
			ss[i] = word
		} else {
			ss[i] = fmt.Sprintf(`"%s"`, string(scanner.Escape(word)))
		}
	}
	return strings.Join(ss, ".")
}

// A Value represents a value in an array or a key-value assignment.
type Value struct {
	Trailer string // a trailing line-comment after the value (empty if none)
	X       Datum  // the concrete value
	Line    int    // the input line where the value is defined (1-based)
}

// MustValue parses s as a TOML value. It panics if parsing fails.  This is
// intended for use at program initialization time, or for static string
// constants that are expected to be always valid.  For all other cases, use
// ParseValue to check the error.
func MustValue(s string) Value {
	v, err := ParseValue(s)
	if err != nil {
		panic(fmt.Errorf("value parse failed: %w", err))
	}
	return v
}

// ParseValue parses s as a TOML value.
func ParseValue(s string) (Value, error) {
	p := New(strings.NewReader(s))
	if _, err := p.require(); err != nil {
		return Value{}, err
	}
	val, err := p.parseValue()
	if err != nil {
		return Value{}, err
	}
	next, err := p.require(scanner.Comment, scanner.Newline)
	if err != nil && err != io.EOF {
		return Value{}, err
	} else if next == scanner.Comment {
		val.Trailer = string(p.sc.Text())
	}
	if _, err := p.require(); err != io.EOF {
		return Value{}, fmt.Errorf("at %s: extra input after value", p.sc.Location().First)
	}
	val.Line = 0
	return val, nil
}

func (Value) isItem()      {}
func (Value) isArrayItem() {}

func (v Value) String() string { return v.X.String() }

// WithComment returns a copy of v with its trailer set to text.
func (v Value) WithComment(text string) Value { v.Trailer = text; return v }

// A Datum is the representation of a data value. The concrete type of a Datum
// is one of Token, Array, or Inline.
type Datum interface {
	isDatum()
	String() string
}

// A Token represents a lexical data element such as a string, integer,
// floating point literal, Boolean, or date/time literal.
type Token struct {
	Type scanner.Token // the lexical type of the token
	text string
}

func (Token) isDatum() {}

func (t Token) String() string {
	if t.Type.IsValue() {
		return t.text
	}
	return t.Type.String()
}

// An ArrayItem is an element in a TOML array value. The concrete type of an
// ArrayItem is one of Comments or Value.
type ArrayItem interface {
	isArrayItem()
}

// An Array represents a (possibly empty) array value.
type Array []ArrayItem

func (Array) isDatum() {}

func (a Array) String() string {
	if len(a) == 0 {
		return "[]"
	}
	var elts []string
	for _, elt := range a {
		if v, ok := elt.(Value); ok {
			elts = append(elts, v.String())
		}
	}
	return `[` + strings.Join(elts, ", ") + `]`
}

// An Inline represents a (possibly empty) inline table value.
type Inline []*KeyValue

func (Inline) isDatum() {}

func (t Inline) String() string {
	if len(t) == 0 {
		return "{}"
	}

	elts := make([]string, len(t))
	for i, elt := range t {
		elts[i] = elt.String()
	}
	return `{` + strings.Join(elts, ", ") + `}`
}
