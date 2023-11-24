// Copyright (C) 2022 Michael J. Fromberger. All Rights Reserved.

// Package scanner implements a lexical scanner for TOML, as defined by the
// TOML v1 specification https://toml.io/en/v1.0.0.
package scanner

import (
	"bufio"
	"bytes"
	"fmt"
	"io"
	"regexp"
	"strconv"
	"strings"
	"unicode"
)

// Token is the type of a lexical token in the TOML grammar.
type Token byte

// IsValue reports whether t can serve as the value in a TOML table.
func (t Token) IsValue() bool { return t >= minValue && t <= maxValue }

// Constants defining the valid Token values.
const (
	Invalid       Token = iota // invalid token
	Comment                    // single line-comment (# xxx)
	Newline                    // line break
	Word                       // unquoted word (foo, 0-bar-1)
	String                     // basic string ("xxx")
	MString                    // multiline basic string ("""xxx""")
	LString                    // literal string ('xxx')
	MLString                   // multiline literal string ('''xxx''')
	Integer                    // integer literal (0x2f, -15)
	Float                      // floating-point literal (6e-9, 0.22)
	DateTime                   // offset date-time (2006-01-02T15:04:05.999999999Z07:00)
	LocalDate                  // local date (2006-01-02)
	LocalTime                  // local time (15:04:05.999999999)
	LocalDateTime              // local date-time (2006-01-02T15:04:05.999999999)
	LBracket                   // left bracket ("[")
	RBracket                   // right bracket ("]")
	LInline                    // inline table open ("{")
	RInline                    // inline table close ("}")
	Equal                      // equal sign ("=")
	Comma                      // comma separator (",")
	Dot                        // key separator (".")

	minValue = Word
	maxValue = LocalDateTime

	// Do not modify the order of these constants. The scanner uses the order of
	// consecutive token types (notably table / array brackets), and the value
	// tokens need to be in a contiguous range.
)

var tokenStr = [...]string{
	Invalid:       "invalid token",
	Comment:       "comment",
	Newline:       "line break",
	Word:          "word",
	String:        "basic string",
	MString:       "multiline basic string",
	LString:       "literal string",
	MLString:      "multiline literal string",
	Integer:       "integer",
	Float:         "float",
	DateTime:      "offset date-time",
	LocalDate:     "local date",
	LocalTime:     "local time",
	LocalDateTime: "local date-time",
	LBracket:      `"["`,
	RBracket:      `"]"`,
	LInline:       `"{"`,
	RInline:       `"}"`,
	Equal:         `"="`,
	Comma:         `","`,
	Dot:           `"."`,
}

func (t Token) String() string {
	v := int(t)
	if v > len(tokenStr) {
		return tokenStr[Invalid]
	}
	return tokenStr[v]
}

// A Scanner reads lexical tokens from an input stream.  Each call to Next
// advances the scanner to the next token, or reports an error.
type Scanner struct {
	r    *bufio.Reader
	buf  bytes.Buffer
	prev Token
	tok  Token
	err  error

	pos, end int // start and end offsets of current token
	last     int // size in bytes of last-read input rune

	// Apparent line and column offsets (0-based)
	pline, pcol int
	eline, ecol int
}

// New constructs a new lexical scanner that consumes input from r.
func New(r io.Reader) *Scanner {
	br, ok := r.(*bufio.Reader)
	if !ok {
		br = bufio.NewReader(r)
	}
	return &Scanner{r: br}
}

