package cbor

import (
	"encoding/hex"
	"net"
	"testing"
)

var enc = Encoder{}

func TestAppendNil(t *testing.T) {
	s := enc.AppendNil([]byte{})
	got := string(s)
	want := "\xf6"
	if got != want {
		t.Errorf("appendNull() = 0x%s, want: 0x%s", hex.EncodeToString(s),
			hex.EncodeToString([]byte(want)))
	}
}

var booleanTestCases = []struct {
	val    bool
	binary string
	json   string
}{
	{true, "\xf5", "true"},
	{false, "\xf4", "false"},
}

func TestAppendBool(t *testing.T) {
	for _, tc := range booleanTestCases {
		s := enc.AppendBool([]byte{}, tc.val)
		got := string(s)
		if got != tc.binary {
			t.Errorf("AppendBool(%s)=0x%s, want: 0x%s",
				tc.json, hex.EncodeToString(s),
				hex.EncodeToString([]byte(tc.binary)))
		}
	}
}

var booleanArrayTestCases = []struct {
	val    []bool
	binary string
	json   string
}{
	{[]bool{true, false, true}, "\x83\xf5\xf4\xf5", "[true,false,true]"},
	{[]bool{true, false, false, true, false, true}, "\x86\xf5\xf4\xf4\xf5\xf4\xf5", "[true,false,false,true,false,true]"},
}

func TestAppendBoolArray(t *testing.T) {
	for _, tc := range booleanArrayTestCases {
		s := enc.AppendBools([]byte{}, tc.val)
		got := string(s)
		if got != tc.binary {
			t.Errorf("AppendBools(%s)=0x%s, want: 0x%s",
				tc.json, hex.EncodeToString(s),
				hex.EncodeToString([]byte(tc.binary)))
		}
	}
}

var integerTestCases = []struct {
	val    int
	binary string
}{
	// Value included in the type.
	{0, "\x00"},
	{1, "\x01"},
	{2, "\x02"},
	{3, "\x03"},
	{8, "\x08"},
	{9, "\x09"},
	{10, "\x0a"},
	{22, "\x16"},
	{23, "\x17"},
	// Value in 1 byte.
	{24, "\x18\x18"},
	{25, "\x18\x19"},
	{26, "\x18\x1a"},
	{100, "\x18\x64"},
	{254, "\x18\xfe"},
	{255, "\x18\xff"},
	// Value in 2 bytes.
	{256, "\x19\x01\x00"},
	{257, "\x19\x01\x01"},
	{1000, "\x19\x03\xe8"},
	{0xFFFF, "\x19\xff\xff"},
	// Value in 4 bytes.
	{0x10000, "\x1a\x00\x01\x00\x00"},
	{0x7FFFFFFE, "\x1a\x7f\xff\xff\xfe"},
	{1000000, "\x1a\x00\x0f\x42\x40"},
	// Negative number test cases.
	// Value included in the type.
	{-1, "\x20"},
	{-2, "\x21"},
	{-3, "\x22"},
	{-10, "\x29"},
	{-21, "\x34"},
	{-22, "\x35"},
	{-23, "\x36"},
	{-24, "\x37"},
	// Value in 1 byte.
	{-25, "\x38\x18"},
	{-26, "\x38\x19"},
	{-100, "\x38\x63"},
	{-254, "\x38\xfd"},
	{-255, "\x38\xfe"},
	{-256, "\x38\xff"},
	// Value in 2 bytes.
	{-257, "\x39\x01\x00"},
	{-258, "\x39\x01\x01"},
	{-1000, "\x39\x03\xe7"},
	// Value in 4 bytes.
	{-0x10001, "\x3a\x00\x01\x00\x00"},
	{-0x7FFFFFFE, "\x3a\x7f\xff\xff\xfd"},
	{-1000000, "\x3a\x00\x0f\x42\x3f"},
}

func TestAppendInt(t *testing.T) {
	for _, tc := range integerTestCases {
		s := enc.AppendInt([]byte{}, tc.val)
		got := string(s)
		if got != tc.binary {
			t.Errorf("AppendInt(0x%x)=0x%s, want: 0x%s",
				tc.val, hex.EncodeToString(s),
				hex.EncodeToString([]byte(tc.binary)))
		}
	}
}

