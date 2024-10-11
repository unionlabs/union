package amino_test

import (
	"bytes"
	"encoding/json"
	"fmt"
	"reflect"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	"github.com/tendermint/go-amino"
)

func registerTransports(cdc *amino.Codec) {
	cdc.RegisterConcrete(&Transport{}, "our/transport", nil)
	cdc.RegisterInterface((*Vehicle)(nil), &amino.InterfaceOptions{AlwaysDisambiguate: true})
	cdc.RegisterInterface((*Asset)(nil), &amino.InterfaceOptions{AlwaysDisambiguate: true})
	cdc.RegisterConcrete(Car(""), "car", nil)
	cdc.RegisterConcrete(insurancePlan(0), "insuranceplan", nil)
	cdc.RegisterConcrete(Boat(""), "boat", nil)
	cdc.RegisterConcrete(Plane{}, "plane", nil)
}

func TestMarshalJSON(t *testing.T) {
	var cdc = amino.NewCodec()
	registerTransports(cdc)
	cases := []struct {
		in      interface{}
		want    string
		wantErr string
	}{
		{&noFields{}, "{}", ""},                        // #0
		{&noExportedFields{a: 10, b: "foo"}, "{}", ""}, // #1
		{nil, "null", ""},                              // #2
		{&oneExportedField{}, `{"A":""}`, ""},          // #3
		{Vehicle(Car("Tesla")),
			`{"type":"car","value":"Tesla"}`, ""}, // #4
		{Car("Tesla"), `{"type":"car","value":"Tesla"}`, ""}, // #5
		{&oneExportedField{A: "Z"}, `{"A":"Z"}`, ""},         // #6
		{[]string{"a", "bc"}, `["a","bc"]`, ""},              // #7
		{[]interface{}{"a", "bc", 10, 10.93, 1e3},
			``, "Unregistered"}, // #8
		{aPointerField{Foo: new(int), Name: "name"},
			`{"Foo":"0","nm":"name"}`, ""}, // #9
		{
			aPointerFieldAndEmbeddedField{intPtr(11), "ap", nil, &oneExportedField{A: "foo"}},
			`{"Foo":"11","nm":"ap","bz":{"A":"foo"}}`, "",
		}, // #10
		{
			doublyEmbedded{
				Inner: &aPointerFieldAndEmbeddedField{
					intPtr(11), "ap", nil, &oneExportedField{A: "foo"},
				},
			},
			`{"Inner":{"Foo":"11","nm":"ap","bz":{"A":"foo"}},"year":0}`, "",
		}, // #11
		{
			struct{}{}, `{}`, "",
		}, // #12
		{
			struct{ A int }{A: 10}, `{"A":"10"}`, "",
		}, // #13
		{
			Transport{},
			`{"type":"our/transport","value":{"Vehicle":null,"Capacity":"0"}}`, "",
		}, // #14
		{
			Transport{Vehicle: Car("Bugatti")},
			`{"type":"our/transport","value":{"Vehicle":{"type":"car","value":"Bugatti"},"Capacity":"0"}}`, "",
		}, // #15
		{
			BalanceSheet{Assets: []Asset{Car("Corolla"), insurancePlan(1e7)}},
			`{"assets":[{"type":"car","value":"Corolla"},{"type":"insuranceplan","value":"10000000"}]}`, "",
		}, // #16
		{
			Transport{Vehicle: Boat("Poseidon"), Capacity: 1789},
			`{"type":"our/transport","value":{"Vehicle":{"type":"boat","value":"Poseidon"},"Capacity":"1789"}}`, "",
		}, // #17
		{
			withCustomMarshaler{A: &aPointerField{Foo: intPtr(12)}, F: customJSONMarshaler(10)},
			`{"fx":"Tendermint","A":{"Foo":"12"}}`, "",
		}, // #18
		{
			func() json.Marshaler { v := customJSONMarshaler(10); return &v }(),
			`"Tendermint"`, "",
		}, // #19

		// We don't yet support interface pointer registration i.e. `*interface{}`
		{
			interfacePtr("a"), "", "Unregistered interface interface {}",
		}, // #20
		{&fp{"Foo", 10}, "<FP-MARSHALJSON>", ""}, // #21
		{(*fp)(nil), "null", ""},                 // #22
		{struct {
			FP      *fp
			Package string
		}{FP: &fp{"Foo", 10}, Package: "bytes"},
			`{"FP":<FP-MARSHALJSON>,"Package":"bytes"}`, "",
		}, // #23,
	}

	for i, tt := range cases {
		t.Logf("Trying case #%v", i)
		blob, err := cdc.MarshalJSON(tt.in)
		if tt.wantErr != "" {
			if err == nil || !strings.Contains(err.Error(), tt.wantErr) {
				t.Errorf("#%d:\ngot:\n\t%q\nwant non-nil error containing\n\t%q", i,
					err, tt.wantErr)
			}
			continue
		}

		if err != nil {
			t.Errorf("#%d: unexpected error: %v\nblob: %v", i, err, tt.in)
			continue
		}
		if g, w := string(blob), tt.want; g != w {
			t.Errorf("#%d:\ngot:\n\t%s\nwant:\n\t%s", i, g, w)
		}
	}
}

