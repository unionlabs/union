// Copyright (C) 2022 Michael J. Fromberger. All Rights Reserved.

// Package tomledit allows structural edits of a TOML document.
//
// # Parsing
//
// To parse a TOML text into a Document, call Parse:
//
//	f, err := os.Open("config.toml")
//	...
//	defer f.Close()
//	doc, err := tomledit.Parse(f)
//	if err != nil {
//	   log.Fatalf("Parse: %v", err)
//	}
//
// Once parsed, the structure of the Document is mutable, and changes to the
// document will be reflected when it is written back out.
//
// Note that the parser does not validate the semantics of the resulting
// document. Issues such as duplicate keys, incorrect table order,
// redefinitions, and so forth are not reported by the parser.
//
// # Formatting
//
// To write a Document back into TOML format, use a Formatter:
//
//	var cfg tomledit.Formatter
//	if err := cfg.Format(os.Stdout, doc); err != nil {
//	   log.Fatalf("Format: %v", err)
//	}
//
// # Structure
//
// A Document consists of one or more sections: A "global" section at the
// beginning of the input, with top-level key-value mappings that are not
// captured in a named table; followed by zero or more sections denoting named
// tables.
//
// All sections except the global section have a Heading, which gives the name
// of the section along with any documentation comments attached to it.  The
// contents of the section are represented as a slice of items, each of which
// is a block of comments (concrete type parser.Comments) or a key-value
// mapping (concrete type *parser.KeyValue). Modifying either of these fields
// updates the structure of the section.
//
// # Comments
//
// Each heading or key-value may have a block comment comment attached to
// it. Block comments are attached to an item if they occur immediately before
// it (with no intervening blanks); otherwise block comments stand alone as
// their own items.
//
// Headings and values may also have trailing line comments attached, if they
// occur on the same line as the value. For example:
//
//	# unattached comment
//	# this block stands alone
//
//	# another unattached comment
//	# this block is separate from the one above
//
//	# attached comment, belongs to the following table
//	[tablename]
//
//	foo = 'bar'  # attached comment, belongs to the preceding mapping
//
// The comments attached to an item move with that item, and retain their
// attachments when the document is formatted. To remove an attached comment,
// set the corresponding field to a zero value.
//
// # Keys and Values
//
// Keys are denoted as slices of strings (parser.Key), representing the
// dot-separated components of a TOML name (e.g., left.center.right).  Use the
// Equal and IsPrefixOf methods of a key to compare it to another key.  It is
// safe to construct key slices programmatically, or use parser.ParseKey.
//
// Values are denoted as parser.Datum implementations. Primitive values
// (parser.Token) are stored as uninterpreted text; this package does not
// convert them into Go values (although you are free to do so). Otherwise, a
// value is either an array (parser.Array) or an inline table (parser.Inline).
// Values in key-value mappings are bound with an optional trailing line
// comment, if one occurs on the same line as the value.
//
// To construct values programmatically, use parser.ParseValue.
package tomledit

import (
	"fmt"
	"io"

	"github.com/creachadair/tomledit/parser"
)

// A Document represents the contents of a TOML document.
// Edits applied to the fields of a document are preserved when the document is
// written using a Formatter.
type Document struct {
	// The global section, containing all keys not in a named table.
	// Setting this field to nil removes the global section.
	Global *Section

	// This slice contains one entry for each named table section.  Modifying
	// the order and content of this slice affects the formatted output.
	Sections []*Section
}

// First returns the first entry in d with the given key, or nil.
func (d *Document) First(key ...string) *Entry {
	want := parser.Key(key)

	var first *Entry
	d.Scan(func(full parser.Key, e *Entry) bool {
		if full.Equals(want) {
			first = e
			return false
		}
		return true
	})
	return first
}

// Find returns a slice of all entries in d with the given key, or nil.
func (d *Document) Find(key ...string) []*Entry {
	want := parser.Key(key)

	var found []*Entry
	d.Scan(func(full parser.Key, e *Entry) bool {
		if full.Equals(want) {
			found = append(found, e)
		}
		return true
	})
	return found
}

// Scan calls f for every key-value pair defined in d, in lexical order.
// The arguments to f are the complete key of the item and the entry.
// Traversal continues until all items have been visited or f returns false.
// Scan reports whether the traversal was stopped early.
//
// Editing the contents of existing sections and mappings is safe.  It is not
// safe to remove or reorder sections or mappings during a scan.
func (d *Document) Scan(f func(parser.Key, *Entry) bool) bool {
	if !d.Global.scan(d, f) {
		return false
	}

	for _, s := range d.Sections {
		if !s.scan(d, f) {
			return false
		}
	}
	return true
}

// A Section represents a section of a TOML document.  A section represents a
// table and all the block comments and key-value pairs it contains.
type Section struct {
	// The heading of the section. For the global section it is nil; otherwise
	// it contains the name of a table. Modifying the contents of this field
	// edits the section within the document.
	*parser.Heading

	// The items comprising the section. Modifying the contents of this slice
	// edits section within its enclosing document.
	Items []parser.Item
}

// IsGlobal reports whether s is a global (top-level) table.
func (s *Section) IsGlobal() bool { return s != nil && s.Heading == nil }

// TableName returns the name of the table defined by s, which is nil for a
// global (top-level) table.
func (s *Section) TableName() parser.Key {
	if s == nil || s.Heading == nil {
		return nil
	}
	return s.Heading.Name
}

