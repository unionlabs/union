package logfmt

import (
	"bufio"
	"bytes"
	"fmt"
	"reflect"
	"strings"
	"testing"
)

type kv struct {
	k, v []byte
}

func (s kv) String() string {
	return fmt.Sprintf("{k:%q v:%q}", s.k, s.v)
}

func TestDecoder_scan(t *testing.T) {
	defaultDecoder := func(s string) *Decoder { return NewDecoder(strings.NewReader(s)) }
	tests := []struct {
		data string
		dec  func(string) *Decoder
		want [][]kv
	}{
		{
			data: "",
			dec:  defaultDecoder,
			want: nil,
		},
		{
			data: "\n\n",
			dec:  defaultDecoder,
			want: [][]kv{nil, nil},
		},
		{
			data: `x= `,
			dec:  defaultDecoder,
			want: [][]kv{{{[]byte("x"), nil}}},
		},
		{
			data: `y=`,
			dec:  defaultDecoder,
			want: [][]kv{{{[]byte("y"), nil}}},
		},
		{
			data: `y`,
			dec:  defaultDecoder,
			want: [][]kv{{{[]byte("y"), nil}}},
		},
		{
			data: `y=f`,
			dec:  defaultDecoder,
			want: [][]kv{{{[]byte("y"), []byte("f")}}},
		},
		{
			data: "y=\"\\tf\"",
			dec:  defaultDecoder,
			want: [][]kv{{{[]byte("y"), []byte("\tf")}}},
		},
		{
			data: "a=1\n",
			dec:  defaultDecoder,
			want: [][]kv{{{[]byte("a"), []byte("1")}}},
		},
		{
			data: `a=1 b="bar" ƒ=2h3s r="esc\t" d x=sf   `,
			dec:  defaultDecoder,
			want: [][]kv{{
				{[]byte("a"), []byte("1")},
				{[]byte("b"), []byte("bar")},
				{[]byte("ƒ"), []byte("2h3s")},
				{[]byte("r"), []byte("esc\t")},
				{[]byte("d"), nil},
				{[]byte("x"), []byte("sf")},
			}},
		},
		{
			data: "y=f\ny=g",
			dec:  defaultDecoder,
			want: [][]kv{
				{{[]byte("y"), []byte("f")}},
				{{[]byte("y"), []byte("g")}},
			},
		},
		{
			data: "y=f  \n\x1e y=g",
			dec:  defaultDecoder,
			want: [][]kv{
				{{[]byte("y"), []byte("f")}},
				{{[]byte("y"), []byte("g")}},
			},
		},
		{
			data: "y= d y=g",
			dec:  defaultDecoder,
			want: [][]kv{{
				{[]byte("y"), nil},
				{[]byte("d"), nil},
				{[]byte("y"), []byte("g")},
			}},
		},
		{
			data: "y=\"f\"\ny=g",
			dec:  defaultDecoder,
			want: [][]kv{
				{{[]byte("y"), []byte("f")}},
				{{[]byte("y"), []byte("g")}},
			},
		},
		{
			data: "y=\"f\\n\"y=g",
			dec:  defaultDecoder,
			want: [][]kv{{
				{[]byte("y"), []byte("f\n")},
				{[]byte("y"), []byte("g")},
			}},
		},
		{
			data: strings.Repeat(`y=f `, 5),
			dec:  func(s string) *Decoder { return NewDecoderSize(strings.NewReader(s), 21) },
			want: [][]kv{{
				{[]byte("y"), []byte("f")},
				{[]byte("y"), []byte("f")},
				{[]byte("y"), []byte("f")},
				{[]byte("y"), []byte("f")},
				{[]byte("y"), []byte("f")},
			}},
		},
	}

	for _, test := range tests {
		var got [][]kv
		dec := test.dec(test.data)

		for dec.ScanRecord() {
			var kvs []kv
			for dec.ScanKeyval() {
				k := dec.Key()
				v := dec.Value()
				if k != nil {
					kvs = append(kvs, kv{k, v})
				}
			}
			got = append(got, kvs)
		}
		if err := dec.Err(); err != nil {
			t.Errorf("got err: %v", err)
		}
		if !reflect.DeepEqual(got, test.want) {
			t.Errorf("\n  in: %q\n got: %+v\nwant: %+v", test.data, got, test.want)
		}
	}
}

