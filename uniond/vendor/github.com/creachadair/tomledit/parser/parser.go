// Copyright (C) 2022 Michael J. Fromberger. All Rights Reserved.

// Package parser implements a parser for TOML, as defined by the TOML v1
// specification https://toml.io/en/v1.0.0.
//
// Unlike other TOML libraries, this parser does not unmarshal into Go values,
// but only constructs an abstract syntax tree for its input. This is meant to
// support manipulating the structure of a TOML document.
package parser

import (
	"bytes"
	"fmt"
	"io"
	"strings"

	"github.com/creachadair/tomledit/scanner"
)

// A Parser is a parser for TOML syntax.
type Parser struct {
	sc *scanner.Scanner
}

// New constructs a new parser that consumes input from r.
func New(r io.Reader) *Parser { return &Parser{sc: scanner.New(r)} }

// Items reads the top-level items from the input.
func (p *Parser) Items() ([]Item, error) {
	var items []Item
	for {
		item, err := p.parseItem()
		if err == io.EOF {
			return items, nil
		} else if err != nil {
			return nil, err
		}
		items = append(items, item)
	}
}

// parseItem parses a single top-level item in a TOML file.
// The concrete type of Item will be Comments, Heading, or KeyValue.
//
// A Comments value is a block of contiguous comments that are not attached to
// some other form. A block comment attached to Heading and KeyValue items is
// attached to the item itself.
func (p *Parser) parseItem() (Item, error) {
	var block []string
	for p.sc.Next() == nil {
		switch p.sc.Token() {
		case scanner.Comment:
			block = append(block, string(p.sc.Text()))
			continue

		case scanner.Newline:
			if len(block) != 0 {
				return Comments(block), nil
			}
			continue

		case scanner.LBracket:
			return p.parseHeading(p.sc.Token(), block)

		case scanner.Word,
			scanner.String, scanner.LString,
			scanner.Integer, scanner.Float,
			scanner.LocalDate:
			return p.parseKeyValue(p.sc.Token(), block)

		default:
			return nil, fmt.Errorf("at %s: unexpected %v", p.sc.Location().First, p.sc.Token())
		}
	}
	if p.sc.Err() == io.EOF && len(block) != 0 {
		return Comments(block), nil
	}
	return nil, p.sc.Err()
}

// parseHeading parses the heading of a table ("[name]") or table-array ("[[name]]").
func (p *Parser) parseHeading(tok scanner.Token, comments []string) (*Heading, error) {
	var isArray bool
	if next, err := p.require(); err != nil {
		return nil, err
	} else if next == tok && p.sc.Prev() == tok { // recognize "[["
		isArray = true
		if _, err := p.advance(next); err != nil {
			return nil, err
		}
	}
	key, line, err := p.parseKey()
	if err != nil {
		return nil, err
	}

	// Check for the matching close brace.
	next, err := p.advance(scanner.RBracket)
	if err != nil && err != io.EOF {
		return nil, err
	}
	if isArray { // require "]]"
		if next != scanner.RBracket || p.sc.Prev() != next {
			return nil, fmt.Errorf(`at %s: got %v, want "]]"`, p.sc.Location().First, next)
		} else if _, err := p.advance(scanner.RBracket); err != nil && err != io.EOF {
			return nil, err
		}
	}

	hd := &Heading{
		Block:   Comments(comments),
		IsArray: isArray,
		Name:    key,
		Line:    line,
	}

	// Check for an optional trailing comment.
	if next == scanner.Comment {
		hd.Trailer = string(p.sc.Text())
	}
	return hd, nil
}

// parseInlineKeyValue parses an undecorated key-value assignment.
func (p *Parser) parseInlineKeyValue(tok scanner.Token) (*KeyValue, error) {
	key, line, err := p.parseKey()
	if err != nil {
		return nil, err
	}
	if _, err := p.advance(scanner.Equal); err != nil {
		return nil, err
	}
	val, err := p.parseValue()
	if err != nil {
		return nil, err
	}
	return &KeyValue{Name: key, Value: val, Line: line}, nil
}

// parseKeyValue parses a key-value assignment ("name = value").
func (p *Parser) parseKeyValue(tok scanner.Token, comments []string) (*KeyValue, error) {
	kv, err := p.parseInlineKeyValue(tok)
	if err != nil {
		return nil, err
	}
	next, err := p.require(scanner.Comment, scanner.Newline)
	if err != nil && err != io.EOF {
		return nil, err
	}
	// Add block and trailing comments, as needed.
	kv.Block = Comments(comments)
	if next == scanner.Comment {
		kv.Value.Trailer = string(p.sc.Text())
	}
	return kv, nil
}

// parseKey parses a (possibly compound) key for a heading or key-value assignment.
func (p *Parser) parseKey() (Key, int, error) {
	var result Key
	var line int
	for {
		text := p.sc.Text()
		if line == 0 {
			line = p.sc.Location().First.Line
		}
		switch p.sc.Token() {
		case scanner.Word, scanner.Integer, scanner.LocalDate:
			result = append(result, string(text))

		case scanner.String:
			unq, err := scanner.Unescape(text[1 : len(text)-1]) // remove quotes, unescape
			if err != nil {
				return nil, 0, fmt.Errorf("at %s: invalid string: %w", p.sc.Location().First, err)
			}
			result = append(result, string(unq))

		case scanner.LString:
			result = append(result, string(text[1:len(text)-1])) // remove quotes

		case scanner.Float:
			// Take apart float literals that have decimal points in them.
			if i := bytes.IndexAny(text, "+"); i >= 0 {
				return nil, 0, fmt.Errorf(`at %s: invalid %q in key`, p.sc.Location().First, text[i])
			}
			result = append(result, strings.Split(string(text), ".")...)

		default:
			return nil, 0, fmt.Errorf("at %s: got %v, want name or string",
				p.sc.Location().First, p.sc.Token())
		}

		// Check for a dotted continuation of the name.
		next, err := p.require()
		if err != nil && err != io.EOF {
			return nil, 0, err
		}
		if next != scanner.Dot {
			return result, line, nil
		}
		if _, err := p.require(
			scanner.Word, scanner.String, scanner.LString,
			scanner.Integer, scanner.Float, scanner.LocalDate,
		); err != nil {
			return nil, 0, err
		}
	}
}

