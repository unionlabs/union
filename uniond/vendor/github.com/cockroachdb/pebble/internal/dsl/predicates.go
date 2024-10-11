// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package dsl

import (
	"fmt"
	"go/token"
	"strconv"
	"strings"
	"sync/atomic"

	"github.com/cockroachdb/errors"
)

// Predicate encodes conditional logic that yields a boolean.
type Predicate[E any] interface {
	Evaluate(E) bool
	String() string
}

// Not returns a Predicate that negates the provided predicate.
func Not[E any](p Predicate[E]) Predicate[E] { return not[E]{Predicate: p} }

// And returns a Predicate that evaluates to true if all its operands evaluate
// to true.
func And[E any](preds ...Predicate[E]) Predicate[E] { return and[E](preds) }

// Or returns a Predicate that evaluates to true if any of its operands evaluate
// true.
func Or[E any](preds ...Predicate[E]) Predicate[E] { return or[E](preds) }

// OnIndex returns a Predicate that evaluates to true on its N-th call.
func OnIndex[E any](n int32) *Index[E] {
	p := new(Index[E])
	p.Int32.Store(n)
	return p
}

// Index is a Predicate that evaluates to true only on its N-th invocation.
type Index[E any] struct {
	atomic.Int32
}

// String implements fmt.Stringer.
func (p *Index[E]) String() string {
	return fmt.Sprintf("(OnIndex %d)", p.Int32.Load())
}

// Evaluate implements Predicate.
func (p *Index[E]) Evaluate(E) bool { return p.Int32.Add(-1) == -1 }

type not[E any] struct {
	Predicate[E]
}

func (p not[E]) String() string    { return fmt.Sprintf("(Not %s)", p.Predicate.String()) }
func (p not[E]) Evaluate(e E) bool { return !p.Predicate.Evaluate(e) }

type and[E any] []Predicate[E]

func (p and[E]) String() string {
	var sb strings.Builder
	sb.WriteString("(And")
	for i := 0; i < len(p); i++ {
		sb.WriteRune(' ')
		sb.WriteString(p[i].String())
	}
	sb.WriteRune(')')
	return sb.String()
}

func (p and[E]) Evaluate(e E) bool {
	ok := true
	for i := range p {
		ok = ok && p[i].Evaluate(e)
	}
	return ok
}

type or[E any] []Predicate[E]

func (p or[E]) String() string {
	var sb strings.Builder
	sb.WriteString("(Or")
	for i := 0; i < len(p); i++ {
		sb.WriteRune(' ')
		sb.WriteString(p[i].String())
	}
	sb.WriteRune(')')
	return sb.String()
}

func (p or[E]) Evaluate(e E) bool {
	ok := false
	for i := range p {
		ok = ok || p[i].Evaluate(e)
	}
	return ok
}

func parseNot[E any](p *Parser[Predicate[E]], s *Scanner) Predicate[E] {
	preds := parseVariadicPredicate(p, s)
	if len(preds) != 1 {
		panic(errors.Newf("dsl: not accepts exactly 1 argument, given %d", len(preds)))
	}
	return not[E]{Predicate: preds[0]}
}

func parseAnd[E any](p *Parser[Predicate[E]], s *Scanner) Predicate[E] {
	return And[E](parseVariadicPredicate[E](p, s)...)
}

func parseOr[E any](p *Parser[Predicate[E]], s *Scanner) Predicate[E] {
	return Or[E](parseVariadicPredicate[E](p, s)...)
}

func parseOnIndex[E any](p *Parser[Predicate[E]], s *Scanner) Predicate[E] {
	i, err := strconv.ParseInt(s.Consume(token.INT).Lit, 10, 32)
	if err != nil {
		panic(err)
	}
	s.Consume(token.RPAREN)
	return OnIndex[E](int32(i))
}

func parseVariadicPredicate[E any](p *Parser[Predicate[E]], s *Scanner) (ret []Predicate[E]) {
	tok := s.Scan()
	for tok.Kind == token.LPAREN || tok.Kind == token.IDENT {
		ret = append(ret, p.ParseFromPos(s, tok))
		tok = s.Scan()
	}
	assertTok(tok, token.RPAREN)
	return ret
}
