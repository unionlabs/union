package json

import (
	"encoding/json"
	"math"
	"math/rand"
	"net"
	"reflect"
	"testing"
)

func TestAppendType(t *testing.T) {
	w := map[string]func(interface{}) []byte{
		"AppendInt":                   func(v interface{}) []byte { return enc.AppendInt([]byte{}, v.(int)) },
		"AppendInt8":                  func(v interface{}) []byte { return enc.AppendInt8([]byte{}, v.(int8)) },
		"AppendInt16":                 func(v interface{}) []byte { return enc.AppendInt16([]byte{}, v.(int16)) },
		"AppendInt32":                 func(v interface{}) []byte { return enc.AppendInt32([]byte{}, v.(int32)) },
		"AppendInt64":                 func(v interface{}) []byte { return enc.AppendInt64([]byte{}, v.(int64)) },
		"AppendUint":                  func(v interface{}) []byte { return enc.AppendUint([]byte{}, v.(uint)) },
		"AppendUint8":                 func(v interface{}) []byte { return enc.AppendUint8([]byte{}, v.(uint8)) },
		"AppendUint16":                func(v interface{}) []byte { return enc.AppendUint16([]byte{}, v.(uint16)) },
		"AppendUint32":                func(v interface{}) []byte { return enc.AppendUint32([]byte{}, v.(uint32)) },
		"AppendUint64":                func(v interface{}) []byte { return enc.AppendUint64([]byte{}, v.(uint64)) },
		"AppendFloat32":               func(v interface{}) []byte { return enc.AppendFloat32([]byte{}, v.(float32), -1) },
		"AppendFloat64":               func(v interface{}) []byte { return enc.AppendFloat64([]byte{}, v.(float64), -1) },
		"AppendFloat32SmallPrecision": func(v interface{}) []byte { return enc.AppendFloat32([]byte{}, v.(float32), 1) },
		"AppendFloat64SmallPrecision": func(v interface{}) []byte { return enc.AppendFloat64([]byte{}, v.(float64), 1) },
	}
	tests := []struct {
		name  string
		fn    string
		input interface{}
		want  []byte
	}{
		{"AppendInt8(math.MaxInt8)", "AppendInt8", int8(math.MaxInt8), []byte("127")},
		{"AppendInt16(math.MaxInt16)", "AppendInt16", int16(math.MaxInt16), []byte("32767")},
		{"AppendInt32(math.MaxInt32)", "AppendInt32", int32(math.MaxInt32), []byte("2147483647")},
		{"AppendInt64(math.MaxInt64)", "AppendInt64", int64(math.MaxInt64), []byte("9223372036854775807")},

		{"AppendUint8(math.MaxUint8)", "AppendUint8", uint8(math.MaxUint8), []byte("255")},
		{"AppendUint16(math.MaxUint16)", "AppendUint16", uint16(math.MaxUint16), []byte("65535")},
		{"AppendUint32(math.MaxUint32)", "AppendUint32", uint32(math.MaxUint32), []byte("4294967295")},
		{"AppendUint64(math.MaxUint64)", "AppendUint64", uint64(math.MaxUint64), []byte("18446744073709551615")},

		{"AppendFloat32(-Inf)", "AppendFloat32", float32(math.Inf(-1)), []byte(`"-Inf"`)},
		{"AppendFloat32(+Inf)", "AppendFloat32", float32(math.Inf(1)), []byte(`"+Inf"`)},
		{"AppendFloat32(NaN)", "AppendFloat32", float32(math.NaN()), []byte(`"NaN"`)},
		{"AppendFloat32(0)", "AppendFloat32", float32(0), []byte(`0`)},
		{"AppendFloat32(-1.1)", "AppendFloat32", float32(-1.1), []byte(`-1.1`)},
		{"AppendFloat32(1e20)", "AppendFloat32", float32(1e20), []byte(`100000000000000000000`)},
		{"AppendFloat32(1e21)", "AppendFloat32", float32(1e21), []byte(`1e+21`)},

		{"AppendFloat64(-Inf)", "AppendFloat64", float64(math.Inf(-1)), []byte(`"-Inf"`)},
		{"AppendFloat64(+Inf)", "AppendFloat64", float64(math.Inf(1)), []byte(`"+Inf"`)},
		{"AppendFloat64(NaN)", "AppendFloat64", float64(math.NaN()), []byte(`"NaN"`)},
		{"AppendFloat64(0)", "AppendFloat64", float64(0), []byte(`0`)},
		{"AppendFloat64(-1.1)", "AppendFloat64", float64(-1.1), []byte(`-1.1`)},
		{"AppendFloat64(1e20)", "AppendFloat64", float64(1e20), []byte(`100000000000000000000`)},
		{"AppendFloat64(1e21)", "AppendFloat64", float64(1e21), []byte(`1e+21`)},

		{"AppendFloat32SmallPrecision(-1.123)", "AppendFloat32SmallPrecision", float32(-1.123), []byte(`-1.1`)},
		{"AppendFloat64SmallPrecision(-1.123)", "AppendFloat64SmallPrecision", float64(-1.123), []byte(`-1.1`)},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got := w[tt.fn](tt.input); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("got %s, want %s", got, tt.want)
			}
		})
	}
}