// Next advances s to the next token of the input, or reports an error.
// At the end of the input, Next returns io.EOF.
func (s *Scanner) Next() error {
	s.buf.Reset()
	s.err = nil
	s.prev, s.tok = s.tok, Invalid
	s.pos, s.pline, s.pcol = s.end, s.eline, s.ecol

	for {
		ch, err := s.rune()
		if err == io.EOF {
			return s.setErr(err)
		} else if err != nil {
			return s.fail(err)
		}

		// Skip whitespace, but keep track of line breaks. Line breaks are
		// significant to the syntax, so they are returned as a token.
		if isSpace(ch) {
			s.pos, s.pline, s.pcol = s.end, s.eline, s.ecol
			if ch == '\n' {
				s.eline++
				s.ecol = 0
				s.tok = Newline
				return nil
			}
			s.prev = Invalid // whitespace interrupts tokens
			continue
		}

		// Handle punctuation.
		if t, ok := selfDelim(ch); ok {
			s.buf.WriteRune(ch)
			s.tok = t
			return nil
		}

		// Handle string and literal string values.
		if ch == '"' {
			return s.scanBasicString(ch)
		} else if ch == '\'' {
			return s.scanLiteralString(ch)
		}

		// Handle comments.
		if ch == '#' {
			return s.scanComment(ch)
		}

		// Handle names, integers, floating point.
		if !isWordish(ch) {
			return s.failf("invalid %q", ch)
		}
		return s.scanWord(ch)
	}
}

// Token returns the type of the current token.
func (s *Scanner) Token() Token { return s.tok }

// Prev returns the type of the immediately prior token, or Invalid.
// Whitespace resets the previous token.
func (s *Scanner) Prev() Token { return s.prev }

// Err returns the last error reported by Next.
func (s *Scanner) Err() error { return s.err }

// Text returns the undecoded text of the current token.  The return value is
// only valid until the next call of Next. The caller must copy the contents of
// the returned slice if it is needed beyond that.
func (s *Scanner) Text() []byte { return s.buf.Bytes() }

// Span returns the location span of the current token.
func (s *Scanner) Span() Span { return Span{Pos: s.pos, End: s.end} }

// Location returns the complete location of the current token.
func (s *Scanner) Location() Location {
	return Location{
		Span:  s.Span(),
		First: LineCol{Line: s.pline + 1, Column: s.pcol},
		Last:  LineCol{Line: s.eline + 1, Column: s.ecol},
	}
}

func (s *Scanner) scanLiteralString(open rune) error {
	isEmpty, nquotes, err := s.isMultiline(open)
	if err != nil {
		return err
	} else if isEmpty {
		s.tok = LString
		return nil
	}

	wantq := nquotes
	resType := LString
	if nquotes > 1 {
		resType = MLString
	}

	for {
		ch, err := s.rune()
		if err != nil {
			return s.fail(err)
		} else if isControl(ch) && !isSpace(ch) {
			return s.failf("invalid control %q", ch)
		}
		s.buf.WriteRune(ch)
		if ch == open {
			wantq--
			if wantq > 0 {
				continue
			} else if nquotes == 1 {
				s.tok = resType
				return nil
			}
			return s.checkMultilineStringEnd(open, resType)
		} else {
			wantq = nquotes // non-quote: reset
		}
	}
}

// checkMultilineStringEnd checks whether there are additional quotation marks
// beyond the apparent end of a multiline string.
//
// After we have seen enough quotes to close the string, we might be done, but
// lexicall grammar allows up to 2 unhedged quotes at the end, so we have to
// check further.  If there are 0, 1, or 2 additional quotes, all is well; any
// more than that is an error.
func (s *Scanner) checkMultilineStringEnd(open rune, resType Token) error {
	var extra int

	if _, err := s.readWhile(func(r rune) bool {
		if r == open {
			extra++
			return true
		}
		return false
	}); err == io.EOF {
		// OK
	} else if err != nil {
		return err
	} else {
		s.unrune()
	}
	if extra >= 3 {
		return s.failf("too many quotes (%d) at end of string", extra)
	}
	s.tok = resType
	return nil
}

// isMultiline checks whether the input begins a multiline string.
//
// The empty flag is true if the input begins with an empty non-multiline
// string literal, in which case nquotes == 0. Otherwise, nquotes is the number
// of quotes that are needed to end the string.
func (s *Scanner) isMultiline(open rune) (empty bool, nquotes int, _ error) {
	s.buf.WriteRune(open)

	// Check for a second quotation mark.
	ch2, err := s.rune()
	if err != nil {
		return false, 0, err
	} else if ch2 != open {
		s.unrune()
		return false, 1, nil
	}
	s.buf.WriteRune(open)

	// Check for a third quotation mark.
	ch3, err := s.rune()
	if err == io.EOF {
		s.unrune() // put back the second quote
		return true, 1, nil
	} else if err != nil {
		return false, 0, err
	} else if ch3 != open {
		s.unrune() // put back the non-quote we read
		return true, 0, nil
	}
	s.buf.WriteRune(open)

	return false, 3, nil
}

