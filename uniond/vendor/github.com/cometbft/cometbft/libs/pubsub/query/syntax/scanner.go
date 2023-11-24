package syntax

import (
	"bufio"
	"bytes"
	"fmt"
	"io"
	"strings"
	"time"
	"unicode"
)

// Token is the type of a lexical token in the query grammar.
type Token byte

const (
	TInvalid  = iota // invalid or unknown token
	TTag             // field tag: x.y
	TString          // string value: 'foo bar'
	TNumber          // number: 0, 15.5, 100
	TTime            // timestamp: TIME yyyy-mm-ddThh:mm:ss([-+]hh:mm|Z)
	TDate            // datestamp: DATE yyyy-mm-dd
	TAnd             // operator: AND
	TContains        // operator: CONTAINS
	TExists          // operator: EXISTS
	TEq              // operator: =
	TLt              // operator: <
	TLeq             // operator: <=
	TGt              // operator: >
	TGeq             // operator: >=

	// Do not reorder these values without updating the scanner code.
)

var tString = [...]string{
	TInvalid:  "invalid token",
	TTag:      "tag",
	TString:   "string",
	TNumber:   "number",
	TTime:     "timestamp",
	TDate:     "datestamp",
	TAnd:      "AND operator",
	TContains: "CONTAINS operator",
	TExists:   "EXISTS operator",
	TEq:       "= operator",
	TLt:       "< operator",
	TLeq:      "<= operator",
	TGt:       "> operator",
	TGeq:      ">= operator",
}

func (t Token) String() string {
	v := int(t)
	if v > len(tString) {
		return "unknown token type"
	}
	return tString[v]
}

const (
	// TimeFormat is the format string used for timestamp values.
	TimeFormat = time.RFC3339

	// DateFormat is the format string used for datestamp values.
	DateFormat = "2006-01-02"
)

// Scanner reads lexical tokens of the query language from an input stream.
// Each call to Next advances the scanner to the next token, or reports an
// error.
type Scanner struct {
	r   *bufio.Reader
	buf bytes.Buffer
	tok Token
	err error

	pos, last, end int
}

// NewScanner constructs a new scanner that reads from r.
func NewScanner(r io.Reader) *Scanner { return &Scanner{r: bufio.NewReader(r)} }

// Next advances s to the next token in the input, or reports an error.  At the
// end of input, Next returns io.EOF.
func (s *Scanner) Next() error {
	s.buf.Reset()
	s.pos = s.end
	s.tok = TInvalid
	s.err = nil

	for {
		ch, err := s.rune()
		if err != nil {
			return s.fail(err)
		}
		if unicode.IsSpace(ch) {
			s.pos = s.end
			continue // skip whitespace
		}
		if '0' <= ch && ch <= '9' {
			return s.scanNumber(ch)
		} else if isTagRune(ch) {
			return s.scanTagLike(ch)
		}
		switch ch {
		case '\'':
			return s.scanString(ch)
		case '<', '>', '=':
			return s.scanCompare(ch)
		default:
			return s.invalid(ch)
		}
	}
}

// Token returns the type of the current input token.
func (s *Scanner) Token() Token { return s.tok }

// Text returns the text of the current input token.
func (s *Scanner) Text() string { return s.buf.String() }

// Pos returns the start offset of the current token in the input.
func (s *Scanner) Pos() int { return s.pos }

// Err returns the last error reported by Next, if any.
func (s *Scanner) Err() error { return s.err }

// scanNumber scans for numbers with optional fractional parts.
// Examples: 0, 1, 3.14
func (s *Scanner) scanNumber(first rune) error {
	s.buf.WriteRune(first)
	if err := s.scanWhile(isDigit); err != nil {
		return err
	}

	ch, err := s.rune()
	if err != nil && err != io.EOF {
		return err
	}
	if ch == '.' {
		s.buf.WriteRune(ch)
		if err := s.scanWhile(isDigit); err != nil {
			return err
		}
	} else {
		s.unrune()
	}
	s.tok = TNumber
	return nil
}

