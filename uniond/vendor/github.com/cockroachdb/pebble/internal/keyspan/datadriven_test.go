// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package keyspan

import (
	"bytes"
	"fmt"
	"go/token"
	"io"
	"reflect"
	"strconv"
	"strings"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/dsl"
)

// This file contains testing facilities for Spans and FragmentIterators. It's
// defined here so that it may be used by the keyspan package to test its
// various FragmentIterator implementations.
//
// TODO(jackson): Move keyspan.{Span,Key,FragmentIterator} into internal/base,
// and then move the testing facilities to an independent package, eg
// internal/itertest.

// probe defines an interface for probes that may inspect or mutate internal
// span iterator behavior.
type probe interface {
	// probe inspects, and possibly manipulates, iterator operations' results.
	probe(*probeContext)
}

func parseProbes(probeDSLs ...string) []probe {
	probes := make([]probe, len(probeDSLs))
	var err error
	for i := range probeDSLs {
		probes[i], err = probeParser.Parse(probeDSLs[i])
		if err != nil {
			panic(err)
		}
	}
	return probes
}

func attachProbes(iter FragmentIterator, pctx probeContext, probes ...probe) FragmentIterator {
	if pctx.log == nil {
		pctx.log = io.Discard
	}
	for i := range probes {
		iter = &probeIterator{
			iter:     iter,
			probe:    probes[i],
			probeCtx: pctx,
		}
	}
	return iter
}

// probeContext provides the context within which a probe is run. It includes
// information about the iterator operation in progress.
type probeContext struct {
	op
	log io.Writer
}

type op struct {
	Kind    OpKind
	SeekKey []byte
	Span    *Span
	Err     error
}

// ErrInjected is an error artificially injected for testing.
var ErrInjected = &errorProbe{name: "ErrInjected", err: errors.New("injected error")}

var probeParser = func() *dsl.Parser[probe] {
	valuerParser := dsl.NewParser[valuer]()
	valuerParser.DefineConstant("StartKey", func() valuer { return startKey{} })
	valuerParser.DefineFunc("Bytes",
		func(p *dsl.Parser[valuer], s *dsl.Scanner) valuer {
			v := bytesConstant{bytes: []byte(s.ConsumeString())}
			s.Consume(token.RPAREN)
			return v
		})

	predicateParser := dsl.NewPredicateParser[*probeContext]()
	predicateParser.DefineFunc("Equal",
		func(p *dsl.Parser[dsl.Predicate[*probeContext]], s *dsl.Scanner) dsl.Predicate[*probeContext] {
			eq := equal{
				valuerParser.ParseFromPos(s, s.Scan()),
				valuerParser.ParseFromPos(s, s.Scan()),
			}
			s.Consume(token.RPAREN)
			return eq
		})
	for i, name := range opNames {
		opKind := OpKind(i)
		predicateParser.DefineConstant(name, func() dsl.Predicate[*probeContext] {
			// An OpKind implements dsl.Predicate[*probeContext].
			return opKind
		})
	}
	probeParser := dsl.NewParser[probe]()
	probeParser.DefineConstant("ErrInjected", func() probe { return ErrInjected })
	probeParser.DefineConstant("noop", func() probe { return noop{} })
	probeParser.DefineFunc("If",
		func(p *dsl.Parser[probe], s *dsl.Scanner) probe {
			probe := ifProbe{
				predicateParser.ParseFromPos(s, s.Scan()),
				probeParser.ParseFromPos(s, s.Scan()),
				probeParser.ParseFromPos(s, s.Scan()),
			}
			s.Consume(token.RPAREN)
			return probe
		})
	probeParser.DefineFunc("Return",
		func(p *dsl.Parser[probe], s *dsl.Scanner) (ret probe) {
			switch tok := s.Scan(); tok.Kind {
			case token.STRING:
				str, err := strconv.Unquote(tok.Lit)
				if err != nil {
					panic(err)
				}
				span := ParseSpan(str)
				ret = returnSpan{s: &span}
			case token.IDENT:
				switch tok.Lit {
				case "nil":
					ret = returnSpan{s: nil}
				default:
					panic(errors.Newf("unrecognized return value %q", tok.Lit))
				}
			}
			s.Consume(token.RPAREN)
			return ret
		})
	probeParser.DefineFunc("Log",
		func(p *dsl.Parser[probe], s *dsl.Scanner) (ret probe) {
			ret = loggingProbe{prefix: s.ConsumeString()}
			s.Consume(token.RPAREN)
			return ret
		})
	return probeParser
}()