func (s *Scanner) scanBasicString(open rune) error {
	isEmpty, nquotes, err := s.isMultiline(open)
	if err != nil {
		return err
	} else if isEmpty {
		s.tok = String
		return nil
	}

	wantq := nquotes
	resType := String
	if nquotes > 1 {
		resType = MString
	}

	var esc bool
	for {
		ch, err := s.rune()
		if err != nil {
			return s.fail(err)
		} else if ch == open {
			s.buf.WriteRune(ch)
			if !esc {
				wantq--
				if wantq == 0 {
					return s.checkMultilineStringEnd(open, resType)
				}
			}
			esc = false
			continue
		}

		wantq = nquotes // non-quote: reset

		// Newlines are OK in a multi-line string, but do not generate a separate
		// token for the newline. If a newline is "escaped" we keep the trailing
		// backslash that preceded it. Either way, however, we need to update the
		// line numbering so subsequent tokens will get correct locations.
		if nquotes > 1 && (ch == '\n' || ch == '\r') {
			if ch == '\n' {
				s.eline++
				s.ecol = 0
			}
			esc = false // consume the escape we wrote out previously
			s.buf.WriteRune(ch)
			continue
		}

		if esc {
			// We are awaiting the completion of a \-escape.
			switch ch {
			case '\\', '\t', ' ', 'b', 'f', 'n', 'r', 't':
				s.buf.WriteByte(byte(ch))
			case 'u', 'U':
				s.buf.WriteByte(byte(ch))
				if err := s.readHex4(); err != nil {
					return s.failf("invalid Unicode escape: %w", err)
				}
				if ch == 'U' {
					if err := s.readHex4(); err != nil {
						return s.failf("invalid long Unicode escape: %w", err)
					}
				}
			default:
				return s.failf("invalid %q after escape", ch)
			}
			esc = false
		} else if isControl(ch) && !isSpace(ch) {
			return s.failf("unescaped control %q", ch)
		} else if ch > unicode.MaxRune {
			return s.failf("invalid Unicode rune %q", ch)
		} else if nquotes == 1 && (ch == '\n' || ch == '\r') {
			return s.failf("unescaped %q in basic string", ch)
		} else {
			s.buf.WriteRune(ch)
			esc = ch == '\\'
		}
	}
}

func (s *Scanner) scanComment(start rune) error {
	s.buf.WriteRune(start)
	for {
		ch, err := s.rune()
		if err == io.EOF {
			break // OK, comment ends at EOF
		} else if ch == '\n' {
			s.eline++
			s.ecol = 0
			break
		} else if isControl(ch) && !isSpace(ch) {
			return s.failf("invalid comment rune %q", ch)
		} else if err != nil {
			return s.fail(err)
		}
		s.buf.WriteRune(ch)
	}
	s.tok = Comment
	return nil
}

