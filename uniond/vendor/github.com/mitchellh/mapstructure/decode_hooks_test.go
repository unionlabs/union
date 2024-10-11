package mapstructure

import (
	"errors"
	"math/big"
	"net"
	"reflect"
	"testing"
	"time"
)

func TestComposeDecodeHookFunc(t *testing.T) {
	f1 := func(
		f reflect.Kind,
		t reflect.Kind,
		data interface{}) (interface{}, error) {
		return data.(string) + "foo", nil
	}

	f2 := func(
		f reflect.Kind,
		t reflect.Kind,
		data interface{}) (interface{}, error) {
		return data.(string) + "bar", nil
	}

	f := ComposeDecodeHookFunc(f1, f2)

	result, err := DecodeHookExec(
		f, reflect.ValueOf(""), reflect.ValueOf([]byte("")))
	if err != nil {
		t.Fatalf("bad: %s", err)
	}
	if result.(string) != "foobar" {
		t.Fatalf("bad: %#v", result)
	}
}

func TestComposeDecodeHookFunc_err(t *testing.T) {
	f1 := func(reflect.Kind, reflect.Kind, interface{}) (interface{}, error) {
		return nil, errors.New("foo")
	}

	f2 := func(reflect.Kind, reflect.Kind, interface{}) (interface{}, error) {
		panic("NOPE")
	}

	f := ComposeDecodeHookFunc(f1, f2)

	_, err := DecodeHookExec(
		f, reflect.ValueOf(""), reflect.ValueOf([]byte("")))
	if err.Error() != "foo" {
		t.Fatalf("bad: %s", err)
	}
}

func TestComposeDecodeHookFunc_kinds(t *testing.T) {
	var f2From reflect.Kind

	f1 := func(
		f reflect.Kind,
		t reflect.Kind,
		data interface{}) (interface{}, error) {
		return int(42), nil
	}

	f2 := func(
		f reflect.Kind,
		t reflect.Kind,
		data interface{}) (interface{}, error) {
		f2From = f
		return data, nil
	}

	f := ComposeDecodeHookFunc(f1, f2)

	_, err := DecodeHookExec(
		f, reflect.ValueOf(""), reflect.ValueOf([]byte("")))
	if err != nil {
		t.Fatalf("bad: %s", err)
	}
	if f2From != reflect.Int {
		t.Fatalf("bad: %#v", f2From)
	}
}

func TestOrComposeDecodeHookFunc(t *testing.T) {
	f1 := func(
		f reflect.Kind,
		t reflect.Kind,
		data interface{}) (interface{}, error) {
		return data.(string) + "foo", nil
	}

	f2 := func(
		f reflect.Kind,
		t reflect.Kind,
		data interface{}) (interface{}, error) {
		return data.(string) + "bar", nil
	}

	f := OrComposeDecodeHookFunc(f1, f2)

	result, err := DecodeHookExec(
		f, reflect.ValueOf(""), reflect.ValueOf([]byte("")))
	if err != nil {
		t.Fatalf("bad: %s", err)
	}
	if result.(string) != "foo" {
		t.Fatalf("bad: %#v", result)
	}
}

func TestOrComposeDecodeHookFunc_correctValueIsLast(t *testing.T) {
	f1 := func(
		f reflect.Kind,
		t reflect.Kind,
		data interface{}) (interface{}, error) {
		return nil, errors.New("f1 error")
	}

	f2 := func(
		f reflect.Kind,
		t reflect.Kind,
		data interface{}) (interface{}, error) {
		return nil, errors.New("f2 error")
	}

	f3 := func(
		f reflect.Kind,
		t reflect.Kind,
		data interface{}) (interface{}, error) {
		return data.(string) + "bar", nil
	}

	f := OrComposeDecodeHookFunc(f1, f2, f3)

	result, err := DecodeHookExec(
		f, reflect.ValueOf(""), reflect.ValueOf([]byte("")))
	if err != nil {
		t.Fatalf("bad: %s", err)
	}
	if result.(string) != "bar" {
		t.Fatalf("bad: %#v", result)
	}
}

func TestOrComposeDecodeHookFunc_err(t *testing.T) {
	f1 := func(
		f reflect.Kind,
		t reflect.Kind,
		data interface{}) (interface{}, error) {
		return nil, errors.New("f1 error")
	}

	f2 := func(
		f reflect.Kind,
		t reflect.Kind,
		data interface{}) (interface{}, error) {
		return nil, errors.New("f2 error")
	}

	f := OrComposeDecodeHookFunc(f1, f2)

	_, err := DecodeHookExec(
		f, reflect.ValueOf(""), reflect.ValueOf([]byte("")))
	if err == nil {
		t.Fatalf("bad: should return an error")
	}
	if err.Error() != "f1 error\nf2 error\n" {
		t.Fatalf("bad: %s", err)
	}
}