func TestMarshalJSONTime(t *testing.T) {
	var cdc = amino.NewCodec()
	registerTransports(cdc)

	type SimpleStruct struct {
		String string
		Bytes  []byte
		Time   time.Time
	}

	s := SimpleStruct{
		String: "hello",
		Bytes:  []byte("goodbye"),
		Time:   time.Now().Round(0).UTC(), // strip monotonic.
	}

	b, err := cdc.MarshalJSON(s)
	assert.Nil(t, err)

	var s2 SimpleStruct
	err = cdc.UnmarshalJSON(b, &s2)
	assert.Nil(t, err)
	assert.Equal(t, s, s2)
}

type fp struct {
	Name    string
	Version int
}

func (f *fp) MarshalJSON() ([]byte, error) {
	return []byte("<FP-MARSHALJSON>"), nil
}

func (f *fp) UnmarshalJSON(blob []byte) error {
	f.Name = string(blob)
	return nil
}

var _ json.Marshaler = (*fp)(nil)
var _ json.Unmarshaler = (*fp)(nil)

type innerFP struct {
	PC uint64
	FP *fp
}

func TestUnmarshalMap(t *testing.T) {
	binBytes := []byte(`dontcare`)
	obj := new(map[string]int)
	cdc := amino.NewCodec()
	// Binary doesn't support decoding to a map...
	// TODO: move out binary tests from json_test.go ...
	assert.Panics(t, func() {
		err := cdc.UnmarshalBinaryBare(binBytes, &obj)
		assert.Fail(t, "should have paniced but got err: %v", err)
	})
	assert.Panics(t, func() {
		err := cdc.UnmarshalBinaryBare(binBytes, obj)
		assert.Fail(t, "should have paniced but got err: %v", err)
	})
	// ... nor encoding it.
	assert.Panics(t, func() {
		bz, err := cdc.MarshalBinaryBare(obj)
		assert.Fail(t, "should have paniced but got bz: %X err: %v", bz, err)
	})

	invalidJSONMapBytes := []byte(`{"some_key": 2}`)
	// we expect quoted values for javascript / JSON numbers:
	err := cdc.UnmarshalJSON(invalidJSONMapBytes, &obj)
	assert.Error(t, err)

	validJSONMapBytes := []byte(`{"some_key": "2"}`)
	err = cdc.UnmarshalJSON(validJSONMapBytes, obj)
	assert.NoError(t, err)

	// ... nor encoding it.
	assert.Panics(t, func() {
		bz, err := cdc.MarshalJSON(obj)
		assert.Fail(t, "should have paniced but got bz: %X err: %v", bz, err)
	})
}

func TestUnmarshalFunc(t *testing.T) {
	binBytes := []byte(`dontcare`)
	jsonBytes := []byte(`"dontcare"`)
	obj := func() {}
	cdc := amino.NewCodec()
	// Binary doesn't support decoding to a func...

	err := cdc.UnmarshalBinaryLengthPrefixed(binBytes, &obj)
	// on length prefixed we return an error:
	assert.Error(t, err)

	assert.Panics(t, func() {
		err := cdc.UnmarshalBinaryBare(binBytes, &obj)
		// panics with "unknown field type Func"
		assert.Fail(t, "should have paniced but got err: %v", err)
	})
	assert.Panics(t, func() {
		err := cdc.UnmarshalBinaryBare(binBytes, obj)
		assert.Fail(t, "should have paniced but got err: %v", err)
	})
	// ... nor encoding it.
	assert.Panics(t, func() {
		bz, err := cdc.MarshalBinaryLengthPrefixed(obj)
		assert.Fail(t, "should have paniced but got bz: %X err: %v", bz, err)
	})
	// JSON doesn't support decoding to a func...
	assert.Panics(t, func() {
		err := cdc.UnmarshalJSON(jsonBytes, &obj)
		assert.Fail(t, "should have paniced but got err: %v", err)
	})

	err = cdc.UnmarshalJSON(jsonBytes, obj)
	// UnmarshalJSON expects a pointer
	assert.Error(t, err)

	// ... nor encoding it.
	assert.Panics(t, func() {
		bz, err := cdc.MarshalJSON(obj)
		assert.Fail(t, "should have paniced but got bz: %X err: %v", bz, err)
	})
}