func Test_appendMAC(t *testing.T) {
	MACtests := []struct {
		input string
		want  []byte
	}{
		{"01:23:45:67:89:ab", []byte(`"01:23:45:67:89:ab"`)},
		{"cd:ef:11:22:33:44", []byte(`"cd:ef:11:22:33:44"`)},
	}
	for _, tt := range MACtests {
		t.Run("MAC", func(t *testing.T) {
			ha, _ := net.ParseMAC(tt.input)
			if got := enc.AppendMACAddr([]byte{}, ha); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("appendMACAddr() = %s, want %s", got, tt.want)
			}
		})
	}
}

func Test_appendIP(t *testing.T) {
	IPv4tests := []struct {
		input net.IP
		want  []byte
	}{
		{net.IP{0, 0, 0, 0}, []byte(`"0.0.0.0"`)},
		{net.IP{192, 0, 2, 200}, []byte(`"192.0.2.200"`)},
	}

	for _, tt := range IPv4tests {
		t.Run("IPv4", func(t *testing.T) {
			if got := enc.AppendIPAddr([]byte{}, tt.input); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("appendIPAddr() = %s, want %s", got, tt.want)
			}
		})
	}
	IPv6tests := []struct {
		input net.IP
		want  []byte
	}{
		{net.IPv6zero, []byte(`"::"`)},
		{net.IPv6linklocalallnodes, []byte(`"ff02::1"`)},
		{net.IP{0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x34}, []byte(`"2001:db8:85a3::8a2e:370:7334"`)},
	}
	for _, tt := range IPv6tests {
		t.Run("IPv6", func(t *testing.T) {
			if got := enc.AppendIPAddr([]byte{}, tt.input); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("appendIPAddr() = %s, want %s", got, tt.want)
			}
		})
	}
}