// scan the contents of a section attached to a document, including the section
// itself as the first entry if it has a name.
func (s *Section) scan(doc *Document, f func(parser.Key, *Entry) bool) bool {
	// Report the section alone, if it has a heading.
	if !s.IsGlobal() {
		if !f(s.TableName(), &Entry{Section: s, parent: &doc.Sections}) {
			return false
		}
	}
	return s.Scan(f)
}

// Scan calls f for every key-value pair defined inside s, not including s
// itself, in lexical order.  The arguments to f are the complete key of the
// item and the entry.  Traversal continues until all items have been visited
// or f returns false.  Scan reports whether the traversal was stopped early.
//
// Editing the contents of existing mappings is safe.  It is not safe to remove
// or reorder items during a scan.
func (s *Section) Scan(f func(parser.Key, *Entry) bool) bool {
	if s == nil {
		return true // nothing to do
	}

	// Scan the contents of the section.
	base := s.TableName()
	for _, item := range s.Items {
		kv, ok := item.(*parser.KeyValue)
		if !ok {
			continue // not a key-value mapping
		}

		key := append(base, kv.Name...)
		if !f(key, &Entry{Section: s, KeyValue: kv, parent: &s.Items}) {
			return false
		}

		// If the value is an inline table, traverse it too.
		if !scanInline(key, s, &kv.Value.X, f) {
			return false
		}
	}
	return true
}

// scanInline recursively scans the contents of an inline table.
func scanInline(root parser.Key, s *Section, par *parser.Datum, f func(parser.Key, *Entry) bool) bool {
	inline, ok := (*par).(parser.Inline)
	if !ok {
		return true
	}
	for _, kv := range inline {
		key := append(root, kv.Name...)
		if !f(key, &Entry{Section: s, KeyValue: kv, parent: par}) {
			return false
		}
		if !scanInline(key, s, &kv.Value.X, f) {
			return false
		}
	}
	return true
}

// An Entry represents the result of looking up a key in a document.
type Entry struct {
	// The section of the document containing this entry.  For entries matching
	// tables, this is the complete result. Modifying the contents of this field
	// edits the document.
	*Section

	// For an entry representing a key-value mapping, this is its definition.
	// This field is nil for entries matching section headers.  Modifying the
	// contents of this field edits the document.
	*parser.KeyValue

	// The container where this entry was found. The concrete type is:
	// For sections: *[]*Section
	// For top-level mappings: *[]parser.Item
	// For inline mappings: *parser.Datum containing parser.Inline
	parent interface{}
}

func (e Entry) String() string {
	if e.IsSection() {
		return e.Section.Heading.String()
	}
	return fmt.Sprintf("%s :: %s", e.Section.Heading, e.KeyValue)
}

// Remove removes the entry from its location in the document, and reports
// whether any change was made in doing so.
func (e *Entry) Remove() bool {
	if e == nil {
		return false
	}
	switch t := e.parent.(type) {
	case nil:
		return false

	case *[]*Section:
		for i, s := range *t {
			if s == e.Section {
				*t = append((*t)[:i], (*t)[i+1:]...)
				e.parent = nil
				return true
			}
		}

	case *[]parser.Item:
		for i, kv := range *t {
			if kv == e.KeyValue {
				*t = append((*t)[:i], (*t)[i+1:]...)
				e.parent = nil
				return true
			}
		}

	case *parser.Datum:
		inline := (*t).(parser.Inline)
		for i, kv := range inline {
			if kv == e.KeyValue {
				*t = append(inline[:i], inline[i+1:]...)
				e.parent = nil
				return true
			}
		}
	}
	return false
}

// IsSection reports whether e represents a section head.
func (e Entry) IsSection() bool { return e.KeyValue == nil }

// IsMapping reports whether e represents a key-value mapping.
func (e Entry) IsMapping() bool { return e.KeyValue != nil }

// IsInline reports whether e is inside an inline table.
func (e Entry) IsInline() bool {
	if e.KeyValue != nil {
		_, ok := e.parent.(*parser.Inline)
		return ok
	}
	return false
}

// Parse parses a TOML document from r.
func Parse(r io.Reader) (*Document, error) {
	items, err := parser.New(r).Items()
	if err != nil {
		return nil, err
	}
	sec := parseSections(items)
	return &Document{Global: sec[0], Sections: sec[1:]}, nil
}

// parseSections parses items into a slice of sections. The result will always
// have at least one item, the first, containing the global section.
func parseSections(items []parser.Item) []*Section {
	// There is always at least one section representing the unnamed but
	// possibly empty table at the beginning of the document.
	sections := []*Section{new(Section)}

	cur := 0
	for i := 0; i < len(items); i++ {
		h, ok := items[i].(*parser.Heading)
		if !ok {
			continue
		}
		if i > cur {
			// N.B. Make a copy so edits do not clobber other sections.
			sections[len(sections)-1].Items = copyItems(items[cur:i])
		}
		sections = append(sections, &Section{Heading: h})
		cur = i
	}

	// Pick up any leftovers at the end of the input.
	if cur+1 <= len(items) {
		sections[len(sections)-1].Items = copyItems(items[cur:])
	}
	return sections
}

func copyItems(items []parser.Item) []parser.Item {
	// When copying the items in a section, don't include the item that denotes
	// the section heading. For tables that have one, it's already stored.
	if _, ok := items[0].(*parser.Heading); ok {
		items = items[1:]
	}
	if len(items) == 0 {
		return nil
	}
	out := make([]parser.Item, len(items))
	copy(out, items)
	return out
}