func TestUnmarshalJSON(t *testing.T) {
	var cdc = amino.NewCodec()
	registerTransports(cdc)
	cases := []struct {
		blob    string
		in      interface{}
		want    interface{}
		wantErr string
	}{
		{ // #0
			`null`, 2, nil, "expects a pointer",
		},
		{ // #1
			`null`, new(int), new(int), "",
		},
		{ // #2
			`"2"`, new(int), intPtr(2), "",
		},
		{ // #3
			`{"null"}`, new(int), nil, "invalid character",
		},
		{ // #4
			`{"type":"our/transport","value":{"Vehicle":null,"Capacity":"0"}}`, new(Transport), new(Transport), "",
		},
		{ // #5
			`{"type":"our/transport","value":{"Vehicle":{"type":"car","value":"Bugatti"},"Capacity":"10"}}`,
			new(Transport),
			&Transport{
				Vehicle:  Car("Bugatti"),
				Capacity: 10,
			}, "",
		},
		{ // #6
			`{"type":"car","value":"Bugatti"}`, new(Car), func() *Car { c := Car("Bugatti"); return &c }(), "",
		},
		{ // #7
			`["1", "2", "3"]`, new([]int), func() interface{} {
				v := []int{1, 2, 3}
				return &v
			}(), "",
		},
		{ // #8
			`["1", "2", "3"]`, new([]string), func() interface{} {
				v := []string{"1", "2", "3"}
				return &v
			}(), "",
		},
		{ // #9
			`[1, "2", ["foo", "bar"]]`,
			new([]interface{}), nil, "Unregistered",
		},
		{ // #10
			`2.34`, floatPtr(2.34), nil, "float* support requires",
		},
		{ // #11
			"<FooBar>", new(fp), &fp{"<FooBar>", 0}, "",
		},
		{ // #12
			"10", new(fp), &fp{Name: "10"}, "",
		},
		{ // #13
			`{"PC":"125","FP":"10"}`, new(innerFP), &innerFP{PC: 125, FP: &fp{Name: `"10"`}}, "",
		},
		{ // #14
			`{"PC":"125","FP":"<FP-FOO>"}`, new(innerFP), &innerFP{PC: 125, FP: &fp{Name: `"<FP-FOO>"`}}, "",
		},
	}

	for i, tt := range cases {
		err := cdc.UnmarshalJSON([]byte(tt.blob), tt.in)
		if tt.wantErr != "" {
			if err == nil || !strings.Contains(err.Error(), tt.wantErr) {
				t.Errorf("#%d:\ngot:\n\t%q\nwant non-nil error containing\n\t%q", i,
					err, tt.wantErr)
			}
			continue
		}

		if err != nil {
			t.Errorf("#%d: unexpected error: %v\nblob: %s\nin: %+v\n", i, err, tt.blob, tt.in)
			continue
		}
		if g, w := tt.in, tt.want; !reflect.DeepEqual(g, w) {
			gb, _ := json.MarshalIndent(g, "", "  ")
			wb, _ := json.MarshalIndent(w, "", "  ")
			t.Errorf("#%d:\ngot:\n\t%#v\n(%s)\n\nwant:\n\t%#v\n(%s)", i, g, gb, w, wb)
		}
	}
}

