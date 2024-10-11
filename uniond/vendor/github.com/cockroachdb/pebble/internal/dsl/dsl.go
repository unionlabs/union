// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

// Package dsl provides facilities for parsing lisp-like domain-specific
// languages (DSL).
package dsl

import (
	"fmt"
	"go/scanner"
	"go/token"
	"strconv"
	"strings"

	"github.com/cockroachdb/errors"
)

// NewParser constructs a new Parser of a lisp-like DSL.
func NewParser[T any]() *Parser[T] {
	p := new(Parser[T])
	p.constants = make(map[string]func() T)
	p.funcs = make(map[string]func(*Parser[T], *Scanner) T)
	return p
}

// NewPredicateParser constructs a new Parser of a Lisp-like DSL, where the
// resulting type implements Predicate[E]. NewPredicateParser predefines a few
// useful functions: Not, And, Or, OnIndex.
func NewPredicateParser[E any]() *Parser[Predicate[E]] {
	p := NewParser[Predicate[E]]()
	p.DefineFunc("Not", parseNot[E])
	p.DefineFunc("And", parseAnd[E])
	p.DefineFunc("Or", parseOr[E])
	p.DefineFunc("OnIndex", parseOnIndex[E])
	return p
}

// A Parser holds the rules and logic for parsing a DSL.
type Parser[T any] struct {
	constants map[string]func() T
	funcs     map[string]func(*Parser[T], *Scanner) T
}

// DefineConstant adds a new constant to the Parser's supported DSL. Whenever
// the provided identifier is used within a constant context, the provided
// closure is invoked to instantiate an appropriate AST value.
func (p *Parser[T]) DefineConstant(identifier string, instantiate func() T) {
	p.constants[identifier] = instantiate
}

// DefineFunc adds a new func to the Parser's supported DSL. Whenever the
// provided identifier is used within a function invocation context, the
// provided closure is invoked to instantiate an appropriate AST value.
func (p *Parser[T]) DefineFunc(identifier string, parseFunc func(*Parser[T], *Scanner) T) {
	p.funcs[identifier] = parseFunc
}

// Parse parses the provided input string.
func (p *Parser[T]) Parse(d string) (ret T, err error) {
	defer func() {
		if r := recover(); r != nil {
			var ok bool
			err, ok = r.(error)
			if !ok {
				panic(r)
			}
		}
	}()

	fset := token.NewFileSet()
	file := fset.AddFile("", -1, len(d))
	var s Scanner
	s.Init(file, []byte(strings.TrimSpace(d)), nil /* no error handler */, 0)
	tok := s.Scan()
	ret = p.ParseFromPos(&s, tok)
	tok = s.Scan()
	if tok.Kind == token.SEMICOLON {
		tok = s.Scan()
	}
	assertTok(tok, token.EOF)
	return ret, err
}

// ParseFromPos parses from the provided current position and associated
// scanner. If the parser fails to parse, it panics. This function is intended
// to be used when composing Parsers of various types.
func (p *Parser[T]) ParseFromPos(s *Scanner, tok Token) T {
	switch tok.Kind {
	case token.IDENT:
		// A constant without any parens, eg. `Reads`.
		p, ok := p.constants[tok.Lit]
		if !ok {
			panic(errors.Errorf("dsl: unknown constant %q", tok.Lit))
		}
		return p()
	case token.LPAREN:
		// Otherwise it's an expression, eg: (OnIndex 1)
		tok = s.Consume(token.IDENT)
		fp, ok := p.funcs[tok.Lit]
		if !ok {
			panic(errors.Errorf("dsl: unknown func %q", tok.Lit))
		}
		return fp(p, s)
	default:
		panic(errors.Errorf("dsl: unexpected token %s; expected IDENT or LPAREN", tok.String()))
	}
}

// A Scanner holds the scanner's internal state while processing a given text.
type Scanner struct {
	scanner.Scanner
}

// Scan scans the next token and returns it.
func (s *Scanner) Scan() Token {
	pos, tok, lit := s.Scanner.Scan()
	return Token{pos, tok, lit}
}

// Consume scans the next token. If the token is not of the provided token, it
// panics. It returns the token itself.
func (s *Scanner) Consume(expect token.Token) Token {
	t := s.Scan()
	assertTok(t, expect)
	return t
}

// ConsumeString scans the next token. It panics if the next token is not a
// string, or if unable to unquote the string. It returns the unquoted string
// contents.
func (s *Scanner) ConsumeString() string {
	lit := s.Consume(token.STRING).Lit
	str, err := strconv.Unquote(lit)
	if err != nil {
		panic(errors.Newf("dsl: unquoting %q: %v", lit, err))
	}
	return str
}

// Token is a lexical token scanned from an input text.
type Token struct {
	pos  token.Pos
	Kind token.Token
	Lit  string
}

// String implements fmt.Stringer.
func (t *Token) String() string {
	if t.Lit != "" {
		return fmt.Sprintf("(%s, %q) at pos %v", t.Kind, t.Lit, t.pos)
	}
	return fmt.Sprintf("%s at pos %v", t.Kind, t.pos)
}

func assertTok(tok Token, expect token.Token) {
	if tok.Kind != expect {
		panic(errors.Errorf("dsl: unexpected token %s; expected %s", tok.String(), expect))
	}
}