func (s *Scanner) scanWord(first rune) error {
	s.buf.WriteRune(first)
	next, err := s.readWhile(isWordish)
	if err == io.EOF {
		// OK
	} else if err != nil {
		return s.fail(err)
	}

	// A colon is only valid as part of a date/time literal.
	if next == ':' {
		return s.scanDateTime(next)
	}

	// If the word so far looks like a datestamp (2006-01-02), it may be that.
	// But if it is followed by a space, there may also be a timestamp after it.
	// To tell, we have to look one rune further. Thanks, Tom.
	if dateRE.Match(s.buf.Bytes()) {
		if next != ' ' {
			// The space is a delimiter, put it back.
			s.unrune()
			s.tok = LocalDate
			return nil
		}
		ch, err := s.rune()
		if err == io.EOF {
			// Extra space at EOF, ignore it.
			s.tok = LocalDate
			return nil
		} else if err != nil {
			return err
		} else if isDigit(ch) {
			// The space is a separator, keep it.  At this point the token can
			// ONLY be valid if it has a timestamp.
			s.buf.WriteRune(next)
			return s.scanDateTime(ch)
		} else {
			s.unrune()
		}
	}

	// The lexical grammar of TOML is slightly context-sensitive: A dot (".")
	// may be either a separator for words in a key, or a decimal point in a
	// float or timestamp literal. To correctly distinguish these cases you must
	// know whether you are parsing a key or a value.
	//
	// Without that context, this implementation bends the grammar slightly: If
	// a dot occurs after a prefix that could be a decimal integer literal, we
	// absorb the dot and scan a floating-point literal. This means a key like:
	//
	//    foo.3.14.bar
	//
	// would scan as
	//
	//    word(foo) dot float(3.14) dot word(bar)
	//
	// whereas the grammar for the "key" production would read it as
	//
	//    word(foo) dot word(3) dot word(14) dot word(bar)
	//
	// The parser therefore has to permit integer and float tokens when parsing
	// a key, and unpack the components if a "." occurs.
	//
	// Ref: https://github.com/toml-lang/toml/blob/1.0.0/toml.abnf
	if next == '.' && intRE.Match(s.buf.Bytes()) {
		return s.scanFloat(next)
	}

	s.unrune()

	// Classify what this word-shaped thing is.
	text := s.buf.Bytes()
	if intRE.Match(text) && !sepCheck.Match(text) {
		s.tok = Integer
	} else if (floatRE.Match(text) || sfloatRE.Match(text)) && !sepCheck.Match(text) {
		s.tok = Float
	} else if tok := dateTimeToken(text); tok != Invalid {
		s.tok = tok
	} else if p := bytes.IndexByte(text, '+'); p >= 0 {
		return s.failf(`invalid %q at position %d`, text[p], p)
	} else {
		s.tok = Word
	}
	return nil
}

func (s *Scanner) scanFloat(next rune) error {
	s.buf.WriteRune(next)
	if _, err := s.readWhile(isWordish); err == io.EOF {
		// OK
	} else if err != nil {
		return s.failf("incomplete %v: %w", Float, err)
	} else {
		s.unrune()
	}
	if !floatRE.Match(s.buf.Bytes()) {
		return s.failf("invalid %v", Float)
	} else if sepCheck.Match(s.buf.Bytes()) {
		return s.failf("invalid floating-point literal")
	}
	s.tok = Float
	return nil
}

func (s *Scanner) scanDateTime(next rune) error {
	s.buf.WriteRune(next)
	if _, err := s.readWhile(isDateTimeRune); err == io.EOF {
		// OK
	} else if err != nil {
		return s.failf("incomplete date/time literal: %w", err)
	} else {
		s.unrune()
	}
	s.tok = dateTimeToken(s.buf.Bytes())
	if s.tok == Invalid {
		return s.failf("invalid date/time literal")
	}
	return nil
}

var (
	intRE    = regexp.MustCompile(`^(0x[A-Fa-f0-9_]+|0o[0-7_]+|0b[01_]+|[-+]?[0-9_]+)$`)
	floatRE  = regexp.MustCompile(`^[-+]?([0-9_]+(\.[0-9_]+)?([eE][-+]?[0-9_]+)?)$`)
	sfloatRE = regexp.MustCompile(`^[-+]?(inf|nan)$`)
	sepCheck = regexp.MustCompile(`(?:^_)|(?:__+)|(?:_$)`)
	timeRE   = regexp.MustCompile(`^\d{2}:\d{2}:\d{2}(\.\d+)?$`)
	dateRE   = regexp.MustCompile(`^\d{4}-\d{2}-\d{2}$`)

	// Date/time literals. Match 1 is date, match 2 is time, match 3 is offset.
	// If offset is missing, it is a "local" date/time.
	// If time is also missing it is a local date only.
	dateTimeRE = regexp.MustCompile(`^(\d{4}-\d{2}-\d{2})([tT ]\d{2}:\d{2}:\d{2}(?:\.\d+)?([zZ]|[-+]\d{2}:\d{2})?)?$`)
)

func (s *Scanner) rune() (rune, error) {
	ch, nb, err := s.r.ReadRune()
	s.last = nb
	s.end += nb
	s.ecol += nb
	return ch, err
}

func (s *Scanner) unrune() {
	s.end -= s.last
	s.ecol -= s.last
	s.last = 0
	s.r.UnreadRune()
}