func Test_appendIPPrefix(t *testing.T) {
	IPv4Prefixtests := []struct {
		input net.IPNet
		want  []byte
	}{
		{net.IPNet{IP: net.IP{0, 0, 0, 0}, Mask: net.IPv4Mask(0, 0, 0, 0)}, []byte(`"0.0.0.0/0"`)},
		{net.IPNet{IP: net.IP{192, 0, 2, 200}, Mask: net.IPv4Mask(255, 255, 255, 0)}, []byte(`"192.0.2.200/24"`)},
	}
	for _, tt := range IPv4Prefixtests {
		t.Run("IPv4", func(t *testing.T) {
			if got := enc.AppendIPPrefix([]byte{}, tt.input); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("appendIPPrefix() = %s, want %s", got, tt.want)
			}
		})
	}
	IPv6Prefixtests := []struct {
		input net.IPNet
		want  []byte
	}{
		{net.IPNet{IP: net.IPv6zero, Mask: net.CIDRMask(0, 128)}, []byte(`"::/0"`)},
		{net.IPNet{IP: net.IPv6linklocalallnodes, Mask: net.CIDRMask(128, 128)}, []byte(`"ff02::1/128"`)},
		{net.IPNet{IP: net.IP{0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x00, 0x00, 0x00, 0x00, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x34},
			Mask: net.CIDRMask(64, 128)},
			[]byte(`"2001:db8:85a3::8a2e:370:7334/64"`)},
	}
	for _, tt := range IPv6Prefixtests {
		t.Run("IPv6", func(t *testing.T) {
			if got := enc.AppendIPPrefix([]byte{}, tt.input); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("appendIPPrefix() = %s, want %s", got, tt.want)
			}
		})
	}
}

func Test_appendMac(t *testing.T) {
	MACtests := []struct {
		input net.HardwareAddr
		want  []byte
	}{
		{net.HardwareAddr{0x12, 0x34, 0x56, 0x78, 0x90, 0xab}, []byte(`"12:34:56:78:90:ab"`)},
		{net.HardwareAddr{0x12, 0x34, 0x00, 0x00, 0x90, 0xab}, []byte(`"12:34:00:00:90:ab"`)},
	}

	for _, tt := range MACtests {
		t.Run("MAC", func(t *testing.T) {
			if got := enc.AppendMACAddr([]byte{}, tt.input); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("appendMAC() = %s, want %s", got, tt.want)
			}
		})
	}
}

func Test_appendType(t *testing.T) {
	typeTests := []struct {
		label string
		input interface{}
		want  []byte
	}{
		{"int", 42, []byte(`"int"`)},
		{"MAC", net.HardwareAddr{0x12, 0x34, 0x00, 0x00, 0x90, 0xab}, []byte(`"net.HardwareAddr"`)},
		{"float64", float64(2.50), []byte(`"float64"`)},
		{"nil", nil, []byte(`"<nil>"`)},
		{"bool", true, []byte(`"bool"`)},
	}

	for _, tt := range typeTests {
		t.Run(tt.label, func(t *testing.T) {
			if got := enc.AppendType([]byte{}, tt.input); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("appendType() = %s, want %s", got, tt.want)
			}
		})
	}
}

func Test_appendObjectData(t *testing.T) {
	tests := []struct {
		dst  []byte
		obj  []byte
		want []byte
	}{
		{[]byte{}, []byte(`{"foo":"bar"}`), []byte(`"foo":"bar"}`)},
		{[]byte(`{"qux":"quz"`), []byte(`{"foo":"bar"}`), []byte(`{"qux":"quz","foo":"bar"}`)},
		{[]byte{}, []byte(`"foo":"bar"`), []byte(`"foo":"bar"`)},
		{[]byte(`{"qux":"quz"`), []byte(`"foo":"bar"`), []byte(`{"qux":"quz","foo":"bar"`)},
	}
	for _, tt := range tests {
		t.Run("ObjectData", func(t *testing.T) {
			if got := enc.AppendObjectData(tt.dst, tt.obj); !reflect.DeepEqual(got, tt.want) {
				t.Errorf("appendObjectData() = %s, want %s", got, tt.want)
			}
		})
	}
}