func TestJSONCodecRoundTrip(t *testing.T) {
	var cdc = amino.NewCodec()
	registerTransports(cdc)
	type allInclusive struct {
		Tr      Transport `json:"trx"`
		Vehicle Vehicle   `json:"v,omitempty"`
		Comment string
		Data    []byte
	}

	cases := []struct {
		in      interface{}
		want    interface{}
		out     interface{}
		wantErr string
	}{
		0: {
			in: &allInclusive{
				Tr: Transport{
					Vehicle: Boat("Oracle"),
				},
				Comment: "To the Cosmos! баллинг в космос",
				Data:    []byte("祝你好运"),
			},
			out: new(allInclusive),
			want: &allInclusive{
				Tr: Transport{
					Vehicle: Boat("Oracle"),
				},
				Comment: "To the Cosmos! баллинг в космос",
				Data:    []byte("祝你好运"),
			},
		},

		1: {
			in:   Transport{Vehicle: Plane{Name: "G6", MaxAltitude: 51e3}, Capacity: 18},
			out:  new(Transport),
			want: &Transport{Vehicle: Plane{Name: "G6", MaxAltitude: 51e3}, Capacity: 18},
		},
	}

	for i, tt := range cases {
		mBlob, err := cdc.MarshalJSON(tt.in)
		if tt.wantErr != "" {
			if err == nil || !strings.Contains(err.Error(), tt.wantErr) {
				t.Errorf("#%d:\ngot:\n\t%q\nwant non-nil error containing\n\t%q", i,
					err, tt.wantErr)
			}
			continue
		}

		if err != nil {
			t.Errorf("#%d: unexpected error after MarshalJSON: %v", i, err)
			continue
		}

		if err := cdc.UnmarshalJSON(mBlob, tt.out); err != nil {
			t.Errorf("#%d: unexpected error after UnmarshalJSON: %v\nmBlob: %s", i, err, mBlob)
			continue
		}

		// Now check that the input is exactly equal to the output
		uBlob, err := cdc.MarshalJSON(tt.out)
		if err := cdc.UnmarshalJSON(mBlob, tt.out); err != nil {
			t.Errorf("#%d: unexpected error after second MarshalJSON: %v", i, err)
			continue
		}
		if !reflect.DeepEqual(tt.want, tt.out) {
			t.Errorf("#%d: After roundtrip UnmarshalJSON\ngot: \t%v\nwant:\t%v", i, tt.out, tt.want)
		}
		if !bytes.Equal(mBlob, uBlob) {
			t.Errorf("#%d: After roundtrip MarshalJSON\ngot: \t%s\nwant:\t%s", i, uBlob, mBlob)
		}
	}
}

func intPtr(i int) *int {
	return &i
}

func floatPtr(f float64) *float64 {
	return &f
}

type noFields struct{}
type noExportedFields struct {
	a int
	b string
}

type oneExportedField struct {
	_Foo int
	A    string
	b    string
}

type aPointerField struct {
	Foo  *int
	Name string `json:"nm,omitempty"`
}

type doublyEmbedded struct {
	Inner *aPointerFieldAndEmbeddedField
	Year  int32 `json:"year"`
}

type aPointerFieldAndEmbeddedField struct {
	Foo  *int
	Name string `json:"nm,omitempty"`
	*oneExportedField
	B *oneExportedField `json:"bz,omitempty"`
}

type customJSONMarshaler int

var _ json.Marshaler = (*customJSONMarshaler)(nil)

func (cm customJSONMarshaler) MarshalJSON() ([]byte, error) {
	return []byte(`"Tendermint"`), nil
}

type withCustomMarshaler struct {
	F customJSONMarshaler `json:"fx"`
	A *aPointerField
}

type Transport struct {
	Vehicle
	Capacity int
}

type Vehicle interface {
	Move() error
}

type Asset interface {
	Value() float64
}

func (c Car) Value() float64 {
	return 60000.0
}

type BalanceSheet struct {
	Assets []Asset `json:"assets"`
}

type Car string
type Boat string
type Plane struct {
	Name        string
	MaxAltitude int64
}
type insurancePlan int

func (ip insurancePlan) Value() float64 { return float64(ip) }

func (c Car) Move() error   { return nil }
func (b Boat) Move() error  { return nil }
func (p Plane) Move() error { return nil }

func interfacePtr(v interface{}) *interface{} {
	return &v
}