// readWhile consumes runes matching f from the input until EOF or until a rune
// not matching f is found. The first non-matching rune (if any) is returned.
// It is the caller's responsibility to unread this rune, if desired.
func (s *Scanner) readWhile(f func(rune) bool) (rune, error) {
	for {
		ch, err := s.rune()
		if err != nil {
			return 0, err
		} else if !f(ch) {
			return ch, nil
		}
		s.buf.WriteRune(ch)
	}
}

// readHex4 reads exactly 4 hexadecimal digits from the input.
func (s *Scanner) readHex4() error {
	for i := 0; i < 4; i++ {
		ch, err := s.rune()
		if err != nil {
			return err
		} else if !isHexDigit(ch) {
			return fmt.Errorf("not a hex digit: %q", ch)
		}
		s.buf.WriteRune(ch)
	}
	return nil
}

func (s *Scanner) setErr(err error) error {
	s.err = err
	return err
}

func (s *Scanner) fail(err error) error {
	return s.setErr(fmt.Errorf("offset %d: unexpected error: %w", s.end, err))
}

func (s *Scanner) failf(msg string, args ...interface{}) error {
	return s.setErr(fmt.Errorf("offset %d: "+msg, append([]interface{}{s.end}, args...)...))
}

func isControl(ch rune) bool { return ch < ' ' || ch == '\x7f' }

func isSpace(ch rune) bool {
	return ch == ' ' || ch == '\r' || ch == '\n' || ch == '\t'
}

func isDigit(ch rune) bool { return '0' <= ch && ch <= '9' }

// N.B. A + is not allowed in an ordinary word, but we accept it here to
// simplify the processing of numbers and date/time literals.
func isWordish(ch rune) bool { return isWordRune(ch) || ch == '+' }

func isWordRune(ch rune) bool {
	return 'A' <= ch && ch <= 'Z' || 'a' <= ch && ch <= 'z' || isDigit(ch) ||
		ch == '-' || ch == '_'
}

func isDateTimeRune(ch rune) bool { return ch == '.' || ch == ':' || isWordish(ch) }

func isHexDigit(ch rune) bool {
	return (ch >= '0' && ch <= '9') || (ch >= 'a' && ch <= 'f') || (ch >= 'A' && ch <= 'F')
}

// N.B. The order of the probe string must match the order of the self slice.
const selfText = "{}=,.[]"

var self = [...]Token{LInline, RInline, Equal, Comma, Dot, LBracket, RBracket}

func selfDelim(ch rune) (Token, bool) {
	i := strings.IndexRune(selfText, ch)
	if i >= 0 {
		return self[i], true
	}
	return Invalid, false
}

// dateTimeToken returns Invalid if text does not contain an invalid timestamp,
// otherwise it reports which timestamp token applies.
func dateTimeToken(text []byte) Token {
	if timeRE.Match(text) {
		return LocalTime
	} else if m := dateTimeRE.FindSubmatch(text); m == nil {
		return Invalid
	} else if len(m[3]) == 0 {
		if len(m[2]) == 0 {
			return LocalDate
		}
		return LocalDateTime
	}
	return DateTime
}

// A Span describes a contiguous span of a source input.
type Span struct {
	Pos int // the start offset, 0-based
	End int // the end offset, 0-based (noninclusive)
}

func (s Span) String() string {
	if s.End <= s.Pos {
		return strconv.Itoa(s.Pos)
	}
	return fmt.Sprintf("%d..%d", s.Pos, s.End)
}

// A LineCol describes the line number and column offset of a location in
// source text.
type LineCol struct {
	Line   int // line number, 1-based
	Column int // byte offset of column in line, 0-based
}

func (lc LineCol) String() string { return fmt.Sprintf("%d:%d", lc.Line, lc.Column) }

// A Location describes the complete location of a range of source text,
// including line and column offsets.
type Location struct {
	Span
	First, Last LineCol
}

func (loc Location) String() string {
	if loc.First.Line == loc.Last.Line {
		return fmt.Sprintf("%d:%d-%d", loc.First.Line, loc.First.Column, loc.Last.Column)
	}
	return loc.First.String() + "-" + loc.Last.String()
}