// probe implementations

type errorProbe struct {
	name string
	err  error
}

func (p *errorProbe) String() string { return p.name }
func (p *errorProbe) Error() error   { return p.err }
func (p *errorProbe) probe(pctx *probeContext) {
	pctx.op.Err = p.err
	pctx.op.Span = nil
}

// ifProbe is a conditional probe. If its predicate evaluates to true, it probes
// using its Then probe. If its predicate evalutes to false, it probes using its
// Else probe.
type ifProbe struct {
	Predicate dsl.Predicate[*probeContext]
	Then      probe
	Else      probe
}

func (p ifProbe) String() string { return fmt.Sprintf("(If %s %s %s)", p.Predicate, p.Then, p.Else) }
func (p ifProbe) probe(pctx *probeContext) {
	if p.Predicate.Evaluate(pctx) {
		p.Then.probe(pctx)
	} else {
		p.Else.probe(pctx)
	}
}

type returnSpan struct {
	s *Span
}

func (p returnSpan) String() string {
	if p.s == nil {
		return "(Return nil)"
	}
	return fmt.Sprintf("(Return %q)", p.s.String())
}

func (p returnSpan) probe(pctx *probeContext) {
	pctx.op.Span = p.s
	pctx.op.Err = nil
}

type noop struct{}

func (noop) String() string           { return "Noop" }
func (noop) probe(pctx *probeContext) {}

type loggingProbe struct {
	prefix string
}

func (lp loggingProbe) String() string { return fmt.Sprintf("(Log %q)", lp.prefix) }
func (lp loggingProbe) probe(pctx *probeContext) {
	opStr := strings.TrimPrefix(pctx.op.Kind.String(), "Op")
	fmt.Fprintf(pctx.log, "%s%s(", lp.prefix, opStr)
	if pctx.op.SeekKey != nil {
		fmt.Fprintf(pctx.log, "%q", pctx.op.SeekKey)
	}
	fmt.Fprint(pctx.log, ") = ")
	if pctx.op.Span == nil {
		fmt.Fprint(pctx.log, "nil")
		if pctx.op.Err != nil {
			fmt.Fprintf(pctx.log, " <err=%q>", pctx.op.Err)
		}
	} else {
		fmt.Fprint(pctx.log, pctx.op.Span.String())
	}
	fmt.Fprintln(pctx.log)
}

// dsl.Predicate[*probeContext] implementations.

type equal struct {
	a, b valuer
}

func (e equal) String() string { return fmt.Sprintf("(Equal %s %s)", e.a, e.b) }
func (e equal) Evaluate(pctx *probeContext) bool {
	return reflect.DeepEqual(e.a.value(pctx), e.b.value(pctx))
}

// OpKind indicates the type of iterator operation being performed.
type OpKind int8

const (
	OpSeekGE OpKind = iota
	OpSeekLT
	OpFirst
	OpLast
	OpNext
	OpPrev
	OpClose
	numOpKinds
)

func (o OpKind) String() string                   { return opNames[o] }
func (o OpKind) Evaluate(pctx *probeContext) bool { return pctx.op.Kind == o }

var opNames = [numOpKinds]string{
	OpSeekGE: "OpSeekGE",
	OpSeekLT: "OpSeekLT",
	OpFirst:  "OpFirst",
	OpLast:   "OpLast",
	OpNext:   "OpNext",
	OpPrev:   "OpPrev",
	OpClose:  "OpClose",
}

// valuer implementations

type valuer interface {
	fmt.Stringer
	value(pctx *probeContext) any
}

type bytesConstant struct {
	bytes []byte
}

func (b bytesConstant) String() string               { return fmt.Sprintf("%q", string(b.bytes)) }
func (b bytesConstant) value(pctx *probeContext) any { return b.bytes }

type startKey struct{}

func (s startKey) String() string { return "StartKey" }
func (s startKey) value(pctx *probeContext) any {
	if pctx.op.Span == nil {
		return nil
	}
	return pctx.op.Span.Start
}