func TestDecoder_errors(t *testing.T) {
	defaultDecoder := func(s string) *Decoder { return NewDecoder(strings.NewReader(s)) }
	tests := []struct {
		data string
		dec  func(string) *Decoder
		want error
	}{
		{
			data: "a=1\n=bar",
			dec:  defaultDecoder,
			want: &SyntaxError{Msg: "unexpected '='", Line: 2, Pos: 1},
		},
		{
			data: "a=1\n\"k\"=bar",
			dec:  defaultDecoder,
			want: &SyntaxError{Msg: "unexpected '\"'", Line: 2, Pos: 1},
		},
		{
			data: "a=1\nk\"ey=bar",
			dec:  defaultDecoder,
			want: &SyntaxError{Msg: "unexpected '\"'", Line: 2, Pos: 2},
		},
		{
			data: "a=1\nk=b\"ar",
			dec:  defaultDecoder,
			want: &SyntaxError{Msg: "unexpected '\"'", Line: 2, Pos: 4},
		},
		{
			data: "a=1\nk=b =ar",
			dec:  defaultDecoder,
			want: &SyntaxError{Msg: "unexpected '='", Line: 2, Pos: 5},
		},
		{
			data: "a==",
			dec:  defaultDecoder,
			want: &SyntaxError{Msg: "unexpected '='", Line: 1, Pos: 3},
		},
		{
			data: "a=1\nk=b=ar",
			dec:  defaultDecoder,
			want: &SyntaxError{Msg: "unexpected '='", Line: 2, Pos: 4},
		},
		{
			data: "a=\"1",
			dec:  defaultDecoder,
			want: &SyntaxError{Msg: "unterminated quoted value", Line: 1, Pos: 5},
		},
		{
			data: "a=\"1\\",
			dec:  defaultDecoder,
			want: &SyntaxError{Msg: "unterminated quoted value", Line: 1, Pos: 6},
		},
		{
			data: "a=\"\\t1",
			dec:  defaultDecoder,
			want: &SyntaxError{Msg: "unterminated quoted value", Line: 1, Pos: 7},
		},
		{
			data: "a=\"\\u1\"",
			dec:  defaultDecoder,
			want: &SyntaxError{Msg: "invalid quoted value", Line: 1, Pos: 8},
		},
		{
			data: "a\ufffd=bar",
			dec:  defaultDecoder,
			want: &SyntaxError{Msg: "invalid key", Line: 1, Pos: 5},
		},
		{
			data: "\x80=bar",
			dec:  defaultDecoder,
			want: &SyntaxError{Msg: "invalid key", Line: 1, Pos: 2},
		},
		{
			data: "\x80",
			dec:  defaultDecoder,
			want: &SyntaxError{Msg: "invalid key", Line: 1, Pos: 2},
		},
		{
			data: "a=1\nb=2",
			dec: func(s string) *Decoder {
				dec := NewDecoderSize(strings.NewReader(s), 1)
				return dec
			},
			want: bufio.ErrTooLong,
		},
	}

	for _, test := range tests {
		dec := test.dec(test.data)

		for dec.ScanRecord() {
			for dec.ScanKeyval() {
			}
		}
		if got, want := dec.Err(), test.want; !reflect.DeepEqual(got, want) {
			t.Errorf("got: %v, want: %v", got, want)
		}
	}
}

func TestDecoder_decode_encode(t *testing.T) {
	tests := []struct {
		in, out string
	}{
		{"", ""},
		{"\n", "\n"},
		{"\n  \n", "\n\n"},
		{
			"a=1\nb=2\n",
			"a=1\nb=2\n",
		},
		{
			"a=1 b=\"bar\" ƒ=2h3s r=\"esc\\t\" d x=sf   ",
			"a=1 b=bar ƒ=2h3s r=\"esc\\t\" d= x=sf\n",
		},
	}

	for _, test := range tests {
		dec := NewDecoder(strings.NewReader(test.in))
		buf := bytes.Buffer{}
		enc := NewEncoder(&buf)

		var err error
	loop:
		for dec.ScanRecord() && err == nil {
			for dec.ScanKeyval() {
				if dec.Key() == nil {
					continue
				}
				if err = enc.EncodeKeyval(dec.Key(), dec.Value()); err != nil {
					break loop
				}
			}
			enc.EndRecord()
		}
		if err == nil {
			err = dec.Err()
		}
		if err != nil {
			t.Errorf("got err: %v", err)
		}
		if got, want := buf.String(), test.out; got != want {
			t.Errorf("\n got: %q\nwant: %q", got, want)
		}
	}
}
