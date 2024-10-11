package syntax_test

import (
	"io"
	"reflect"
	"strings"
	"testing"

	"github.com/cometbft/cometbft/libs/pubsub/query/syntax"
)

func TestScanner(t *testing.T) {
	tests := []struct {
		input string
		want  []syntax.Token
	}{
		// Empty inputs
		{"", nil},
		{"  ", nil},
		{"\t\n ", nil},

		// Numbers
		{`0 123`, []syntax.Token{syntax.TNumber, syntax.TNumber}},
		{`0.32 3.14`, []syntax.Token{syntax.TNumber, syntax.TNumber}},

		// Tags
		{`foo foo.bar`, []syntax.Token{syntax.TTag, syntax.TTag}},
		{`foo foo-foo.bar`, []syntax.Token{syntax.TTag, syntax.TTag}},
		{`foo foo._bar_bar`, []syntax.Token{syntax.TTag, syntax.TTag}},

		// Strings (values)
		{` '' x 'x' 'x y'`, []syntax.Token{syntax.TString, syntax.TTag, syntax.TString, syntax.TString}},
		{` 'you are not your job' `, []syntax.Token{syntax.TString}},

		// Comparison operators
		{`< <= = > >=`, []syntax.Token{
			syntax.TLt, syntax.TLeq, syntax.TEq, syntax.TGt, syntax.TGeq,
		}},

		// Mixed values of various kinds.
		{`x AND y`, []syntax.Token{syntax.TTag, syntax.TAnd, syntax.TTag}},
		{`x.y CONTAINS 'z'`, []syntax.Token{syntax.TTag, syntax.TContains, syntax.TString}},
		{`foo EXISTS`, []syntax.Token{syntax.TTag, syntax.TExists}},
		{`and AND`, []syntax.Token{syntax.TTag, syntax.TAnd}},

		// Timestamp
		{`TIME 2021-11-23T15:16:17Z`, []syntax.Token{syntax.TTime}},

		// Datestamp
		{`DATE 2021-11-23`, []syntax.Token{syntax.TDate}},
	}

	for _, test := range tests {
		s := syntax.NewScanner(strings.NewReader(test.input))
		var got []syntax.Token
		for s.Next() == nil {
			got = append(got, s.Token())
		}
		if err := s.Err(); err != io.EOF {
			t.Errorf("Next: unexpected error: %v", err)
		}

		if !reflect.DeepEqual(got, test.want) {
			t.Logf("Scanner input: %q", test.input)
			t.Errorf("Wrong tokens:\ngot:  %+v\nwant: %+v", got, test.want)
		}
	}
}

func TestScannerErrors(t *testing.T) {
	tests := []struct {
		input string
	}{
		{`'incomplete string`},
		{`-23`},
		{`&`},
		{`DATE xyz-pdq`},
		{`DATE xyzp-dq-zv`},
		{`DATE 0000-00-00`},
		{`DATE 0000-00-000`},
		{`DATE 2021-01-99`},
		{`TIME 2021-01-01T34:56:78Z`},
		{`TIME 2021-01-99T14:56:08Z`},
		{`TIME 2021-01-99T34:56:08`},
		{`TIME 2021-01-99T34:56:11+3`},
	}
	for _, test := range tests {
		s := syntax.NewScanner(strings.NewReader(test.input))
		if err := s.Next(); err == nil {
			t.Errorf("Next: got %v (%#q), want error", s.Token(), s.Text())
		}
	}
}

// These parser tests were copied from the original implementation of the query
// parser, and are preserved here as a compatibility check.
func TestParseValid(t *testing.T) {
	tests := []struct {
		input string
		valid bool
	}{
		{"tm.events.type='NewBlock'", true},
		{"tm.events.type = 'NewBlock'", true},
		{"tm.events.name = ''", true},
		{"tm.events.type='TIME'", true},
		{"tm.events.type='DATE'", true},
		{"tm.events.type='='", true},
		{"tm.events.type='TIME", false},
		{"tm.events.type=TIME'", false},
		{"tm.events.type==", false},
		{"tm.events.type=NewBlock", false},
		{">==", false},
		{"tm.events.type 'NewBlock' =", false},
		{"tm.events.type>'NewBlock'", false},
		{"", false},
		{"=", false},
		{"='NewBlock'", false},
		{"tm.events.type=", false},

		{"tm.events.typeNewBlock", false},
		{"tm.events.type'NewBlock'", false},
		{"'NewBlock'", false},
		{"NewBlock", false},
		{"", false},

		{"tm.events.type='NewBlock' AND abci.account.name='Igor'", true},
		{"tm.events.type='NewBlock' AND", false},
		{"tm.events.type='NewBlock' AN", false},
		{"tm.events.type='NewBlock' AN tm.events.type='NewBlockHeader'", false},
		{"AND tm.events.type='NewBlock' ", false},

		{"abci.account.name CONTAINS 'Igor'", true},

		{"tx.date > DATE 2013-05-03", true},
		{"tx.date < DATE 2013-05-03", true},
		{"tx.date <= DATE 2013-05-03", true},
		{"tx.date >= DATE 2013-05-03", true},
		{"tx.date >= DAT 2013-05-03", false},
		{"tx.date <= DATE2013-05-03", false},
		{"tx.date <= DATE -05-03", false},
		{"tx.date >= DATE 20130503", false},
		{"tx.date >= DATE 2013+01-03", false},
		// incorrect year, month, day
		{"tx.date >= DATE 0013-01-03", false},
		{"tx.date >= DATE 2013-31-03", false},
		{"tx.date >= DATE 2013-01-83", false},

		{"tx.date > TIME 2013-05-03T14:45:00+07:00", true},
		{"tx.date < TIME 2013-05-03T14:45:00-02:00", true},
		{"tx.date <= TIME 2013-05-03T14:45:00Z", true},
		{"tx.date >= TIME 2013-05-03T14:45:00Z", true},
		{"tx.date >= TIME2013-05-03T14:45:00Z", false},
		{"tx.date = IME 2013-05-03T14:45:00Z", false},
		{"tx.date = TIME 2013-05-:45:00Z", false},
		{"tx.date >= TIME 2013-05-03T14:45:00", false},
		{"tx.date >= TIME 0013-00-00T14:45:00Z", false},
		{"tx.date >= TIME 2013+05=03T14:45:00Z", false},

		{"account.balance=100", true},
		{"account.balance >= 200", true},
		{"account.balance >= -300", false},
		{"account.balance >>= 400", false},
		{"account.balance=33.22.1", false},

		{"slashing.amount EXISTS", true},
		{"slashing.amount EXISTS AND account.balance=100", true},
		{"account.balance=100 AND slashing.amount EXISTS", true},
		{"slashing EXISTS", true},

		{"hash='136E18F7E4C348B780CF873A0BF43922E5BAFA63'", true},
		{"hash=136E18F7E4C348B780CF873A0BF43922E5BAFA63", false},

		{"cosm-wasm.transfer_amount=100", true},
	}

	for _, test := range tests {
		q, err := syntax.Parse(test.input)
		if test.valid != (err == nil) {
			t.Errorf("Parse %#q: valid %v got err=%v", test.input, test.valid, err)
		}

		// For valid queries, check that the query round-trips.
		if test.valid {
			qstr := q.String()
			r, err := syntax.Parse(qstr)
			if err != nil {
				t.Errorf("Reparse %#q failed: %v", qstr, err)
			}
			if rstr := r.String(); rstr != qstr {
				t.Errorf("Reparse diff\nold: %#q\nnew: %#q", qstr, rstr)
			}
		}
	}
}