// parseValue parses a value on the right-hand side of a key-value assignment.
func (p *Parser) parseValue() (Value, error) {
	var datum Datum
	var err error

	next := p.sc.Token()
	line := p.sc.Location().First.Line
	if next.IsValue() {
		// Special case: Bare words are not allowed except true and false.
		text := string(p.sc.Text())
		if next == scanner.Word && text != "true" && text != "false" {
			return Value{}, fmt.Errorf("at %s: got %v (%q), wanted value, array, or inline table",
				p.sc.Location().First, next, text)
		}
		datum = Token{Type: next, text: text}
	} else if next == scanner.LBracket {
		datum, err = p.parseArrayValue()
	} else if next == scanner.LInline {
		datum, err = p.parseInlineValue()
	} else {
		return Value{}, fmt.Errorf("at %s: got %v, wanted value, array, or inline table",
			p.sc.Location().First, next)
	}
	if err != nil {
		return Value{}, err
	}
	return Value{X: datum, Line: line}, nil
}

// parseArrayValue parses an inline array value on the right-hand side of a
// key-value assignment. The starting token must be the opening "[".
func (p *Parser) parseArrayValue() (Array, error) {
	var block []string
	var result Array
	pop := func() {
		if len(block) != 0 {
			result = append(result, Comments(block))
			block = nil
		}
	}

	var itemLine int
	var wantComma bool
	for {
		next, err := p.require()
		if err == io.EOF {
			return nil, fmt.Errorf("at %v: unclosed array value", p.sc.Location().First)
		} else if err != nil {
			return nil, err
		}
		switch next {
		case scanner.RBracket:
			pop()
			return result, nil

		case scanner.Comment:
			// If we see a line comment on the same line as the previous item,
			// attach it as a trailer on that item. The legerdemain here is
			// because the comment may not occur till after a comma.
			if line := p.sc.Location().First.Line; line == itemLine {
				pop()
				last := len(result) - 1
				v := result[last].(Value)
				v.Trailer = string(p.sc.Text())
				result[last] = v
			} else {
				block = append(block, string(p.sc.Text()))
			}

		case scanner.Newline:
			pop()

		case scanner.Comma:
			if !wantComma {
				return nil, fmt.Errorf("at %v: unexpected %v", p.sc.Location().First, scanner.Comma)
			}
			wantComma = false

		default:
			if wantComma {
				return nil, fmt.Errorf("at %v: got %v, want %v", p.sc.Location().First, next, scanner.Comma)
			}
			item, err := p.parseValue()
			if err != nil {
				return nil, err
			}
			pop()
			itemLine = p.sc.Location().Last.Line
			wantComma = true
			result = append(result, item)
		}
	}
}

// parsesInlineValue parses an inline table. The starting token must be the
// opening "{" for the table.
func (p *Parser) parseInlineValue() (Inline, error) {
	if next, err := p.require(); err != nil {
		return nil, err
	} else if next == scanner.RInline {
		return nil, nil // OK, empty
	}

	var result Inline
	for {
		kv, err := p.parseInlineKeyValue(p.sc.Token())
		if err != nil {
			return nil, err
		}
		result = append(result, kv)

		// Unlike arrays, inline tables do not permit trailing commas.
		if next, err := p.require(scanner.Comma, scanner.RInline); err != nil {
			return nil, err
		} else if next == scanner.RInline {
			return result, nil
		} else if _, err := p.require(); err != nil {
			return nil, err
		}
		// Reaching here, we saw a comma; go for another value.
	}
}

// require advances to the next token in the input, reporting an error if there
// are no further tokens available. If token types are provided, the next token
// type must equal one of them or an error is reported naming those tokens;
// otherwise any token type is accepted.
func (p *Parser) require(tokens ...scanner.Token) (scanner.Token, error) {
	if err := p.sc.Next(); err != nil {
		return scanner.Invalid, err
	} else if len(tokens) != 0 {
		got := p.sc.Token()
		for _, want := range tokens {
			if got == want {
				return got, nil
			}
		}
		return scanner.Invalid, fmt.Errorf("at %s: got %v, wanted %v",
			p.sc.Location().First, got, tokLabel(tokens))
	}
	return p.sc.Token(), nil
}

// advance checks that the current token matches the given type and advances to
// the next token in the input, reporting an error if no further tokens are
// available.
func (p *Parser) advance(tok scanner.Token) (scanner.Token, error) {
	if got := p.sc.Token(); got != tok {
		return tok, fmt.Errorf("at %s: got %v, wanted %v", p.sc.Location().First, got, tok)
	} else if err := p.sc.Next(); err != nil {
		return scanner.Invalid, err
	}
	return p.sc.Token(), nil
}

// tokLabel makes a human-readable summary string for the given token types.
func tokLabel(tokens []scanner.Token) string {
	if len(tokens) == 0 {
		return "more input"
	} else if len(tokens) == 1 {
		return tokens[0].String()
	}
	last := len(tokens) - 1
	ss := make([]string, len(tokens)-1)
	for i, tok := range tokens[:last] {
		ss[i] = tok.String()
	}
	return strings.Join(ss, ", ") + " or " + tokens[last].String()
}