type probeIterator struct {
	iter     FragmentIterator
	err      error
	probe    probe
	probeCtx probeContext
}

// Assert that probeIterator implements the fragment iterator interface.
var _ FragmentIterator = (*probeIterator)(nil)

func (p *probeIterator) handleOp(preProbeOp op) *Span {
	p.probeCtx.op = preProbeOp
	if preProbeOp.Span == nil && p.iter != nil {
		p.probeCtx.op.Err = p.iter.Error()
	}

	p.probe.probe(&p.probeCtx)
	p.err = p.probeCtx.op.Err
	return p.probeCtx.op.Span
}

func (p *probeIterator) SeekGE(key []byte) *Span {
	op := op{
		Kind:    OpSeekGE,
		SeekKey: key,
	}
	if p.iter != nil {
		op.Span = p.iter.SeekGE(key)
	}
	return p.handleOp(op)
}

func (p *probeIterator) SeekLT(key []byte) *Span {
	op := op{
		Kind:    OpSeekLT,
		SeekKey: key,
	}
	if p.iter != nil {
		op.Span = p.iter.SeekLT(key)
	}
	return p.handleOp(op)
}

func (p *probeIterator) First() *Span {
	op := op{Kind: OpFirst}
	if p.iter != nil {
		op.Span = p.iter.First()
	}
	return p.handleOp(op)
}

func (p *probeIterator) Last() *Span {
	op := op{Kind: OpLast}
	if p.iter != nil {
		op.Span = p.iter.Last()
	}
	return p.handleOp(op)
}

func (p *probeIterator) Next() *Span {
	op := op{Kind: OpNext}
	if p.iter != nil {
		op.Span = p.iter.Next()
	}
	return p.handleOp(op)
}

func (p *probeIterator) Prev() *Span {
	op := op{Kind: OpPrev}
	if p.iter != nil {
		op.Span = p.iter.Prev()
	}
	return p.handleOp(op)
}

func (p *probeIterator) Error() error {
	return p.err
}

func (p *probeIterator) Close() error {
	op := op{Kind: OpClose}
	if p.iter != nil {
		op.Err = p.iter.Close()
	}

	p.probeCtx.op = op
	p.probe.probe(&p.probeCtx)
	p.err = p.probeCtx.op.Err
	return p.err
}

// runIterCmd evaluates a datadriven command controlling an internal
// keyspan.FragmentIterator, returning a string with the results of the iterator
// operations.
func runIterCmd(t *testing.T, td *datadriven.TestData, iter FragmentIterator) string {
	var buf bytes.Buffer
	lines := strings.Split(strings.TrimSpace(td.Input), "\n")
	for i, line := range lines {
		if i > 0 {
			fmt.Fprintln(&buf)
		}
		line = strings.TrimSpace(line)
		i := strings.IndexByte(line, '#')
		iterCmd := line
		if i > 0 {
			iterCmd = string(line[:i])
		}
		runIterOp(&buf, iter, iterCmd)
	}
	return buf.String()
}

var iterDelim = map[rune]bool{',': true, ' ': true, '(': true, ')': true, '"': true}

func runIterOp(w io.Writer, it FragmentIterator, op string) {
	fields := strings.FieldsFunc(op, func(r rune) bool { return iterDelim[r] })
	var s *Span
	switch strings.ToLower(fields[0]) {
	case "first":
		s = it.First()
	case "last":
		s = it.Last()
	case "seekge", "seek-ge":
		if len(fields) == 1 {
			panic(fmt.Sprintf("unable to parse iter op %q", op))
		}
		s = it.SeekGE([]byte(fields[1]))
	case "seeklt", "seek-lt":
		if len(fields) == 1 {
			panic(fmt.Sprintf("unable to parse iter op %q", op))
		}
		s = it.SeekLT([]byte(fields[1]))
	case "next":
		s = it.Next()
	case "prev":
		s = it.Prev()
	default:
		panic(fmt.Sprintf("unrecognized iter op %q", fields[0]))
	}
	if s == nil {
		fmt.Fprint(w, "<nil>")
		if err := it.Error(); err != nil {
			fmt.Fprintf(w, " err=<%s>", it.Error())
		}
		return
	}
	fmt.Fprint(w, s)
}