var float64Tests = []struct {
	Name string
	Val  float64
	Want string
}{
	{
		Name: "Positive integer",
		Val:  1234.0,
		Want: "1234",
	},
	{
		Name: "Negative integer",
		Val:  -5678.0,
		Want: "-5678",
	},
	{
		Name: "Positive decimal",
		Val:  12.3456,
		Want: "12.3456",
	},
	{
		Name: "Negative decimal",
		Val:  -78.9012,
		Want: "-78.9012",
	},
	{
		Name: "Large positive number",
		Val:  123456789.0,
		Want: "123456789",
	},
	{
		Name: "Large negative number",
		Val:  -987654321.0,
		Want: "-987654321",
	},
	{
		Name: "Zero",
		Val:  0.0,
		Want: "0",
	},
	{
		Name: "Smallest positive value",
		Val:  math.SmallestNonzeroFloat64,
		Want: "5e-324",
	},
	{
		Name: "Largest positive value",
		Val:  math.MaxFloat64,
		Want: "1.7976931348623157e+308",
	},
	{
		Name: "Smallest negative value",
		Val:  -math.SmallestNonzeroFloat64,
		Want: "-5e-324",
	},
	{
		Name: "Largest negative value",
		Val:  -math.MaxFloat64,
		Want: "-1.7976931348623157e+308",
	},
	{
		Name: "NaN",
		Val:  math.NaN(),
		Want: `"NaN"`,
	},
	{
		Name: "+Inf",
		Val:  math.Inf(1),
		Want: `"+Inf"`,
	},
	{
		Name: "-Inf",
		Val:  math.Inf(-1),
		Want: `"-Inf"`,
	},
	{
		Name: "Clean up e-09 to e-9 case 1",
		Val:  1e-9,
		Want: "1e-9",
	},
	{
		Name: "Clean up e-09 to e-9 case 2",
		Val:  -2.236734e-9,
		Want: "-2.236734e-9",
	},
}

func TestEncoder_AppendFloat64(t *testing.T) {
	for _, tc := range float64Tests {
		t.Run(tc.Name, func(t *testing.T) {
			var b []byte
			b = (Encoder{}).AppendFloat64(b, tc.Val, -1)
			if s := string(b); tc.Want != s {
				t.Errorf("%q", s)
			}
		})
	}
}

func FuzzEncoder_AppendFloat64(f *testing.F) {
	for _, tc := range float64Tests {
		f.Add(tc.Val)
	}
	f.Fuzz(func(t *testing.T, val float64) {
		actual := (Encoder{}).AppendFloat64(nil, val, -1)
		if len(actual) == 0 {
			t.Fatal("empty buffer")
		}

		if actual[0] == '"' {
			switch string(actual) {
			case `"NaN"`:
				if !math.IsNaN(val) {
					t.Fatalf("expected %v got NaN", val)
				}
			case `"+Inf"`:
				if !math.IsInf(val, 1) {
					t.Fatalf("expected %v got +Inf", val)
				}
			case `"-Inf"`:
				if !math.IsInf(val, -1) {
					t.Fatalf("expected %v got -Inf", val)
				}
			default:
				t.Fatalf("unexpected string: %s", actual)
			}
			return
		}

		if expected, err := json.Marshal(val); err != nil {
			t.Error(err)
		} else if string(actual) != string(expected) {
			t.Errorf("expected %s, got %s", expected, actual)
		}

		var parsed float64
		if err := json.Unmarshal(actual, &parsed); err != nil {
			t.Fatal(err)
		}

		if parsed != val {
			t.Fatalf("expected %v, got %v", val, parsed)
		}
	})
}

