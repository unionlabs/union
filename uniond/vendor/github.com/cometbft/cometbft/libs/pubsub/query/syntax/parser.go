package syntax

import (
	"fmt"
	"io"
	"math/big"
	"strings"
	"time"
)

// Parse parses the specified query string. It is shorthand for constructing a
// parser for s and calling its Parse method.
func Parse(s string) (Query, error) {
	return NewParser(strings.NewReader(s)).Parse()
}

// Query is the root of the parse tree for a query.  A query is the conjunction
// of one or more conditions.
type Query []Condition

func (q Query) String() string {
	ss := make([]string, len(q))
	for i, cond := range q {
		ss[i] = cond.String()
	}
	return strings.Join(ss, " AND ")
}

// A Condition is a single conditional expression, consisting of a tag, a
// comparison operator, and an optional argument. The type of the argument
// depends on the operator.
type Condition struct {
	Tag string
	Op  Token
	Arg *Arg

	opText string
}

func (c Condition) String() string {
	s := c.Tag + " " + c.opText
	if c.Arg != nil {
		return s + " " + c.Arg.String()
	}
	return s
}

// An Arg is the argument of a comparison operator.
type Arg struct {
	Type Token
	text string
}

func (a *Arg) String() string {
	if a == nil {
		return ""
	}
	switch a.Type {
	case TString:
		return "'" + a.text + "'"
	case TTime:
		return "TIME " + a.text
	case TDate:
		return "DATE " + a.text
	default:
		return a.text
	}
}

// Number returns the value of the argument text as a number, or nil if the
// text does not encode a valid number value.
func (a *Arg) Number() *big.Float {
	if a == nil {
		return nil
	}
	intVal := new(big.Int)
	if _, ok := intVal.SetString(a.text, 10); !ok {
		f, _, err := big.ParseFloat(a.text, 10, 125, big.ToNearestEven)
		if err != nil {
			return nil
		}
		return f
	}
	// If it is indeed a big integer, we make sure to convert it to a float with enough precision
	// to represent all the bits
	bitLen := uint(intVal.BitLen())
	var f *big.Float
	var err error
	if bitLen <= 64 {
		f, _, err = big.ParseFloat(a.text, 10, 0, big.ToNearestEven)
	} else {
		f, _, err = big.ParseFloat(a.text, 10, bitLen, big.ToNearestEven)
	}
	if err != nil {
		return nil
	}
	return f

}

// Time returns the value of the argument text as a time, or the zero value if
// the text does not encode a timestamp or datestamp.
func (a *Arg) Time() time.Time {
	var ts time.Time
	if a == nil {
		return ts
	}
	var err error
	switch a.Type {
	case TDate:
		ts, err = ParseDate(a.text)
	case TTime:
		ts, err = ParseTime(a.text)
	}
	if err == nil {
		return ts
	}
	return time.Time{}
}

// Value returns the value of the argument text as a string, or "".
func (a *Arg) Value() string {
	if a == nil {
		return ""
	}
	return a.text
}

// Parser is a query expression parser. The grammar for query expressions is
// defined in the syntax package documentation.
type Parser struct {
	scanner *Scanner
}

// NewParser constructs a new parser that reads the input from r.
func NewParser(r io.Reader) *Parser {
	return &Parser{scanner: NewScanner(r)}
}

// Parse parses the complete input and returns the resulting query.
func (p *Parser) Parse() (Query, error) {
	cond, err := p.parseCond()
	if err != nil {
		return nil, err
	}
	conds := []Condition{cond}
	for p.scanner.Next() != io.EOF {
		if tok := p.scanner.Token(); tok != TAnd {
			return nil, fmt.Errorf("offset %d: got %v, want %v", p.scanner.Pos(), tok, TAnd)
		}
		cond, err := p.parseCond()
		if err != nil {
			return nil, err
		}
		conds = append(conds, cond)
	}
	return conds, nil
}

// parseCond parses a conditional expression: tag OP value.
func (p *Parser) parseCond() (Condition, error) {
	var cond Condition
	if err := p.require(TTag); err != nil {
		return cond, err
	}
	cond.Tag = p.scanner.Text()
	if err := p.require(TLeq, TGeq, TLt, TGt, TEq, TContains, TExists); err != nil {
		return cond, err
	}
	cond.Op = p.scanner.Token()
	cond.opText = p.scanner.Text()

	var err error
	switch cond.Op {
	case TLeq, TGeq, TLt, TGt:
		err = p.require(TNumber, TTime, TDate)
	case TEq:
		err = p.require(TNumber, TTime, TDate, TString)
	case TContains:
		err = p.require(TString)
	case TExists:
		// no argument
		return cond, nil
	default:
		return cond, fmt.Errorf("offset %d: unexpected operator %v", p.scanner.Pos(), cond.Op)
	}
	if err != nil {
		return cond, err
	}
	cond.Arg = &Arg{Type: p.scanner.Token(), text: p.scanner.Text()}
	return cond, nil
}

// require advances the scanner and requires that the resulting token is one of
// the specified token types.
func (p *Parser) require(tokens ...Token) error {
	if err := p.scanner.Next(); err != nil {
		return fmt.Errorf("offset %d: %w", p.scanner.Pos(), err)
	}
	got := p.scanner.Token()
	for _, tok := range tokens {
		if tok == got {
			return nil
		}
	}
	return fmt.Errorf("offset %d: got %v, wanted %s", p.scanner.Pos(), got, tokLabel(tokens))
}

// tokLabel makes a human-readable summary string for the given token types.
func tokLabel(tokens []Token) string {
	if len(tokens) == 1 {
		return tokens[0].String()
	}
	last := len(tokens) - 1
	ss := make([]string, len(tokens)-1)
	for i, tok := range tokens[:last] {
		ss[i] = tok.String()
	}
	return strings.Join(ss, ", ") + " or " + tokens[last].String()
}

// ParseDate parses s as a date string in the format used by DATE values.
func ParseDate(s string) (time.Time, error) {
	return time.Parse("2006-01-02", s)
}

// ParseTime parses s as a timestamp in the format used by TIME values.
func ParseTime(s string) (time.Time, error) {
	return time.Parse(time.RFC3339, s)
}