func TestComposeDecodeHookFunc_safe_nofuncs(t *testing.T) {
	f := ComposeDecodeHookFunc()
	type myStruct2 struct {
		MyInt int
	}

	type myStruct1 struct {
		Blah map[string]myStruct2
	}

	src := &myStruct1{Blah: map[string]myStruct2{
		"test": {
			MyInt: 1,
		},
	}}

	dst := &myStruct1{}
	dConf := &DecoderConfig{
		Result:      dst,
		ErrorUnused: true,
		DecodeHook:  f,
	}
	d, err := NewDecoder(dConf)
	if err != nil {
		t.Fatal(err)
	}
	err = d.Decode(src)
	if err != nil {
		t.Fatal(err)
	}
}

func TestStringToSliceHookFunc(t *testing.T) {
	f := StringToSliceHookFunc(",")

	strValue := reflect.ValueOf("42")
	sliceValue := reflect.ValueOf([]byte("42"))
	cases := []struct {
		f, t   reflect.Value
		result interface{}
		err    bool
	}{
		{sliceValue, sliceValue, []byte("42"), false},
		{strValue, strValue, "42", false},
		{
			reflect.ValueOf("foo,bar,baz"),
			sliceValue,
			[]string{"foo", "bar", "baz"},
			false,
		},
		{
			reflect.ValueOf(""),
			sliceValue,
			[]string{},
			false,
		},
	}

	for i, tc := range cases {
		actual, err := DecodeHookExec(f, tc.f, tc.t)
		if tc.err != (err != nil) {
			t.Fatalf("case %d: expected err %#v", i, tc.err)
		}
		if !reflect.DeepEqual(actual, tc.result) {
			t.Fatalf(
				"case %d: expected %#v, got %#v",
				i, tc.result, actual)
		}
	}
}

func TestStringToTimeDurationHookFunc(t *testing.T) {
	f := StringToTimeDurationHookFunc()

	timeValue := reflect.ValueOf(time.Duration(5))
	strValue := reflect.ValueOf("")
	cases := []struct {
		f, t   reflect.Value
		result interface{}
		err    bool
	}{
		{reflect.ValueOf("5s"), timeValue, 5 * time.Second, false},
		{reflect.ValueOf("5"), timeValue, time.Duration(0), true},
		{reflect.ValueOf("5"), strValue, "5", false},
	}

	for i, tc := range cases {
		actual, err := DecodeHookExec(f, tc.f, tc.t)
		if tc.err != (err != nil) {
			t.Fatalf("case %d: expected err %#v", i, tc.err)
		}
		if !reflect.DeepEqual(actual, tc.result) {
			t.Fatalf(
				"case %d: expected %#v, got %#v",
				i, tc.result, actual)
		}
	}
}

func TestStringToTimeHookFunc(t *testing.T) {
	strValue := reflect.ValueOf("5")
	timeValue := reflect.ValueOf(time.Time{})
	cases := []struct {
		f, t   reflect.Value
		layout string
		result interface{}
		err    bool
	}{
		{reflect.ValueOf("2006-01-02T15:04:05Z"), timeValue, time.RFC3339,
			time.Date(2006, 1, 2, 15, 4, 5, 0, time.UTC), false},
		{strValue, timeValue, time.RFC3339, time.Time{}, true},
		{strValue, strValue, time.RFC3339, "5", false},
	}

	for i, tc := range cases {
		f := StringToTimeHookFunc(tc.layout)
		actual, err := DecodeHookExec(f, tc.f, tc.t)
		if tc.err != (err != nil) {
			t.Fatalf("case %d: expected err %#v", i, tc.err)
		}
		if !reflect.DeepEqual(actual, tc.result) {
			t.Fatalf(
				"case %d: expected %#v, got %#v",
				i, tc.result, actual)
		}
	}
}

func TestStringToIPHookFunc(t *testing.T) {
	strValue := reflect.ValueOf("5")
	ipValue := reflect.ValueOf(net.IP{})
	cases := []struct {
		f, t   reflect.Value
		result interface{}
		err    bool
	}{
		{reflect.ValueOf("1.2.3.4"), ipValue,
			net.IPv4(0x01, 0x02, 0x03, 0x04), false},
		{strValue, ipValue, net.IP{}, true},
		{strValue, strValue, "5", false},
	}

	for i, tc := range cases {
		f := StringToIPHookFunc()
		actual, err := DecodeHookExec(f, tc.f, tc.t)
		if tc.err != (err != nil) {
			t.Fatalf("case %d: expected err %#v", i, tc.err)
		}
		if !reflect.DeepEqual(actual, tc.result) {
			t.Fatalf(
				"case %d: expected %#v, got %#v",
				i, tc.result, actual)
		}
	}
}