var integerArrayTestCases = []struct {
	val    []int
	binary string
	json   string
}{
	{[]int{-1, 0, 200, 20}, "\x84\x20\x00\x18\xc8\x14", "[-1,0,200,20]"},
	{[]int{-200, -10, 200, 400}, "\x84\x38\xc7\x29\x18\xc8\x19\x01\x90", "[-200,-10,200,400]"},
	{[]int{1, 2, 3}, "\x83\x01\x02\x03", "[1,2,3]"},
	{[]int{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25},
		"\x98\x19\x01\x02\x03\x04\x05\x06\x07\x08\x09\x0a\x0b\x0c\x0d\x0e\x0f\x10\x11\x12\x13\x14\x15\x16\x17\x18\x18\x18\x19",
		"[1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25]"},
}

func TestAppendIntArray(t *testing.T) {
	for _, tc := range integerArrayTestCases {
		s := enc.AppendInts([]byte{}, tc.val)
		got := string(s)
		if got != tc.binary {
			t.Errorf("AppendInts(%s)=0x%s, want: 0x%s",
				tc.json, hex.EncodeToString(s),
				hex.EncodeToString([]byte(tc.binary)))
		}
	}
}

var float32TestCases = []struct {
	val    float32
	binary string
}{
	{0.0, "\xfa\x00\x00\x00\x00"},
	{-0.0, "\xfa\x00\x00\x00\x00"},
	{1.0, "\xfa\x3f\x80\x00\x00"},
	{1.5, "\xfa\x3f\xc0\x00\x00"},
	{65504.0, "\xfa\x47\x7f\xe0\x00"},
	{-4.0, "\xfa\xc0\x80\x00\x00"},
	{0.00006103515625, "\xfa\x38\x80\x00\x00"},
}

func TestAppendFloat32(t *testing.T) {
	for _, tc := range float32TestCases {
		s := enc.AppendFloat32([]byte{}, tc.val, -1)
		got := string(s)
		if got != tc.binary {
			t.Errorf("AppendFloat32(%f)=0x%s, want: 0x%s",
				tc.val, hex.EncodeToString(s),
				hex.EncodeToString([]byte(tc.binary)))
		}
	}
}

var ipAddrTestCases = []struct {
	ipaddr net.IP
	text   string // ASCII representation of ipaddr
	binary string // CBOR representation of ipaddr
}{
	{net.IP{10, 0, 0, 1}, "\"10.0.0.1\"", "\xd9\x01\x04\x44\x0a\x00\x00\x01"},
	{net.IP{0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3, 0x0, 0x0, 0x0, 0x0, 0x8a, 0x2e, 0x03, 0x70, 0x73, 0x34},
		"\"2001:db8:85a3::8a2e:370:7334\"",
		"\xd9\x01\x04\x50\x20\x01\x0d\xb8\x85\xa3\x00\x00\x00\x00\x8a\x2e\x03\x70\x73\x34"},
}

func TestAppendNetworkAddr(t *testing.T) {
	for _, tc := range ipAddrTestCases {
		s := enc.AppendIPAddr([]byte{}, tc.ipaddr)
		got := string(s)
		if got != tc.binary {
			t.Errorf("AppendIPAddr(%s)=0x%s, want: 0x%s",
				tc.ipaddr, hex.EncodeToString(s),
				hex.EncodeToString([]byte(tc.binary)))
		}
	}
}

var macAddrTestCases = []struct {
	macaddr net.HardwareAddr
	text    string // ASCII representation of macaddr
	binary  string // CBOR representation of macaddr
}{
	{net.HardwareAddr{0x12, 0x34, 0x56, 0x78, 0x90, 0xab}, "\"12:34:56:78:90:ab\"", "\xd9\x01\x04\x46\x12\x34\x56\x78\x90\xab"},
	{net.HardwareAddr{0x20, 0x01, 0x0d, 0xb8, 0x85, 0xa3}, "\"20:01:0d:b8:85:a3\"", "\xd9\x01\x04\x46\x20\x01\x0d\xb8\x85\xa3"},
}