func (s *Scanner) scanString(first rune) error {
	// discard opening quote
	for {
		ch, err := s.rune()
		if err != nil {
			return s.fail(err)
		} else if ch == first {
			// discard closing quote
			s.tok = TString
			return nil
		}
		s.buf.WriteRune(ch)
	}
}

func (s *Scanner) scanCompare(first rune) error {
	s.buf.WriteRune(first)
	switch first {
	case '=':
		s.tok = TEq
		return nil
	case '<':
		s.tok = TLt
	case '>':
		s.tok = TGt
	default:
		return s.invalid(first)
	}

	ch, err := s.rune()
	if err == io.EOF {
		return nil // the assigned token is correct
	} else if err != nil {
		return s.fail(err)
	}
	if ch == '=' {
		s.buf.WriteRune(ch)
		s.tok++ // depends on token order
		return nil
	}
	s.unrune()
	return nil
}

func (s *Scanner) scanTagLike(first rune) error {
	s.buf.WriteRune(first)
	var hasSpace bool
	for {
		ch, err := s.rune()
		if err == io.EOF {
			break
		} else if err != nil {
			return s.fail(err)
		}
		if !isTagRune(ch) {
			hasSpace = ch == ' ' // to check for TIME, DATE
			break
		}
		s.buf.WriteRune(ch)
	}

	text := s.buf.String()
	switch text {
	case "TIME":
		if hasSpace {
			return s.scanTimestamp()
		}
		s.tok = TTag
	case "DATE":
		if hasSpace {
			return s.scanDatestamp()
		}
		s.tok = TTag
	case "AND":
		s.tok = TAnd
	case "EXISTS":
		s.tok = TExists
	case "CONTAINS":
		s.tok = TContains
	default:
		s.tok = TTag
	}
	s.unrune()
	return nil
}

func (s *Scanner) scanTimestamp() error {
	s.buf.Reset() // discard "TIME" label
	if err := s.scanWhile(isTimeRune); err != nil {
		return err
	}
	if ts, err := time.Parse(TimeFormat, s.buf.String()); err != nil {
		return s.fail(fmt.Errorf("invalid TIME value: %w", err))
	} else if y := ts.Year(); y < 1900 || y > 2999 {
		return s.fail(fmt.Errorf("timestamp year %d out of range", ts.Year()))
	}
	s.tok = TTime
	return nil
}

func (s *Scanner) scanDatestamp() error {
	s.buf.Reset() // discard "DATE" label
	if err := s.scanWhile(isDateRune); err != nil {
		return err
	}
	if ts, err := time.Parse(DateFormat, s.buf.String()); err != nil {
		return s.fail(fmt.Errorf("invalid DATE value: %w", err))
	} else if y := ts.Year(); y < 1900 || y > 2999 {
		return s.fail(fmt.Errorf("datestamp year %d out of range", ts.Year()))
	}
	s.tok = TDate
	return nil
}

func (s *Scanner) scanWhile(ok func(rune) bool) error {
	for {
		ch, err := s.rune()
		if err == io.EOF {
			return nil
		} else if err != nil {
			return s.fail(err)
		} else if !ok(ch) {
			s.unrune()
			return nil
		}
		s.buf.WriteRune(ch)
	}
}

func (s *Scanner) rune() (rune, error) {
	ch, nb, err := s.r.ReadRune()
	s.last = nb
	s.end += nb
	return ch, err
}

func (s *Scanner) unrune() {
	_ = s.r.UnreadRune()
	s.end -= s.last
}

func (s *Scanner) fail(err error) error {
	s.err = err
	return err
}

func (s *Scanner) invalid(ch rune) error {
	return s.fail(fmt.Errorf("invalid input %c at offset %d", ch, s.end))
}

func isDigit(r rune) bool { return '0' <= r && r <= '9' }

func isTagRune(r rune) bool {
	return r == '.' || r == '_' || unicode.IsLetter(r) || unicode.IsDigit(r)
}

func isTimeRune(r rune) bool {
	return strings.ContainsRune("-T:+Z", r) || isDigit(r)
}

func isDateRune(r rune) bool { return isDigit(r) || r == '-' }