// Test to ensure that Amino codec's time encoding/decoding roundtrip
// produces the same result as the standard library json's.
func TestAminoJSONTimeEncodeDecodeRoundTrip(t *testing.T) {
	loc, _ := time.LoadLocation("America/Los_Angeles")
	din := time.Date(2008, 9, 15, 14, 13, 12, 11109876, loc).Round(time.Millisecond).UTC()

	cdc := amino.NewCodec()
	blobAmino, err := cdc.MarshalJSON(din)
	require.Nil(t, err, "amino.Codec.MarshalJSON should succeed")
	var tAminoOut time.Time
	require.Nil(t, cdc.UnmarshalJSON(blobAmino, &tAminoOut), "amino.Codec.UnmarshalJSON should succeed")
	require.NotEqual(t, tAminoOut, time.Time{}, "amino.marshaled definitely isn't equal to zero time")
	require.Equal(t, tAminoOut, din, "expecting marshaled in to be equal to marshaled out")

	blobStdlib, err := json.Marshal(din)
	require.Nil(t, err, "json.Marshal should succeed")
	var tStdlibOut time.Time
	require.Nil(t, json.Unmarshal(blobStdlib, &tStdlibOut), "json.Unmarshal should succeed")
	require.NotEqual(t, tStdlibOut, time.Time{}, "stdlib.marshaled definitely isn't equal to zero time")
	require.Equal(t, tStdlibOut, din, "expecting stdlib.marshaled to be equal to time in")

	require.Equal(t, tAminoOut, tStdlibOut, "expecting amino.unmarshaled to be equal to json.unmarshaled")
}

//----------------------------------------

func TestMarshalJSONMap(t *testing.T) {
	var cdc = amino.NewCodec()

	type SimpleStruct struct {
		Foo int
		Bar []byte
	}

	type MapsStruct struct {
		Map1      map[string]string
		Map1nil   map[string]string
		Map1empty map[string]string

		Map2      map[string]SimpleStruct
		Map2nil   map[string]SimpleStruct
		Map2empty map[string]SimpleStruct

		Map3      map[string]*SimpleStruct
		Map3nil   map[string]*SimpleStruct
		Map3empty map[string]*SimpleStruct

		/*
			NOT SUPPORTED YET.  FIRST, DEFINE SPEC.
			Map4      map[int]*SimpleStruct
			Map4nil   map[int]*SimpleStruct
			Map4empty map[int]*SimpleStruct
		*/
	}

	ms := MapsStruct{
		Map1:      map[string]string{"foo": "bar"},
		Map1nil:   (map[string]string)(nil),
		Map1empty: map[string]string{},

		Map2:      map[string]SimpleStruct{"foo": {Foo: 1, Bar: []byte("bar")}},
		Map2nil:   (map[string]SimpleStruct)(nil),
		Map2empty: map[string]SimpleStruct{},

		Map3:      map[string]*SimpleStruct{"foo": {Foo: 1, Bar: []byte("bar")}},
		Map3nil:   (map[string]*SimpleStruct)(nil),
		Map3empty: map[string]*SimpleStruct{},

		/*
			Map4:      map[int]*SimpleStruct{123: &SimpleStruct{Foo: 1, Bar: []byte("bar")}},
			Map4nil:   (map[int]*SimpleStruct)(nil),
			Map4empty: map[int]*SimpleStruct{},
		*/
	}

	// ms2 is expected to be this.
	ms3 := MapsStruct{
		Map1:      map[string]string{"foo": "bar"},
		Map1nil:   map[string]string{},
		Map1empty: map[string]string{},

		Map2:      map[string]SimpleStruct{"foo": {Foo: 1, Bar: []byte("bar")}},
		Map2nil:   map[string]SimpleStruct{},
		Map2empty: map[string]SimpleStruct{},

		Map3:      map[string]*SimpleStruct{"foo": {Foo: 1, Bar: []byte("bar")}},
		Map3nil:   map[string]*SimpleStruct{},
		Map3empty: map[string]*SimpleStruct{},

		/*
			Map4:      map[int]*SimpleStruct{123: &SimpleStruct{Foo: 1, Bar: []byte("bar")}},
			Map4nil:   (map[int]*SimpleStruct)(nil),
			Map4empty: map[int]*SimpleStruct{},
		*/
	}

	b, err := cdc.MarshalJSON(ms)
	assert.Nil(t, err)

	var ms2 MapsStruct
	err = cdc.UnmarshalJSON(b, &ms2)
	assert.Nil(t, err)
	assert.Equal(t, ms3, ms2)
}

func TestMarshalJSONIndent(t *testing.T) {
	var cdc = amino.NewCodec()
	registerTransports(cdc)
	obj := Car("Tesla")
	indent := "  "
	expected := fmt.Sprintf(`{
%s"type": "car",
%s"value": "Tesla"
}`, indent, indent)

	blob, err := cdc.MarshalJSONIndent(obj, "", "  ")
	assert.Nil(t, err)
	assert.Equal(t, expected, string(blob))
}