func TestAppendMACAddr(t *testing.T) {
	for _, tc := range macAddrTestCases {
		s := enc.AppendMACAddr([]byte{}, tc.macaddr)
		got := string(s)
		if got != tc.binary {
			t.Errorf("AppendMACAddr(%s)=0x%s, want: 0x%s",
				tc.macaddr.String(), hex.EncodeToString(s),
				hex.EncodeToString([]byte(tc.binary)))
		}
	}
}

var IPPrefixTestCases = []struct {
	pfx    net.IPNet
	text   string // ASCII representation of pfx
	binary string // CBOR representation of pfx
}{
	{net.IPNet{IP: net.IP{0, 0, 0, 0}, Mask: net.CIDRMask(0, 32)}, "\"0.0.0.0/0\"", "\xd9\x01\x05\xa1\x44\x00\x00\x00\x00\x00"},
	{net.IPNet{IP: net.IP{192, 168, 0, 100}, Mask: net.CIDRMask(24, 32)}, "\"192.168.0.100/24\"",
		"\xd9\x01\x05\xa1\x44\xc0\xa8\x00\x64\x18\x18"},
}

func TestAppendIPPrefix(t *testing.T) {
	for _, tc := range IPPrefixTestCases {
		s := enc.AppendIPPrefix([]byte{}, tc.pfx)
		got := string(s)
		if got != tc.binary {
			t.Errorf("AppendIPPrefix(%s)=0x%s, want: 0x%s",
				tc.pfx.String(), hex.EncodeToString(s),
				hex.EncodeToString([]byte(tc.binary)))
		}
	}
}

func BenchmarkAppendInt(b *testing.B) {
	type st struct {
		sz  byte
		val int64
	}
	tests := map[string]st{
		"int-Positive": {sz: 0, val: 10000},
		"int-Negative": {sz: 0, val: -10000},
		"uint8":        {sz: 1, val: 100},
		"uint16":       {sz: 2, val: 0xfff},
		"uint32":       {sz: 4, val: 0xffffff},
		"uint64":       {sz: 8, val: 0xffffffffff},
		"int8":         {sz: 21, val: -120},
		"int16":        {sz: 22, val: -1200},
		"int32":        {sz: 23, val: 32000},
		"int64":        {sz: 24, val: 0xffffffffff},
	}
	for name, str := range tests {
		b.Run(name, func(b *testing.B) {
			buf := make([]byte, 0, 100)
			for i := 0; i < b.N; i++ {
				switch str.sz {
				case 0:
					_ = enc.AppendInt(buf, int(str.val))
				case 1:
					_ = enc.AppendUint8(buf, uint8(str.val))
				case 2:
					_ = enc.AppendUint16(buf, uint16(str.val))
				case 4:
					_ = enc.AppendUint32(buf, uint32(str.val))
				case 8:
					_ = enc.AppendUint64(buf, uint64(str.val))
				case 21:
					_ = enc.AppendInt8(buf, int8(str.val))
				case 22:
					_ = enc.AppendInt16(buf, int16(str.val))
				case 23:
					_ = enc.AppendInt32(buf, int32(str.val))
				case 24:
					_ = enc.AppendInt64(buf, int64(str.val))
				}
			}
		})
	}
}

func BenchmarkAppendFloat(b *testing.B) {
	type st struct {
		sz  byte
		val float64
	}
	tests := map[string]st{
		"Float32": {sz: 4, val: 10000.12345},
		"Float64": {sz: 8, val: -10000.54321},
	}
	for name, str := range tests {
		b.Run(name, func(b *testing.B) {
			buf := make([]byte, 0, 100)
			for i := 0; i < b.N; i++ {
				switch str.sz {
				case 4:
					_ = enc.AppendFloat32(buf, float32(str.val), -1)
				case 8:
					_ = enc.AppendFloat64(buf, str.val, -1)
				}
			}
		})
	}
}