func TestStringToIPNetHookFunc(t *testing.T) {
	strValue := reflect.ValueOf("5")
	ipNetValue := reflect.ValueOf(net.IPNet{})
	var nilNet *net.IPNet = nil

	cases := []struct {
		f, t   reflect.Value
		result interface{}
		err    bool
	}{
		{reflect.ValueOf("1.2.3.4/24"), ipNetValue,
			&net.IPNet{
				IP:   net.IP{0x01, 0x02, 0x03, 0x00},
				Mask: net.IPv4Mask(0xff, 0xff, 0xff, 0x00),
			}, false},
		{strValue, ipNetValue, nilNet, true},
		{strValue, strValue, "5", false},
	}

	for i, tc := range cases {
		f := StringToIPNetHookFunc()
		actual, err := DecodeHookExec(f, tc.f, tc.t)
		if tc.err != (err != nil) {
			t.Fatalf("case %d: expected err %#v", i, tc.err)
		}
		if !reflect.DeepEqual(actual, tc.result) {
			t.Fatalf(
				"case %d: expected %#v, got %#v",
				i, tc.result, actual)
		}
	}
}

func TestWeaklyTypedHook(t *testing.T) {
	var f DecodeHookFunc = WeaklyTypedHook

	strValue := reflect.ValueOf("")
	cases := []struct {
		f, t   reflect.Value
		result interface{}
		err    bool
	}{
		// TO STRING
		{
			reflect.ValueOf(false),
			strValue,
			"0",
			false,
		},

		{
			reflect.ValueOf(true),
			strValue,
			"1",
			false,
		},

		{
			reflect.ValueOf(float32(7)),
			strValue,
			"7",
			false,
		},

		{
			reflect.ValueOf(int(7)),
			strValue,
			"7",
			false,
		},

		{
			reflect.ValueOf([]uint8("foo")),
			strValue,
			"foo",
			false,
		},

		{
			reflect.ValueOf(uint(7)),
			strValue,
			"7",
			false,
		},
	}

	for i, tc := range cases {
		actual, err := DecodeHookExec(f, tc.f, tc.t)
		if tc.err != (err != nil) {
			t.Fatalf("case %d: expected err %#v", i, tc.err)
		}
		if !reflect.DeepEqual(actual, tc.result) {
			t.Fatalf(
				"case %d: expected %#v, got %#v",
				i, tc.result, actual)
		}
	}
}

func TestStructToMapHookFuncTabled(t *testing.T) {
	var f DecodeHookFunc = RecursiveStructToMapHookFunc()

	type b struct {
		TestKey string
	}

	type a struct {
		Sub b
	}

	testStruct := a{
		Sub: b{
			TestKey: "testval",
		},
	}

	testMap := map[string]interface{}{
		"Sub": map[string]interface{}{
			"TestKey": "testval",
		},
	}

	cases := []struct {
		name     string
		receiver interface{}
		input    interface{}
		expected interface{}
		err      bool
	}{
		{
			"map receiver",
			func() interface{} {
				var res map[string]interface{}
				return &res
			}(),
			testStruct,
			&testMap,
			false,
		},
		{
			"interface receiver",
			func() interface{} {
				var res interface{}
				return &res
			}(),
			testStruct,
			func() interface{} {
				var exp interface{} = testMap
				return &exp
			}(),
			false,
		},
		{
			"slice receiver errors",
			func() interface{} {
				var res []string
				return &res
			}(),
			testStruct,
			new([]string),
			true,
		},
		{
			"slice to slice - no change",
			func() interface{} {
				var res []string
				return &res
			}(),
			[]string{"a", "b"},
			&[]string{"a", "b"},
			false,
		},
		{
			"string to string - no change",
			func() interface{} {
				var res string
				return &res
			}(),
			"test",
			func() *string {
				s := "test"
				return &s
			}(),
			false,
		},
	}

	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			cfg := &DecoderConfig{
				DecodeHook: f,
				Result:     tc.receiver,
			}

			d, err := NewDecoder(cfg)
			if err != nil {
				t.Fatalf("unexpected err %#v", err)
			}

			err = d.Decode(tc.input)
			if tc.err != (err != nil) {
				t.Fatalf("expected err %#v", err)
			}

			if !reflect.DeepEqual(tc.expected, tc.receiver) {
				t.Fatalf("expected %#v, got %#v",
					tc.expected, tc.receiver)
			}
		})

	}
}

func TestTextUnmarshallerHookFunc(t *testing.T) {
	cases := []struct {
		f, t   reflect.Value
		result interface{}
		err    bool
	}{
		{reflect.ValueOf("42"), reflect.ValueOf(big.Int{}), big.NewInt(42), false},
		{reflect.ValueOf("invalid"), reflect.ValueOf(big.Int{}), nil, true},
		{reflect.ValueOf("5"), reflect.ValueOf("5"), "5", false},
	}

	for i, tc := range cases {
		f := TextUnmarshallerHookFunc()
		actual, err := DecodeHookExec(f, tc.f, tc.t)
		if tc.err != (err != nil) {
			t.Fatalf("case %d: expected err %#v", i, tc.err)
		}
		if !reflect.DeepEqual(actual, tc.result) {
			t.Fatalf(
				"case %d: expected %#v, got %#v",
				i, tc.result, actual)
		}
	}
}