var float32Tests = []struct {
	Name string
	Val  float32
	Want string
}{
	{
		Name: "Positive integer",
		Val:  1234.0,
		Want: "1234",
	},
	{
		Name: "Negative integer",
		Val:  -5678.0,
		Want: "-5678",
	},
	{
		Name: "Positive decimal",
		Val:  12.3456,
		Want: "12.3456",
	},
	{
		Name: "Negative decimal",
		Val:  -78.9012,
		Want: "-78.9012",
	},
	{
		Name: "Large positive number",
		Val:  123456789.0,
		Want: "123456790",
	},
	{
		Name: "Large negative number",
		Val:  -987654321.0,
		Want: "-987654340",
	},
	{
		Name: "Zero",
		Val:  0.0,
		Want: "0",
	},
	{
		Name: "Smallest positive value",
		Val:  math.SmallestNonzeroFloat32,
		Want: "1e-45",
	},
	{
		Name: "Largest positive value",
		Val:  math.MaxFloat32,
		Want: "3.4028235e+38",
	},
	{
		Name: "Smallest negative value",
		Val:  -math.SmallestNonzeroFloat32,
		Want: "-1e-45",
	},
	{
		Name: "Largest negative value",
		Val:  -math.MaxFloat32,
		Want: "-3.4028235e+38",
	},
	{
		Name: "NaN",
		Val:  float32(math.NaN()),
		Want: `"NaN"`,
	},
	{
		Name: "+Inf",
		Val:  float32(math.Inf(1)),
		Want: `"+Inf"`,
	},
	{
		Name: "-Inf",
		Val:  float32(math.Inf(-1)),
		Want: `"-Inf"`,
	},
	{
		Name: "Clean up e-09 to e-9 case 1",
		Val:  1e-9,
		Want: "1e-9",
	},
	{
		Name: "Clean up e-09 to e-9 case 2",
		Val:  -2.236734e-9,
		Want: "-2.236734e-9",
	},
}

func TestEncoder_AppendFloat32(t *testing.T) {
	for _, tc := range float32Tests {
		t.Run(tc.Name, func(t *testing.T) {
			var b []byte
			b = (Encoder{}).AppendFloat32(b, tc.Val, -1)
			if s := string(b); tc.Want != s {
				t.Errorf("%q", s)
			}
		})
	}
}

func FuzzEncoder_AppendFloat32(f *testing.F) {
	for _, tc := range float32Tests {
		f.Add(tc.Val)
	}
	f.Fuzz(func(t *testing.T, val float32) {
		actual := (Encoder{}).AppendFloat32(nil, val, -1)
		if len(actual) == 0 {
			t.Fatal("empty buffer")
		}

		if actual[0] == '"' {
			val := float64(val)
			switch string(actual) {
			case `"NaN"`:
				if !math.IsNaN(val) {
					t.Fatalf("expected %v got NaN", val)
				}
			case `"+Inf"`:
				if !math.IsInf(val, 1) {
					t.Fatalf("expected %v got +Inf", val)
				}
			case `"-Inf"`:
				if !math.IsInf(val, -1) {
					t.Fatalf("expected %v got -Inf", val)
				}
			default:
				t.Fatalf("unexpected string: %s", actual)
			}
			return
		}

		if expected, err := json.Marshal(val); err != nil {
			t.Error(err)
		} else if string(actual) != string(expected) {
			t.Errorf("expected %s, got %s", expected, actual)
		}

		var parsed float32
		if err := json.Unmarshal(actual, &parsed); err != nil {
			t.Fatal(err)
		}

		if parsed != val {
			t.Fatalf("expected %v, got %v", val, parsed)
		}
	})
}

func generateFloat32s(n int) []float32 {
	floats := make([]float32, n)
	for i := 0; i < n; i++ {
		floats[i] = rand.Float32()
	}
	return floats
}

func generateFloat64s(n int) []float64 {
	floats := make([]float64, n)
	for i := 0; i < n; i++ {
		floats[i] = rand.Float64()
	}
	return floats
}

// this is really just for the memory allocation characteristics
func BenchmarkEncoder_AppendFloat32(b *testing.B) {
	floats := append(generateFloat32s(5000), float32(math.NaN()), float32(math.Inf(1)), float32(math.Inf(-1)))
	dst := make([]byte, 0, 128)

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		for _, f := range floats {
			dst = (Encoder{}).AppendFloat32(dst[:0], f, -1)
		}
	}
}

// this is really just for the memory allocation characteristics
func BenchmarkEncoder_AppendFloat64(b *testing.B) {
	floats := append(generateFloat64s(5000), math.NaN(), math.Inf(1), math.Inf(-1))
	dst := make([]byte, 0, 128)

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		for _, f := range floats {
			dst = (Encoder{}).AppendFloat64(dst[:0], f, -1)
		}
	}
}
